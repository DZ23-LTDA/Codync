# Primeiro uso — todos os números num lugar só

Antes disto, os números do primeiro uso estavam em quatro arquivos e cada um contava metade da história. Este é o único lugar:
o que foi medido, com que teste, e o que **não** foi.

A régua do goal (§18.5): **primeiro uso em menos de 3 minutos**, sem ajuda. Ninguém executou os três minutos ainda — o app
instalado continua em **2.33.2** e o item 10 está bloqueado. O que existe é o tempo do **host** e do **renderer**, medido,
com o teste que o produz.

## As nove etapas do roteiro

| # | Etapa | Medido | Como | O que ainda falta |
|---|---|---|---|---|
| 1 | abrir o app | **2204 ms** até o primeiro conteúdo | Playwright + `performance` no renderer em dev | o **app instalado** (Electron empacotado) |
| 2 | detectar o agente | — | — | só o app instalado mostra a lista de agentes |
| 3 | criar um bot | **4 ms** | `first_use_e2e.rs`, host real | — |
| 4 | mensagem → primeira resposta | **569 ms** (processo do agente + handshake ACP) | `first_use_e2e.rs`, agente que responde na hora | — |
| 5 | primeiro token | p95 **4 ms** · mediana **3 ms** · pior **587 ms** | `first_token_e2e.rs`, 20 rodadas | — |
| 6 | aprovar uma ação | cartão **602–644 ms** · sim → turno **21–28 ms** | `approval_latency_e2e.rs` | a **decisão humana** (só no app) |
| 7 | parear o celular | QR **3–75 ms** · connect **13 ms** · redeem **1 ms** | `pairing_latency_e2e.rs`, `e2e.rs` | **escanear o QR** (só no app) |
| 8 | matar e reabrir | — | — | só o app instalado |
| 9 | desinstalar sem serviço órfão | — | — | só o app instalado |

**Do lado do host, as etapas 3–7 somam menos de 1,5 s.** O resto dos três minutos é humano: ler, digitar, escolher.

## O que estes números são, e o que não são

**São** medições reproduzíveis, cada uma com o teste que a produz e o ambiente descrito:

```
host/tests/first_use_e2e.rs       createBot + primeira resposta
host/tests/first_token_e2e.rs     primeiro token, 20 rodadas (ignored por padrão)
host/tests/pairing_latency_e2e.rs QR de pareamento
host/tests/e2e.rs                 connect + redeem do código
host/tests/approval_latency_e2e.rs cartão de aprovação + sim
demo + Playwright                 abertura do renderer
```

**Não são** o tempo de uma pessoa no app instalado: este ambiente é Windows, com host de teste, agente de teste e renderer
em modo dev. Um número do host não pode ser apresentado como "o primeiro uso leva X" — só como "o host gasta X".

## Hardware e ambiente

Windows, build **debug** (`cargo test`), MSVC 14.44, Rust 1.99.0, Chromium via Playwright para o renderer. Máquina de
desenvolvimento, não uma máquina limpa: os números de primeira execução (processo do agente nascendo) tendem a ser **piores**
numa máquina recém-instalada, não melhores.

## Próximo passo, quando o item 10 destravar

Executar `first-use-runbook.md` no app instalado, cronometrar as nove etapas, e só então dizer se os três minutos fecham. Até
lá, o item 5 continua `IN_PROGRESS` — com números, não com adjetivos.
