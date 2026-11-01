# Primeiro uso — a parte que é do host

Medido em 01/11 com `host/tests/first_use_e2e.rs`: host real, agente de teste que responde na hora (para o modelo não
entrar na conta).

```
createBot                      4 ms
mensagem → primeira resposta  569 ms
total do host                 573 ms
```

O que são esses 569 ms: **o processo do agente nascendo e o handshake ACP** — o mesmo custo que apareceu como caso isolado no
teste de primeiro token (587–678 ms). Não é o modelo, não é a rede: é subir um processo.

## O que isso quer dizer para os três minutos

O roteiro de primeiro uso dá **três minutos** para a pessoa inteira: abrir o app, ver o agente, criar o bot, mandar a
mensagem, receber a resposta e aprovar uma ação.

O host gasta **573 ms** na parte que lhe cabe (criar o bot e conversar). O resto dos três minutos é humano: ler, digitar,
escolher — e é isso que o app tem de manter curto.

## O que não está neste número

- a **abertura do app** (2204 ms no renderer em dev — não é o app instalado);
- o **pareamento** (QR 3–75 ms · connect 13 ms · redeem 1 ms);
- a **aprovação de uma ação**, que depende de o agente pedir permissão — o teste de primeiro uso não cobre;
- e o **app instalado**, que continua em 2.33.2.

## Pendência

Um número por etapa já existe para o host; o item 5 só fecha quando alguém executar o roteiro no app instalado e disser
onde os três minutos foram gastos de verdade.
