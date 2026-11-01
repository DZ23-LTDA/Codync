# B-3PO — checkpoint de continuidade

> Fonte de direção: **goal v3.2** (`docs/plans/B3PO-GOAL.md`). Este arquivo é o checkpoint da seção 5: estado medido,
> evidência, o que não foi validado e por quê.

## Sessão de 31/10/2026, 22:10Z

`dev` = `feaf8c73`. PRs abertos: **0**. Árvore limpa (só `docs/plans/preview-window.md`, de outra sessão, intocado).

### Gate do Release 1

| # | Item | Estado | Evidência |
|---|---|---|---|
| 1 | Scanner de i18n corrigido e recontado | IMPLEMENTED + TESTED | PR #46 · `314` strings (244 atributos + 70 texto) · 10/10 testes |
| 2 | Release do fork separado do upstream | IMPLEMENTED + TESTED | PR #47 · guarda aborta fora de `DZ23-LTDA/Codync`, antes da rede |
| 3 | Banco em todos os estados | PARCIAL | PRs #43, #48, #50 · `store::schema` 9/9 · falta registrar "downgrade não suportado" |
| 6 | Erros humanos | IN_PROGRESS (medido) | PRs #53–#57 · scanner: **0 stacks, 0 valores crus** · frases não lidas, estados vazios não auditados |
| 4, 5, 7, 8, 9 | instalação, primeiro uso, pt-BR visual, performance, release | NOT_STARTED / parcial | ver `release-state.json` |

### O que **não** foi validado

- **Instalação limpa, atualização, rollback** nos três sistemas: nenhuma tentativa.
- **Primeiro uso, QR, reconexão, desinstalação**: exige app aberto.
- **Inspeção visual** de qualquer tela: nenhuma nesta série.
- **Tempo de abertura do app**: NÃO MEDIDO.
- **iOS/Android**: `BLOCKED_BY_EXTERNAL_DEPENDENCY` (sem Xcode, sem Android SDK 37).

### Bloqueio de infraestrutura (afeta a regra 16)

**O GitHub não cria runs para PRs novos** desde `41252692797` (21:11Z). Verificado em `host/**` (#43, #48) e `apps/**`
(#53–#57), com workflows `active` e `allowed_actions: all`. Consequência: os últimos merges usaram **verificação local**
(`tsc`, `cargo test`, scanners) com o desvio declarado no PR. **Nenhum item pode ser `VERIFIED`** enquanto isso durar.

Achado registrado: a run do **#38** terminou com `windows-latest | failure` — o PR foi mergeado com a run em andamento e
ela vermelha.

### Números medidos

- strings fora da tradução: **314**
- stacks no renderer: **0** · valores crus impressos: **0**
- testes: `store::schema` **9/9** · `l10n-unwrapped` **10/10** · `error-messages` **4/4** · `human-error` **5/5**
- tempo de abertura: **não medido**

### Próxima tarefa concreta

Auditar **estados vazios e offline** no renderer com um scanner próprio (mesmo padrão de `error-messages.mjs`): medir
quantas telas que podem ficar vazias têm estado vazio hoje, antes de escrever qualquer uma. Sem isso, o item 6 continua
sem a segunda metade que o gate pede.
