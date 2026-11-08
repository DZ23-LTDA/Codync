# VISUAL-01 — inventário visual do B-3PO (estado em `7037e8fd`)

Auditoria que o adendo visual manda fazer **antes** de implementar. Tudo aqui foi lido do código e dos assets, não de
impressão.

## 1. O que existe hoje, por cliente

| Cliente | Marca na tela | Mascotes | Tema | Componentes de UI |
|---|---|---|---|---|
| desktop (Electron/React) | marca no estado vazio (#226); ícone do app é a marca | **nenhum** — `CharacterAvatar` desenha formas procedurais | escuro; tokens em CSS (`--surface`, `--text`, …) | `apps/desktop/src/renderer/components/` (Button, Sheet, Dialog, AnchoredMenu, Controls, Icon) |
| Android (Compose) | wordmark no boas-vindas (#228) | **nenhum** — `CharacterAvatar` procedural | escuro; `Palette`/`Type` em `design/` | `apps/android/design/` (`CodyncSheet`, `ModalHeader`, `CodyncDialog`, `DropdownMenu`, `Controls.kt`, `Symbol`) |
| iPhone (SwiftUI) | wordmark no boas-vindas (#228, **não compilado**) | **nenhum** — `CharacterAvatar` procedural | `Palette` em `CodyncKit` | `apps/ios/Kit/Sources/CodyncUI/Controls.swift`, `Chrome.swift` |
| TUI (ratatui) | o nome **B-3PO** no estado vazio (#228) | não se aplica | cores do terminal | `host/src/tui/view/` |

## 2. Assets existentes

| Arquivo | Tamanho | Alfa | Uso |
|---|---|---|---|
| `assets/brand/logomarca-b3po.png` | 1536×1024, 1,2 MB | **não** (24bpp RGB) | fonte da marca |
| `assets/brand/b3po-mark-512.png` | 512×512 | **não** | símbolo |
| `assets/brand/b3po-icon-1024.png` | 1024×1024, 649 KB | **não** | ícone do app (desktop/Android/iOS) |
| `assets/mascots/*.png` | 13 × 512×512 | **sim**, fundo transparente | adicionados nesta entrega; antes a pasta tinha só README+NOTICE |

## 3. Problemas encontrados (com causa raiz)

### P1 — a logo aparece num retângulo preto sobre fundo claro

**Causa raiz:** **nenhum** asset de marca tem canal alfa. Os quatro são `Format24bppRgb`, com o fundo preto da arte
gravado nos pixels. Num fundo claro, o retângulo preto é o próprio PNG.

Provado com amostragem de pixels (1640 amostras em `logomarca-b3po.png`, `comAlpha=False`, 0 pixels transparentes).

**Não se corrige com chroma-key:** o robô tem viseira preta; remover "o preto" apaga o personagem. O adendo proíbe
exatamente isso.

**Saídas possíveis:**
1. Exportar a arte com alfa de verdade (precisa do arquivo-fonte, que não temos).
2. Tratar o retângulo como **placa de logo** deliberada (cantos arredondados, elevação) — legítimo, mas é uma decisão de
   arte, não um conserto.
3. Usar o **mascote** (que tem alfa) no lugar da logo em fundos claros.

### P2 — o engine de avatares não está ligado ao produto

O pacote `dz23-bot-avatar-engine` traz **13 PNGs 512×512 com fundo transparente**, exatamente os 13 ids de
`host/src/agent/avatar.rs`. Eles agora estão em `assets/mascots/` e derivados para os quatro clientes por
`tools/mascot-assets.mjs --derive`.

**Mas nenhum cliente os desenha ainda:** todos usam `CharacterAvatar` (formas procedurais). Ligar isso é VISUAL-04/05.

### P3 — a derivação quebrava o Android

`--derive` escrevia `steve-jobs.png`, `byte.png` etc. em `drawable-nodpi`. Nome de recurso Android não aceita hífen
**e** recusa palavra reservada Java:

```
byte.png: Error: byte is not a valid resource name (reserved Java keyword)
```

Corrigido em `tools/mascot-assets.mjs` (`clientFile`), com 5 testes novos. `:app:assembleDebug` passa.

### P4 — elementos da identidade anterior convivendo sem hierarquia

O onboarding do Android mostra o "Crew" (cinco blobs pontilhados) **e** a marca nova. Não há regra de qual manda. Decisão
pendente de arte (VISUAL-07).

## 4. O que NÃO existe (e o adendo pede)

- Família de avatares **desenhada** por papel (Architect, QA, …): hoje o avatar é forma+cor.
- Motor de animação com estados (IDLE, THINKING, WORKING, …): existe CSS que move a imagem inteira.
- Tema claro: os quatro clientes são escuros.
- Tokens compartilhados entre plataformas: cada cliente tem o seu `Palette`.
- Testes de componente/tema/responsividade.

## 5. Plano incremental (VISUAL-02 … 11)

| Etapa | Entrega | Depende de |
|---|---|---|
| VISUAL-02 | tokens oficiais (paleta, tipografia, espaçamento) e variantes de logo com alfa | arquivo-fonte da arte |
| VISUAL-03 | componentes compartilhados e estados | 02 |
| VISUAL-04 | mascote principal + variantes | 02 |
| VISUAL-05 | avatares por papel | 04 |
| VISUAL-06 | motor de animação ligado aos **eventos reais** do host | 04 + eventos |
| VISUAL-07 | onboarding | 02, 04 |
| VISUAL-08 | aplicar às telas inventariadas | 03, 07 |
| VISUAL-09 | paridade mobile/desktop/web | 08 |
| VISUAL-10 | QA: responsividade, acessibilidade, performance | 09 |
| VISUAL-11 | consolidação | 10 |

**Bloqueio externo declarado:** as variantes de logo com transparência real e uma família de avatares desenhada exigem
arte-fonte; o que existe no repositório são imagens rasterizadas sem alfa.
