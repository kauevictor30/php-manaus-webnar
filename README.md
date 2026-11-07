# Selo — reputação soulbound na Stellar

Selo emite badges **intransferíveis** na Stellar testnet. Eles formam a reputação pública de quem participa de uma comunidade.
É a dApp do webinar da Comunidade PHP Manaus: um contrato Soroban em Rust, um backend Laravel e um frontend React, construídos com agentes de IA.

> Endereço `G...` = passaporte · badge = carimbo · contrato = cartório · admin = chefe do cartório · emissor = funcionário credenciado · Laravel = arquivo morto.

| Peça | Pasta | O que faz |
|---|---|---|
| Contrato (cartório) | `selo/` | Emite e revoga badges e guarda a reputação on-chain |
| Bindings TS | `bindings/selo/` | Cliente TypeScript gerado a partir do contrato |
| Backend (arquivo morto) | `backend/` | Indexa os eventos do contrato e expõe ranking e histórico |
| Frontend (balcão) | `frontend/` | Telas de Ranking, Perfil e Emissor |

**Contrato na testnet:** [`CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q`](https://stellar.expert/explorer/testnet/contract/CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q)

A especificação completa está em `requirements.md`, `design.md` e `tasks.md`.

## Pré-requisitos

- Node >= 20 e npm
- PHP 8.3+ com as extensões `gmp` e `bcmath` (`php -m | grep -E '^(gmp|bcmath)$'`) e Composer
- Para mexer no contrato: Rust com o target `wasm32v1-none` e Stellar CLI >= 23

Para rodar o app não é preciso Rust nem Stellar CLI: o contrato já está publicado na testnet.

## Como rodar (clone limpo)

### 1. Bindings do contrato (antes do frontend)
```bash
cd bindings/selo
npm install && npm run build
cd ../..
```
A pasta `dist/` dos bindings não é versionada. Sem esse passo, o build do frontend quebra.

### 2. Backend Laravel — http://localhost:8000
```bash
cd backend
composer install
cp .env.example .env
php artisan key:generate
touch database/database.sqlite
php artisan migrate
# no .env, preencha SELO_ADMIN_TOKEN com um valor aleatório (ex.: openssl rand -hex 32)
php artisan serve
```
Em outro terminal, dentro de `backend/`:
```bash
php artisan selo:index        # copia os eventos do contrato para o banco (manual)
php artisan schedule:work     # ou: roda o selo:index a cada minuto
```

### 3. Frontend React — http://localhost:5173
```bash
cd frontend
npm install
cp .env.example .env.local
npm run dev
```

Abra `/` (Ranking) e `/m/<endereço G...>` (Perfil). Ranking e Perfil funcionam sem carteira.

## API

| Método | Rota | Observação |
|---|---|---|
| GET | `/api/events` | Lista os eventos da comunidade |
| POST | `/api/events` | Exige o header `X-Admin-Token` igual a `SELO_ADMIN_TOKEN` |
| GET | `/api/members/{address}` | Histórico completo, incluindo revogados, e pontos ativos |
| GET | `/api/ranking` | Top 20 por pontos de badges ativos |

Para criar o evento usado na demo:
```bash
curl -X POST http://localhost:8000/api/events \
  -H "X-Admin-Token: <SELO_ADMIN_TOKEN>" -H "Content-Type: application/json" \
  -d '{"name":"PHP Manaus — Webinar Selo","date":"2026-11-07","location":"Online"}'
```

## Contrato

```bash
cd selo
cargo test                         # testes do contrato
stellar contract build             # gera target/wasm32v1-none/release/selo.wasm
# leitura on-chain (qualquer identidade serve, pois não altera estado):
stellar contract invoke --id CDOLW5XCTGHWGZWIDSBXGIUGSDKQOWILSCU7D6FNHV4NH2IGD73W5B3Q \
  --network testnet --source <sua-identidade> -- reputation --member G...
```

Só o **admin** credencia emissores e revoga badges, e só um **emissor credenciado** emite. Num clone, você usa o contrato publicado só para leitura. Para emitir, publique o seu próprio contrato (veja a T07 do `tasks.md`) e aponte as variáveis `SELO_CONTRACT_ID`, `SELO_START_LEDGER` e `VITE_SELO_CONTRACT_ID` para ele.

| Badge | Peso |
|---|---|
| Participante | 1 |
| Mentor | 3 |
| Organizador | 3 |
| Palestrante | 5 |

## Roteiro da demo (5 passos)

1. **Conectar:** abra `/emitir` e conecte a carteira Varial com a conta emissora credenciada.
2. **Carimbar:** escolha o evento, cole o `G...` de quem está no chat, escolha **Participante** e emita. A tela passa por aguardando assinatura → enviando → confirmado.
3. **Conferir no explorer:** clique no link do stellar.expert e mostre a transação na testnet.
4. **Arquivo morto:** rode `php artisan selo:index` e mostre que o badge entrou no banco.
5. **On-chain = off-chain:** abra `/m/<G...>` e mostre que a reputação lida do contrato bate com a do Laravel. Depois mostre o Ranking.

## Segurança

- Nenhuma chave privada no repositório. As identidades ficam no keystore do Stellar CLI.
- `.env` e `.env.local` não são versionados. Os `.env.example` têm só valores públicos da testnet.
- A dApp não movimenta fundos. É só testnet.
