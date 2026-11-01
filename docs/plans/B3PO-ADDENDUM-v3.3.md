# B-3PO — ADENDO v3.3: conectores, skills e custo externo

Complementa o GOAL v3.2 (`docs/plans/B3PO-GOAL.md`). **Não altera o gate do Release 1** (seção 18); acrescenta as regras
da camada de ferramentas e uma tarefa opcional fora do gate.

Registrado em 31/10/2026 a partir do texto do dono. Quando este arquivo divergir do texto colado, o texto colado vence até
a reconciliação.

## A. Inventário medido (31/10, nesta máquina) — verificado pelo agente

| Camada | Estado | Como foi medido |
|---|---|---|
| Agentes no catálogo ACP | **43** | `$HOME\.codync\registry.json` → 43 entradas |
| Skills instaladas | **48** | `$HOME\.codync\skills` → 48 pastas |
| Conectores MCP instalados | **0** | não existe `mcp.json` |

Catálogo não é instalação: agente disponível ≠ agente detectado; conector presente na lista ≠ conector com `tools/list`
respondido.

## B. Camadas de um bot

1. **Agente/harness** — o cérebro; um por bot, com modelo e parâmetros.
2. **Conectores MCP** — ferramentas externas; instalados **por computador**.
3. **Skills** — pastas com `SKILL.md` que entram no perfil do agente como instrução.
4. **Ferramentas embutidas** — `chat`, `memory`, `team`, `computer` (quando ligado).
5. **Rotinas** — cron por bot.
6. **Política e limites** — permissões, orçamento em token e em dinheiro, workspace.
7. **Canais** — `reply_webhook` (saída assinada) e `/hooks/bots/{secret}` (entrada).

## C. Regras de conector

1. **Catálogo antes de colar JSON.** Item do MCP Registry tem versão pinada e é atualizado pelo upstream; conector
   personalizado **nunca** se atualiza sozinho. JSON colado só para o que não estiver no catálogo.
2. **Credencial só no formulário do conector.** O host cifra `env`, headers e argumentos no vault (XChaCha20-Poly1305,
   chave no Keychain/Secret Service); o agente recebe um comando proxy com ambiente vazio. **Nunca** colar chave em chat,
   README, commit, log ou prompt.
3. **Instalar liga em todos os bots** — é o comportamento do produto. Depois de instalar, desligar nos bots que não
   precisam, sobretudo os pagos.
4. **Máximo ~5 conectores por bot.** Cada servidor entra na lista de ferramentas do agente; mais conectores piora a
   escolha de ferramenta e consome contexto.
5. **Um driver de browser por instalação.** Playwright e Chrome DevTools juntos = dois browsers, dois perfis, dois
   conflitos. Padrão: **Chrome DevTools**.
6. **Bot que atende cliente não recebe automação de browser.** Conteúdo de página é entrada não confiável; browser no
   mesmo bot que fala com cliente é vetor de injeção. Browser automation é para bot de desenvolvimento.
7. **Conector de prontidão duvidosa não fica instalado.** O check do host (90 s, `initialize` + `tools/list`) precisa
   passar. Conector listado e falhando não conta como capacidade do bot.

## D. Kit aprovado (4 MCPs reais; 1 da lista não é MCP)

| Nome | Pacote | Versão | Requisito extra |
|---|---|---|---|
| perplexity | `@perplexity-ai/mcp-server` | 1.3.0 | `PERPLEXITY_API_KEY` |
| firecrawl | `firecrawl-mcp` | 3.36.0 | `FIRECRAWL_API_KEY`; **Node ≥ 22** |
| chrome-devtools | `chrome-devtools-mcp` | 1.12.0 | Chrome/Chromium; Node ≥ 20.19 |
| playwright | `@playwright/mcp` | 0.0.84 | `npx playwright install chromium` |
| **HyperFrames** | — | — | **não é servidor MCP**; é CLI/framework de vídeo |

`heygen-com/hyperframes` **não** entra em `mcpServers`: não responde `initialize` e o check falha. Se for usado, é como
ferramenta/skill, não como conector.

```json
{
  "mcpServers": {
    "perplexity": {
      "command": "npx",
      "args": ["-y", "@perplexity-ai/mcp-server@1.3.0"],
      "env": { "PERPLEXITY_API_KEY": "pplx-COLE_AQUI" }
    },
    "firecrawl": {
      "command": "npx",
      "args": ["-y", "firecrawl-mcp@3.36.0"],
      "env": { "FIRECRAWL_API_KEY": "fc-COLE_AQUI" }
    },
    "chrome-devtools": {
      "command": "npx",
      "args": ["-y", "chrome-devtools-mcp@1.12.0", "--isolated"]
    }
  }
}
```

Trocar as chaves **no formulário do conector**, nunca neste arquivo.

## E. Efeito no trabalho em curso

Nada do Release 1 (seção 18 do v3.2) muda de prioridade: o gate continua sendo os 9 itens. Este adendo passa a valer para
qualquer tarefa que toque conectores, skills, browser automation ou custo externo — inclusive quando o item 5 (primeiro
uso) chegar em "detectar agente → criar bot".
