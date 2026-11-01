# As 45 strings que a automação não envolve — e por quê

Medido em `693bc7a5` com `node tools/l10n-unwrapped.mjs`. Cada uma foi olhada; nenhuma foi "deixada para depois" sem motivo.
O erro que não vou repetir: tentar envolver estas em lote. Já tentei, e o resultado foram dois `tsc` vermelhos e **zero**
string resolvida.

## 1. Listas em nível de módulo, traduzidas pelo consumidor (18 expressões)

`SettingsView.tsx`, `IslandWindow.tsx` e afins montam arrays assim:

```tsx
const pages = [{ id: 'computers', title: 'Computers & devices' }, …]
…
<Text>{tr(item.label ?? item.title)}</Text>          // o consumidor traduz
```

Envolver `tr('Computers & devices')` **aqui** põe um hook em escopo de módulo — foi exatamente o defeito que quebrou o
app no commit `1c2ec6b6`. E seria tradução dupla. **Estas não devem ser envolvidas: já estão traduzidas.**

Ação correta: ensinar o scanner a reconhecer "array de módulo cujo valor é consumido por `tr()`", precisando de um teste
que prove que continua pegando o caso não traduzido. É trabalho de ferramenta, não de PR de tradução.

## 2. Texto em nós JSX multilinha dentro de componentes sem tradutor (22 textos)

`Update now`, `Cancel`, `Done`, `Members`, `Switch back`, `No usage yet.`, `Loading…`… Cada uma está num componente que
**não** declara `const tr = useTr()`, e o script de escopo insere o tradutor na função errada (o nó está em JSX de um
`return` aninhado ou de um componente interno). O `tsc` responde `TS6133` (declarado e não usado) e `TS2304` (usado e não
declarado) — os dois lados do mesmo erro.

Ação correta: um por um, olhando o componente. São 22 pequenas edições, não um lote.

## 3. O que a medição NÃO diz

- não diz que a interface está em inglês: estes 45 são uma amostra de telas de configuração, memória, atualização e rede;
- não diz que o resto está certo: 0 de 656 frases `tr()` sem tradução é cobertura de **tabela**, não de tela.

## Próximo passo, na ordem

1. Fechar as 22 à mão (uma por vez, `tsc` entre elas).
2. Ensinar o scanner a reconhecer o caso 1, com teste — sem isso, o número nunca chega a zero honesto.
3. Só então declarar o item 7 do desktop fechado **depois** de abrir as telas no demo.


## Atualização: 84 strings de configuração (01/11)

`tools/l10n-missing.mjs` passou a ler também `title:`/`label:`/`detail:`/`empty:` literais — o texto que chega à tela
por uma lista de itens (`{ title: 'Support', … }` renderizado por `tr(item.title)`). O literal nunca passa por `tr()`,
então **uma chave faltando ali aparecia em inglês com todos os contadores em zero** — foi assim que `Support` ficou meses
em inglês no menu da conta.

Resultado: **84 strings de configuração sem entrada no `pt.json`**. Muitas são rótulos reais (`Black`, `Blue`, `Brown`,
`Allow`, `Approximate location`); outras são valores de unidade (`1 s`, `2.5 s`). É o quarto ponto cego do mesmo tipo:
o texto mora fora do JSX e fora de `tr()`.
