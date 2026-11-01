# B-3PO — checkpoint de continuidade

> Fonte de direção: **goal v3.2** + **adendos v3.3 e v3.4**. O gate do Release 1 tem **11 itens** (9 do v3.2 + 10 e 11 do
> v3.4).

## Sessão de 31/10/2026, 23:30Z

`dev` = `0fa01f85`. PRs abertos: **0**. Árvore limpa (só `docs/plans/preview-window.md`, de outra sessão, intocado).

### Versão instalada — a regra 0 do v3.4

| | |
|---|---|
| **App instalado** | **v2.33.2** (`ProductVersion 2.33.2.0`, arquivos de 30/10) — **não contém nada do `dev` de hoje** |
| **Release publicada** | **v2.34.0** — https://github.com/DZ23-LTDA/Codync/releases/tag/v2.34.0 · `codync-windows-x64.exe` (128,9 MB) |
| Instalar nesta máquina | **não concluiu**: 3 tentativas, `/S` nunca retorna. Não é UAC (instalação per-user), não é processo vivo, não é `TEMP`. **Bloqueio de ambiente**, declarado em `docs/plans/release-v2.34.0.md` |

### Gate (11 itens)

| # | Item | Estado | Evidência |
|---|---|---|---|
| 1 | Scanner de i18n | IMPLEMENTED + TESTED | PRs #46, #76 · **298** strings visíveis fora do `tr()` |
| 2 | Release do fork ≠ upstream | IMPLEMENTED + TESTED | PR #47 · guarda aborta fora de `DZ23-LTDA/Codync` |
| 3 | Banco em todos os estados | **TESTED** | PRs #43, #48, #50, #62 · 10 testes |
| 6 | Erros humanos | IN_PROGRESS | PRs #53–#57, #64, #73, #75 · 0 stacks / 0 valores crus na tela; 3 vazamentos corrigidos e vistos no app |
| 7 | pt-BR no escopo | IN_PROGRESS | PRs #71, #72, #73 · tela de acesso **0 inglês** verificado; `pt` 253/1468 na tabela compartilhada |
| 10 | Ciclo de entrega | **IN_PROGRESS** | release publicada; instalação e update in-app **não** |
| 11 | Cancelar e destravar | IN_PROGRESS | `stop` existe (host, store, botão); falta **watchdog** e registro do motivo |
| 4, 5, 8, 9 | instalação/primeiro uso/perf/release | NOT_STARTED / parcial | host pronto em 334 ms; resto depende de VM e app |

**3 de 11 com implementação e teste · 0 verified** (CI não cria runs desde `41255618924`).

### Medidas do adendo v3.4 (§2)

1. **Primeiro uso em máquina limpa**: **não medido** — exige VM Windows.
2. **pt-BR**: **feito** — `docs/plans/ptbr-coverage-report.md` (253/1468 chaves; 298 strings fora do `tr()`).
3. **Travamento**: **metade** — `docs/plans/stuck-turn-report.md` (mecanismo existe; travamento real não reproduzido).

### Inspeção visual real (Playwright + Electron)

Receita que funciona: `TMP` no workspace → `npm run build` → `node tools/account-config.mjs dev` → Electron com
`--no-sandbox --disable-gpu` → Playwright `_electron.launch`. Telas vistas e corrigidas: boas-vindas, acesso ao computador
(pt completo), estados de host (pt + sem vazamento de comando). Evidências em `evidencias/` no workspace.

### Próxima tarefa concreta

**Item 11 — o watchdog**, o único item do gate que posso levar a `TESTED` sem release, sem VM e sem sessão interativa:
temporizador por turno sem evento no host + registro do motivo no transcript, testado com um agente falso que aceita o
turno e nunca responde.
