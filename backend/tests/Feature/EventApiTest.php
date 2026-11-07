<?php

namespace Tests\Feature;

use App\Models\Event;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class EventApiTest extends TestCase
{
    use RefreshDatabase;

    private const TOKEN = 'token-de-teste-bem-aleatorio';

    protected function setUp(): void
    {
        parent::setUp();
        config([
            'selo.admin_token' => self::TOKEN,
            'cors.allowed_origins' => ['http://localhost:5173'],
        ]);
    }

    private function payload(array $override = []): array
    {
        return array_merge([
            'name' => 'PHP Manaus — Webinar Selo',
            'date' => '2026-11-07',
            'location' => 'Online',
            'description' => 'Workshop Stellar',
        ], $override);
    }

    public function test_get_lista_eventos_no_formato_do_design(): void
    {
        Event::create($this->payload());

        $this->getJson('/api/events')
            ->assertOk()
            ->assertExactJson([[
                'id' => 1,
                'name' => 'PHP Manaus — Webinar Selo',
                'date' => '2026-11-07',
                'location' => 'Online',
                'description' => 'Workshop Stellar',
            ]]);
    }

    public function test_get_sem_eventos_retorna_lista_vazia(): void
    {
        $this->getJson('/api/events')->assertOk()->assertExactJson([]);
    }

    public function test_post_com_token_valido_cria_evento_201(): void
    {
        $this->withHeader('X-Admin-Token', self::TOKEN)
            ->postJson('/api/events', $this->payload(['description' => null]))
            ->assertCreated()
            ->assertJson(['name' => 'PHP Manaus — Webinar Selo', 'date' => '2026-11-07', 'description' => null])
            ->assertJsonStructure(['id', 'name', 'date', 'location', 'description']);

        $this->assertDatabaseCount('events', 1);
    }

    public function test_post_sem_token_retorna_401_e_nao_cria(): void
    {
        $this->postJson('/api/events', $this->payload())
            ->assertUnauthorized()
            ->assertExactJson(['message' => 'Unauthorized']);

        $this->assertDatabaseCount('events', 0);
    }

    public function test_post_com_token_errado_retorna_401_e_nao_cria(): void
    {
        $this->withHeader('X-Admin-Token', 'errado')
            ->postJson('/api/events', $this->payload())
            ->assertUnauthorized()
            ->assertExactJson(['message' => 'Unauthorized']);

        $this->assertDatabaseCount('events', 0);
    }

    public function test_post_com_token_vazio_no_env_sempre_401(): void
    {
        config(['selo.admin_token' => '']);

        $this->withHeader('X-Admin-Token', '')
            ->postJson('/api/events', $this->payload())
            ->assertUnauthorized();

        $this->assertDatabaseCount('events', 0);
    }

    public function test_post_invalido_retorna_422(): void
    {
        $this->withHeader('X-Admin-Token', self::TOKEN)
            ->postJson('/api/events', ['name' => '', 'date' => '07/11/2026'])
            ->assertUnprocessable()
            ->assertJsonValidationErrors(['name', 'date', 'location']);

        $this->assertDatabaseCount('events', 0);
    }

    public function test_cors_libera_somente_a_origem_do_frontend(): void
    {
        $this->withHeader('Origin', 'http://localhost:5173')
            ->getJson('/api/events')
            ->assertHeader('Access-Control-Allow-Origin', 'http://localhost:5173');

        // Com uma única origem permitida, o CORS sempre devolve essa origem fixa;
        // o navegador de outra origem bloqueia a resposta. Nunca '*' nem eco da origem.
        $this->withHeader('Origin', 'http://evil.example')
            ->getJson('/api/events')
            ->assertHeader('Access-Control-Allow-Origin', 'http://localhost:5173');
    }
}
