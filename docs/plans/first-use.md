# Primeiro uso — o que já foi medido sem a janela

Registro do que foi exercitado no host **de verdade** (com o binário rodando e o token da API em mão), para o item 5 do
gate não depender apenas de impressão de tela.

## Método

```powershell
$env:CODYNC_HOME = "<diretório vazio>"
Start-Process host\target\debug\codync-host.exe -ArgumentList 'serve','--port','19503'
# token: <CODYNC_HOME>\token
# POST http://127.0.0.1:19503/api/hello   com  Authorization: Bearer <token>
```

## Resultado medido (31/10/2026, 22:41Z)

| Passo | Resultado |
|---|---|
| Host cria identidade e sobe | `GET /health` → 200 em **334 ms** (frio) |
| Autenticação da API | `POST /api/hello` com `Bearer <token>` → **200** |
| `POST /api/hello` completo | **2583 ms** (inclui inicialização de sessão após o start) |
| Catálogo de agentes | **48 harnesses** listados |
| Harnesses **instalados** nesta máquina | **4**: `opencode`, `copilot`, `hermes`, `dsh` |
| Harnesses com `version` / `auth` preenchidos | **0** — os campos vieram vazios no `hello` |

## O que isso prova, e o que não prova

**Prova**: o host sobe do zero, cria identidade, autentica a API por token e **detecta agentes instalados** — a primeira
etapa do item 5 ("detectar agente") funciona e é mensurável por comando, sem tela.

**Não prova**: que um bot criado responde. O fluxo `criar bot → mensagem → resposta → aprovar uma ação` **não foi
executado** nesta sessão. Também não foi verificado se `version`/`auth` vazios no `hello` são intencionais (o método
`agentAuth`/`agentSetup` existe separadamente) ou um campo não preenchido por defeito.

## Próximo passo concreto

1. Chamar `POST /api/agentSetup` para o backend `dsh` e conferir se `version`/`auth` aparecem.
2. Criar um bot pela API, enviar uma mensagem e medir **primeiro token** — isso fecha a metade "resposta" do item 5 no
   que o host controla.
3. Só a aprovação **no celular** e o QR exigem o app; essas ficam para uma sessão com a janela.

Este documento não é o item 5 fechado: é a evidência do primeiro terço dele, com o método que qualquer pessoa repete.
