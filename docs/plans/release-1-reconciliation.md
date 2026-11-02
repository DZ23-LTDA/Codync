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
| 3 | Banco em todos os estados | `VERIFIED` | #43/#48/#50/#62 (store: ausente, vazio, válido, danificado, truncado, migrações, cópia) + #181 (interrupção de escrita determinística, rajada morta no meio) + #182 (processo recusa schema mais novo e arquivo danificado/truncado, sem tocar no arquivo nem criar um vazio no lugar) | — |
| 4 | Instalação/update/rollback em 3 sistemas | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | `codync-windows-x64.exe` publicado, instalar, abrir, atualizar, desinstalar — ação do dono |
| 5 | Primeiro uso em 3 min, QR, reconexão | `IN_PROGRESS` | onboarding do Android percorrido no emulador (1/2 → 2/2 → permissão de câmera negada → campo para colar `codync-dev://pair`); desktop: 2 passos até a janela, com host real | cronometrar o fluxo inteiro com host pareado, nos dois clientes |
| 6 | Erros humanos, vazios e offline | `IN_PROGRESS` | telas de erro/offline exercitadas pela suíte de UI (127 testes); `error-messages`: 0 stack traces; `empty-states`: 28 de 63 views | inspeção visual com captura, por cliente |
| 7 | pt-BR no escopo liberado | `TESTED` | #166 (scanner + 27 frases), #178 (português virou idioma de verdade nos apps, com o catálogo empacotado no iOS), #179 (catálogo completo: 1673 chaves, 0 faltando; tela de instalação e boas-vindas conferidas no emulador em pt-BR) | inspeção humana das telas internas (com host pareado) |
| 8 | Performance medida | `IN_PROGRESS` | primeiras medições reais (abaixo) | primeiro token com agente real; carga/concorrência |
| 9 | Checkpoint, bump, WhatsNew, tag | `NOT_STARTED` | `B3PO-CHECKPOINT.md` atualizado nesta sessão | bump + WhatsNew + tag (é a publicação: exige autorização do dono) |
| 10 | Ciclo de entrega | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | depende do 4 e da autorização de publicação |
| 11 | Cancelar um turno travado | `VERIFIED` | #86/#87/#88 (watchdog de 180 s, decisão coberta por 6 testes, E2E com agente silencioso) + E2E de app rodando (02/11): bot com `silent_agent.py`, mensagem enviada, botão **Stop** clicado na janela real → o turno fecha e a conversa mostra "Stopped." | dica de "você pode parar" no iOS/Android/TUI (item de UX aberto, abaixo) |

Nota do item 7: o catálogo zh-Hans (a referência) **não tem 205 strings** que os apps pedem — por isso elas aparecem em inglês para todo mundo menos o português, que as carrega. Corrigir isso exige traduzir as 205 para as outras seis línguas; está registrado no teste de catálogo.

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

## Pareamento de celular — bloqueado nesta máquina (02/11/2026)

O host sobe e responde em `127.0.0.1:19223`, e `codync-host pair --json` entrega o link
`codync-dev://pair?...` com os endereços de LAN (`192.168.100.3`, `172.20.16.1`). O emulador **não
alcança** o host porque o Windows Firewall tem duas regras de entrada para `codync-host.exe` com ação
**Block** (criadas por um "Bloquear" no aviso do firewall, perfis privado/público, TCP e UDP).

Consequência: o item 5 (pareamento por QR, primeiro uso) e a inspeção visual das telas internas do
celular ficam **BLOCKED_BY_EXTERNAL_DEPENDENCY** até o dono liberar:

> Windows Security → Firewall e proteção de rede → Permitir um aplicativo → marcar `codync-host` nas
> redes privada e pública (ou remover as duas regras "codync-host" de bloqueio no firewall avançado).

Não alterei as regras: mudar permissão de firewall exige autorização explícita.

O que **foi** verificado sem pareamento, no desktop (que fala com o host por loopback):

- Tela **Extensões** em pt-BR: "Extensões", "Credenciais", "Buscar agentes, conectores e skills",
  "Agentes vêm do registro ACP e são instalados em Dz23.", "Não instalado", "Instalado".
- Conversa: "Carregar mensagens anteriores", "Hoje", "Perguntar a Codync".
- 0 erros de console e 0 de página no passeio inteiro.

Dois achados do passeio:

1. **O desktop empacota os catálogos no build** (`import … with { type: 'json' }`). Uma build antiga
   mostrava uma string misturada ("Not instalados"); depois de reconstruir com o catálogo atual, a
   mesma linha lê "Não instalado". Consequência prática: mudança de catálogo exige rebuild do desktop
   para validação visual (o site e os apps de celular não têm esse passo de build para o catálogo, o
   iOS sim: é recurso empacotado).
2. **Linhas do histórico geradas pelo host continuam em inglês** ("Created routine: Check in"). É
   conteúdo, não interface — mas é visível ao usuário e não passa por `tr()`. Fica registrado como
   decisão de produto, não como bug de tradução.

## Banco em todos os estados (item 3) — o que as provas novas mostram

`write_interruption_e2e.rs` acrescenta duas propriedades que faltavam:

1. **Morte com a escrita impossível de commitar**: outro processo segura o `BEGIN IMMEDIATE` do SQLite, o
   host tenta gravar (fica preso no caminho de escrita) e é morto com `kill` seco. Depois: o arquivo abre,
   `PRAGMA integrity_check` = `ok`, o bot que já tinha sido gravado está lá, o que nunca commitou não
   aparece, e o host sobe de novo na mesma pasta sem pôr nada "de lado" como corrompido.
2. **Rajada de escritas + morte no meio**: as escritas são disparadas em sequência e o processo morre no
   meio da rajada; ao voltar, toda escrita que o cliente recebeu como sucesso está no arquivo, nenhum bot
   que ninguém pediu apareceu, toda linha tem config JSON válida e o host lista exatamente o que o arquivo
   tem.

Nível de processo (o que a pessoa vê quando o host sobe numa pasta que não consegue ler), testado em
`database_damage_e2e.rs`: schema mais novo → o processo **sai com erro** e diz "written by a newer version…
Nothing was changed"; arquivo com header destruído → sai e diz "may be damaged… Nothing was deleted";
arquivo truncado → recusa. Nos três, o arquivo fica **byte a byte igual** e nenhum banco vazio é criado no
lugar.

Detalhe que os testes descobriram: matar o host deixa um `codync.db-wal` cheio, e o SQLite **reproduz o
WAL** — com o `codync.db` destruído e o WAL intacto, o banco ainda abre com os dados. Para o cenário de
"arquivo danificado" ser real, o WAL tem de ser dobrado de volta antes (o teste faz `wal_checkpoint`).

## Parar um turno travado (item 11) — o que o E2E de app rodando mostrou

Feito com a janela real (`tools/e2e-desktop/e2e-stop.cjs`, capturas em `reports/desktop-e2e/stop/`):

1. Um bot roda `silent_agent.py`, um agente ACP que aceita o prompt e **nunca responde**.
2. A mensagem é enviada pela própria janela; o bot entra em "Pensando…".
3. O botão **Stop** é clicado na interface.
4. O turno fecha e a conversa mostra **"Stopped."** (`stopReason: "cancelled"`); o botão de parar desaparece.

### Dois achados que ficam registrados (não corrigidos aqui)

- **São 17 segundos** entre clicar em Stop e o turno realmente fechar quando o agente ignora o
  `session/cancel`: o host espera a janela de cancelamento antes de matar o processo. Durante esse tempo
  a janela **não dá nenhum sinal** — continua "Pensando…". Para quem clica, parece que o botão não
  funcionou. Mudança sugerida (decisão do dono, é produto): um aviso imediato "Parando…" ou o botão em
  estado de espera.
- **"Stopped." é texto do host, em inglês**, como "Created routine: Check in". A conversa em pt-BR mistura
  os dois idiomas. É conteúdo gerado pelo host, não interface — corrigir exige decidir se o texto dessas
  linhas vem traduzido do host ou se o cliente formata a partir do `stopReason`.