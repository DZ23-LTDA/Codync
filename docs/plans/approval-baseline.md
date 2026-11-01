# Aprovação de uma ação — o número que faltava

Medido em 01/11 com um teste próprio (`host/tests/approval_latency_e2e.rs`), host real e `fake_agent.py`, que pede
permissão em todo turno.

## O que foi medido

| Etapa | O que conta | Número |
|---|---|---|
| pedido | da mensagem enviada até o cartão de permissão aparecer no transcript | **602–644 ms** |
| resposta | do `respondPermission` até o turno terminar com a resposta do agente | **21–28 ms** |

## Por que importa

O goal (§13) diz que a aprovação humana é o centro do produto: nenhuma ação sensível acontece sem ela. Se o cartão demorar,
a pessoa fica olhando uma tela que não diz nada; se a resposta demorar, ela não sabe se aprovou ou não.

Este é o único passo do roteiro de primeiro uso que toca a regra mais importante do produto, e até agora não tinha número.


## Como ler

- **602–644 ms** para o cartão: é o processo do agente nascendo e o handshake ACP, o mesmo custo do primeiro token (569–678 ms).
  Não é o agente "pensando" — ele pede permissão antes de qualquer trabalho.
- **21–28 ms** do sim até o turno terminar: o caminho da aprovação em si (validar o cartão, liberar o agente, registrar o
  resultado). Esse é o número que mede a regra de governança, e ele é rápido.

## O que não está aqui

- a **decisão humana**: o tempo entre o cartão aparecer e a pessoa tocar — só o app instalado mede;
- a **ação aprovada rodando** de verdade (o agente de teste responde "pronto" depois do sim, não toca em arquivo);
- e o caminho pelo **celular**: aqui tudo é loopback, sem relay nem canal E2E.


## Duas execuções, para não apresentar um número só

```
1ª: message to card 644 ms · yes to finished turn 21 ms
2ª: message to card 602 ms · yes to finished turn 28 ms
```

O cartão varia com o processo do agente (o mesmo custo de 569–678 ms do primeiro token); a aprovação em si fica na casa das
dezenas de milissegundos.
