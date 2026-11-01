# B-3PO — checkpoint de continuidade

> Fonte de direção: **goal v3.2** + **adendos v3.3 e v3.4**. O gate do Release 1 tem **11 itens**
> (9 do v3.2 + 10 e 11 do v3.4). O relatório de gate sai de `node tools/release-report.mjs`.

## Sessão de 01/11/2026, 21:00Z — auditoria de recuperação e primeiro ciclo de correção

`dev` = `51cb2e1c44db098517235aa294064b88798b0f33`. PRs abertos: **#169** e o deste checkpoint. Nenhum
segredo real encontrado; nenhuma reescrita de histórico necessária.

### Recuperação — reconciliada

O `Codync-dev.zip` é um snapshot fiel do `dev` em `2e69b6c2`: **3511 de 3513 arquivos idênticos**
por SHA-256, **zero** arquivos exclusivos do ZIP. Os dois restantes são symlinks
(`apps/screen-macos-tests/Sources/*.swift`) que o extrator recusou por apontarem para fora da
pasta. Não há trabalho recuperável só no ZIP, e o checkpoint anterior (que citava `0fc8e166`) estava
desatualizado em relação ao remoto.

### Ambiente construído nesta máquina

| Peça | Versão | Observação |
|---|---|---|
| Rust | 1.99.0, host `x86_64-pc-windows-gnu` | o repo fixa a 1.99.0; não há MSVC C++ nem admin aqui, então o host GNU é a adaptação local |
| Android SDK | platform 37.2, build-tools 37.0.0, platform-tools 37.0.1 | instalado em `projects/b3po-codync/tools/android-sdk` |
| JDK | Temurin 21.0.13 | exigido pelo AGP 9.4.1 |
| Node / npm | 26.13.0 / 11.20.0 | |

### O que passou a rodar (e o que isso revelou)

| Suíte | Resultado em 01/11 | O que estava quebrado |
|---|---|---|
| Host | **840 passando, 0 falhando, 2 ignorados**, 34 binários | 2 testes de unidade + a suíte E2E travava no Windows (PR #161) |
| Desktop | **326 passando, 0 falhando**; typecheck limpo; `electron-vite build` ok | 13 testes de fixture falhavam no Windows (PR #160) |
| Web | 43 páginas, lint e docs ok; 0 critical / 0 high | `next` 16.2.1 com 1 critical (PR #159) |
| Cloud | 125 testes, typecheck ok | 8 high no toolchain (PR #167) |
| Relay / private relay | todas as suítes verdes; 0 achados | 5 high + 1 moderate (PR #163) |
| Android | `:core:test` 192 (1 ignorado), `:app:testDebugUnitTest`, `:assembleDebug`, `:lintDebug` verdes | **o app não compilava no `dev`** (PR #165); um teste de timing não passava no Windows (PR #168) |

O achado mais grave do ciclo não veio de um teste: `.github/workflows/android.yml` estava vermelho
em todo push para `dev` desde o PR #149, porque o app Android não compilava (`@Composable` faltando
em `stateWord`). O merge levou a run verde do próprio PR junto.

### Entrega — o que estava apontando para o upstream

`electron-builder.yml` publicava com `owner: leepokai` (isso vira o feed do `electron-updater`), o
instalador, o tap do Homebrew, o manifesto de update do host e o release gate também eram do
upstream. O PR #164 aponta tudo para `DZ23-LTDA/Codync` e o gate passa a exigir a variável
`RELEASE_GATE_ENABLED` — sem ela o job continua skipped, nada é publicado.

### Segurança — triagem concluída

Relatório completo em `projects/b3po-codync/reports/SECURITY-TRIAGE.md` (workspace do agente):

- **156 achados do GitGuard** (scan de `main@7f33f3c`) reproduzidos em `dev` com `npm audit`,
  Gitleaks 8.30.1 e Semgrep 1.183.0.
- **Nenhuma credencial real**: as 4 chaves do PostHog são `phc_` (públicas por design; `phx_` não
  existe no repositório), o PEM do teste de FCM é gerado em tempo de execução, o resto é fixture ou
  bundle de terceiro. Nada a revogar, nada a reescrever no histórico.
- Dependências: `web` 0/0, `relay` 0, `private-relay` 0, `cloud` 0 high, `desktop` 0 high.
- GitHub Actions: 64 referências pinadas por SHA (`chore/pin-action-shas`, aguardando PR).

### Gate (11 itens)

| # | Item | Estado | Evidência desta sessão |
|---|---|---|---|
| 1 | Scanner de i18n | IMPLEMENTED | o scanner **estava errado** (descartava toda frase de 5+ palavras); corrigido + 27 traduções (PR #166). `pt` = 100% da tela do desktop |
| 2 | Release do fork ≠ upstream | IMPLEMENTED | PR #164 (URLs, tap e gate) |
| 3 | Banco em todos os estados | TESTED | suíte do host verde nas 3 OSes da CI (PR #161) |
| 4 | Instalação/update/rollback em 3 sistemas | BLOCKED_BY_EXTERNAL_DEPENDENCY | precisa do app instalado (§ abaixo) |
| 5 | Primeiro uso em 3 min, QR, reconexão | NOT_STARTED | sem app instalado |
| 6 | Erros humanos, vazios e offline | IN_PROGRESS | `error-messages`: 0 stack traces; `empty-states`: 28 de 63 views com estado vazio (o medidor é heurístico e superestima) |
| 7 | pt-BR no escopo | IN_PROGRESS | `pt` 100% da tela do desktop; iOS/Android 0 chaves sem tradução; falta inspeção visual com o app rodando |
| 8 | Performance medida | NOT_STARTED | números antigos existem, mas de renderer dev, não do app instalado |
| 9 | Checkpoint, bump, WhatsNew, tag | NOT_STARTED | este documento é a parte de checkpoint |
| 10 | Ciclo de entrega | BLOCKED_BY_EXTERNAL_DEPENDENCY | além do instalador, o app publicado buscaria update no upstream até o PR #164 entrar |
| 11 | Cancelar/destravar turno | TESTED | PRs #86–#88 + E2E de host |

**Nenhum item VERIFIED.** Itens 4, 5, 8 e 10 dependem de rodar o app instalado.

### Bloqueios com ação do dono

| Bloqueio | Ação que libera |
|---|---|
| Instalação do app publicado (itens 4, 5, 8, 10) | rodar `codync-windows-x64.exe` uma vez; três tentativas via harness nunca retornaram |
| Release gate (marcar release como latest, apontar o tap) | criar o tap `DZ23-LTDA/homebrew-codync`, o secret `TAP_TOKEN` e a variável `RELEASE_GATE_ENABLED=true` |
| iOS (build, testes, telas) | macOS + Xcode; não existe nesta máquina |
| Android instrumentado | imagem de sistema + AVD (~2 GB) |
| Assinatura macOS/Windows | certificados; não são desta máquina |
| Deploy de `cloud/`/`relay/` | autorização explícita |

### Próxima tarefa concreta

1. Fechar os PRs #166–#168 e abrir o PR das GitHub Actions pinadas.
2. Com autorização do dono: instalar o app publicado e medir primeiro uso, abertura e update
   (itens 4, 5, 8). É o único caminho para sair de NOT_STARTED/BLOCKED nesses quatro itens.
3. Sem a instalação: traduzir as 115 frases que faltam nas outras seis línguas e conferir os
   estados vazios apontados por `tools/empty-states.mjs` (triagem manual, o medidor é heurístico).
