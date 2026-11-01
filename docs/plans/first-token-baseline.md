# Primeiro token — medido no host, sem modelo

Medido em 01/11 com `host/tests/first_token_e2e.rs` e o agente de teste `tests/fixtures/fast_reply_agent.py`, que responde
na hora. Assim o tempo entre `send` e a primeira entrada no transcript mede **só o caminho do host**: subir o processo,
handshake ACP, converter o `session/update` em entrada.

## Resultado (5 rodadas, um bot)

```
first word (ms): [3, 3, 3, 3, 678]  ·  mais rápido 3 ms  ·  p95 678 ms
```

- a **primeira** chamada custa **678 ms** — é o processo do agente nascendo e o handshake ACP;
- as **seguintes** custam **3 ms** cada;
- o teste falha se o p95 passar de 5000 ms, para pegar regressão grande sem quebrar em máquina ocupada.

Hardware: Windows, build debug (`cargo test`), MSVC 14.44, Rust 1.99.0.

## O que este número é, e o que não é

**É** a latência do host até a primeira palavra aparecer no transcript, com agente local instantâneo.
**Não é** "primeiro token" de um modelo real: com Claude ou Codex, o tempo dominante é o do provedor, não o daqui.

O item 8 do gate pede primeiro token **do app**; o que existe agora é o **piso do host**. Com a abertura medida no renderer
(`perf-baseline.md`, 2204 ms até o primeiro conteúdo), dá para separar o que é nosso do que é do modelo — que é a única
forma de um limite de performance ter sentido.
