# Selo — dApp de reputação soulbound (Stellar testnet)

Kit do webinar da Comunidade PHP Manaus: construir uma dApp na Stellar com agentes de IA.
Selo = badges intransferíveis emitidos on-chain que formam a reputação pública de membros de comunidade.

Analogia: endereço G... = passaporte · badge = carimbo · contrato = cartório · admin = chefe do cartório · emissor = funcionário credenciado · Laravel = arquivo morto.

## Stack
- Contrato: Rust + soroban-sdk, testnet. Toolchain: **Stellar CLI** (sem Caatinga).
- Backend: Laravel em `/backend` + soneso/stellar-php-sdk (decodifica XDR dos eventos).
- Frontend: React + TypeScript com bindings de `stellar contract bindings typescript`.
- Carteira: Varial (antiga Freighter). Pacote de conexão: verificar no npm antes de instalar.

## Fonte da verdade
- `requirements.md`, `design.md`, `tasks.md` (SDD) na raiz. Gerados por `prompts/01-gerar-sdd.md`.
- Implementação segue `prompts/02-harness.md`: Tech Lead delega a `.claude/agents/` (dione, luiz, felipe; sarah revisa cada fase e não escreve código).
- Visão geral e ordem da aula: `COMO-USAR.md`.

## Regras
- Nada fora do SDD. Não inventar API, flag ou nome de pacote — na dúvida, `--help` ou npm.
- Uma tarefa por vez; após cada uma: rodar critério de pronto, marcar [x], commit `feat(T0X): ...`.
- [PREP] antes da aula; [DEMO] (tela do Emissor + emissão) só ao vivo.
- Mesmo comando falhou 2x do mesmo jeito → parar e mostrar o erro.
- Nunca gravar chave privada / .env em arquivo versionado. Identidades ficam no keystore do Stellar CLI.
- Não instalar nada sem perguntar ao usuário.

## Ambiente (verificado em 2026-11-07)
node 24.12 · npm 11.6 · rustc/cargo 1.99 + wasm32v1-none · stellar CLI 27.1.0 (>= 23 exigido; 22.x quebra invoke; 25.2.0 não compila no rustc 1.99) · php 8.5.4 · composer 2.9.5

## Como rodar
> Caminhos exatos serão confirmados após `stellar contract init selo` e registrados no topo do `tasks.md`.

```bash
# Contrato
stellar contract build
cargo test
stellar contract deploy --wasm <wasm> --source <identidade> --network testnet --alias selo
stellar contract bindings typescript --contract-id selo --network testnet --output-dir <pasta-frontend>
stellar contract invoke --id selo --source <identidade> --network testnet -- reputation --member G...

# Backend
cd backend && composer install && php artisan migrate && php artisan serve
php artisan selo:index        # indexa eventos do contrato (SELO_START_LEDGER no .env)

# Frontend
npm install && npm run dev
```
