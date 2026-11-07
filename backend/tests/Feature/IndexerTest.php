<?php

namespace Tests\Feature;

use App\Models\BadgeRecord;
use App\Models\IndexerCursor;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Http\Client\Request;
use Illuminate\Support\Facades\Http;
use Soneso\StellarSDK\Crypto\KeyPair;
use Soneso\StellarSDK\Soroban\Address;
use Soneso\StellarSDK\Xdr\XdrSCMapEntry;
use Soneso\StellarSDK\Xdr\XdrSCVal;
use Tests\TestCase;

/**
 * selo:index com o RPC simulado (Http::fake). O XDR é gerado com o SDK,
 * no mesmo formato do #[contractevent] do contrato (design §2.4).
 */
class IndexerTest extends TestCase
{
    use RefreshDatabase;

    private const CONTRACT = 'CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q';

    private const RPC = 'https://soroban-testnet.stellar.org';

    private const START = 5611351;

    /** @var array<int, array<string, mixed>> eventos "on-chain" visíveis ao RPC falso */
    private array $chain = [];

    private int $seq = 0;

    /** @var array<string, mixed>|null quando definido, o RPC falso responde com este erro */
    private ?array $rpcError = null;

    private string $member;

    private string $issuer;

    protected function setUp(): void
    {
        parent::setUp();

        config([
            'selo.contract_id' => self::CONTRACT,
            'selo.rpc_url' => self::RPC,
            'selo.start_ledger' => self::START,
        ]);

        $this->member = KeyPair::random()->getAccountId();
        $this->issuer = KeyPair::random()->getAccountId();

        // RPC falso: o cursor "cur-N" significa "depois dos N primeiros eventos".
        Http::fake([self::RPC.'*' => function (Request $request) {
            if ($this->rpcError !== null) {
                return Http::response(['jsonrpc' => '2.0', 'id' => 1, 'error' => $this->rpcError]);
            }

            $params = json_decode($request->body(), true)['params'];
            $cursor = $params['pagination']['cursor'] ?? null;
            $offset = $cursor === null ? 0 : (int) substr($cursor, 4);
            $limit = $params['pagination']['limit'] ?? 100;
            $page = array_slice($this->chain, $offset, $limit);

            return Http::response([
                'jsonrpc' => '2.0',
                'id' => 1,
                'result' => [
                    'events' => $page,
                    'cursor' => 'cur-'.($offset + count($page)),
                    'latestLedger' => self::START + 1000,
                    'oldestLedger' => self::START - 1000,
                ],
            ]);
        }]);
    }

    // ---------- helpers de XDR (via SDK, sem parser manual) ----------

    private function sym(string $s): XdrSCVal
    {
        return XdrSCVal::forSymbol($s);
    }

    private function addr(string $g): XdrSCVal
    {
        return Address::fromAccountId($g)->toXdrSCVal();
    }

    private function kind(string $k): XdrSCVal
    {
        return XdrSCVal::forVec([$this->sym($k)]);
    }

    /** @param array<string, XdrSCVal> $fields */
    private function map(array $fields): string
    {
        $entries = [];
        foreach ($fields as $k => $v) {
            $entries[] = new XdrSCMapEntry($this->sym($k), $v);
        }

        return XdrSCVal::forMap($entries)->toBase64Xdr();
    }

    private function pushEvent(string $name, string $topicAddress, string $value, int $ledger, string $contract = self::CONTRACT): string
    {
        $id = sprintf('%019d-%010d', $ledger * 4096, $this->seq++);
        $this->chain[] = [
            'type' => 'contract',
            'ledger' => $ledger,
            'ledgerClosedAt' => '2026-11-07T19:44:01Z',
            'contractId' => $contract,
            'id' => $id,
            'operationIndex' => 0,
            'transactionIndex' => 1,
            'txHash' => str_repeat(dechex($this->seq % 16), 64),
            'topic' => [$this->sym($name)->toBase64Xdr(), $this->addr($topicAddress)->toBase64Xdr()],
            'value' => $value,
        ];

        return $id;
    }

    private function issued(string $kind, int $eventId, int $ledger, int $issuedAt = 1794081911): string
    {
        return $this->pushEvent('issued', $this->member, $this->map([
            'kind' => $this->kind($kind),
            'event_id' => XdrSCVal::forU32($eventId),
            'issuer' => $this->addr($this->issuer),
            'issued_at' => XdrSCVal::forU64($issuedAt),
        ]), $ledger);
    }

    private function revoked(string $kind, int $eventId, int $ledger): string
    {
        return $this->pushEvent('revoked', $this->member, $this->map([
            'kind' => $this->kind($kind),
            'event_id' => XdrSCVal::forU32($eventId),
        ]), $ledger);
    }

    private function issuerEvent(int $ledger): void
    {
        $this->pushEvent('issuer', $this->issuer, $this->map([
            'enabled' => XdrSCVal::forTrue(),
        ]), $ledger);
    }

    // ---------- testes ----------

    public function test_issued_cria_registro_com_os_campos_do_evento(): void
    {
        $id = $this->issued('Mentor', 7, self::START + 10, 1794081911);

        $this->artisan('selo:index')->assertSuccessful();

        $record = BadgeRecord::sole();
        $this->assertSame($id, $record->rpc_event_id);
        $this->assertSame($this->member, $record->member_address);
        $this->assertSame('Mentor', $record->kind->value);
        $this->assertSame(7, $record->event_id);
        $this->assertSame($this->issuer, $record->issuer);
        $this->assertSame('issued', $record->status);
        $this->assertSame(self::START + 10, $record->ledger);
        $this->assertSame(64, strlen($record->tx_hash));
        $this->assertSame(1794081911, $record->issued_at->getTimestamp());
        $this->assertNull($record->revoked_at);
    }

    public function test_rodar_duas_vezes_nao_duplica_e_salva_cursor(): void
    {
        $this->issuerEvent(self::START + 5);
        $this->issued('Participante', 1, self::START + 10);
        $this->issued('Palestrante', 1, self::START + 11);

        $this->artisan('selo:index')->assertSuccessful();
        $this->assertSame(2, BadgeRecord::count());
        $this->assertSame('cur-3', IndexerCursor::where('contract_id', self::CONTRACT)->value('cursor'));

        $this->artisan('selo:index')->assertSuccessful();
        $this->assertSame(2, BadgeRecord::count());
        $this->assertSame(1, IndexerCursor::count());
    }

    public function test_reprocessar_sem_cursor_continua_idempotente(): void
    {
        $this->issued('Participante', 1, self::START + 10);
        $this->revoked('Participante', 1, self::START + 20);

        $this->artisan('selo:index')->assertSuccessful();
        IndexerCursor::query()->delete(); // força reler tudo desde SELO_START_LEDGER
        $this->artisan('selo:index')->assertSuccessful();

        $this->assertSame(1, BadgeRecord::count());
        $this->assertSame('revoked', BadgeRecord::sole()->status);
    }

    public function test_primeira_execucao_usa_start_ledger_e_depois_so_o_cursor(): void
    {
        $this->issued('Participante', 1, self::START + 10);

        $this->artisan('selo:index')->assertSuccessful();
        $this->artisan('selo:index')->assertSuccessful();

        $bodies = collect(Http::recorded())->map(fn ($pair) => json_decode($pair[0]->body(), true)['params']);

        $first = $bodies->first();
        $this->assertSame(self::START, $first['startLedger']);
        $this->assertArrayNotHasKey('cursor', $first['pagination']);
        $this->assertSame(100, $first['pagination']['limit']);
        $this->assertSame([['type' => 'contract', 'contractIds' => [self::CONTRACT]]], $first['filters']);

        $bodies->skip(1)->each(function ($params) {
            $this->assertArrayNotHasKey('startLedger', $params);
            $this->assertArrayHasKey('cursor', $params['pagination']);
        });
    }

    public function test_revoked_depois_de_issued_muda_o_status(): void
    {
        $this->issued('Organizador', 3, self::START + 10);
        $this->artisan('selo:index')->assertSuccessful();
        $this->assertSame('issued', BadgeRecord::sole()->status);

        $this->revoked('Organizador', 3, self::START + 20);
        $this->artisan('selo:index')->assertSuccessful();

        $record = BadgeRecord::sole();
        $this->assertSame('revoked', $record->status);
        $this->assertSame('2026-11-07T19:44:01+00:00', $record->revoked_at->toIso8601String());
    }

    public function test_revoke_antigo_nao_revoga_badge_reemitido_d07(): void
    {
        $first = $this->issued('Mentor', 1, self::START + 10);
        $this->revoked('Mentor', 1, self::START + 20);
        $second = $this->issued('Mentor', 1, self::START + 30);

        $this->artisan('selo:index')->assertSuccessful();
        IndexerCursor::query()->delete(); // reprocessa o revoke antigo
        $this->artisan('selo:index')->assertSuccessful();

        $this->assertSame(2, BadgeRecord::count());
        $this->assertSame('revoked', BadgeRecord::where('rpc_event_id', $first)->value('status'));
        $this->assertSame('issued', BadgeRecord::where('rpc_event_id', $second)->value('status'));
    }

    public function test_evento_issuer_e_de_outro_contrato_sao_ignorados(): void
    {
        $this->issuerEvent(self::START + 5);
        $this->pushEvent('issued', $this->member, $this->map([
            'kind' => $this->kind('Palestrante'),
            'event_id' => XdrSCVal::forU32(1),
            'issuer' => $this->addr($this->issuer),
            'issued_at' => XdrSCVal::forU64(1),
        ]), self::START + 6, 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');

        $this->artisan('selo:index')->assertSuccessful();

        $this->assertSame(0, BadgeRecord::count());
        $this->assertNotNull(IndexerCursor::first());
    }

    public function test_erro_do_rpc_sem_cursor_pede_para_atualizar_start_ledger(): void
    {
        $this->rpcError = ['code' => -32600, 'message' => 'startLedger must be within the ledger range'];

        $this->artisan('selo:index')
            ->expectsOutputToContain('SELO_START_LEDGER')
            ->assertFailed();

        $this->assertSame(0, IndexerCursor::count());
    }
}
