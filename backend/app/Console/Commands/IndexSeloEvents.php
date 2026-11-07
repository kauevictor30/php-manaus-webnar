<?php

namespace App\Console\Commands;

use App\Enums\BadgeKind;
use App\Models\BadgeRecord;
use App\Models\IndexerCursor;
use Carbon\CarbonImmutable;
use GuzzleHttp\Client;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use RuntimeException;
use Soneso\StellarSDK\Soroban\Address;
use Soneso\StellarSDK\Soroban\Requests\EventFilter;
use Soneso\StellarSDK\Soroban\Requests\EventFilters;
use Soneso\StellarSDK\Soroban\Requests\GetEventsRequest;
use Soneso\StellarSDK\Soroban\Requests\PaginationOptions;
use Soneso\StellarSDK\Soroban\Responses\EventInfo;
use Soneso\StellarSDK\Soroban\Responses\GetEventsResponse;
use Soneso\StellarSDK\Soroban\SorobanServer;
use Soneso\StellarSDK\Xdr\XdrSCVal;

/**
 * Copia os eventos do contrato Selo para o "arquivo morto" (design §3.4).
 * A fonte da verdade do estado é o contrato; aqui só guardamos o histórico.
 */
class IndexSeloEvents extends Command
{
    protected $signature = 'selo:index';

    protected $description = 'Indexa os eventos issued/revoked do contrato Selo via Stellar RPC getEvents';

    private const PAGE_LIMIT = 100;

    /** @var array{issued:int, revoked:int, ignored:int} */
    private array $stats = ['issued' => 0, 'revoked' => 0, 'ignored' => 0];

    public function handle(): int
    {
        $contractId = (string) config('selo.contract_id');
        $startLedger = (int) config('selo.start_ledger');

        if ($contractId === '' || $startLedger <= 0) {
            $this->error('Configure SELO_CONTRACT_ID e SELO_START_LEDGER no .env.');

            return self::FAILURE;
        }

        $server = $this->makeServer();
        $filters = new EventFilters(new EventFilter('contract', [$contractId]));
        $pages = 0;

        while (true) {
            $saved = IndexerCursor::where('contract_id', $contractId)->first();

            // startLedger e cursor são mutuamente exclusivos no RPC.
            $request = $saved
                ? new GetEventsRequest(null, null, $filters, new PaginationOptions($saved->cursor, self::PAGE_LIMIT))
                : new GetEventsRequest($startLedger, null, $filters, new PaginationOptions(null, self::PAGE_LIMIT));

            $response = $server->getEvents($request);

            if ($error = $response->getError()) {
                $message = 'Erro do RPC getEvents: '.$error->getMessage();
                if (! $saved) {
                    $message .= " (SELO_START_LEDGER={$startLedger}). Se o ledger saiu da janela do RPC, atualize SELO_START_LEDGER no .env.";
                }
                $this->error($message);

                return self::FAILURE;
            }

            if (! $saved && $response->getOldestLedger() !== null && $startLedger < $response->getOldestLedger()) {
                $this->error("SELO_START_LEDGER={$startLedger} é anterior ao ledger mais antigo do RPC ({$response->getOldestLedger()}). Atualize SELO_START_LEDGER no .env.");

                return self::FAILURE;
            }

            $events = $response->getEvents() ?? [];
            $this->processPage($contractId, $events, $response);
            $pages++;

            if ($events === [] || $response->getCursor() === null) {
                break;
            }
        }

        $this->info(sprintf(
            'selo:index ok — páginas: %d · issued: %d · revoked: %d · ignorados: %d · badge_records: %d',
            $pages,
            $this->stats['issued'],
            $this->stats['revoked'],
            $this->stats['ignored'],
            BadgeRecord::count(),
        ));

        return self::SUCCESS;
    }

    /**
     * Cliente Guzzle com a handler stack do Laravel: em teste, Http::fake() intercepta.
     */
    private function makeServer(): SorobanServer
    {
        $client = new Client([
            'handler' => Http::buildHandlerStack(),
            'timeout' => 30,
            'connect_timeout' => 5,
        ]);

        return new SorobanServer((string) config('selo.rpc_url'), $client);
    }

    /**
     * Uma transação por página: registros e cursor gravados juntos.
     *
     * @param  array<EventInfo>  $events
     */
    private function processPage(string $contractId, array $events, GetEventsResponse $response): void
    {
        DB::transaction(function () use ($contractId, $events, $response) {
            foreach ($events as $event) {
                $this->processEvent($contractId, $event);
            }

            if ($response->getCursor() !== null) {
                IndexerCursor::updateOrCreate(
                    ['contract_id' => $contractId],
                    ['cursor' => $response->getCursor()],
                );
            }
        });
    }

    private function processEvent(string $contractId, EventInfo $event): void
    {
        // Defesa extra contra "indexador envenenado" (design §5).
        if ($event->getContractId() !== $contractId || $event->getInSuccessfulContractCall() === false) {
            $this->stats['ignored']++;

            return;
        }

        $topic = $event->getTopic();
        if (count($topic) < 2) {
            $this->stats['ignored']++;

            return;
        }

        $name = XdrSCVal::fromBase64Xdr($topic[0])->getSym();

        match ($name) {
            'issued' => $this->handleIssued($event, $topic[1]),
            'revoked' => $this->handleRevoked($event, $topic[1]),
            default => $this->stats['ignored']++, // "issuer" e outros: fora do escopo
        };
    }

    private function handleIssued(EventInfo $event, string $memberXdr): void
    {
        $fields = $this->decodeMap($event->getValue());

        $record = BadgeRecord::firstOrCreate(
            ['rpc_event_id' => $event->getId()],
            [
                'member_address' => $this->decodeAddress(XdrSCVal::fromBase64Xdr($memberXdr)),
                'kind' => $this->decodeKind($fields['kind']),
                'event_id' => $this->requireInt($fields['event_id']->getU32(), 'event_id'),
                'issuer' => $this->decodeAddress($fields['issuer']),
                'status' => BadgeRecord::STATUS_ISSUED,
                'tx_hash' => $event->getTxHash(),
                'ledger' => $event->getLedger(),
                'issued_at' => CarbonImmutable::createFromTimestampUTC(
                    $this->requireInt($fields['issued_at']->getU64(), 'issued_at'),
                ),
            ],
        );

        if ($record->wasRecentlyCreated) {
            $this->stats['issued']++;
        }
    }

    /**
     * D-07: marca o issued mais recente com ledger <= ledger do revoke.
     * Reprocessar o mesmo revoke não acha mais issued correspondente (idempotente).
     */
    private function handleRevoked(EventInfo $event, string $memberXdr): void
    {
        $fields = $this->decodeMap($event->getValue());

        $record = BadgeRecord::query()
            ->where('member_address', $this->decodeAddress(XdrSCVal::fromBase64Xdr($memberXdr)))
            ->where('kind', $this->decodeKind($fields['kind'])->value)
            ->where('event_id', $this->requireInt($fields['event_id']->getU32(), 'event_id'))
            ->where('status', BadgeRecord::STATUS_ISSUED)
            ->where('ledger', '<=', $event->getLedger())
            ->orderByDesc('ledger')
            ->orderByDesc('id')
            ->first();

        if ($record === null) {
            return;
        }

        $record->update([
            'status' => BadgeRecord::STATUS_REVOKED,
            'revoked_at' => CarbonImmutable::parse($event->getLedgerClosedAt())->utc(),
        ]);
        $this->stats['revoked']++;
    }

    /**
     * @return array<string, XdrSCVal> ScMap indexado pelo nome (Symbol) do campo
     */
    private function decodeMap(string $base64): array
    {
        $map = XdrSCVal::fromBase64Xdr($base64)->getMap();
        if ($map === null) {
            throw new RuntimeException('Valor do evento não é um ScMap.');
        }

        $fields = [];
        foreach ($map as $entry) {
            $fields[(string) $entry->getKey()->getSym()] = $entry->getVal();
        }

        return $fields;
    }

    /** BadgeKind chega como ScVec[Symbol("Mentor")]. */
    private function decodeKind(?XdrSCVal $val): BadgeKind
    {
        $vec = $val?->getVec();
        $symbol = $vec[0] ?? null;

        return BadgeKind::from((string) $symbol?->getSym());
    }

    private function decodeAddress(?XdrSCVal $val): string
    {
        if ($val === null) {
            throw new RuntimeException('Endereço ausente no evento.');
        }
        $address = Address::fromXdrSCVal($val);

        return $address->getAccountId() ?? $address->toStrKey();
    }

    private function requireInt(?int $value, string $field): int
    {
        if ($value === null) {
            throw new RuntimeException("Campo {$field} ausente ou com tipo inesperado no evento.");
        }

        return $value;
    }
}
