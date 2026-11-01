# Marca — B-3PO / BOT PLATFORM

Identidade visual do produto da DZ23, aplicada sobre a base herdada do Codync. A fonte é
`assets/brand/logomarca-b3po.png` (1536×1024), entregue pelo dono; tudo o mais é gerado por
`tools/apply-brand-icons.py`, no workspace do agente.

## O que a marca diz

| Elemento | Valor |
|---|---|
| Nome | **B-3PO** |
| Descritor | **BOT PLATFORM** |
| Marca curta | a cabeça do robô (usada como ícone de app) |
| Campo | quase-preto `#000105` |
| Acento | ciano `#73C4EA`; azul profundo `#0454A0` |
| Metal | prata `#D6DBE6`, branco `#FCFDFE` |

## Onde já está aplicada

| Cliente | Arquivo | Como aparece |
|---|---|---|
| Android | `apps/android/app/src/main/res/drawable-nodpi/ic_launcher_foreground.png` + `values/launcher.xml` | ícone adaptativo: cabeça do robô sobre `#000105` |
| iOS | `apps/ios/Resources/Assets.xcassets/AppIcon.appiconset/image.png` | ícone 1024 (o appiconset aponta para essa imagem) |
| Desktop | `apps/desktop/resources/icon.png` | ícone que o empacotador converte em `.ico`/`.icns`/PNG |
| Site | `web/public/icon.png`, `web/public/apple-touch-icon.png` | favicon e ícone de tela inicial |
| Repositório | `assets/brand/` | logo original, ícones 1024/512 e a marca curta |

## O que **não** foi trocado, de propósito

Identificadores técnicos continuam os do Codync. Trocá-los sem plano de migração quebraria
instalações e atualizações existentes:

| Item | Valor mantido | Por quê |
|---|---|---|
| Bundle / applicationId | `com.pokai.Codync`, `com.pokai.Codync.android` | um id novo é outro app para macOS, Windows, Android e iOS: sem atualização in-place, sem continuidade de dados |
| Pasta de dados | `~/.codync`, `~/.codync-dev` | exigiria migrador de banco e dos agentes instalados |
| URL scheme | `codync://`, `com.pokai.Codync://` | está em QR codes, deep links e atalhos já distribuídos |
| Nome do pacote e produto empacotado | `Codync` | define pasta de instalação e nome do artefato que o updater procura |
| URLs e domínios | `codync.dev`, `mcp.codync.dev` | infraestrutura publicada e DNS |

O nome **exibido** (marca) é B-3PO; a **identidade de instalação** segue Codync até uma release com
migração planejada. Ver `docs/plans/` para o estado do gate.

## Como regenerar

```powershell
# no workspace do agente (o mesmo que guarda o logo original)
python tools/apply-brand-icons.py
```

O script recorta a cabeça do robô para o ícone adaptativo do Android dentro da zona segura de 66%,
gera os quadrados 1024/512/180 a partir do logo inteiro e regrava `assets/brand/`.

## Regras de uso

1. Não redesenhar, recolorir nem esticar o robô; só escala proporcional.
2. Sobre fundo claro, usar o ícone quadrado (que carrega o campo escuro); nunca o recorte da cabeça
   solto sobre branco.
3. O descritor vem sempre depois do nome, menor, com espaçamento entre letras.
4. Nada saturado sobre saturado: o contraste da marca vem de claro sobre quase-preto.
