# Selo — Requisitos

> SDD 1/3 · Reputação soulbound para comunidades · Stellar testnet

## 1. Visão

Comunidades de tecnologia não têm registro confiável de quem contribui: certificado em PDF se falsifica e reputação em Discord some. O **Selo** emite badges **intransferíveis** on-chain que formam a reputação pública de cada membro.

**Analogia (usada em toda a documentação):**

| Mundo real | Selo |
|---|---|
| Passaporte | Endereço Stellar `G...` do membro |
| Carimbo | Badge |
| Cartório | Contrato Soroban |
| Chefe do cartório | Admin |
| Funcionário credenciado | Emissor |
| Arquivo morto | Backend Laravel (histórico e dados que não vão para a blockchain) |

## 2. Atores

| Ator | O que faz | Assina transação? |
|---|---|---|
| Admin | Faz o deploy, credencia/descredencia emissores, revoga badges | Sim |
| Emissor | Carimba (emite badges) | Sim |
| Membro | Recebe badges | **Não** |
| Visitante | Consulta perfis e ranking | Não (sem carteira) |

## 3. User stories e critérios de aceite (EARS)

### US-01 — Inicializar o cartório (Admin)
Como admin, quero inicializar o contrato com meu endereço para ser o único chefe do cartório.
- QUANDO `init(admin)` for chamado pela primeira vez, O SISTEMA DEVE gravar `admin` como administrador.
- QUANDO `init` for chamado com admin já gravado, O SISTEMA DEVE falhar com `AlreadyInitialized` (1).
- QUANDO qualquer função administrativa for chamada antes do `init`, O SISTEMA DEVE falhar com `NotInitialized` (2).

### US-02 — Credenciar emissor (Admin)
Como admin, quero credenciar e descredenciar emissores para controlar quem pode carimbar.
- QUANDO o admin chamar `set_issuer(issuer, true)` com sua assinatura, O SISTEMA DEVE marcar o emissor como ativo e emitir o evento `issuer`.
- QUANDO o admin chamar `set_issuer(issuer, false)`, O SISTEMA DEVE marcar o emissor como inativo e emitir o evento `issuer`.
- QUANDO `set_issuer` for chamado sem a assinatura do admin, O SISTEMA DEVE rejeitar a transação (falha de auth).

### US-03 — Emitir badge (Emissor)
Como emissor, quero carimbar o passaporte de um membro para registrar sua contribuição.
- QUANDO um emissor ativo chamar `issue(issuer, member, kind, event_id)` com sua assinatura, O SISTEMA DEVE gravar o badge, adicioná-lo à lista do membro, somar `weight(kind)` à reputação e emitir o evento `issued`.
- QUANDO o emissor não estiver ativo, O SISTEMA DEVE falhar com `NotIssuer` (4).
- QUANDO `issuer == member`, O SISTEMA DEVE falhar com `SelfIssue` (5).
- QUANDO o membro já tiver badge do mesmo `kind` no mesmo `event_id`, O SISTEMA DEVE falhar com `AlreadyHasBadge` (6).
- QUANDO `issue` for chamado sem a assinatura do `issuer`, O SISTEMA DEVE rejeitar a transação.
- O SISTEMA NÃO DEVE exigir assinatura do membro.

### US-04 — Revogar badge (Admin)
Como admin, quero revogar um badge emitido por engano.
- QUANDO o admin chamar `revoke(member, kind, event_id)` com sua assinatura, O SISTEMA DEVE apagar o badge, removê-lo da lista do membro **buscando pelo valor `(kind, event_id)`**, subtrair `weight(kind)` da reputação e emitir o evento `revoked`.
- QUANDO o badge não existir, O SISTEMA DEVE falhar com `BadgeNotFound` (7).
- QUANDO `revoke` for chamado sem a assinatura do admin, O SISTEMA DEVE rejeitar a transação.
- QUANDO um badge for revogado, O SISTEMA DEVE permitir que o mesmo badge seja emitido novamente.

### US-05 — Consultar reputação on-chain (Visitante)
- QUANDO `reputation(member)` for chamado, O SISTEMA DEVE retornar o contador do membro, ou `0` se não houver.
- QUANDO `badges_of(member)` for chamado, O SISTEMA DEVE retornar a lista de `Badge` ativos do membro (vazia se não houver).

### US-06 — Indexar o histórico (Laravel)
Como mantenedor, quero que o arquivo morto copie os eventos do cartório.
- QUANDO `php artisan selo:index` rodar, O SISTEMA DEVE ler os eventos do contrato via Stellar RPC `getEvents` a partir do último cursor salvo e gravar `issued`/`revoked` em `badge_records`.
- QUANDO não houver cursor salvo, O SISTEMA DEVE começar no ledger `SELO_START_LEDGER` do `.env`.
- QUANDO o indexador rodar 2 vezes sobre os mesmos eventos, O SISTEMA NÃO DEVE duplicar registros (chave única = `id` do evento RPC).
- QUANDO um evento `revoked` for indexado, O SISTEMA DEVE marcar o registro emitido correspondente como `revoked` com `revoked_at`, mantendo-o no histórico.
- O SISTEMA DEVE agendar `selo:index` a cada minuto.

### US-07 — Gerenciar eventos da comunidade (Admin, off-chain)
- QUANDO `GET /api/events` for chamado, O SISTEMA DEVE listar os eventos.
- QUANDO `POST /api/events` for chamado com header `X-Admin-Token` igual ao do `.env`, O SISTEMA DEVE criar o evento e retornar 201.
- QUANDO o header faltar ou for diferente, O SISTEMA DEVE retornar 401 sem criar nada.

### US-08 — Ver perfil (Visitante)
- QUANDO o visitante abrir `/m/:address`, O SISTEMA DEVE mostrar lado a lado a reputação **lida direto do contrato** (rótulo "on-chain") e o histórico vindo do Laravel, incluindo revogados (rótulo "off-chain").
- QUANDO o endereço for inválido, O SISTEMA DEVE mostrar mensagem de erro sem chamar contrato nem API.

### US-09 — Ver ranking (Visitante)
- QUANDO o visitante abrir `/`, O SISTEMA DEVE mostrar o top 20 de membros por pontos de badges **ativos**, vindo do Laravel.

### US-10 — Emitir pela tela (Emissor) — [DEMO]
- QUANDO o emissor abrir `/emitir`, O SISTEMA DEVE permitir conectar a carteira Varial, escolher evento, colar endereço `G...` e escolher o tipo.
- QUANDO o endereço colado for inválido, O SISTEMA DEVE bloquear o envio antes de pedir assinatura.
- QUANDO o emissor enviar, O SISTEMA DEVE exibir os estados: **aguardando assinatura → enviando → confirmado** (com link para stellar.expert testnet) **ou erro**.
- QUANDO o contrato retornar um `contracterror`, O SISTEMA DEVE exibir a mensagem traduzida para português a partir do código.

## 4. Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | `init` só uma vez. |
| RN-02 | Badge intransferível. Não existe transferência e o contrato **não** implementa a interface de token (SEP-41). |
| RN-03 | 1 badge do mesmo tipo por membro por evento. |
| RN-04 | Emissor não emite para si mesmo. |
| RN-05 | Reputação é contador: +peso no `issue`, −peso no `revoke`. Pesos: Participante=1, Mentor=3, Organizador=3, Palestrante=5. |
| RN-06 | `revoke` apaga o badge e a chave de dedup, então ele pode ser reemitido. O histórico fica no Laravel via evento `revoked`. |
| RN-07 | Emissor desativado não emite, mas os badges que ele já emitiu continuam válidos. |
| RN-08 | `issue`, `revoke` e `set_issuer` sempre emitem evento. |
| RN-09 | `event_id` é o ID do evento no Laravel. O contrato não valida se ele existe. |
| RN-10 | A dApp não movimenta fundos. |

## 5. Requisitos não funcionais

- Rede: Stellar **testnet** apenas.
- Toolchain do contrato: Stellar CLI (>= 23; testado com 27.1.0).
- Nenhuma chave privada em arquivo versionado; identidades no keystore do Stellar CLI; segredos do Laravel só no `.env`.
- Interface legível em tela compartilhada (fonte grande, contraste alto).

## 6. Fora de escopo

Votação · multisig do admin (próxima versão) · expiração de badge · emissão em lote · login/SEP-10 · edição de perfil · mainnet.
