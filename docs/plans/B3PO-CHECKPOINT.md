# B-3PO — checkpoint de continuidade

Fonte de direção: o documento mestre *B-3PO BOT PLATFORM — goal único e definitivo* (mantido pelo proprietário).
Este arquivo é o checkpoint operacional exigido pela seção 11 daquele documento: estado real, evidência e próxima tarefa.

## Onda e fase atuais

- **Onda 1 — concluída** (validada por CI): matriz de capacidades, `session/resume` do DeepSeek, orçamento de tokens com gate, fan-out e ledger do `run`.
- **Onda 2 — em andamento**: O2-A **concluída**; O2-B com gate/contabilidade prontos (reserva analisada em
  `o2b-reservation.md`); O2-D completa no host, shims bloqueados em credenciais; O2-C bloqueada em credenciais;
  O2-E com auditoria feita (`o2e-ptbr-audit.md`) e a fundação de i18n como próxima entrega.
- **Identidade B-3PO** — iniciada no host: mascote, cargo e equipe modelados e validados.

## Entregas concluídas e evidência

| PR | Entrega | Estado |
|---|---|---|
| #1 | matriz de capacidades (adapter + memória própria no catálogo) | mergeado, ubuntu verde |
| #2 | `session/resume` para harness sem `loadSession` | mergeado, ubuntu verde |
| #3 | orçamento diário de tokens com gate | mergeado, ubuntu verde |
| #5 | contrato do `run` | mergeado (docs) |
| #6 | `FORK.md` (o que difere, como verificar, como compilar) | mergeado (docs) |
| #23 | análise da reserva atômica de O2-B | mergeado (docs) |
| #25 | auditoria de strings de O2-E | mergeado (docs) |
| #26 | prazo vencido lido como `expired` no status | mergeado, CI do fork verde |
| #7 | seams de implementação do `run` | mergeado (docs) |
| #8 | `startRun` + ledger em `kv` | mergeado, ubuntu verde |
| #9 | join determinístico das respostas | mergeado, ubuntu verde |
| #10 | selo de aprovação humana (**O2-A**) | mergeado, ubuntu verde |
| #20 | selo vinculado ao hash do payload + expiração em 24 h | mergeado, ubuntu verde |
| #21 | aprovação single-flight (dois aprovadores simultâneos) | mergeado, ubuntu verde |
| #22 | estados `revoked` e `executed`, com motivo | mergeado, ubuntu verde |
| #11 | teto de gasto em dinheiro (**O2-B**) | mergeado, ubuntu verde |
| #13 | plano das pontes de chat | mergeado (docs) |
| #14 | uso por tipo de token + preço de cache | mergeado, ubuntu verde |
| #15 | `reply_webhook` assinado (primeira metade de **O2-D**) | mergeado, ubuntu verde |
| #16 | `avatarId`, `roleTitle`, `team` no host (identidade) | mergeado, ubuntu verde |
| #12 | `tools/import-skills.py` (47 skills instaladas nesta máquina) | mergeado (tooling) |

Nenhum PR aberto. `dev` é a branch padrão do fork e está em `61621e94`.

## Testes e CI

- Espelho privado `DZ23-LTDA/codync-ci` roda, por push: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` e `cargo nextest --profile ci` em ubuntu e windows. `network_e2e` é pulado no windows (não alcança pares no runner).
- Suíte do host: ~834 testes; todos os PRs acima têm ubuntu verde no commit final.
- **Nunca** foi declarado verde sem execução; windows fica pendente em vários PRs porque leva 30–40 min no runner.

## Pendências e bloqueios

- **O2-C (conectores sociais/vendas)**: `BLOCKED_BY_EXTERNAL_DEPENDENCY` — exige chaves OAuth/Composio do proprietário.
- **O2-A** está completa: selo, hash do payload, expiração, single-flight, `revoked`/`executed` e proteção contra replay
  (teste unitário no módulo `chat::run`).
- **Onda 1 e O2-D (pontes)**: falta o endpoint de entrada (`/hooks/bots/{secret}`) e os shims de Slack/Discord/Telegram.
- **O2-E (pt-BR)**: não iniciada; exige os quatro clientes e quebra a regra de paridade se feita isolada.
- **iOS/Android**: `BLOCKED_BY_EXTERNAL_DEPENDENCY` — este ambiente é Windows, sem Xcode nem Android SDK 37.
- **GitHub Actions do fork**: workflows listados mas sem disparo em PR; verificação segue no espelho.
- **Toolchain local**: sem MSVC (`link.exe`), então não há build/test local; `cargo fmt` funciona.
- **Marca**: "B-3PO" é decisão do projeto, não liberação jurídica; busca de anterioridade (INPI) e revisão de PI continuam necessárias antes de lançamento público.

## Próxima tarefa concreta

**O2-D, segunda metade**: `POST /hooks/bots/{secret}`, validando o segredo por bot guardado no `kv`, corpo `{text, from}` limitado, mensagem enfileirada no ator com origem `bridge` (nunca como o usuário), e teste com host de fixture. Depois disso, os shims de Slack/Discord/Telegram.

## Riscos e decisões pendentes

- Um endpoint público de entrada é superfície de ataque: segredo longo, corpo limitado, nenhuma ferramenta de escrita liberada por esse caminho, e o mesmo gate de orçamento/selo dos outros turnos.
- Laço bot→canal→bot precisa de marca de origem e limite por minuto.
- A integração dos 13 mascotes (assets + galeria no desktop) depende de decisão sobre empacotar os PNGs no repositório.

## Verificação (atualizado)

- O **CI do fork roda de verdade** desde que o proprietário habilitou Actions: workflow `codync-host`, em ubuntu, macOS e
  windows, disparado em PR. O repositório espelho `codync-ci` foi **arquivado** (a exclusão exige o escopo
  `delete_repo`, que o token local não tem).
- **Toolchain local funcionando** (MSVC Build Tools 14.44 + Windows SDK 10.0.26100): `cargo build`, `cargo clippy
  --workspace --all-targets -D warnings` e testes por módulo rodam nesta máquina. Para linkar, exportar `LIB`/`INCLUDE`
  do MSVC + SDK e apontar `TMP`/`TEMP` para dentro do workspace (o linker não escreve no `%TEMP%` do sistema).
- Suíte completa local tem três falhas **ambientais** conhecidas: symlink sem privilégio (precisa de Developer Mode),
  arquivo em uso por `TMP` compartilhado, e um teste que assume o `%TEMP%` padrão.

## Próximas tarefas concretas

1. O2-E: fundação de i18n no desktop (catálogo tipado, `t()`, teste de cobertura) e extração de diálogos/erros/estados
   vazios — sem mudança visual ainda.
2. O2-B: reconciliação com custo confirmado do provedor quando houver fonte confiável; hoje o custo é estimado.
3. Identidade B-3PO (fase B): depende da decisão do proprietário sobre versionar os 13 PNGs.
4. O2-C e shims de O2-D: dependem de credenciais do proprietário (`BLOCKED_BY_EXTERNAL_DEPENDENCY`).
