# Ver a tela: o que já funciona e onde trava

Objetivo: o item 4, 5 e 7 do Release 1 precisam de inspeção visual. Este arquivo registra o caminho técnico testado —
para a próxima sessão não repetir a mesma investigação.

## O que funciona (verificado em 31/10/2026, 22:5xZ)

| Passo | Comando | Resultado |
|---|---|---|
| Playwright disponível | `npx --no-install playwright --version` | **1.64.1** (global, em `%APPDATA%\npm\node_modules`) |
| Bundle do desktop compila | `npm run build` em `apps/desktop` | **ok** com `TMP`/`TEMP` dentro do workspace |
| Binário do Electron | `node node_modules/electron/install.js` | **ok** (`path.txt` = `electron.exe`) |

**A pegadinha do build**: `electron-vite build` falha com `remove C:\Users\...\AppData\Local\Temp\esbuild-...: Acesso
negado` quando `TMP`/`TEMP` apontam para o perfil do usuário. Com `TMP`/`TEMP` dentro do workspace, o build passa em ~5 s
(`out/main`, `out/preload`, `out/renderer`).

## Onde trava

1. **`ELECTRON_RUN_AS_NODE=1` está herdado neste ambiente.** Com ele, o "Electron" roda como Node puro e o app morre em
   `TypeError: Cannot read properties of undefined (reading 'isPackaged')` — o `electron` do `require` é `undefined`.
   Precisa ser removido do ambiente **antes** de lançar.
2. Mesmo sem essa variável, o processo sai imediatamente com **código `-2147483645`** e nenhuma saída em stdout/stderr.
   É falha de inicialização do processo gráfico — a mesma família de bloqueio de sandbox que atinge a criação de arquivos
   fora do workspace. `Playwright._electron.launch()` responde `Process failed to launch!`.

## Consequência para o gate

Enquanto o processo gráfico não subir, **nenhuma inspeção visual pode ser feita por mim**. Não é falta de ferramenta: é o
ambiente que não inicia a janela. O que já existe e pode ser usado assim que a janela abrir:

- `apps/desktop/out/**` construído;
- script de captura com Playwright (janela → `page.title()`, `document.body.innerText`, screenshot em `evidencias/`);
- host rodando com `CODYNC_HOME` dentro do workspace e API autenticada por token.

## O que ainda não foi tentado

- Lançar o app **empacotado** (`npm run dist:win:dev`) em vez do bundle, caso o crash seja do modo desempacotado.
- Rodar com um desktop virtual (não disponível nesta máquina até onde foi verificado).

Registro honesto: sem a janela, itens 4, 5 (a metade do app), 6 (estados vazios na tela) e 7 (inspeção pt-BR) **não podem
sair de `NOT_STARTED`/`IN_PROGRESS`** por mim.
