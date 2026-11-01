# Performance baseline — host readiness

Medido em 31/10/2026, 22:35Z (01:35Z UTC), nesta máquina, com o host compilado em modo `debug` (perfil `dev`, sem
otimização). Serve de referência inicial: **números reais, método reproduzível, e o que ainda não foi medido escrito na
cara**.

## O que foi medido

**Tempo até o host responder `GET /health` com 200**, a partir do `Start-Process` de `codync-host serve --port <p>`, com
`CODYNC_HOME` apontando para um diretório vazio (primeira execução) e para o mesmo diretório depois (execuções seguintes).

| Execução | Data dir | Tempo até HTTP 200 |
|---|---|---|
| fria (diretório vazio) | novo | **334 ms** |
| quente (mesmo diretório) | existente | **587 ms** |
| quente (repetição) | existente | **589 ms** |

Interpretação honesta: a primeira execução é mais rápida porque ainda não há nada a reconciliar; nas seguintes o host
continua trabalhando depois de abrir a porta (registro no relay, verificação de agentes, lead bot), e o `/health` responde
enquanto isso. **O número medido é "aceita conexão", não "está pronto para o primeiro turno".**

## Hardware e ambiente

- Windows, host Rust em `debug` (não é build de release — um build otimizado deve ser mais rápido; isso **não** foi medido).
- MSVC Build Tools 14.44 + Windows SDK 10.0.26100.
- `CODYNC_HOME` em diretório dentro do workspace (o processo não consegue criar `%USERPROFILE%\.codync` neste ambiente —
  erro `creating C:\Users\lmpra\.codync-dev` registrado durante a tentativa).

## O que **não** foi medido, e por quê

- **Abertura da janela do app** (Electron) até primeiro conteúdo desenhado: exige a janela; não há automação de browser
  nesta sessão e ninguém com a tela aberta para cronometrar.
- **Primeiro token** e **p95 de mensagem**: exigem um agente real respondendo, com o bot configurado.
- **Build de release**: `cargo build --release` não foi executado nesta máquina.

## Como repetir

```powershell
$env:CODYNC_HOME = "<diretório vazio>"
$sw = [Diagnostics.Stopwatch]::StartNew()
Start-Process host\target\debug\codync-host.exe -ArgumentList 'serve','--port','19401'
# repetir GET http://127.0.0.1:19401/health até 200 e parar o cronômetro
```

O gate do Release 1 (item 8) pede abertura, primeiro conteúdo, primeiro token e p95. **Este documento cobre um quinto
disso**: o host aceitar conexão. Não é o item 8 fechado.
