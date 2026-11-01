# Roteiro de primeiro uso (item 5) — para executar quando o app instalar

O item 10 está bloqueado (o instalador não conclui nesta máquina, `install-blocked.md`). Este roteiro existe para que,
no minuto em que instalar, o teste do item 5 seja **execução**, não improviso.

## Pré-condição

| | |
|---|---|
| App instalado | `%LOCALAPPDATA%\Programs\codync-desktop\Codync.exe` com `ProductVersion 2.34.0` |
| Como conferir | `(Get-Item "$env:LOCALAPPDATA\Programs\codync-desktop\Codync.exe").VersionInfo.ProductVersion` |
| Se ainda disser 2.33.2 | **pare**: a instalação não aconteceu, e todo o resto do roteiro mede o código velho |

## Cronômetro

O gate pede **< 3 min sem ajuda**. Marque o tempo em cada passo e anote onde parou, mesmo que termine.

| # | Passo | O que deve aparecer (pt-BR) | Prova | Orçamento |
|---|---|---|---|---|
| 1 | Abrir o app | janela com a lista de bots vazia e a frase de estado vazio | print + ms até primeiro conteúdo | 5 s |
| 2 | O app detecta o agente | o agente aparece como **Instalado** em Extensões → Agentes | print da lista | 10 s |
| 3 | Criar um bot | nome, projeto e agente escolhidos; bot aparece na lista | print do bot na lista | 30 s |
| 4 | Mandar uma mensagem | a mensagem do usuário aparece e o bot entra em **Trabalhando** | print das duas bolhas | 10 s |
| 5 | Receber a resposta | texto do agente no transcript | print da resposta | até 60 s |
| 6 | Aprovar uma ação | cartão de permissão com **Aprovar** / **Negar**; após aprovar, a ação roda | print antes e depois | 30 s |
| 7 | Parear o celular (se houver iPhone) | QR na tela, leitura pelo app | print do QR | 20 s |
| 8 | Matar o app e reabrir | reconecta sozinho, sem pedir nada | print do estado após reabrir | 15 s |
| 9 | Desinstalar | sai de Programas, **sem serviço órfão** | `Get-Service | Where-Object { $_.Name -match 'codync' }` vazio | 30 s |

**Total sem os passos 6–7:** ~1 min 55 s. Com aprovação e QR: ~2 min 45 s — dentro dos 3 min, mas **sem margem**: se o
passo 4 demorar mais de 30 s, o limite já era.

## O que registrar em cada passo

- o **texto exato** que apareceu (não "abriu normal"), porque é onde idioma e estado vazio se provam;
- o **tempo** até o primeiro conteúdo e até a primeira resposta;
- qualquer coisa em **inglês** — o medidor cobre 0 strings em componente, mas o app instalado pode rodar outro bundle;
- qualquer erro com stack, caminho interno ou token (proibido pelo item 6).

## Modos de falha que este roteiro procura

| Sintoma | O que significa | Onde registrar |
|---|---|---|
| abre em inglês | o bundle instalado é outro | item 7 volta a `IN_PROGRESS` com prova |
| lista vazia sem frase | estado vazio ausente | item 6, tela 1 |
| nada em Extensões → Agentes | detecção de agente falhou | item 5, passo 2 |
| bot fica em **Trabalhando** sem resposta | timeout/watchdog | item 11 (já testado no host, não no app) |
| serviço fica depois de desinstalar | órfão | item 5, passo 9 |
| janela não abre | pior caso: o release não presta | item 4 |

## O que este documento não é

Não é prova de nada. É o teste **preparado** — e vale zero até alguém executar. Enquanto o app instalado disser 2.33.2,
o item 5 continua `NOT_STARTED`, independentemente de quantas linhas este arquivo tenha.
