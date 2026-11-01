# Inspeção visual — telas 1 e 2, com o que está errado

Capturas: `evidencias/desktop-01.png` e `evidencias/desktop-02.png` (workspace, fora do repositório).

## Tela 1 — boas-vindas

| Observação | Evidência |
|---|---|
| Título e corpo em **inglês** (*Your coding agents, as teammates.*) | texto do DOM + captura |
| As três opções de entrada são **botões** com rótulo no DOM: `Continue on this computer`, `Continuar com a Apple`, `Continuar com o Google` | `desktop-01-buttons.json` |
| Os dois botões sociais aparecem **sem texto visível** na captura, só um indicador circular | captura 1 |
| Já há **mistura de idiomas na mesma tela**: um rótulo em inglês, dois em português | captura 1 |

## Tela 2 — acesso ao computador (depois de "Continue")

| Observação | Evidência |
|---|---|
| Título **"Acesso ao computador"** em português | captura 2 |
| O resto em **inglês**: `Computer control`, `GitHub Star`, `Bots can use apps on this computer…`, `Waiting for your computer`, `Set up later`, `Permissions stay on this computer…` | captura 2 |
| Estado de espera honesto: **"Waiting for your computer"** com instrução de reiniciar o host | captura 2 |

## Conclusão sobre o item 7 (pt-BR)

A interface **não está traduzida por completo** e não está nem consistente dentro da mesma tela: duas frases em português
convivem com dez em inglês. Isso confirma, com imagem, o que a seção 11 do goal já avisava — **cobertura de `tr()` não é
cobertura de interface**: há frases dentro do `tr()` sem tradução no `pt.json` (ou com fallback para o inglês), e há
frases fora do `tr()`.

Isso é um **defeito medido**, não uma impressão: a lista de frases visíveis em inglês nas duas telas está acima, e cada uma
pode ser procurada no `pt.json`.

## O que ainda não foi verificado

- Se o app tem seletor de idioma e se ele muda essas frases (o caminho de configuração não foi aberto).
- Por que os dois botões sociais aparecem sem texto visível apesar de terem rótulo no DOM (contraste? sobreposição do
  indicador?).
- O host não estava conectado nesta sessão (`Waiting for your computer`), então o fluxo de primeiro uso **parou aqui** —
  criar bot e receber resposta continua não executado no app.

## Próximo passo

1. Abrir Settings → idioma, escolher pt-BR e repetir as duas capturas; a diferença mostra o que é falta de tradução e o
   que é falta de seleção.
2. Cruzar cada frase em inglês da lista acima com `apps/ios/Kit/Sources/CodyncKit/Resources/Languages/pt.json`.
3. Subir o host (`CODYNC_HOME` no workspace) antes de abrir o app, para o primeiro uso prosseguir.
