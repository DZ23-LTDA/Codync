# B-3PO — goal ativo

> Este documento substitui integralmente qualquer goal anterior (v1, v3, v3.1) e o adendo v3.2.
> É o único goal ativo. **Conclusão = Release 1 publicado**; depois disso a série para.

## Texto recebido do dono (fonte de verdade, íntegro)

Este goal SUBSTITUI integralmente qualquer goal anterior (v1, v3, v3.1) e o adendo v3.2.
É o único goal ativo. Conclusão = Release 1 publicado; depois disso, pare.
Primeira ação: commitar este documento em `docs/plans/B3PO-GOAL.md` e reconciliar o PR #43, que está aberto agora.

## O que isto significa na prática

1. **Escopo**: apenas o Release 1 (o checklist de "pronto" já registrado no checkpoint). Nada do backlog das Fases C–G
   começa sem o dono editar este arquivo.
2. **Critério de parada**: Release 1 publicado — instalar, usar e atualizar sem terminal — e então a série encerra.
3. **Onde está o checklist verificável**: `docs/plans/B3PO-CHECKPOINT.md`, seção "Sessão de 31/10/2026, 21:20Z", com os
   números medidos e o que ainda **não** foi validado.
4. **Vocabulário**: o estado é "distribuição assistida (não assinada)". Não existe liberação de marca nem de arte.
5. **Regras de execução e armadilhas** continuam as do goal anterior (anti-fabricação, CI lida antes do merge, proibição de
   `git add -A` e de `Remove-Item -Recurse` no repositório, PRs pequenos para `dev`, nunca `main`).

## Estado no momento em que este documento é commitado

- `dev` em `b5916793` (checkpoint do goal v3) + PR #43 aberto (mensagem humana para banco corrompido).
- 271 strings fora da tradução; 13 mascotes com 0 arte commitada; `pt` em 100% da superfície `tr()`.
- CI do host verde nos três sistemas no último run lido (41252692797); empacotamento verde (41254036197).

## Nota de fidelidade

O dono enviou, nesta mensagem, apenas o texto acima citado — não um documento longo com seções. Este arquivo preserva esse
texto **literalmente** e não inventa seções que não vieram; se o dono enviar a versão completa, ela substitui este arquivo.
