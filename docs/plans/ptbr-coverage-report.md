# Cobertura pt-BR — medido, não estimado

Primeira das três medidas que o adendo v3.4 (§2) exige antes de qualquer melhoria de usabilidade. Comandos e saídas reais,
em 31/10/2026, `dev` em `65e3b5eb`.

## 1. Tabela compartilhada (iOS + Android + tabelas do desktop)

```
$ node tools/l10n-coverage.mjs
de        1468 keys  100.0% of de     0 missing    0 not in de
es        1468 keys  100.0% of de     0 missing    0 not in de
fr        1468 keys  100.0% of de     0 missing    0 not in de
ja        1468 keys  100.0% of de     0 missing    0 not in de
ko        1468 keys  100.0% of de     0 missing    0 not in de
pt         253 keys   15.4% of de  1242 missing   27 not in de
zh-Hans   1468 keys  100.0% of de     0 missing    0 not in de
zh-Hant   1468 keys  100.0% of de     0 missing    0 not in de
```

**pt-BR é a única língua incompleta da tabela compartilhada**: 253 de 1468 chaves (**15,4%**), com **1242 faltando** e
**27 chaves que não existem no alemão** (texto nosso, criado depois).

## 2. Tela do desktop

```
desktop shows 229 distinct phrases through tr()
pt       100.0% of the desktop screen     0 missing
```

Toda frase que passa por `tr()` no desktop **tem** entrada em pt. Isso não é cobertura de interface — é cobertura do
tradutor.

## 3. Texto que nunca chega ao tradutor

```
$ node tools/l10n-unwrapped.mjs
298 visible strings never reach the translation
```

**298 strings visíveis** (atributos `title`/`placeholder`/`aria-label` e texto JSX) **não passam por `tr()`** — logo não
têm como ser traduzidas, em nenhuma língua.

## 4. Leitura honesta dos três números juntos

| Camada | Número | O que significa |
|---|---|---|
| Chaves da tabela compartilhada | **253 / 1468 (15,4%)** | iOS, Android e o que o desktop usa da tabela estão majoritariamente em inglês |
| Frases do desktop via `tr()` | **100%** | o que já é traduzível está traduzido |
| Strings fora do `tr()` | **298** | não são traduzíveis hoje, independentemente da tabela |

Conclusão medida: **o problema não é tradução faltando no `pt.json` do desktop — é string que nunca foi envolvida (298) e
tabela compartilhada incompleta (1242 chaves)**. As duas coisas explicam a mistura de idiomas vista nas telas.

## 5. O que ainda falta medir (§2 do adendo)

1. **Primeiro uso em máquina limpa** (VM Windows sem Node nem agente): não medido — exige VM, não tenho uma neste ambiente.
2. **Efeito de um travamento**: parcialmente observado na prática (bot preso em "Waiting for your computer" e em
   "Starting the host…"), mas **sem turno real travado** com agente que não responde. Falta o teste com agente instalado.

Essas duas ficam como pendência declarada até haver ambiente; nenhuma meta do adendo vale sem elas.
