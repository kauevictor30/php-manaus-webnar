# Selo — Tarefas

> SDD 3/3 · Executar na ordem, uma por vez. Depois de cada tarefa: rodar o critério de pronto e mostrar a saída, marcar `[x]` e commitar `feat(T0X): descrição`.
> **[PREP]** = antes da aula · **[DEMO]** = ao vivo (só a tela do Emissor e a emissão).

## Caminhos reais
> Confirmado na T02 com o que o `stellar contract init selo --name selo` gerou (CLI 27.1.0, soroban-sdk 27.0.6). Nenhum crate de exemplo extra foi gerado.

| O quê | Caminho |
|---|---|
| Workspace do contrato | `selo/` (`selo/Cargo.toml`, `selo/Cargo.lock`) |
| Crate do contrato | `selo/contracts/selo/` (`src/lib.rs`, `src/test.rs`, `Makefile`) |
| WASM gerado | _(preencher depois da T07)_ |
| Bindings TS | `bindings/selo/` |
| Laravel | `backend/` |
| React | `frontend/` |
| Contract ID / ledger do deploy | _(preencher depois da T07)_ |

---

## Fase 1 — Contrato (o cartório)

- [x] **T01 [PREP] Identidades na testnet**
  - Descrição: criar e financiar as identidades `admin` e `emissor` no keystore do Stellar CLI (`stellar keys generate <nome> --network testnet --fund`). Nenhuma chave em arquivo do repositório.
  - Arquivos: nenhum (keystore do CLI)
  - Depende de: —
  - Agente: **soroban-dev**
  - Pronto: `stellar keys address admin && stellar keys address emissor` imprime dois endereços `G...`

- [x] **T02 [PREP] Workspace do contrato**
  - Descrição: `stellar contract init selo --name selo`; inspecionar a estrutura gerada, remover o exemplo que não for o crate `selo` e preencher a tabela "Caminhos reais".
  - Arquivos: `selo/**`, `tasks.md`
  - Depende de: —
  - Agente: **soroban-dev**
  - Pronto: `cd selo && cargo test` passa (com os testes do template)

- [ ] **T03 [PREP] Tipos, storage, erros e pesos**
  - Descrição: `BadgeKind`, `weight()`, `Badge`, `DataKey`, `Error`, as constantes de TTL e as structs de evento (§2.1 a §2.4). Testes de `weight()`.
  - Arquivos: `selo/contracts/selo/src/lib.rs` (ou módulos ao lado), `src/test.rs`
  - Depende de: T02
  - Agente: **soroban-dev**
  - Pronto: `cd selo && cargo test -p selo weight`

- [ ] **T04 [PREP] `init` e `set_issuer`**
  - Descrição: testes primeiro (feliz, sem auth, `AlreadyInitialized`, `NotInitialized`, evento `issuer`), depois a implementação (§2.5).
  - Arquivos: `selo/contracts/selo/src/lib.rs`, `src/test.rs`
  - Depende de: T03
  - Agente: **soroban-dev**
  - Pronto: `cd selo && cargo test -p selo`

- [ ] **T05 [PREP] `issue`**
  - Descrição: testes primeiro (feliz, sem auth, `NotIssuer`, `SelfIssue`, `AlreadyHasBadge`, emissor desativado mantém os badges antigos, evento `issued`), depois a implementação, com o TTL das 3 chaves.
  - Arquivos: `selo/contracts/selo/src/lib.rs`, `src/test.rs`
  - Depende de: T04
  - Agente: **soroban-dev**
  - Pronto: `cd selo && cargo test -p selo`

- [ ] **T06 [PREP] `revoke`, `reputation`, `badges_of`**
  - Descrição: testes primeiro (emitir 3 e revogar o do **meio**, `BadgeNotFound`, sem auth, revogar e reemitir, evento `revoked`, reputação 0 para quem não tem badge), depois a implementação. A remoção é **por valor** (`first_index_of`).
  - Arquivos: `selo/contracts/selo/src/lib.rs`, `src/test.rs`
  - Depende de: T05
  - Agente: **soroban-dev**
  - Pronto: `cd selo && cargo test -p selo` com todos os testes da §2.6 verdes

- [ ] **T07 [PREP] Build, deploy e inicialização na testnet**
  - Descrição: `stellar ledger latest --network testnet` (anotar como `SELO_START_LEDGER`) → `stellar contract build` → `stellar contract deploy --wasm <wasm> --source admin --network testnet --alias selo` → `init` com o admin → `set_issuer` com o emissor e `true`. Registrar o contract ID, o ledger e o caminho do WASM em "Caminhos reais". Conferir a sintaxe dos argumentos com `stellar contract invoke --id selo --network testnet -- <função> --help`.
  - Arquivos: `tasks.md`
  - Depende de: T06
  - Agente: **soroban-dev**
  - Pronto: `stellar contract invoke --id selo --source admin --network testnet -- reputation --member $(stellar keys address emissor)` retorna `0`, e um 2º `init` retorna o erro `#1` (AlreadyInitialized)

- [ ] **T08 [PREP] Bindings TypeScript**
  - Descrição: `stellar contract bindings typescript --contract-id selo --network testnet --output-dir bindings/selo`, depois instalar e buildar o pacote gerado.
  - Arquivos: `bindings/selo/**`
  - Depende de: T07
  - Agente: **soroban-dev**
  - Pronto: `cd bindings/selo && npm install && npm run build` termina sem erro

> 🔍 **QA Fase 1:** `qa-revisor` revisa o contrato contra o `design.md` §2 e §5. Só avança se ele aprovar.

## Fase 2 — Laravel (o arquivo morto)

- [ ] **T09 [PREP] Projeto Laravel e migrations**
  - Descrição: `composer create-project laravel/laravel backend`, `composer require soneso/stellar-php-sdk`, `php artisan install:api`, as 4 migrations (§3.2), o enum `BadgeKind` com `weight()` e o `.env.example` com as chaves da §3.1 sem segredos.
  - Arquivos: `backend/**`
  - Depende de: T07
  - Agente: **laravel-dev**
  - Pronto: `cd backend && php artisan migrate:fresh --force && php artisan migrate:status`

- [ ] **T10 [PREP] API de eventos + CORS**
  - Descrição: `GET /api/events` e `POST /api/events` com `X-Admin-Token` comparado via `hash_equals` (401/422/201), e CORS restrito a `FRONTEND_URL` (§3.3). Feature tests.
  - Arquivos: `backend/routes/api.php`, `app/Http/**`, `config/cors.php`, `tests/Feature/EventApiTest.php`
  - Depende de: T09
  - Agente: **laravel-dev**
  - Pronto: `cd backend && php artisan test --filter=EventApiTest`

- [ ] **T11 [PREP] Indexador `selo:index`**
  - Descrição: comando da §3.4 (cursor, `SELO_START_LEDGER`, transação por página, `issued`/`revoked`, idempotência D-07) e agendamento a cada minuto. Testes com a resposta do RPC simulada (`issued`, `revoked`, rodar 2x).
  - Arquivos: `backend/app/Console/Commands/IndexSeloEvents.php`, `routes/console.php`, `tests/Feature/IndexerTest.php`
  - Depende de: T09
  - Agente: **laravel-dev**
  - Pronto: `cd backend && php artisan test --filter=IndexerTest && php artisan selo:index && php artisan selo:index` (2ª execução não cria registros novos)

- [ ] **T12 [PREP] Perfil e ranking na API**
  - Descrição: `GET /api/members/{address}` (validação StrKey, histórico com revogados, `points` dos ativos) e `GET /api/ranking` (top 20), conforme §3.3. Feature tests, incluindo o teste que fixa os pesos (D-06).
  - Arquivos: `backend/routes/api.php`, `app/Http/**`, `tests/Feature/MemberApiTest.php`, `tests/Feature/RankingApiTest.php`
  - Depende de: T10, T11
  - Agente: **laravel-dev**
  - Pronto: `cd backend && php artisan test`

> 🔍 **QA Fase 2:** `qa-revisor` revisa o Laravel contra a §3 e a §5.

## Fase 3 — React (o balcão de atendimento)

- [ ] **T13 [PREP] Base do frontend**
  - Descrição: `npm create vite@latest frontend -- --template react-ts`, `react-router-dom`, dependência `file:../bindings/selo`, `@creit.tech/stellar-wallets-kit@2.8.0`, `.env.example` (§4), cliente da API, client do contrato a partir das envs, rotas `/`, `/m/:address` e `/emitir` (placeholder) e o tema de fonte grande e alto contraste.
  - Arquivos: `frontend/**`
  - Depende de: T08, T12
  - Agente: **react-dev**
  - Pronto: `cd frontend && npm run build`

- [ ] **T14 [PREP] Tela Ranking (`/`)**
  - Descrição: lista do `GET /api/ranking` com os estados carregando, vazio, lista e erro (§4.1). Cada linha leva ao perfil.
  - Arquivos: `frontend/src/pages/Ranking.tsx` (+ componentes)
  - Depende de: T13
  - Agente: **react-dev**
  - Pronto: `cd frontend && npm run build`

- [ ] **T15 [PREP] Tela Perfil (`/m/:address`)**
  - Descrição: colunas "on-chain" (`reputation` + `badges_of` pelos bindings, sem carteira) e "off-chain" (`GET /api/members/{address}` com revogados), lado a lado, com validação do endereço e estados independentes por coluna.
  - Arquivos: `frontend/src/pages/Perfil.tsx` (+ componentes)
  - Depende de: T13
  - Agente: **react-dev**
  - Pronto: `cd frontend && npm run build`

> 🔍 **QA Fase 3 (PREP):** `qa-revisor` revisa o Ranking e o Perfil contra a §4 e a §5.

- [ ] **T16 [DEMO] Emissor: conectar a Varial e formulário**
  - Descrição: `StellarWalletsKit.init({ modules: [new VarialModule()], network: Networks.TESTNET })`, botão conectar/desconectar, seleção de evento (`GET /api/events`), campo `G...` com validação, seleção do tipo.
  - Arquivos: `frontend/src/pages/Emitir.tsx`, `frontend/src/wallet.ts`
  - Depende de: T15
  - Agente: **react-dev**
  - Pronto: `cd frontend && npm run build`

- [ ] **T17 [DEMO] Emissor: transação e erros traduzidos**
  - Descrição: `issue` pelos bindings, assinado com `signTransaction` do Kit. Estados aguardando assinatura → enviando → confirmado (link stellar.expert testnet) ou erro traduzido pela tabela §4.2.
  - Arquivos: `frontend/src/pages/Emitir.tsx`, `frontend/src/errors.ts`
  - Depende de: T16
  - Agente: **react-dev**
  - Pronto: `cd frontend && npm run build`

> 🔍 **QA Fase 3 (DEMO):** `qa-revisor` revisa o Emissor contra a §4.1, a §4.2 e a §5.

- [ ] **T18 [DEMO] Ponta a ponta ao vivo**
  - Descrição: a Varial conecta como `emissor` → emite PARTICIPANTE para um `G...` do chat → link no explorer → `php artisan selo:index` → o badge aparece no Perfil e no Ranking.
  - Arquivos: `README.md` (contract ID e roteiro da demo em 5 passos)
  - Depende de: T17
  - Agente: **react-dev**
  - Pronto: a reputação on-chain bate com a do Laravel:
    `stellar contract invoke --id selo --source admin --network testnet -- reputation --member G...` == `curl -s localhost:8000/api/members/G... | jq .points`
