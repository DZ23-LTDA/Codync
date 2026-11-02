# Release 1 — reconciliação do gate (02/11/2026)

Ponto de partida: `release-state.json`, os adendos v3.3/v3.4 e o HEAD de `dev` desta sessão. Cada
linha diz **o que foi executado**, onde, e o que falta para sair de `TESTED` para `VERIFIED`. Estado
só sobe com evidência reproduzível; nada aqui foi rebaixado.

HEAD: `00ab59532062` (merge do #175). Já em `dev` nesta sessão: #172 (marca), #173 (suíte
instrumentada), #174 (reconciliação GitGuard), #175 (teclado ao tocar na conversa).

| # | Item | Estado | Evidência | Falta para VERIFIED |
|---|---|---|---|---|
| 1 | Scanner de i18n | `IMPLEMENTED` | #166: o scanner descartava toda frase de 5+ palavras; corrigido, 27 traduções entraram, `pt` em 100% da tela do desktop conferida | inspeção visual das telas restantes, por cliente |
| 2 | Release do fork ≠ upstream | `IMPLEMENTED` | #164 (URLs/tap/gate recusam outro repositório) e #171 (clientes, tray, SSH, cloud) | o **workflow** ainda tem `github.repository == 'leepokai/Codync'`; vira VERIFIED quando o job rodar no fork (depende do tap e do secret) |
| 3 | Banco em todos os estados | `TESTED` | 10 testes (#43/#48/#50/#62), suíte do host verde nas 3 OSes da CI | restore real de backup, não só validar o arquivo gerado |
| 4 | Instalação/update/rollback em 3 sistemas | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | `codync-windows-x64.exe` publicado, instalar, abrir, atualizar, desinstalar — ação do dono |
| 5 | Primeiro uso em 3 min, QR, reconexão | `IN_PROGRESS` | onboarding do Android percorrido no emulador (1/2 → 2/2 → permissão de câmera negada → campo para colar `codync-dev://pair`); desktop: 2 passos até a janela, com host real | cronometrar o fluxo inteiro com host pareado, nos dois clientes |
| 6 | Erros humanos, vazios e offline | `IN_PROGRESS` | telas de erro/offline exercitadas pela suíte de UI (127 testes); `error-messages`: 0 stack traces; `empty-states`: 28 de 63 views | inspeção visual com captura, por cliente |
| 7 | pt-BR no escopo liberado | `IN_PROGRESS` | #166 (scanner + 27 frases); iOS/Android sem chave faltando | 297 strings ainda fora de `tr()`; inspeção humana |
| 8 | Performance medida | `IN_PROGRESS` | primeiras medições reais (abaixo) | primeiro token com agente real; carga/concorrência |
| 9 | Checkpoint, bump, WhatsNew, tag | `NOT_STARTED` | `B3PO-CHECKPOINT.md` atualizado nesta sessão | bump + WhatsNew + tag (é a publicação: exige autorização do dono) |
| 10 | Ciclo de entrega | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | depende do 4 e da autorização de publicação |
| 11 | Recuperação após encerramento abrupto | `TESTED` | #158: mata o host, confere dois bots e histórico após reinício | interrupção em ponto de escrita determinístico, downgrade de schema, corrupção real |

## Performance (item 8) — primeira medição real

Ambiente: Windows 11, Rust 1.99.0 `x86_64-pc-windows-gnu` (build `debug`), Node 26.13.0, JDK 21.0.13,
Android SDK 37, emulador `dsh_pixel` (API 35, x86_64, GPU de software).

| Medida | Resultado |
|---|---|
| Host `GET /health` (60 amostras) | p50 **0,9 ms** · p95 1,3 ms |
| Host `POST /api/history` (60 amostras) | p50 **1,0 ms** · p95 1,5 ms |
| Host, working set | 50 MB |
| Android, abertura a frio (3×) | 2494 / 2552 / 2283 ms |
| Android, memória | PSS 127 MB (RSS 225 MB) |

Leitura: o transporte do host não é gargalo (1 ms de p50 num endpoint que toca o banco). O Android
foi medido em `debug` num emulador com GPU de software — é o pior caso, não o que o usuário vê.

## Mudanças de estado nesta sessão

Nenhum item subiu para `VERIFIED`. Duas coisas mudaram de fato:

- **#173** tirou a suíte instrumentada do Android da inércia (127 testes em emulador real, 126 verdes,
  24 pulados por ambiente). O `dev` anterior nunca a tinha executado — e ela escondia uma regressão
  real (conversa curta não ficava junto ao compositor).
- **#175** fechou o teclado ao tocar na conversa, com o contrato do gesto exposto e testado.

## Bloqueios que dependem do dono

| Bloqueio | Ação que libera |
|---|---|
| 4, 9, 10 | publicar/apontar o instalador e autorizar a publicação da release |
| 2 | criar o tap `DZ23-LTDA/homebrew-codync`, o secret `TAP_TOKEN` e a variável `RELEASE_GATE_ENABLED` |
| parte de 4/5/7 | conta Apple + Mac com Xcode (não existem nesta máquina) |
