# Primeiro token — medido no host, sem modelo

Medido em 01/11 com `host/tests/first_token_e2e.rs` e o agente de teste `tests/fixtures/fast_reply_agent.py`, que responde
na hora. Assim o tempo entre `send` e a primeira entrada no transcript mede **só o caminho do host**: subir o processo,
handshake ACP, converter o `session/update` em entrada.

## Resultado

Cinco rodadas (teste normal, com teto de 5 s):

```
[3, 3, 3, 3, 678] ms
```

Vinte rodadas (`-- --ignored --nocapture`):

```
[3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 587] ms
p95 = 4 ms · mediana 3 ms · pior caso 587 ms
```

## Como ler isso

- **caminho quente**: 3–4 ms. É o host transformando um `session/update` em entrada do transcript;
- **uma vez por execução**, ~590–680 ms: o processo do agente nascendo e o handshake ACP. Não é a primeira rodada por
  definição — a posição varia (apareceu no fim nas 20 rodadas, no início nas 5), o que diz que é o processo sendo (re)criado;
- o teto do teste é **5 s**, para pegar regressão grande sem quebrar em máquina ocupada.

Hardware: Windows, build debug (`cargo test`), MSVC 14.44, Rust 1.99.0.

## O que este número é, e o que não é

**É** a latência do host até a primeira palavra aparecer no transcript, com agente local instantâneo. É o **piso**: nenhum
provedor pode ser mais rápido que isso.
**Não é** "primeiro token" de um modelo real: com Claude ou Codex, o tempo dominante é o do provedor.

Com a abertura medida no renderer (`perf-baseline.md`, 2204 ms até o primeiro conteúdo), os quatro números do item 8 existem
para **host + renderer**. O que falta é medi-los **no app instalado**, que depende do item 10.
