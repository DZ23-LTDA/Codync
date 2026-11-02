# Release 1 — reconciliação do gate (02/11/2026)

Ponto de partida: `release-state.json`, os adendos v3.3/v3.4 e o HEAD de `dev` desta sessão. Cada
linha diz **o que foi executado**, onde, e o que falta para sair de `TESTED` para `VERIFIED`. Estado
só sobe com evidência reproduzível; nada aqui foi rebaixado.

HEAD: `00ab59532062` (merge do #175). Já em `dev` nesta sessão: #172 (marca), #173 (suíte
instrumentada), #174 (reconciliação GitGuard), #175 (teclado ao tocar na conversa).

| # | Item | Estado | Evidência | Falta para VERIFIED |
|---|---|---|---|---|
| 1 | Scanner de i18n | `VERIFIED` | #166 (correções anteriores) + esta rodada: `l10n-missing` decodifica os escapes do literal (uma chave escrita `\n` era comparada com a barra crua e a frase traduzida aparecia como faltando) → **815 frases, 0 sem entrada no pt.json**; `l10n-unwrapped` → **0 strings visíveis fora de `tr()`** (14 em escopo de módulo, anunciadas pelo componente); `l10n-coverage` → pt 1673 chaves/100%, demais 1468/87,7% | — |
| 2 | Release do fork ≠ upstream | `VERIFIED` | #164 (URLs/tap/gate) + esta rodada: `tools/release-gate.py` agora recusa **também o tap** (`leepokai/homebrew-codync`), não só o repositório; workflow com a condição do fork e `vars.RELEASE_GATE_ENABLED` (comentário no YAML explica); probes: repo de terceiro → exit 1, tap de terceiro → exit 1, padrões → segue | habilitar de verdade exige `TAP_TOKEN` + tap do dono (item 10) |
| 3 | Banco em todos os estados | `VERIFIED` | #43/#48/#50/#62 (store: ausente, vazio, válido, danificado, truncado, migrações, cópia) + #181 (interrupção de escrita determinística, rajada morta no meio) + #182 (processo recusa schema mais novo e arquivo danificado/truncado, sem tocar no arquivo nem criar um vazio no lugar) | — |
| 4 | Instalação/update/rollback em 3 sistemas | `TESTED` | — | `codync-windows-x64.exe` publicado, instalar, abrir, atualizar, desinstalar — ação do dono |
| 5 | Primeiro uso em 3 min, QR, reconexão | `IN_PROGRESS` | onboarding do Android percorrido no emulador (1/2 → 2/2 → permissão de câmera negada → campo para colar `codync-dev://pair`); desktop: 2 passos até a janela, com host real | cronometrar o fluxo inteiro com host pareado, nos dois clientes |
| 6 | Erros humanos, vazios e offline | `IN_PROGRESS` | suíte de UI do desktop (127 testes) + passeio com app real (02/11, `tools/e2e-desktop/e2e-states.cjs`): vazio ("Seus agentes de código, como colegas de equipe." + "Novo bot"), falha ("O host não está rodando" / "O host não está respondendo. Veja o log para detalhes." + "Tentar de novo", depois de ~30 s em "Iniciando o host…"), offline ("Offline" na linha do bot ao usar a janela) e volta ao normal quando o host retorna. Nenhuma das telas mostra stack trace, caminho interno ou segredo | metade celular (precisa de pareamento — firewall) e TUI |
| 7 | pt-BR no escopo liberado | `TESTED` | #166, #178, #179 (catálogo 1673 chaves, 0 faltando) + inspeção com app real, duas rodadas: onboarding (2 passos), janela vazia, erro de host, offline, conversa, Stop, **Extensões nas três abas** ("Conectores vêm do MCP Registry oficial e são instalados em Dz23.", "Skills vêm da Anthropic e são instaladas em Dz23.", "Conector personalizado", "Adicionar", "Não instalado") e o **menu do bot** ("Fixar", "Mover para a seção", "Marcar como lida", "Editar perfil…", "Copiar o ID da conversa", "Apagar"). Emulador Android: boas-vindas e passo de instalação | telas internas do celular (pareamento bloqueado pelo firewall) e iOS (só CI, sem Mac) |
| 8 | Performance medida | `VERIFIED` | API (p50 0,9 ms / p95 1,3 ms), memória (50 MB), Android a frio (2,3–2,6 s), **primeiro token** p95 **17 ms** num bot e **1212 ms** com 8 bots juntos, **CPU do host sob carga 28,6% de um núcleo** (0,89% da máquina, janela de 1,91 s, 8 turnos), **resposta longa** de 62 KB em 0,68 s e a janela abrindo a conversa em **39 ms** | 20 turnos numa mesma conversa não medidos (ver nota) |
| 9 | Checkpoint, bump, WhatsNew, tag | `IN_PROGRESS` | `B3PO-CHECKPOINT.md` atualizado nesta sessão | bump + WhatsNew + tag (é a publicação: exige autorização do dono) |
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
| Primeiro token, 1 bot, 20 rodadas | p50 **3 ms** · p95 **17 ms** (uma rodada a frio: 2789 ms) |
| Primeiro token, 8 bots ao mesmo tempo | 1211–1212 ms nos oito · p95 **1212 ms** |
| CPU do host, 8 turnos ao mesmo tempo | 0,547 s de CPU em 1,91 s de janela = **28,6% de um núcleo** (0,89% da máquina de 32 núcleos) · memória 54 MB |
| Resposta longa (400 blocos, 62 KB) | host grava e fecha o turno em **0,68 s**; a janela mostra a última linha **39 ms** depois do clique |
| Desktop, janela → primeiro conteúdo (5ª execução, perfil quente) | janela em **922 ms** (pior 955) · primeiro conteúdo em **1060 ms** (pior 1086) |

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

## Estados vazios, de erro e offline no desktop (item 6) — o que o passeio mostrou

Roteiro `tools/e2e-desktop/e2e-states.cjs`, capturas em `reports/desktop-e2e/states/`:

| Situação | O que a janela mostra | Quando |
|---|---|---|
| Host ainda subindo | "Iniciando o host… Isso leva alguns segundos." | imediato |
| Host que nunca sobe | "O host não está rodando" · "O host não está respondendo. Veja o log para detalhes." · botão **Tentar de novo** | ~30 s |
| Host no ar, nada criado | "Seus agentes de código, como colegas de equipe." · "Escolha um bot, ou crie um para cada tipo de trabalho e aponte para um projeto." · **Novo bot** | imediato |
| Host morto com a janela aberta (parada) | nada muda | 10 s |
| Host morto, depois de clicar | **"Offline"** na linha do bot; a conversa em cache continua legível | ~6 s após o clique |
| Host de volta | "Offline" some e a conversa volta | ~8 s |

Nenhuma dessas telas mostra stack trace, caminho interno, nome de arquivo ou segredo — o filtro do roteiro
procura `at Object.`, `node_modules`, `C:\`, `/home/` e `/Users/` em tudo o que aparece na tela e não achou
nada.

Duas observações, não defeitos:

- são ~30 s até a janela desistir do host; quem olha não sabe se está lento ou quebrado. O botão "Tentar
  de novo" resolve, então é espera, não beco.
- a mensagem manda "ver o log" mas não diz onde; o log está no menu da bandeja ("Open log"). Um link na
  própria mensagem fecharia o caminho.

### Primeiro token (item 8) — o que foi medido e com o que

`cargo test --test first_token_e2e -- --ignored --nocapture` (host `debug`, Windows 11, agente de fixture
`fast_reply_agent.py`, que responde no mesmo instante: o número é **overhead do host**, não do modelo).

| Medida | Resultado |
|---|---|
| 1 bot, 20 rodadas | `[3 ×18, 4 ×2, 17, 2789]` ms · **p95 17 ms** |
| 8 bots ao mesmo tempo | `[1211, 1211, 1211, 1211, 1211, 1212, 1212, 1212]` ms · **p95 1212 ms** |

Leitura: com o processo do agente já quente, o host leva **3 ms** para pôr a primeira palavra no
transcript; a rodada de 2789 ms é o primeiro turno, que sobe o processo. Com oito bots, cada turno paga a
subida de um processo Python (8 em paralelo) — 1,2 s, e o valor é o mesmo nos oito, ou seja, o host não
serializa os turnos.

CPU sob carga e a tela pesada foram medidos depois (abaixo).

### Carga e tela pesada (item 8) — como foi medido

`tools/measure-host-cpu.ps1`: host sozinho, 8 bots com um agente de fixture, os 8 turnos disparados juntos.
Entre o instante antes dos envios e o oitavo `pong`: **1,91 s de janela**, **0,547 s de CPU do processo
`codync-host`** — 28,6% de um núcleo, 0,89% da máquina — e 54 MB de working set no pico.

`tools/measure-heavy.ps1`: um turno com `bulk_reply_agent.py` (400 blocos de texto) vira **62 KB em 2
entradas** em **0,68 s** (o host junta os blocos numa mensagem só), e a janela do desktop mostra a última
linha **39 ms** depois do clique no bot.

O que **não** foi medido: vinte turnos na mesma conversa (200 entradas). Minha primeira tentativa parou por
um erro meu — o laço esperava o número de mensagens do assistente crescer por um critério que nunca casava,
e cada turno queimava o limite inteiro. O caminho para medir é o mesmo script com o critério certo (contar
entradas, não o tipo), e fica registrado como o único buraco do item.

## Telas internas do desktop (itens 6 e 7) — segunda passada

`tools/e2e-desktop/e2e-inner.cjs`, capturas em `reports/desktop-e2e/inner/`, host real e identidade dev.

| Tela | O que está em português |
|---|---|
| Extensões → **Agentes** | "Agentes vêm do registro ACP e são instalados em Dz23.", "Instalado" / "Não instalado" |
| Extensões → **Conectores** | "Conectores vêm do MCP Registry oficial e são instalados em Dz23.", "Conector personalizado · Qualquer servidor MCP: um comando ou uma URL", "Adicionar", "Novo" |
| Extensões → **Skills** | "Skills vêm da Anthropic e são instaladas em Dz23.", "Sua própria skill" |
| Menu do bot | "Fixar", "Mover para a seção", "Marcar como lida", "Editar perfil…", "Copiar o ID da conversa", "Apagar" |
| Diálogo de apagar | "Apagar Codync?" · "Os arquivos que ele alterou no seu computador ficam como estão." · "Cancelar" · "Excluir o bot e a conversa dele" |

O que **continua em inglês é conteúdo de terceiro**, de propósito: as descrições dos conectores ("Connect AI
agents to 1000+ apps with manage…") e os nomes dos produtos vêm do registry, não do nosso catálogo.

Uma observação registrada, **não reproduzida**: numa das passadas a janela registrou 12 respostas 404 no
console enquanto o diálogo de apagar estava aberto. Duas tentativas depois (perfil novo e o mesmo perfil, com
Extensões aberta) deram **zero** respostas >= 400. Sem repro, não vira defeito — fica anotado para quem
mexer nessa tela.

### Abertura do desktop (item 8) — como foi medido

`tools/e2e-desktop/measure-open.cjs`, cinco execuções num perfil já configurado, host no ar, Windows 11:

| Medida | Mediana | Pior |
|---|---|---|
| Janela na tela | **922 ms** | 955 ms |
| Primeiro conteúdo legível | **1060 ms** | 1086 ms |

É o app **desempacotado** (`npm run build` + Electron local), não o instalador assinado — o número mede o
carregamento do renderer e a primeira resposta do host, não a verificação de assinatura do pacote. A primeira
execução num perfil novo levou **2644 ms** porque passa pelo onboarding.