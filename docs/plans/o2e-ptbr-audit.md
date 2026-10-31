# O2-E — correção da auditoria: a tradução já existe, falta o português

## O que eu errei na primeira auditoria (#25)

O documento anterior dizia "não há i18n". **Está errado.** O desktop traduz hoje, e por um mecanismo maduro:

- `apps/desktop/src/renderer/lib/l10n.ts` — `translate(texto, preferência, sistema)` e o hook `useTr()`; a chave é o
  **próprio texto em inglês** (`tr('Something went wrong')`), com casamento exato e templating para placeholders.
- `apps/ios/Kit/Sources/CodyncKit/Resources/Languages/*.json` — a tabela de cada língua, **compartilhada com o iOS**
  (o desktop importa os mesmos arquivos). O Android, portado do iOS, consome o mesmo conjunto.
- Línguas hoje: `zh-Hans`, `zh-Hant`, `ja`, `ko`, `fr`, `de`, `es` — **sete, sem português**.

## O número que importa

`node tools/l10n-coverage.mjs` mede completude contra a maior tabela:

```
de        1468 keys  100.0%   0 missing   0 not in de
es        1468 keys  100.0%   0 missing   0 not in de
fr        1468 keys  100.0%   0 missing   0 not in de
ja        1468 keys  100.0%   0 missing   0 not in de
ko        1468 keys  100.0%   0 missing   0 not in de
zh-Hans   1468 keys  100.0%   0 missing   0 not in de
zh-Hant   1468 keys  100.0%   0 missing   0 not in de
```

Todas as sete tabelas têm **exatamente as mesmas 1.468 chaves**: são espelhos completos das frases usadas pelos apps.

## O que isso muda no plano

O critério de O2-E proíbe mistura de idiomas. Uma tabela `pt.json` parcial produz exatamente isso: uma tela em que
algumas frases estão em português e o resto cai no inglês pelo fallback. Então o português **não entra aos poucos na
interface**: ou a tabela está completa, ou a língua não aparece no seletor.

A entrega de O2-E passa a ser:

1. **`pt.json` com as 1.468 frases** traduzidas para pt-BR, no mesmo conjunto compartilhado (o que já serve iOS, Android
   e desktop de uma vez) — é trabalho de tradução, com revisão, não de código.
2. `pt` em `appLanguages`, `resolvedLanguage` e `catalogs` no `l10n.ts`, mais o teste de que `pt-BR` do sistema resolve
   para `pt`.
3. `node tools/l10n-coverage.mjs` deve mostrar `pt 1468 keys 100.0%` antes de a língua ser oferecida.
4. Formatação de data, hora e número por locale nas telas que mostram tempo.
5. Inspeção visual nas telas mais densas: português é ~20% mais longo e quebra layout que o inglês não quebra.

## Decisão

Não criei uma segunda infraestrutura de tradução. A tentativa de fundação própria (`lib/i18n/`) foi **removida** neste
PR: ela duplicava o `l10n.ts`, e duplicar o lugar onde as strings vivem é como a mistura de idiomas começa.
