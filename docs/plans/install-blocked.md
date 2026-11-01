# Item 10 — por que o app instalado continua em 2.33.2

Estado em 01/11, com a árvore em `29c41766` (dev) e o instalador 2.34.0 já construído em
`apps/desktop/dist/codync-windows-x64.exe` (135 MB, 31/10 23:10).

## O que foi tentado, e o que aconteceu

| Tentativa | Comando | Resultado observado |
|---|---|---|
| 1 | `Start-Process … -ArgumentList /S` e 25 s depois ler o processo | o comando do harness **não retornou**; virou job em segundo plano |
| 2 | script `probe.ps1` com `WaitForExit(90000)` dentro de arquivo | **também não retornou**; sem arquivo de log escrito |
| 3 | inspeção por fora, em outro processo | nenhum `codync*`, `nsis*` ou `Un_A*` vivo; **versão instalada continua 2.33.2** |

Três tentativas, três vezes sem saída. O padrão não é "o instalador demora": é que **o processo que o inicia nunca
retorna** neste ambiente, e o instalador também não deixa rastro de processo vivo.

## O que se sabe de fato

- o binário instalado é **2.33.2.0** (`ProductVersion` de `%LOCALAPPDATA%\Programs\codync-desktop\Codync.exe`);
- o pacote em `dist/` é **2.34.0** (`apps/desktop/package.json`), com `codync-host.exe` e `Codync.exe` de 31/10;
- **nenhum** processo Codync/NSIS fica vivo depois;
- nenhuma escrita em `%LOCALAPPDATA%\Programs\codync-desktop` aconteceu (a versão não mudou).

## O que NÃO se sabe

Onde exatamente trava. Sem um processo observável e sem log do NSIS, qualquer afirmação sobre a causa seria invenção —
e já houve três "concluído" falsos nesta série por causa desse tipo de suposição.

## Classificação

`BLOCKED_BY_EXTERNAL_DEPENDENCY`, no sentido da seção 4 do goal:

- **o que falta**: uma execução do instalador que o harness consiga supervisionar (ou um log do NSIS);
- **por que é necessário**: o item 10 pede o ciclo entrega→instalado→usado, e a medição final é a versão instalada;
- **o que foi implementado**: o instalador 2.34.0 está construído e publicado como Release;
- **o que foi testado sem ela**: build (`dist` completo), app não empacotado rodando via `npm run dev`, superfície de
  tradução e testes (313 passam);
- **o que não pôde ser validado**: instalação, atualização in-app, rollback e desinstalação sem serviço órfão;
- **ação do dono que libera**: rodar `apps/desktop/dist/codync-windows-x64.exe` (sem `/S`, dois cliques) nesta máquina e
  dizer o que aparece. Com isso o ciclo fecha ou aparece o erro real.

## Alternativa testada e descartada

Copiar `dist/win-unpacked` sobre a instalação existente. Descartada por princípio: isso não é instalar — pula registro,
`install-host`, atalhos e a migração do serviço. Faria a versão parecer atualizada sem provar nada, que é o tipo de
"pronto" que esta série proíbe.
