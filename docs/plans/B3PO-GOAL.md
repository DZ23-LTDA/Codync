# B-3PO BOT PLATFORM — GOAL v3.2 (único, finito, vinculante)

Este é o **único goal ativo**. Ele substitui integralmente o goal v1, o v3 e o v3.1.
Nada é apagado: o histórico vira a seção 9 e o backlog a seção 20.
**Conclusão = Release 1 publicado (seção 18). Depois disso, o agente para.**

Arquivo canônico: `docs/plans/B3PO-GOAL.md`, versionado no próprio repositório.

---

## 1. MISSÃO E ESCOPO

Um bot que roda na máquina do usuário, com o agente que ele já paga, comandado do celular,
onde toda ação sensível passa por aprovação humana — e onde o resultado é visível e provado.

**Não é** — e está proibido empurrar para lá — um cliente de chat, nem substituto de Claude
Desktop, Codex ou ChatGPT. O B-3PO nunca chama API de modelo com o login do usuário.
Qualquer tarefa nessa direção é recusada citando esta seção.

Vantagem a provar, medível: **agente na máquina + visualização real do projeto + aprovação
do celular.**

## 2. PAPEL DO AGENTE E O QUE CONTA COMO CONCLUSÃO

Atue como engenheiro principal: Rust, TypeScript/React/Electron, Swift, Kotlin, backend,
segurança, QA, release e UX.

Siga sempre: **AUDITAR → ENTENDER → PLANEJAR → IMPLEMENTAR → TESTAR → INSPECIONAR →
CORRIGIR → RETESTAR → DOCUMENTAR → ENTREGAR.**

Não encerre por: compila · teste unitário passa · CI verde · endpoint responde · tela
desenhada · documento atualizado. Encerre quando o comportamento pedido foi **demonstrado**.

## 3. ESTADO REAL (medido em 31/10, 21:28)

| | |
|---|---|
| `dev` | `b5916793` (PR #42 mergeado 21:22) |
| PRs abertos | **1** — #43 `fix/corrupt-db-message` (aberto 21:24) |
| Mergeados em 31/10 | **38** — e **nenhuma release** |
| CI do host (3 sistemas) | verde no run **41252692797** |
| Release desktop (3 sistemas) | verde no run **41254036197**; `codync-macos-unsigned` (299 MB) |
| String fora da tradução | **271 medido, mas é piso** (ver 3.1) |
| Mascotes | 13 ids no catálogo; **0 com arte** no repo; pacote do dono entregue |
| Toolchain local | MSVC 14.44 + SDK 10.0.26100; build e clippy OK |
| App instalado | `Programs\codync-desktop` v2.33.2 — binário, não é o código |

### 3.1 Quatro achados de auditoria, verificados

1. **`tools/l10n-unwrapped.mjs` tem ponto cego.** Linha 16: `if (line.includes("tr(")) continue;`
   pula a linha inteira, escondendo `title`, `placeholder` e `aria-label` na mesma linha de um
   `tr()`. **O número 271 é piso, não total.** Corrigir antes de usar qualquer número como gate.
2. **Release gate é do upstream.** `tools/release-gate.py` L29–30: `REPO = "leepokai/Codync"`,
   `TAP = "leepokai/homebrew-codync"`; `.github/workflows/release-gate.yml` L25:
   `if: github.repository == 'leepokai/Codync'`. No fork o job nunca roda — mas execução manual
   faria `git clone` + push no **tap Homebrew de terceiro**.
3. **`dev` sem proteção** (`protected: false`).
4. **`docs/plans/preview-window.md` é untracked** e pertence a outra sessão: nunca staged,
   nunca tratado como especificação oficial até ser commitado por decisão do dono.

## 4. REGRAS INEGOCIÁVEIS

**Sem funcionalidade falsa.** Proibido: botão sem ação, formulário sem persistência, tela vazia
como entrega, integração simulada, resultado hardcoded como real, resposta fictícia de agente,
indicador de sucesso sem operação. Mock só em teste isolado e identificado.

**Sem esconder problema.** Proibido: remover teste para ficar verde, desligar verificação de
segurança, engolir exceção, reduzir permissão para contornar defeito, classificar falha real
como ambiental sem evidência. Corrigir a causa raiz.

**Preservar o que funciona.** Antes de alterar: inspecionar, achar consumidores, entender
contrato e dados persistidos, verificar compatibilidade com o upstream. Não reescrever módulo
sem necessidade demonstrada.

**Dependência externa** → exatamente `BLOCKED_BY_EXTERNAL_DEPENDENCY`, com: o que falta, por que
é necessário, o que foi implementado, o que foi testado sem ela, o que não pôde ser validado e
qual ação do dono libera.

**Ação perigosa** exige autorização explícita: excluir dados, migração irreversível, apagar
repositório, mudar permissão crítica, release público, operação financeira real, envio externo
em nome do usuário. O resto é autônomo.

## 5. RECONCILIAR ANTES DE AGIR

Toda sessão começa por auditoria — o SIA de um checkpoint nunca é prova.

1. `git fetch`; confirmar `origin/dev`; comparar com o checkpoint.
2. Listar PRs abertos, merges recentes, runs de CI, arquivos modificados e untracked.
3. Verificar se o commit de referência é ancestral do HEAD; identificar o que chegou depois.
4. Se divergir, reconciliar a documentação **antes** de tarefa dependente.
5. Nunca reset destrutivo; nunca descartar alteração do dono.
6. Registrar o SHA efetivamente usado.

Ao terminar, `docs/plans/B3PO-CHECKPOINT.md` atualizado: data UTC, branch, SHA completo, último
PR, PRs abertos, CI real, estado por onda, feito/em curso/pendente, dependências, testes
executados e não executados, evidências, **próxima tarefa concreta**.
Sem versões contraditórias dentro do mesmo documento.

## 6. ARQUITETURA — O HOST É DONO DA LÓGICA

- **Host (Rust)**: bots, processos de agente, sessões, execuções, orquestração, roteamento,
  memória, MCP, conectores, rotinas, aprovações, limites, persistência, API, estado
  compartilhado. É o dono da regra de negócio.
- **Desktop (Electron/React/TS)**, **iOS (SwiftUI)**, **Android (Kotlin/Compose)**, **TUI**:
  apresentam estado do host e enviam intenções. Nunca uma segunda cópia da regra.
- Serviços complementares no repo (cloud, relay, private-relay, web, packaging, screen
  helpers): auditar cada um antes de mexer; não assumir que estão funcionando no fork.
- Mudança de UI em um cliente exige a correspondente nos outros, na mesma entrega.

## 7. BOTS E EQUIPES

Sem número fixo de bots ou equipes. O usuário define nome, avatar, identidade, cargo, equipe,
instruções, harness, modelo, projeto, diretório, permissões, memória, limites e política de
aprovação. A plataforma sugere presets; nunca impõe.

Cargo é metadado operacional: **nunca** concede autorização.

Isolamento obrigatório e testado: um bot não lê diretório de outro, não vê memória alheia, não
reusa credencial de outro, não chama ferramenta fora do escopo.

## 8. PROVEDORES, HARNESS E EXECUÇÃO

Arquitetura extensível, sem acoplar a um provedor. Contratos claros para AgentProvider,
HarnessAdapter, Chat/Embedding/Transcription/TTS/Vision/Reranking/Tool providers.

Catálogo por harness: id, nome, versão, instalação, compatibilidade, capacidades, sessões,
resume, streaming, ferramentas, MCP, memória, limitações, credencial exigida, saúde.
**Nunca declarar capacidade sem teste ou evidência.**

Roteamento: seleção explícita, por capacidade, fallback controlado, timeout, retry limitado,
circuit breaker, registro de execução, orçamento. Nunca trocar de provedor em silêncio em
operação sensível. Sessão: identidade, contexto, resume quando suportado, estado, histórico,
erro, cancelamento, origem. **Nunca fingir continuidade quando o harness não suporta.**

## 9. CONCLUÍDO — NÃO REABRIR

**Onda 1** — matriz de 30 harnesses; adapter e memória própria no catálogo; `session/resume`
para harness sem `loadSession`; orçamento de tokens com gate; fan-out; contrato do `run`;
`startRun`; ledger; join determinístico.

**O2-A** — selo, hash do payload, expiração 24 h, single-flight, `revoked`/`executed`,
anti-replay, motivo. PRs #10, #20, #21, #22, #24.

**O2-B** — teto em dinheiro por bot/dia, gate, uso por tipo de token, preço de cache, resumo de
uso. PRs #11, #14. Reserva atômica **analisada e deferida** com justificativa em
`docs/plans/o2b-reservation.md`.

**O2-D no host** — `POST /hooks/bots/{secret}` (#18), saída HMAC (#15), rate limit 30/min (#19),
e2e 401/413/200.

**Identidade no host** — `avatarId`, `roleTitle`, `team` (#16); 13 ids em `avatar.rs`.

**Mascotes (pipeline)** — `assets/mascots/` + `NOTICE` + `tools/mascot-assets.mjs` (#38).

**Distribuição** — macOS sem certificado quando `DESKTOP_MACOS_SIGNING` ausente (#41).

**CI/higiene** — windows do host consertado (#39); guards de triage e assinatura (#28, #31);
phrase dump removido (#37); checkpoint corrigido (#40, #42).

**i18n** — fundação compartilhada, `pt.json` com 210 frases, `l10n-coverage.mjs` (#30, #32,
#33, #34).

Revalidar quando uma alteração tocar esses pontos. Não reabrir sem defeito comprovado.

## 10. ONDA 2 — O QUE AINDA FALTA

**O2-A**: validar todos os caminhos sensíveis (não só os testes do módulo).

**O2-B**: separar estimado / informado pelo provedor / confirmado / reservado / consumido.
**Nunca apresentar estimativa como confirmado.** Reconciliação quando houver fonte confiável.

**O2-C**: arquitetura, contratos, testes isolados e documentação que não exijam credencial.
Credencial ausente → `BLOCKED_BY_EXTERNAL_DEPENDENCY`. Nunca dizer "conector pronto" por
manifesto ou UI.

**O2-D**: shims Slack/Discord/Telegram, cada um validado por si. Segredo longo, armazenamento
seguro, redaction, validação de payload, limite de tamanho, rate limit, HMAC, anti-replay,
idempotência, origem, proteção de laço bot→canal→bot, permissões, orçamento, auditoria.
**Mensagem de bridge nunca é tratada como mensagem autenticada do dono.**

**O2-E**: ver seção 11.

## 11. i18n — O SCANNER PRIMEIRO

**P0**: corrigir `tools/l10n-unwrapped.mjs` (ponto cego da linha 16) e cobrir com testes:
texto JSX direto, `placeholder`, `title`, `aria-label`, multilinha, template literal,
condicional, componente composto, linha com `tr()` **e** literal, string construída
dinamicamente, e texto que não deve ser traduzido.
**Recontar depois da correção** — o número novo é a base; o 271 não serve.

Depois, envolver as strings em `tr()` por lotes temáticos, com entrada em `pt.json`.
**Regra permanente**: toda string nova nasce com `tr()` no mesmo PR.
Aceite: zero visível não traduzido no escopo liberado, fallback testado, inspeção visual por
tela, formatação de data/hora/número em `pt-BR`.
Relatório separa: encontradas · traduzíveis · cobertas · não traduzidas · excluídas com
justificativa · telas inspecionadas · clientes cobertos.
**Nunca apresentar cobertura de `tr()` como cobertura da interface.**

## 12. MASCOTES E IDENTIDADE

**Dois modos no `mascot-assets.mjs`**: desenvolvimento (arte ausente é aviso, fallback aparece)
e release (exige as 13 artes válidas, valida PNG, dimensão 256–512, nome, integridade e
distribuição para todos os clientes; falha se faltar).

**Arte entregue pelo dono** em `C:\Users\lmpra\Downloads\dz23-bot-avatar-engine-completo.zip`:
13 PNGs 512×512 com fundo transparente, ids idênticos aos de `avatar.rs`, dentro do contrato.
- Integrar **só a arte**: copiar para `assets/mascots/<id>.png` e rodar
  `node tools/mascot-assets.mjs --derive`.
- **Não importar** `manifest.json`, `types/avatar.ts`, `BotAvatar.tsx`, `BotTeam.tsx`: a fonte
  dos ids é `avatar.rs`; o renderer é `Avatar.tsx` (desktop) e `CharacterAvatar` (iOS/Android).
  Dois renderers quebram a paridade.
- O valor a portar são os keyframes dos 8 estados (o `avatar.css` do repo não tem nenhum) — e o
  **mapa status→8 estados vive no host**, não no componente.
- `demo/index.html` é referência do dono; não entra no repo.
- **Antes de commitar**: decisão de elenco **A** (uso interno, mantém) ou **B** (elenco
  próprio). O repositório é **público** — commitar arte de pessoas reais é publicá-la.
- Não renomear ids sem decisão do dono.

**Logo** recebido é arte promocional (1536×1024, com wordmark): não é ícone. Derivar marca
quadrada sem wordmark, 1024 iOS **sem alfa**, `.ico`/`.icns`, PNGs 16–512 e **ícone de bandeja
monocromático**. Remover o glifo "X" nos ícones de produto. Nome, cargo e equipe são
configuração do usuário, nunca dos assets.

## 13. SEGURANÇA E GOVERNANÇA

Aprovação humana **no backend**, nunca só na UI. Operação sensível = escrita externa,
publicação, envio em nome do usuário, alteração destrutiva, comando perigoso, recurso
protegido, operação financeira, mudança de permissão, ferramenta com efeito externo.

Selo valida: aprovador, hash do payload, expiração, escopo, bot, ferramenta, parâmetros,
estado, revogação, single-flight, idempotência, replay, execução duplicada, concorrência,
auditoria. Mudar parâmetro depois da aprovação invalida o selo.

Threat model conforme superfície: access control, IDOR, escalada, injeção, XSS, CSRF, SSRF,
path traversal, upload, força bruta, sessão/JWT, CORS, replay, webhook, exposição de
credencial, vazamento, supply chain, dependência vulnerável, log inseguro.

Credencial nunca em frontend, código, git, doc pública, log, URL, erro ou telemetria sem
redaction. Vault existente. Menor privilégio. **Nunca extrair cookie ou sessão de app de
terceiro por mecanismo não autorizado.**

## 14. ORÇAMENTO E FINOPS

Por bot: limite diário de tokens, limite financeiro diário, limite por execução, por provedor,
concorrência, política de fallback e de interrupção.
Registrar: tokens de entrada/saída/cache, provedor, modelo, moeda, preço, fonte do preço,
estimado, confirmado, reserva, consumo, falha, ajuste.
Nenhuma execução continua em silêncio depois de limite obrigatório.
Testar: duas execuções simultâneas, duas reservas, falha no meio, cancelamento, timeout, retry,
provedor sem custo, preço alterado, limite atingido, reconciliação.

## 15. RELEASE — FORK ≠ UPSTREAM

`release-gate.py` aponta para `leepokai/Codync` e `leepokai/homebrew-codync`; o workflow só roda
com `github.repository == 'leepokai/Codync'`. Portanto:
1. **Não executar** o release gate do fork como está.
2. Adicionar guarda que aborta quando `REPO != DZ23-LTDA/Codync`.
3. Nunca usar token do tap do upstream; nunca escrever no repositório original.
4. Criar pipeline próprio do fork; até existir, publicar **só artefatos locais** (Release
   desktop), sem marcar "latest" no upstream.
5. Não usar URL do upstream como download oficial B-3PO.
6. Separar desenvolvimento, distribuição de teste e lançamento público.
7. Build não assinado **nunca** é chamado de release seguro ou certificado.

## 16. BRANCH E CI

`dev` está sem proteção (`protected: false`). Enquanto não houver ruleset — e não alterar
configuração administrativa sem autorização: exigir CI verde **no SHA do PR**, revisão do diff,
conferência de conflitos e evidência de aprovação.

Nunca: `main` direto, force push em branch compartilhada, descartar trabalho do dono, merge sem
critério. **PR mergeado não é prova de qualidade funcional.**

Antes do merge: checks obrigatórios verdes, SHA conferido, execução final lida (os três jobs),
falha investigada. `skipped` não é sucesso; `cancelled` não é sucesso; teste que passa só em
retry é suspeito de flaky. Não mergear com CI em andamento.

## 17. EVIDÊNCIA — MATRIZ OBRIGATÓRIA

| Critério | Implementação | Teste | Evidência | Estado |
|---|---|---|---|---|

Estados: `NOT_STARTED`, `IN_PROGRESS`, `IMPLEMENTED`, `TESTED`, `VERIFIED`,
`BLOCKED_BY_EXTERNAL_DEPENDENCY`, `FAILED`, `DEFERRED_BY_OWNER`.
**Implementado ≠ testado ≠ verificado ≠ aprovado para release.**

Cada entrega registra: SHA completo, PR, arquivos, testes rodados, comandos, resultado, URL do
CI, ambiente, limitações, resultado visual quando houver.
CI verde em SHA diferente do código entregue **não** é evidência.

## 18. RELEASE 1 — O QUE FECHA

### Gate obrigatório (só isto bloqueia)
1. **Scanner de i18n corrigido e recontado** (seção 11).
2. **Release do fork separado do upstream** (seção 15, itens 1–2).
3. **Banco**: inexistente, vazio, válido, corrompido, migração, interrupção, recuperação.
   Corrompido **avisa e não apaga**; preserva o arquivo original antes de qualquer tentativa.
   (PR #43 cobre parte.)
4. **Instalação limpa, atualização e rollback** nos três sistemas, com o estado não assinado
   documentado (`xattr -cr`, SmartScreen).
5. **Primeiro uso em < 3 min sem ajuda**: detectar agente → criar bot → mensagem → resposta →
   aprovar uma ação; QR e reconexão após matar o app; desinstalação sem serviço órfão.
6. **Erros humanos**: sem stack trace, sem caminho interno, sem token, sempre com ação.
   Estados vazios e offline em toda tela que possa ficar vazia.
7. **pt-BR no escopo liberado**: zero visível não traduzido, inspeção visual registrada.
8. **Performance medida**: baseline reproduzível de abertura, primeiro conteúdo, primeiro token
   e p95; limite só **depois** do baseline, com hardware descrito.
9. **Checkpoint reconciliado** + versão bumpada + `WhatsNew` dos dois apps + tag.

### Prática contínua (registrar como risco, **não** bloqueia)

Build reproduzível, health checks, SLO, threat model integral, WCAG 2.2 AA completo,
backup/restore automatizado, observabilidade com painéis. Entram na matriz com prioridade — não
no gate.

## 19. RELEASE 1.5 — PREVIEW (bloqueado até decisão)

`docs/plans/preview-window.md` está **untracked** e é de outra sessão: não staged, não é
especificação oficial. Se o dono autorizar, o primeiro passo é **commitar o plano** como spec,
com critérios de aceite, e só então implementar.

Desenho já aprovado, quando autorizado: servir (não executar) · manifesto
`<workspace>/.codync/preview.json` · capability em loopback por sessão de bot · watcher com
debounce → evento com `rev` · falha mantém a última versão boa · UI isolada sem preload (nunca
iframe same-origin) · build automático com travas (consentimento por projeto, só o `build` do
manifesto, hash do workspace, environment limpo, timeout, fila, não bloqueia a resposta) ·
aditivo, `minApp` não sobe.

Se a sessão paralela estiver ativa, ela é dona de `host/src/preview/**`, do braço `previewInfo`,
da tool `set_preview` no MCP `chat` e da chave do evento no `wire-shape.json`.

## 20. RELEASE 2 E BACKLOG

**Release 2 — atendimento** (hipótese): WhatsApp **oficial** (Cloud API por cliente; fora da
janela de 24 h só template aprovado; cobrança por mensagem entregue; o MCP oficial da Meta é
**provisionamento**, não inbox) · CRM próprio (R3) no host, fora de namespace de bot, leitura
por artifact · agenda via MCP · LGPD (retenção, exclusão, opt-in) · **risco nº 1: mensagem de
cliente é entrada não confiável → isolamento de contexto por conversa**.

**Backlog (Fases C–G e pilares)**: Agent Studio completo · Model Gateway com roteamento por
custo/latência · Memory/Knowledge Gateway · observabilidade com SLOs · workflows visuais ·
marketplace e SDK · RBAC/multi-tenant · hardening, carga, backup/restore · benchmarks.

**Regra**: nada daqui começa sem o dono editar este arquivo.

## 21. PROCESSO — GIT, PR, SESSÃO

Branch por tarefa a partir do `dev` do fork (**nunca** de `upstream/dev`). Um PR por assunto.
Não misturar correção não relacionada.

**Proibido**: `Remove-Item -Recurse` / `rm -rf` no repositório; `git add -A` / `git add .`.
Usar `git clean -nd` antes de `-f`; diretório de teste fora do repo. Nunca `stash`/`reset`/
`checkout`/`rebase` de hunk alheio.

**Armadilhas já pagas**: tool nova em servidor MCP pode quebrar turnos de grupo/request (PR #4,
revertido) — rodar `files_e2e` e `network_e2e` antes · campo aditivo exige
`host/tests/fixtures/wire-shape.json` · dead code é erro no crate · não inventar necessidade:
antes de renomear ou refatorar, mostrar o defeito medido.

**Disciplina de sessão**: no máximo **3 PRs simultâneos**; feche o que abriu antes de abrir o
próximo; se o orçamento de rodadas terminar, deixe a árvore limpa e o checkpoint escrito.
38 PRs num dia sem release não é progresso — **um item do Release 1 fechado vale mais que dez
PRs laterais**.

## 22. AUTONOMIA

Não peça confirmação para: nome de variável, organização de arquivo, correção de lint, teste,
refatoração pequena, correção de bug, documentação técnica, branch e PR normal.

Peça para: excluir dados, mudança irreversível, custo relevante, publicação pública, acesso
externo sensível, alteração administrativa crítica, decisão de produto do dono, mudança de
escopo entre releases.

**Não pedir autorização repetidamente para algo já autorizado.**

## 23. RELATÓRIO

Ao fim de cada tarefa: **Resultado** · **Alterações** · **Testes** (comando e saída) ·
**Evidências** (PR, SHA, CI) · **Segurança** · **Pendências** · **Bloqueios** · **Próxima
tarefa**.

Obrigatoriamente **três números medidos** e reproduzíveis: strings fora da tradução (scanner
corrigido), testes que passaram, tempo de abertura.

Criar `tools/release-report.mjs` que imprime o estado do Release 1 (os 9 itens da seção 18) com
os números — o checklist deixa de depender de alguém declarar.

Proibido no relatório: "100% pronto", "totalmente seguro", "sem erros", "produção garantida".

## 24. ENCERRAMENTO

Release 1 é concluído quando os 9 itens do gate estiverem demonstrados **e** a tag publicada:
registrar no checkpoint com as 9 linhas de evidência, **parar**, e o dono abre o próximo goal.

Trabalho fora da seção 18 antes disso é recusado citando esta seção. Ambiguidade **dentro** da
seção 18 é decidida pelo agente e registrada no PR — não é motivo de parada. Só a seção 4
(dependência externa) justifica parar e perguntar.

## 25. PRIMEIRA EXECUÇÃO

1. **Auditar**: SHA real, PRs, CI, checkpoint, untracked.
2. **Investigar divergências**: scanner de i18n · release gate do upstream · estado do
   `preview-window.md` · proteção de branch · critério de release dos mascotes · identidade vs
   branding herdado · instalação e primeiro uso · segurança das bridges · estado real de cada
   item da Onda 2.
3. **Matriz de situação**: | Área | Estado real | Evidência | Problema | Prioridade | Próxima
   ação |
4. **Plano**: P0 · P1 · bloqueios · decisões do dono · escopo futuro.
5. **Implementar** — começando por commitar este goal em `docs/plans/B3PO-GOAL.md` e reconciliar
   o PR #43 em curso.

Preserve todo trabalho válido. Prossiga incrementalmente até fechar o Release 1.
