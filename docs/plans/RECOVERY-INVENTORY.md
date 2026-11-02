# Inventário de recuperação — B-3PO / Codync

Gerado em 2026-11-01 (America/Sao_Paulo) sobre o fork `DZ23-LTDA/Codync`.

## 1. Artefatos congelados

| Artefato | Origem | Bytes | SHA-256 |
|---|---|---|---|
| `Codync-dev.zip` | `C:\Users\lmpra\Downloads\Codync-dev.zip` | 17.399.142 | `2afa23a06a27679579e72929463b1190c054a91e4421149085b39f775cf0a3e9` |
| `gitguard-scans.txt` | `C:\Users\lmpra\Downloads\gitguard-scans.txt` | 18.631 | `b03ef3844a5862865dc1e3afd02fccc0551e87059a2319dc596767e78a3539c9` |
| `dz23-bot-avatar-engine-completo.zip` | Downloads | 1.971.920 | `0d09c3a09de8565c44ba44b38e5e3c2c3543e76a2fc8d5d5ce0e7ad029a5e3b8` |
| `dz23-bot-avatar-engine-demo.html` | Downloads | 2.636.255 | `838f7f231873f8a4f86f10581ba28a07d1ef89446c8d7585e099d6df5b8745d1` |
| `logomarca-b3po.png` (marca) | anexo do dono | 2.241.985 | `a89606e96db51502eb1a8587007cc3e9255620f83d6dd098a8658c9de73f1b0d` |

Cópias de trabalho em `evidence/` e `brand/`. Os originais em `Downloads` não foram alterados.

## 2. Estado do remoto no início do ciclo

| Item | Valor |
|---|---|
| Branch padrão | `dev` |
| `dev` HEAD | `2e69b6c2c033f16ed7189198e6bc09493473da4e` — merge do PR #158 (`test/interruption-recovery`) |
| `main` HEAD | `7f33f3cb419a85e509b988b779856cdc4d5d2fab` — commit que o GitGuard escaneou |
| PRs abertos | 0 |
| Release mais recente | `v2.34.0`, 2026-11-01T02:10:44Z, `codync-windows-x64.exe` (135.113.208 bytes, não assinado) |
| Último push em `dev` | 2026-11-01T05:36:19Z |

`main` e `dev` divergem: o GitGuard escaneou `main`, os achados só valem para `dev` depois de
reproduzidos lá. O `npm audit` deste ciclo foi rodado em `dev`.

## 3. ZIP × `origin/dev` — resultado

Comparação por caminho e SHA-256 (hash bruto e hash com CRLF dobrado para LF, para o checkout
Windows não parecer alteração de conteúdo). Relatório completo em `zip-vs-head.md`, dados em
`zip-vs-head.csv`.

| Estado | Arquivos |
|---|---|
| idênticos | 3511 |
| só no ZIP | 0 |
| só no repositório | 0 |
| conteúdo diferente | 2 |

Os dois "diferentes" são `apps/screen-macos-tests/Sources/HostLink.swift` e
`.../SessionCandidate.swift`: no Git são symlinks para `../../screen-macos/*.swift`; o 7-Zip
recusou recriá-los ("Dangerous link path was ignored", alvo fora da pasta) e o ZIP não carrega o
conteúdo do link. O ZIP também traz o comentário `2e69b6c2…`, o mesmo HEAD.

**Conclusão: o ZIP é um snapshot fiel do `dev` em `2e69b6c2`. Não existe trabalho exclusivo do
ZIP, nada precisa ser recuperado dele, e o checkpoint que citava `0fc8e166` está desatualizado em
relação ao remoto.**

## 4. O que o checkout permite validar nesta máquina

Criado por `git clone` novo em `work/Codync` (nunca por cima do ZIP), apontando para o mesmo SHA.

| Alvo | Comando | Situação |
|---|---|---|
| Host (Rust) | `cargo build --workspace`, `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` | **verde**: compila, fmt e clippy limpos, 840 testes passando, 0 falhando, 2 ignorados, com rustup 1.99.0 **host GNU** (o repo fixa 1.99.0; a máquina não tem MSVC C++ nem admin para instalar) |
| Desktop | `npm run typecheck`, `npm test` | typecheck limpo; 345 testes, 326 passando, 0 falhando após corrigir os fixtures (ver §5) |
| Web | `npm ci`, `npm run build`, `npm run lint`, `npm run check:docs` | verdes |
| Cloud / relay / private-relay | `npm ci` | instalam; testes a rodar |
| iOS | `xcodebuild` | **impossível**: exige macOS + Xcode |
| Android | `./gradlew …` | exige Android SDK + emulador, ainda não instalado |
| Instalação do app publicado | `codync-windows-x64.exe` | continua `BLOCKED_BY_EXTERNAL_DEPENDENCY` desde o checkpoint: o instalador v2.34.0 não chegou a rodar; ação do dono |

## 5. Corrigido neste ciclo

| Achado | Correção | Evidência |
|---|---|---|
| 13 testes do desktop falhavam no Windows | os fixtures simulavam macOS/Linux com as regras de path do host; agora recebem as regras e os caminhos da plataforma simulada | `npm test`: 326 passando, 0 falhando (PR #160) |
| 2 testes do host falhavam e a suíte travava no Windows | prefixo de home no lugar da pasta de dados real; symlink exigindo privilégio; `stop()` chamando o `kill` do Unix, que não existe no Windows e deixava hosts órfãos segurando os pipes | `cargo test --workspace`: 840 passando, 0 falhando, 0 processo sobrando (PR #161) |
| `web/` com 1 critical e 9 high | `next` 16.2.1 → 16.4.3 (+ `@next/mdx`, `eslint-config-next`), `npm audit fix` sem `--force` | `npm audit`: 0 critical, 0 high; `next build` 43 páginas; lint e check:docs verdes (PR #159) |

## 6. Próximas ações concretas

1. Rodar `cloud`, `relay`, `private-relay` (testes + typecheck) e tratar suas dependências vulneráveis (`undici`, `ws`, `wrangler`, `@hono/node-server`).
2. Fechar a auditoria de dependências do `apps/desktop` (`electron`, `vite`, `undici`).
3. Rodar o site construído num navegador antes de pedir merge do PR #159.
4. Instalar o Android SDK e rodar `:core:test`, `:app:testDebugUnitTest`, `:assembleDebug`, `:lintDebug`.
5. Fixar as 68 GitHub Actions por SHA de commit (PR próprio, mexe em todo o CI).
6. Declarar `BLOCKED_BY_EXTERNAL_DEPENDENCY` para iOS e para a instalação do app, com a ação do dono em cada caso.

---

## 7. Onde as cópias vivem

Os artefatos congelados (ZIPs, o HTML de demonstração, a logomarca) ficam **fora do repositório**, em
`evidence/` e `brand/` na pasta de trabalho do projeto, com os SHA-256 acima. Não são commitados: o ZIP tem
17 MB e a licença dos binários não é do repositório. Os originais em `Downloads` não foram alterados.
