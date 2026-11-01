# Pareamento — o tempo até o código aparecer

Medido em 01/11 com `host/tests/pairing_latency_e2e.rs`: cinco chamadas a `pairing` com o host real, contando de request a
resposta com o `pairingUrl` pronto.

## Antes

```
[2030, 2031, 2039, 2054, 2105] ms
```

**Dois segundos, toda vez.** A causa estava medida no código: `service::addresses()` roda
`tailscale status --json` — **um processo novo por chamada**, e no Windows isso custa ~2 s.

## Depois de ler o nome do Tailscale uma vez por processo

```
[3, 3, 3, 3, 2046] ms
```

**3 ms** nas chamadas seguintes. A primeira ainda paga ~2 s: é o processo do `tailscale` nascendo.

## O que isso significa para o item 5

Mostrar o QR é um passo do primeiro uso, e ele estava dois segundos parado — com o celular na mão. Agora o primeiro toque
depois de iniciar o host ainda paga ~2 s; os seguintes são instantâneos.

**Próximo passo, medido e não feito**: aquecer o nome do Tailscale na subida do host, em segundo plano, para que nem o
primeiro toque pague. Não fiz porque muda o caminho de inicialização e este é o lugar para decidir com calma, não no meio de
um lote de tradução.

## Um defeito de lint que só o clippy achou

Rodar `cargo clippy --all-targets -- -D warnings` reprovou **o meu próprio teste de primeiro token** (PR #132): três
`cast` de `f64`/`usize` que o crate recusa. Corrigido com aritmética inteira (`div_ceil`). O `cargo test` passava — o lint
não. Como o CI deste fork não roda, ninguém veria isso até alguém rodar clippy à mão.


## Aquecendo na subida do host

```
[3, 3, 3, 3, 1830] ms
```

O primeiro toque deixou de pagar a leitura do Tailscale: os quatro primeiros ficam em 3 ms. Mas sobrou **um caso lento por
execução**, sempre na **quinta** chamada — e ele **não** é o Tailscale:

- se fosse a leitura preguiçosa do nome, a primeira chamada pagaria, não a quinta;
- `issue()` só remove o código mais antigo quando passa de cinco, o que é uma operação de vetor;
- o valor (~1,8 s) é estável entre execuções, então não é ruído.

**Não sei o que é.** Fica medido, não explicado — e é o próximo passo antes de declarar o pareamento rápido.
