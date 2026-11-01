# Paridade dos widgets — o que foi medido

Pergunta: os widgets das duas plataformas dizem a mesma coisa? Medido em 01/11.

## O que existe em cada lado

| Estado | iOS (`apps/ios/Widgets`) | Android (`CodyncWidgets.kt`) |
|---|---|---|
| sem computador pareado | `Connect Codync to your computer` · `Connect a computer` | `Connect a computer in Codync.` · `Open Codync to pair with your computer.` |
| sem uso | `No usage yet` | `No usage yet. Open Codync to refresh.` |
| sem bots | — (lista vazia some) | `Create your first bot in Codync.` |
| títulos de janela de uso | `L10n.tr(item.window.title)` | `phrase(limit.window.title)` |

Os dois lados **têm** os mesmos estados; o texto difere em detalhe, não em cobertura.

## O buraco que a medição achou

`tools/l10n-clients.mjs` **não varria `apps/ios/Widgets`**: só olhava `Kit/Sources`. Ou seja, o widget do iPhone — a tela que a
pessoa vê sem abrir o app — nunca entrou em número nenhum.

Agora entra. O que ele mostrou de imediato: **6 strings em inglês** (`Connect Codync to your computer` ×2, `Connect a
computer`, `No usage yet`, `End call`) e **2 chaves sem tradução** (`Open Codync to pair with your computer.`, `Open Codync to
connect a computer and check {provider} usage.`). Todas corrigidas.

## Limite

Kotlin e Swift continuam sem compilação nesta máquina (sem macOS, sem Android SDK). O que está verificado é que as chaves
existem no `pt.json` e o diff foi lido.


## Correção, no mesmo dia

A primeira versão desta nota dizia que o iOS **não tinha** estado "sem bots" e que o Android tinha. **Estava errado.**

Os dois calculam a mesma frase numa propriedade — `Needs you` · `Need you` · `Working` · `No bots yet` · `All quiet` — e
nenhum dos dois traduzia. Ou seja: a cobertura era igual; o que faltava era **tradução**, não estado.

Corrigido nos dois lados (`L10n.tr` no iOS, `phrase(...)` no Android), junto com `Failed` e `Ready` do Android. Nomes em
pt: **Precisa de você**, **Precisam de você**, **Trabalhando**, **Nenhum bot ainda**, **Tudo em paz**, **Falhou**, **Pronto**.

Lição: a diferença que eu vi era de **texto**, e eu a descrevi como diferença de **comportamento**. Uma nota de paridade
que erra isso manda o próximo agente "consertar" um estado que já existe.
