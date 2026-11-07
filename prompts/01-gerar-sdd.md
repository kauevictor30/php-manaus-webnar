# Prompt 1 — Gerar o SDD

> Rode ANTES da aula. Salve os 3 arquivos gerados (requirements.md, design.md, tasks.md) na raiz do projeto.

```
Você é um arquiteto de software sênior especialista em Stellar/Soroban, Laravel e React. Gere o SDD (Spec Driven Development) da dApp abaixo. O design JÁ ESTÁ DECIDIDO: seu trabalho é formalizar, não reinventar. Se algo for impossível na API real do soroban-sdk, registre em "Decisões" e proponha a correção mínima.

# A dApp: Selo — Reputação Soulbound para Comunidades

## Problema
Comunidades de tecnologia não têm registro confiável de quem contribui. Certificado em PDF se falsifica, reputação em Discord some. Queremos badges intransferíveis emitidos on-chain que formam a reputação pública de cada membro.

## Analogia (use-a na documentação)
Endereço G... = passaporte. Badge = carimbo. Contrato = cartório. Admin = chefe do cartório. Emissor = funcionário credenciado. Laravel = arquivo morto (histórico e dados que não vão para a blockchain).

## Atores
- Admin: faz o deploy, credencia emissores, revoga badges.
- Emissor: carimba (emite badges).
- Membro: recebe badges. NÃO assina nada para receber.
- Visitante: consulta perfis e ranking sem carteira.

## Stack
- Contrato: Rust + soroban-sdk, testnet
- Toolchain: Stellar CLI (`stellar contract init | build | deploy | bindings typescript | invoke`). Contract ID salvo como alias `selo` via `stellar contract deploy --alias selo`; identidades no keystore do Stellar CLI
- Frontend: React + TypeScript usando os bindings gerados por `stellar contract bindings typescript`
- Carteira: Varial (antiga Freighter, rebrand em 29/10/2026). Verifique no npm qual SDK de conexão está publicado e funcional hoje (@stellar/freighter-api ou sucessor); prefira o Stellar Wallets Kit se ele suportar a Varial. Não invente nome de pacote.
- Backend: Laravel + soneso/stellar-php-sdk (para decodificar XDR dos eventos)

## Contrato — design fechado

Tipos:
- enum BadgeKind { Participante, Mentor, Organizador, Palestrante }
- pesos: Participante=1, Mentor=3, Organizador=3, Palestrante=5 (função weight(kind) -> u32)
- struct Badge { kind: BadgeKind, event_id: u32, issuer: Address, issued_at: u64 }

Storage (enum DataKey):
- Admin -> Address (instance)
- Issuer(Address) -> bool (persistent)
- Badge(Address, BadgeKind, u32) -> Badge (persistent) — também serve de chave de dedup
- BadgeList(Address) -> Vec<(BadgeKind, u32)> (persistent) — índice para listar os badges do membro
- Reputation(Address) -> u32 (persistent) — contador
- Estender TTL das chaves persistent sempre que forem escritas.

Erros (#[contracterror]):
AlreadyInitialized=1, NotInitialized=2, NotAdmin=3, NotIssuer=4, SelfIssue=5, AlreadyHasBadge=6, BadgeNotFound=7

Funções:
- init(admin): grava admin. Falha com AlreadyInitialized se já existir.
- set_issuer(issuer, enabled: bool): admin.require_auth(). Evento "issuer".
- issue(issuer, member, kind, event_id): issuer.require_auth(); emissor precisa estar ativo; issuer != member; dedup pela chave Badge. Grava Badge, adiciona em BadgeList, soma peso em Reputation. Evento "issued".
- revoke(member, kind, event_id): admin.require_auth(). Apaga a chave Badge, remove o item de BadgeList BUSCANDO PELO VALOR (kind, event_id), nunca por posição, e subtrai o peso de Reputation. Evento "revoked".
- reputation(member) -> u32: leitura, retorna 0 se não houver.
- badges_of(member) -> Vec<Badge>: leitura.

## Regras de negócio
- RN-01: init só uma vez.
- RN-02: badge intransferível. Não existe transferência e o contrato NÃO implementa a interface de token (SEP-41).
- RN-03: 1 badge do mesmo tipo por membro por evento.
- RN-04: emissor não emite para si mesmo.
- RN-05: reputação é contador: +peso no issue, -peso no revoke.
- RN-06: revoke apaga o badge e a chave de dedup, então pode ser reemitido. O histórico fica no Laravel via evento "revoked".
- RN-07: emissor desativado não emite, mas os badges que ele já emitiu continuam válidos.
- RN-08: issue, revoke e set_issuer sempre emitem evento.
- RN-09: event_id é o ID do evento no Laravel. O contrato não valida se ele existe.
- RN-10: a dApp não movimenta fundos.

## Laravel
Tabelas:
- events (id, name, date, location, description)
- members (address unique, nickname null, avatar_url null)
- badge_records (rpc_event_id unique, member_address, kind, event_id, issuer, status issued|revoked, tx_hash, ledger, issued_at, revoked_at null)
- indexer_cursors (contract_id, cursor)

Endpoints:
- GET /api/events
- POST /api/events (protegido por header X-Admin-Token comparado ao .env)
- GET /api/members/{address} (histórico completo, incluindo revogados)
- GET /api/ranking (top 20 por pontos de badges ativos)

Indexador: comando `php artisan selo:index` que lê os eventos do contrato via Stellar RPC getEvents (testnet), decodifica com o stellar-php-sdk e salva o cursor. Idempotente: o `id` do evento retornado pelo RPC é a chave única (rodar 2x não duplica). Primeira execução começa no ledger do deploy (SELO_START_LEDGER no .env), porque o RPC só retém eventos por uma janela limitada. Agendado a cada minuto e rodável manualmente na demo. CORS liberado para a origem do frontend.

## React — 3 telas
1. Perfil (/m/:address): reputação lida DIRETO do contrato pelos bindings e histórico vindo do Laravel, lado a lado, para ensinar on-chain vs off-chain.
2. Ranking (/): vem do Laravel.
3. Emissor (/emitir): conectar carteira, escolher evento, colar endereço G..., escolher tipo, emitir. Estados visíveis: aguardando assinatura → enviando → confirmado (link para stellar.expert testnet) ou erro traduzido a partir do código do contracterror.

## Fora de escopo
Votação, multisig do admin (citar como próxima versão), expiração, emissão em lote, login/SEP-10, edição de perfil, mainnet.

# Entregue 3 arquivos markdown

1. requirements.md: visão, atores, user stories com critérios de aceite em EARS ("QUANDO... O SISTEMA DEVE..."), RNs, fora de escopo.
2. design.md: arquitetura em Mermaid; contrato completo (tipos, storage, erros, assinaturas com pré/pós-condições e evento); diagramas de sequência Mermaid para issue, revoke e indexação; Laravel (migrations, endpoints com request/response); React (telas e estados); seção "Ameaças e defesas" (carimbo indevido, auto-emissão, duplicidade, revoke na posição errada, admin único); seção "Decisões".
3. tasks.md: no máximo 18 tarefas pequenas, cada uma com ID, descrição, arquivos, dependência, AGENTE RESPONSÁVEL (soroban-dev, laravel-dev ou react-dev) e CRITÉRIO DE PRONTO como comando verificável. Ordem: doctor/init → contrato + testes → build/deploy → bindings → Laravel → React → teste ponta a ponta. Marque [PREP] (fazer antes da aula) e [DEMO] (fazer ao vivo). DEMO = apenas a tela do Emissor e a emissão ao vivo.

Não escreva código de implementação, apenas assinaturas e interfaces.
```
