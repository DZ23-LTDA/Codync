# B-3PO — checkpoint de continuidade

> Fonte de direção: **goal v3** (Release 1 obrigatório; o resto listado e bloqueado por decisão). Este arquivo é o
> checkpoint operacional da seção 15: estado medido, evidência com URL/sha, o que não foi validado e por quê.

## Gate do Release 1 — estado em 31/10/2026, 22:40Z

`dev` = `d8c04119`. Nenhum PR aberto.

| # | Item do gate (seção 18) | Estado | Evidência |
|---|---|---|---|
| 1 | Scanner de i18n corrigido e recontado | **IMPLEMENTED + TESTED** | PR #46 · `node tools/l10n-unwrapped.mjs` → **314** (244 atributos + 70 texto) · 10/10 testes |
| 2 | Release do fork separado do upstream | **IMPLEMENTED + TESTED** | PR #47 · guarda aborta com `RELEASE_GATE_REPO=leepokai/Codync`, exit 1, antes de qualquer rede |
| 3 | Banco: inexistente, vazio, válido, corrompido, migração, interrupção, recuperação | **PARCIAL** | PR #43 (corrompido) + PR #48 (inexistente, vazio, válido, truncado) → `store::schema` **8/8**; falta downgrade e recuperação a partir de cópia |
| 4 | Instalação/atualização/rollback nos 3 sistemas | NOT_STARTED | — |
| 5 | Primeiro uso < 3 min, QR, reconexão, desinstalação | NOT_STARTED | exige app aberto |
| 6 | Erros humanos + estados vazios/offline | IN_PROGRESS | mensagem de banco corrompido (#43); resto não auditado |
| 7 | pt-BR: zero visível não traduzido + inspeção visual | IN_PROGRESS | base recontada em 314; **0 strings envolvidas**, 0 telas inspecionadas |
| 8 | Performance medida | NOT_STARTED | tempo de abertura **NÃO MEDIDO** |
| 9 | Checkpoint + bump + WhatsNew + tag | NOT_STARTED | — |

### Problema de infraestrutura em aberto (bloqueia a regra de CI)

O último run de **`codync-host`** em qualquer branch foi **41252692797** (21:11Z). Desde então, PRs que alteram
`host/**` (#43, #48) **não geraram run**, com workflow `active` e `allowed_actions: all`; os demais workflows
(`issue-triage`, `contributor`, `Release desktop`, `Release gate`) continuam disparando. Os dois PRs foram mergeados com
**verificação local** e a limitação declarada no PR — desvio registrado da regra 16 do goal.

Achado lido agora: a run **41251574838** (PR #38) terminou com `windows-latest | failure`, no teste de importação que o
#39 corrigiu. #38 foi mergeado com a run em andamento e ela vermelha.

### Números medidos

- **Strings fora da tradução: 314** (244 atributos + 70 textos) — scanner corrigido.
- **Testes que passaram**: `store::schema` **8/8**; `node --test tools/l10n-unwrapped.test.mjs` **10/10**.
- **Tempo de abertura do app: NÃO MEDIDO.**

### Próxima tarefa concreta

Fechar o item 3: teste de **recuperação** (copiar o arquivo danificado antes de tentar, abrir a cópia) e registrar
explicitamente que **downgrade não é suportado**. Depois, item 6: auditar as mensagens de erro que chegam ao renderer e
provar que nenhuma carrega caminho interno, stack ou token.
