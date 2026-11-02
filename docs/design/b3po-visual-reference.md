# B-3PO — referência visual oficial e critérios de aceitação de UI/UX

O proprietário enviou duas imagens como **referência visual oficial** do B-3PO. Elas valem como **critério de
aceitação de UI/UX** para a Release 1, ao lado dos outros 11 itens do gate — sem mudar a arquitetura nem remover
funcionalidade do Codync.

| Arquivo | O que é |
|---|---|
| [`reference/b3po-logo.webp`](reference/b3po-logo.webp) | a marca: robô branco e preto, olhos em V, anel de luz azul, wordmark `B-3PO` com o traço azul no hífen, `BOT PLATFORM` embaixo |
| [`reference/desktop-empty-state.png`](reference/desktop-empty-state.png) | o app desktop **sem bot selecionado**: a marca ao centro, o título, a frase de apoio, o botão `New Bot`, a lista à esquerda e a linha da conta embaixo |

## O que a referência exige, item por item

### 1. A marca

| Critério | Como se verifica |
|---|---|
| O robô é branco/prata com viseira preta; os olhos formam um **V** | olhar a imagem ao lado do render |
| O anel de luz é **azul elétrico**, atrás do robô, cortado pelo corpo | idem |
| O wordmark diz `B-3PO`, com o hífen em **azul** | idem |
| Embaixo, `BOT PLATFORM` em caixa alta, espaçado | idem |
| Sobre fundo escuro, a marca não ganha contorno nem sombra extra | idem |

Os assets em [`assets/brand/`](../../assets/brand) já são esta família (`b3po-icon-1024.png`, `b3po-mark-512.png`,
`logomarca-b3po.png`). A referência **não pede arte nova**; pede que a arte existente apareça onde a pessoa está.

### 2. A tela vazia do desktop (o critério principal)

Da referência, na ordem vertical:

1. **A marca** ao centro, com largura de aproximadamente 27% da área de conteúdo (≈350 px num conteúdo de 1217 px),
   centralizada verticalmente no terço superior.
2. **Título**: `Your coding agents, as teammates.` — mesma frase que o app já usa
   ([`AccountWelcomeView.tsx`](../../apps/desktop/src/renderer/views/AccountWelcomeView.tsx)).
3. **Apoio**: uma frase que diz o que fazer — *"Pick a bot, or create one for each kind of work and point it at a
   project."*
4. **Ação primária**: um botão, **um só**, `New Bot`, logo abaixo do texto.
5. **Fundo**: preto/grafite chapado, sem gradiente chamativo, sem card com borda.
6. **Barra lateral**: lista de bots com **avatar redondo**, nome em uma linha e `harness · projeto` em cinza na
   segunda; ferramenta no topo (computador, busca, novo); **linha de conta** fixa embaixo com avatar, `Extensions` e o
   selo `BETA`.
7. **Nada de placeholder falso**: na referência, os quatro bots são exemplos. No app real, essa lista mostra os bots
   que existem — vazio, mostra o estado vazio.

### 3. Paridade (regra do repositório)

A mesma identidade tem de aparecer nos quatro clientes, adaptando o que não se aplica: desktop, iPhone, Android e
TUI. O que muda é o meio, não a marca nem a frase.

| Cliente | O que a referência pede |
|---|---|
| desktop | a composição acima, literalmente |
| iPhone / Android | a marca na tela de boas-vindas e no estado vazio, o mesmo título e a mesma frase de apoio, um botão primário |
| TUI | a marca não cabe em células de terminal; o estado vazio mostra o título e a instrução em texto, com o nome B-3PO |

## Diferença entre o que existe hoje e a referência

Levantado no código em `3ff0a6d7`, não por impressão:

| Item | Hoje | Referência | Ação |
|---|---|---|---|
| Marca na tela vazia do desktop | **não aparece**: o estado vazio mostra três avatares de personagem (`ChatWindow.tsx:357-361`) | a marca, ao centro | **feito** (#226): a marca substituiu os avatares |
| Título | `Your coding agents, as teammates.` já existe | igual | nenhuma |
| Apoio | `Your bots will…` / `Give each one a name and a project…` | "Pick a bot, or create one for each kind of work and point it at a project." | alinhar o texto |
| Botão | `New bot` | `New Bot` | capitalização |
| Selo `BETA` na linha de conta | conferir em `sidebar`/conta | presente | conferir |
| Marca no iPhone/Android/TUI | conferir por cliente | presente | adaptar |

Nada nesta lista exige mudar arquitetura, API, host ou persistência. É composição de tela e texto.

## Como isso entra no aceite

- O item 6 do gate (`human errors, empty and offline states`) passa a ter **este documento como critério visual**: a
  tela vazia só conta como aceita quando mostrar a marca, o título e uma ação, como a referência.
- O item 7 (`pt-BR no escopo liberado`) cobre as frases novas: elas precisam existir no `pt.json` como todas as
  outras.
- Uma mudança de UI entra nos quatro clientes na mesma entrega, como manda o `AGENTS.md`; onde não se aplica, o
  resumo do PR diz por quê.

### Como medir esta tela (e por que a captura engana)

O app roda com `devicePixelRatio` **1,5** nesta máquina, então uma janela de 1086 CSS px ocupa **1629 px** de tela. Se a
tela for menor que isso, o Windows corta a janela **sem avisar o renderer**: o layout continua correto, mas a captura
mostra só a parte esquerda.

Medido por dentro do app (console do renderer, `npm run dev`):

```
innerWidth 1086 · devicePixelRatio 1.5
.empty-detail  x:267  width:819  right:1086   ← encaixa exatamente no viewport
```

Ou seja: o painel de detalhe começa depois da barra lateral (267) e termina na borda (1086); o conteúdo é centralizado
nele. **A captura em `desktop-empty-state-with-mark.png` mostra a janela cortada pela tela, não um erro de layout.**
Para conferir com o olho, a janela precisa caber na tela — ou o zoom tem de descer até caber.