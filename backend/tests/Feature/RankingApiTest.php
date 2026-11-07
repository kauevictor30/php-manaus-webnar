<?php

namespace Tests\Feature;

use App\Models\BadgeRecord;
use App\Models\Member;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Soneso\StellarSDK\Crypto\KeyPair;
use Tests\TestCase;

class RankingApiTest extends TestCase
{
    use RefreshDatabase;

    private int $seq = 0;

    private function record(string $member, string $kind, int $eventId = 1, string $status = 'issued'): void
    {
        $this->seq++;
        BadgeRecord::create([
            'rpc_event_id' => 'rpc-'.$this->seq,
            'member_address' => $member,
            'kind' => $kind,
            'event_id' => $eventId,
            'issuer' => str_repeat('G', 56),
            'status' => $status,
            'tx_hash' => str_repeat('b', 64),
            'ledger' => 5611400 + $this->seq,
            'issued_at' => now(),
            'revoked_at' => $status === 'revoked' ? now() : null,
        ]);
    }

    public function test_ranking_vazio(): void
    {
        $this->getJson('/api/ranking')->assertOk()->assertExactJson([]);
    }

    public function test_ranking_ordena_por_pontos_de_badges_ativos(): void
    {
        $ana = KeyPair::random()->getAccountId();
        $bia = KeyPair::random()->getAccountId();
        $caio = KeyPair::random()->getAccountId();
        Member::create(['address' => $bia, 'nickname' => 'bia']);

        // ana: Palestrante(5) + Mentor(3) = 8 · Palestrante revogado não conta
        $this->record($ana, 'Palestrante', 1);
        $this->record($ana, 'Mentor', 1);
        $this->record($ana, 'Palestrante', 2, 'revoked');
        // bia: Organizador(3) + Participante(1) = 4
        $this->record($bia, 'Organizador', 1);
        $this->record($bia, 'Participante', 1);
        // caio: só revogado → fora do ranking
        $this->record($caio, 'Palestrante', 1, 'revoked');

        $this->getJson('/api/ranking')
            ->assertOk()
            ->assertExactJson([
                ['position' => 1, 'address' => $ana, 'nickname' => null, 'points' => 8, 'badges' => 2],
                ['position' => 2, 'address' => $bia, 'nickname' => 'bia', 'points' => 4, 'badges' => 2],
            ]);
    }

    public function test_ranking_limita_ao_top_20(): void
    {
        for ($i = 0; $i < 25; $i++) {
            $address = KeyPair::random()->getAccountId();
            for ($j = 0; $j <= $i; $j++) {
                $this->record($address, 'Participante', $j + 1);
            }
        }

        $response = $this->getJson('/api/ranking')->assertOk()->assertJsonCount(20);

        $this->assertSame(25, $response->json('0.points'));
        $this->assertSame(1, $response->json('0.position'));
        $this->assertSame(6, $response->json('19.points'));
        $this->assertSame(20, $response->json('19.position'));
    }
}
