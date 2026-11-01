# Inspeção visual — a primeira tela, vista de verdade

Em 31/10/2026, 22:4xZ, o app desktop **abriu** e a tela foi capturada por Playwright. Primeiro registro visual da série.

## Como foi possível

```powershell
cd apps/desktop
$env:TMP = "<workspace>\_tools\tmp"     # sem isso o esbuild morre em 'Acesso negado'
npm run build                            # out/main, out/preload, out/renderer
node tools/account-config.mjs dev        # sem resources/account-config.json o app morre em ENOENT
# Electron com --no-sandbox --disable-gpu: sem as duas flags o processo morre com -2147483645
# Playwright: _electron.launch({ executablePath: node_modules/electron/dist/electron.exe,
#   args: ["apps/desktop", "--no-sandbox", "--disable-gpu"] })
```

Captura fora do repositório: `evidencias/desktop-01.png` no workspace.

## O que a tela mostra

1. **É a tela de boas-vindas / primeiro uso**: título *Your coding agents, as teammates.* e o botão **Continue on this
   computer**. É o começo exato do fluxo do item 5.
2. **A interface está em inglês.** As frases em português ("Corrigir o teste de login instável", "Pronto. Era uma
   corrida…") são **conteúdo de conversa de exemplo**, não texto de interface. Para o item 7, o app abre fora do pt-BR.
3. **Dois botões abaixo do principal estão sem rótulo**: cinza, com apenas um indicador circular. Ou é carregamento
   permanente, ou botão sem texto — **candidato a defeito**, que só se decide com o app rodando.

## O que isso muda no gate

- Item 5 saiu de "sem evidência": a primeira tela é observável e reproduzível por comando.
- Item 7 ganhou o primeiro dado real: **a UI inicia em inglês**; falta ver se pt-BR existe e traduz tudo.
- Item 6 ganhou uma pista concreta: botões sem rótulo na primeira tela.

## Próximo passo

1. Trocar o idioma para pt-BR pela interface e capturar a mesma tela.
2. Investigar os dois botões vazios pelo snapshot de acessibilidade (`aria-label`/`title`).
3. Percorrer o primeiro uso e medir o tempo até a primeira resposta.
