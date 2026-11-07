<?php

namespace Tests\Feature;

use App\Enums\BadgeKind;
use App\Models\BadgeRecord;
use App\Models\Event;
use App\Models\Member;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Soneso\StellarSDK\Crypto\KeyPair;
use Tests\TestCase;

class MemberApiTest extends TestCase
{
    use RefreshDatabase;

    private int $seq = 0;

    private function record(string $member, string $kind, int $eventId, string $status = 'issued', array $extra = []): BadgeRecord
    {
        $this->seq++;

        return BadgeRecord::create(array_merge([
            'rpc_event_id' => 'rpc-'.$this->seq,
            'member_address' => $member,
            'kind' => $kind,
            'event_id' => $eventId,
            'issuer' => KeyPair::random()->getAccountId(),
            'status' => $status,
            'tx_hash' => str_repeat('a', 64),
            'ledger' => 5611400 + $this->seq,
            'issued_at' => '2026-11-07 19:10:00',
            'revoked_at' => $status === 'revoked' ? '2026-11-07 20:00:00' : null,
        ], $extra));
    }

    public function test_pesos_fixados_iguais_ao_contrato_d06(): void
    {
        $this->assertSame(1, BadgeKind::Participante->weight());
        $this->assertSame(3, BadgeKind::Mentor->weight());
        $this->assertSame(3, BadgeKind::Organizador->weight());
        $this->assertSame(5, BadgeKind::Palestrante->weight());
        $this->assertCount(4, BadgeKind::cases());
    }

    public function test_endereco_invalido_retorna_422(): void
    {
        $valid = KeyPair::random()->getAccountId();
        $badChecksum = substr($valid, 0, 55).($valid[55] === 'A' ? 'B' : 'A');

        foreach (['abc', 'CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q', $badChecksum, strtolower($valid)] as $address) {
            $this->getJson('/api/members/'.$address)
                ->assertUnprocessable()
                ->assertJsonValidationErrors(['address']);
        }
    }

    public function test_endereco_sem_registros_retorna_200_vazio(): void
    {
        $address = KeyPair::random()->getAccountId();

        $this->getJson('/api/members/'.$address)
            ->assertOk()
            ->assertExactJson([
                'address' => $address,
                'nickname' => null,
                'avatar_url' => null,
                'points' => 0,
                'badges' => [],
            ]);
    }

    public function test_perfil_com_historico_revogados_e_pontos_so_dos_ativos(): void
    {
        $address = KeyPair::random()->getAccountId();
        $event = Event::create(['name' => 'Webinar Selo', 'date' => '2026-11-07', 'location' => 'Online']);
        Member::create(['address' => $address, 'nickname' => 'kaue', 'avatar_url' => 'https://exemplo/a.png']);

        $mentor = $this->record($address, 'Mentor', $event->id);
        $this->record($address, 'Participante', $event->id);
        $this->record($address, 'Palestrante', 999, 'revoked'); // revogado e sem evento no Laravel
        $this->record(KeyPair::random()->getAccountId(), 'Palestrante', $event->id); // outro membro

        $response = $this->getJson('/api/members/'.$address)->assertOk();

        $response->assertJson([
            'address' => $address,
            'nickname' => 'kaue',
            'avatar_url' => 'https://exemplo/a.png',
            'points' => 4, // Mentor 3 + Participante 1; o Palestrante revogado não conta
        ]);
        $response->assertJsonCount(3, 'badges');

        $badges = collect($response->json('badges'));
        $revoked = $badges->firstWhere('status', 'revoked');
        $this->assertSame('Palestrante', $revoked['kind']);
        $this->assertSame(5, $revoked['weight']);
        $this->assertNull($revoked['event']);
        $this->assertSame('2026-11-07T20:00:00Z', $revoked['revoked_at']);

        $this->assertSame([
            'kind' => 'Mentor',
            'weight' => 3,
            'event' => ['id' => $event->id, 'name' => 'Webinar Selo'],
            'issuer' => $mentor->issuer,
            'status' => 'issued',
            'tx_hash' => str_repeat('a', 64),
            'ledger' => $mentor->ledger,
            'issued_at' => '2026-11-07T19:10:00Z',
            'revoked_at' => null,
        ], $badges->firstWhere('kind', 'Mentor'));
    }

    public function test_pontos_somam_os_pesos_de_cada_tipo_d06(): void
    {
        $address = KeyPair::random()->getAccountId();
        foreach (BadgeKind::cases() as $i => $kind) {
            $this->record($address, $kind->value, $i + 1);
        }

        $this->getJson('/api/members/'.$address)
            ->assertOk()
            ->assertJsonPath('points', 12); // 1 + 3 + 3 + 5
    }
}
