# iPhone e Android — o que foi medido e o que continua aberto

Medido em 01/11 com `node tools/l10n-clients.mjs`. Duas perguntas, duas precisões.

## Números

| | iOS | Android |
|---|---|---|
| chaves que chegam ao tradutor | 235 | 124 |
| chaves **sem** entrada no `pt.json` | **0** | **0** |
| chaves montadas em runtime (interpolação, cron) | 27 | 1 |
| **literais indo direto para uma view** | **92** | **208** |

As chaves são exatas: a tabela tem a frase ou não tem. Os literais são **grosseiros** — a regex conta separadores e
valores formatados —, mas o piso é alto o bastante para dizer que as telas de celular não estão no estado do desktop.

## O que já foi corrigido

- **51 chaves** que chegavam a `L10n.tr(...)`/`tr(...)` sem entrada: renderizavam em inglês. PR #143.
- **13 literais** da folha de ícone personalizado (`CustomIconEditor.swift`): PR desta rodada.

## Limite desta máquina, declarado

O build do iPhone é macOS (`xcodebuild`); **esta máquina é Windows**. Nenhuma alteração Swift desta série foi compilada
aqui. O que foi verificado: a chave usada em `L10n.tr(...)` existe no `pt.json` (ferramenta), e a revisão do diff. O que
**não** foi verificado: que o arquivo compila e que a tela abre. Isso precisa de um Mac antes de qualquer release.

## Próximo lote

`CustomIconEditor` → `ArtifactView`/`ArtifactRenameSheet` → `BotDetailsView`/`BotEditorView` → `FallbackRows` →
`AgentModelPicker`. Cada um é uma tela pequena, e a regra continua: nada de traduzir sem ler a tela.
