# Banco: interrupção e recuperação

O gate (§18.3) pede o banco em sete estados: inexistente, vazio, válido, **corrompido, migração, interrupção, recuperação**.
Os primeiros tinham teste; interrupção e recuperação, não.

## O que foi exercitado

`host/tests/interruption_e2e.rs`:

1. sobe o host num home novo, cria dois bots;
2. **mata o processo sem shutdown** (`Child::kill`, sem flush algum — o equivalente a uma queda de energia);
3. sobe de novo no mesmo home;
4. exige que os dois bots estejam lá e que `history` responda;
5. exige que **nada** tenha sido posto de lado como corrompido;
6. mata de novo, sobe de novo, e exige que a lista tenha **o mesmo tamanho** (nem menor, nem duplicada).

```
cargo test --test interruption_e2e   → 1 passed
```

## O que o teste descobriu sobre a minha expectativa

Na primeira versão eu afirmei que restariam **2 bots**. O host cria, num home novo, **um bot principal** por conta própria — então
a contagem certa nunca era 2. O teste foi corrigido para comparar **estabilidade** (mesma lista depois de cada reinício) em vez
de um número mágico. O erro era meu, não do host.

## O que continua fora

- **interrupção no meio de uma escrita específica** (não no meio de uma operação qualquer): provocar isso de forma
  determinística exige instrumentar o arquivo, e não foi feito;
- **migração de esquema mais novo para mais velho** — há teste que recusa, mas não um que faça downgrade;
- **arquivo corrompido de verdade** lido por este teste: existe teste separado (PR #43) para ele, e o gate exige
  "avisa e não apaga", que não é verificado aqui.

Estado do item 3: `TESTED` para os sete caminhos **com estas lacunas nomeadas**, não `VERIFIED`.
