# Uma lacuna fechada: o scanner de sintaxe

Os quatro medidores anteriores leem **texto** (regex). Por isso cinco strings montadas com template literal
(`Remove ${bot.name}`, `Computers, ${text}. Filter conversations`) ficaram em inglês sem aparecer em número nenhum.

## O que existe agora

`tools/l10n-ast.mjs` lê a **árvore de sintaxe** (TypeScript, já dependência do renderer) e conta só onde a pessoa lê:

| Conta | Não conta |
|---|---|
| texto entre tags fora de `tr()` | `className`/`style`/`key` montados com template |
| `title`/`aria-label`/`placeholder`/… com literal | o que já está dentro de `tr(...)` |
| template literal em posição de filho JSX | comentário JSX |
| template literal em atributo de cópia | número, URL, chave |

`tools/l10n-ast.test.mjs` fixa exatamente esses cinco casos (`node --test tools/l10n-ast.test.mjs` → **5 passam**).

## O que ele encontrou de imediato

**5 defeitos**, todos em `title`/`aria-label` montados com interpolação:

- `Remove ${bot.name}` → **Remover {name}**
- `Add ${bot.name}` → **Adicionar {name}**
- `Delete ${bot?.name ?? 'bot'}?` → **Apagar {name}?**
- `About ${item.name}` → **Sobre {name}**
- `Voice by ${providerName(...)}` → **Voz de {name}**

## O número honesto, e o que ainda falta

Antes da correção: **66** achados. Depois: **61**. Dos 61, **32 são texto JSX** — frases quebradas em volta de uma
interpolação (`notes, over the` + valor + `-note cap…`) — e **23 são atributos com template**, dos quais uma parte é
legítima (nome de host + rótulo de conexão, que é dado, não cópia).

Ou seja: a ferramenta nova **não** prova cobertura zero. Ela prova que a categoria que estava invisível agora é contada,
e dá a lista exata para o próximo lote — que é onde o trabalho continua.

## Limite declarado

O scanner não segue variável: `const title = 'Foo'` usado depois em `title={title}` continua invisível. Isso precisa de
inferência de fluxo, não de árvore de sintaxe, e está fora do escopo deste passo.
