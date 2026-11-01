# O demo do site é um laboratório de inspeção visual

Descoberto nesta sessão e é o que faltava para o item 7: o renderer pode rodar **sem o host**, com um host de memória, e é
a mesma UI de produção.

## Como abrir

```powershell
cd apps/desktop
$env:TMP = "<workspace>\_tools\tmp"; $env:TEMP = $env:TMP
npx vite --config vite.demo.config.ts --port 5178 --strictPort
# http://localhost:5178/demo/desktop/
```

Playwright abre a URL, lê `document.body.innerText` e tira print. Ver `evidencias/demo-01.png`.

## O que a primeira captura mostra

A tela de conversa do desktop, com o host de memória:

| Elemento | Idioma |
|---|---|
| "Carregar mensagens anteriores" | **pt-BR** |
| "Ontem 23:41" (data relativa) | **pt-BR** |
| "Extensões" (rodapé lateral) | **pt-BR** |
| "Pergunte a todos · use @ para perguntar a um bot" | **pt-BR** |
| Mensagens e nomes dos bots | conteúdo do demo, não chrome |

**Zero chrome em inglês nesta tela.** Console do navegador: **0 erros**.

## Duas strings em inglês que a captura levantou, e o que são

1. **"Needs approval: Edit src/…"** no item da lista lateral — vem de `src/renderer/demo/fake-host.ts`, é **dado do
   demo**, não texto de interface. Não é defeito de produto.
2. **"Needs approval in System Settings"** em `tray-summary.ts:218` — texto de produção, e **não verifiquei** se passa
   pelo tradutor. Fica como suspeita até alguém olhar o caminho do menu da bandeja.

## Por que isso importa para os itens 6 e 7

Antes, inspecionar uma tela exigia o app com host instalado — que falha nesta máquina no registro `HKCU\...\Run`. O demo
remove essa dependência: qualquer tela do renderer pode ser aberta, lida e capturada em segundos. É o caminho para os
**36 estados vazios** e para a inspeção de pt-BR tela por tela, com prova visual.

## Limite honesto

O demo roda no Chromium, não no Electron: janela, bandeja, atalhos e integração com o sistema **não** são exercitados
aqui. Para isso continua sendo necessário o app instalado.
