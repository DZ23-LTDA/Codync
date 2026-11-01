# B-3PO — checkpoint de continuidade

> Fonte de direção: **goal v3.2** + **adendos v3.3 e v3.4**. O gate do Release 1 tem **11 itens** (9 do v3.2 + 10 e 11 do
> v3.4).

## Sessão de 01/11/2026, 01:00Z

`dev` = `ce8a690d`. PRs abertos: **0**. Árvore limpa (só `docs/plans/preview-window.md`, de outra sessão, intocado).

### Versão instalada — a regra 0 do v3.4

| | |
|---|---|
| **App instalado** | **v2.33.2** (`ProductVersion 2.33.2.0`, arquivos de 30/10) — **não contém nada do `dev` de hoje** |
| **Release publicada** | **v2.34.0** — https://github.com/DZ23-LTDA/Codync/releases/tag/v2.34.0 · `codync-windows-x64.exe` (128,9 MB) |
| Instalar nesta máquina | **não concluiu**: 3 tentativas, `/S` nunca retorna. Não é UAC (instalação per-user), não é processo vivo, não é `TEMP`. **Bloqueio de ambiente**, declarado em `docs/plans/release-v2.34.0.md` |

### Gate (11 itens)

| # | Item | Estado | Evidência |
|---|---|---|---|
| 1 | Scanner de i18n | **IMPLEMENTED + TESTED** | PRs #46, #76, #102, #112 · **45** strings visíveis fora do `tr()`; o scanner passou a ver `footer`/`detail`/`empty`/`label`/`message`, a linha real do nó de texto e expressões JSX · 14 testes |
| 2 | Release do fork ≠ upstream | IMPLEMENTED + TESTED | PR #47 · guarda aborta fora de `DZ23-LTDA/Codync` |
| 3 | Banco em todos os estados | **TESTED** | PRs #43, #48, #50, #62 · 10 testes |
| 6 | Erros humanos | **IN_PROGRESS** | PRs #53–#57, #64, #73, #75, #91–#93, #101 · **0 chamadas de `humanError` no fallback genérico** (30 frases próprias, traduzidas); estados vazios: 36 no medidor heurístico, `NetworkSection` conferido e **tem** estado vazio — o medidor precisa de verificação na tela |
| 7 | pt-BR no escopo | **IN_PROGRESS** | PRs #71–#73, #95–#99, #101, #103–#105, #111, #113, #114, #116, #118, #121 · **0 de 656** frases `tr()` sem tradução (669 chaves) · **45** strings fora do `tr()`, com causa escrita em `docs/plans/i18n-leftovers.md` · 5 defeitos de idioma achados só na tela |
| 10 | Ciclo de entrega | **BLOCKED_BY_EXTERNAL_DEPENDENCY** | release 2.34.0 publicada; instalador construir OK; **3 tentativas de instalar não retornam** (nenhum processo, nenhum log) — `docs/plans/install-blocked.md`; ação do dono: rodar o .exe sem `/S` |
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

### Próxima tarefa concreta — watchdog do item 11

Ancorada no código de hoje (`3ef93bd6`), para não recomeçar a investigação:

| Peça | Onde |
|---|---|
| Laço periódico do ator (tick) | `host/src/agent/bot/mod.rs:382` (`flush_tick.tick()`) |
| Padrão de "prazo vencido → mata o processo" que já existe | `mod.rs:389-398` (`routine_deadline`, `cancel_deadline`) |
| Campos do ator | `mod.rs:118-133` — não há `last_event`; é o campo que falta criar |
| Eventos chegam por | `host/src/agent/bot/updates.rs` (é onde `last_event` seria tocado) |
| Cancelamento já registra no chat | `host/src/agent/bot/turn.rs:322-323` — `stop_notice("Stopped.", "cancelled")` |
| Metade já entregue no desktop | `apps/desktop/src/renderer/views/thread/long-turn.ts` + `WorkingIndicator` (aviso após 2 min) |

O que implementar: campo `last_event: Instant` tocado a cada `session/update`; no tick, se `turn.is_some()` e
`last_event.elapsed() >= STALL_AFTER`, marcar `stop_requested`, matar o processo (mesmo caminho do `cancel_deadline`) e
gravar o motivo real no turno. Teste: agente falso que aceita o turno e nunca responde.

**Paridade pendente e declarada**: o aviso de turno longo existe só no desktop; iOS/Android e TUI não receberam — a regra
de paridade do repositório exige a correspondente no mesmo tipo de entrega, e isso não foi feito.
