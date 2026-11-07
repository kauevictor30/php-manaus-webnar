# Selo — Kit da aula

## O que tem aqui
- `prompts/01-gerar-sdd.md` → gera requirements.md, design.md e tasks.md
- `prompts/02-harness.md` → manda o Tech Lead implementar com a equipe
- `.claude/agents/` → a equipe: soroban-dev, laravel-dev, react-dev, qa-revisor

## A equipe (analogia para a plateia)
| Agente | Papel no cartório |
|---|---|
| Tech Lead (o prompt 2) | Gerente da obra: lê a planta (SDD) e distribui o serviço |
| soroban-dev | Constrói o cartório (contrato) |
| laravel-dev | Monta o arquivo morto (histórico) |
| react-dev | Faz o balcão de atendimento (telas) |
| qa-revisor | Fiscal: não constrói nada, só reprova o que está errado |

O fiscal ser separado é de propósito: quem escreveu o código não deve aprovar o próprio código.

## Ordem
1. Copie a pasta `.claude/` para a raiz do projeto.
2. Rode o Prompt 1 e salve os 3 arquivos na raiz. LEIA o design.md.
3. Rode o Prompt 2 só até terminar as tarefas [PREP].
4. Garanta fundos na conta emissora (friendbot da testnet).
5. Commit numa branch `final` com tudo funcionando.
6. Ao vivo: volte para a branch das tarefas [DEMO] e deixe o Tech Lead fazer a tela do Emissor.
7. Emita o badge PARTICIPANTE para quem mandar o G... no chat.

## Se der errado ao vivo
`git checkout final` e siga a aula. Ninguém precisa saber.
