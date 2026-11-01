# Travamento e cancelamento — o que já existe, medido no código

Terceira medida do adendo v3.4 (§2): *"turno cujo agente não responde — o que o usuário vê, e como sai disso"*.

**Ressalva de método**: não consegui reproduzir um turno real travado (o app nesta máquina não chegou a rodar um agente —
parou antes, sem host instalado). O que está abaixo é **inspeção do caminho de cancelamento**, não observação de um
travamento. É menos do que o adendo pede, e está dito.

## O que existe hoje (verificado no código)

| Peça | Onde | Estado |
|---|---|---|
| Método no host | `host/src/api/bots.rs` — `"stop" => { let bot = str_arg(&b, "botId")?; … groups.stop(…) }` | **existe**, cobre bot e grupo |
| Chamada do cliente | `apps/desktop/src/renderer/store/bot-store.ts:163` — `this.perform((c) => c.call('stop', { botId }), true)` | **existe** |
| Botão na interface | `apps/desktop/src/renderer/views/thread/Composer.tsx:182` — o botão de envio vira `stop` quando `working && !draft`; linha 309 chama `store.stop(botId)` | **existe** |

Ou seja: **"bot preso em trabalhando pode ser cancelado pela UI" já é verdade no desktop** — o botão de enviar se
transforma em *parar* enquanto o bot trabalha.

## O que **não** existe (as duas metades do item 11)

1. **Watchdog por tempo sem evento.** Não há, no caminho lido, um temporizador que detecte "nenhum evento há N segundos" e
   ofereça cancelar sozinho. Hoje o cancelamento depende de a pessoa perceber e clicar.
2. **Registro do cancelamento com motivo no chat.** O `stop` interrompe; o que fica na conversa depois do cancelamento
   **não foi verificado** — e o aceite do adendo pede que o chat registre o cancelamento com motivo.

## O que isso muda no item 11

O item não começa do zero: o caminho host → store → botão existe e é curto. O que falta é **detecção** (watchdog) e
**explicação** (o registro na conversa), que são justamente as duas frases do aceite.

## O que ainda falta medir (honesto)

- **Travamento real**: exige um bot com agente instalado que aceite o turno e não responda. Não reproduzido aqui.
- **O que o usuário vê** durante o travamento: a tela ficou em `Starting the host…` / `Waiting for your computer` nas
  tentativas desta sessão, que são estados **anteriores** a um turno, não um turno travado.

Até esses dois existirem, este documento é a metade da medida que eu consigo provar.
