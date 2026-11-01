# Baseline de abertura — primeira medição real

Medido em 01/11 no **demo** (`vite` dev server servindo `src/renderer/demo`, Chromium via Playwright, 1440×900), com
`performance.getEntriesByType("paint")`.

| Medida | Valor |
|---|---|
| `domContentLoaded` | **2152 ms** |
| `load` | **2155 ms** |
| `first-paint` | **2156 ms** |
| `first-contentful-paint` | **2204 ms** |
| Primeiro conteúdo reconhecível ("Ship room" visível) | **2333 ms** |

## O que este número é, e o que não é

**É** o tempo de abertura do **renderer em modo dev**, com host de memória, sem Electron e sem host real.
**Não é** o tempo de abertura do app instalado: esse continua sem medição, porque o instalador não conclui nesta máquina
(`docs/plans/install-blocked.md`).

O item 8 do gate pede baseline reproduzível de abertura, primeiro conteúdo, primeiro token e p95 **com hardware descrito**.
O que existe agora é uma linha de base do renderer em dev — serve para detectar regressão no renderer, não para fechar o item.

## Primeiro token (host) — ver `first-token-baseline.md`

`[3, 3, 3, 3, 678]` ms em 5 rodadas com agente local: **678 ms** na primeira (processo + handshake ACP) e **3 ms** nas
seguintes. É o piso do host, não o tempo de um modelo real.
