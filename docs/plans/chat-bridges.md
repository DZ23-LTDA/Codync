# Pontes de chat: Slack, Discord, Telegram

Um bot do Codync hoje é alcançável pelo app, pela rede do Codync e por webhook de rotina. Falta o caminho inverso do produto: **a pessoa fala com o bot onde ela já está**, e a resposta volta para lá.

## O que já existe e serve de base

- `POST /hooks/routines/{id}` (`api/http.rs`) — entrada não autenticada por token, com verificação de assinatura dentro de `routines::hooks`. Já aceita um evento externo e dispara trabalho.
- `Hub` + `Cmd::Prompt`: o mesmo caminho que o app usa para mandar mensagem a um bot.
- Conectores MCP e o vault: credenciais de plataforma já têm onde morar.

## Desenho proposto

Duas peças, nesta ordem — a primeira já resolve metade dos casos.

1. **Resposta de saída** (`BotConfig.reply_webhook`): ao fim de um turno no chat do bot, o host faz `POST` do texto final para uma URL configurada, assinado com HMAC do segredo do bot. É o que faz um bot acionado por webhook (Telegram `setWebhook`, Discord, Zapier) **responder de volta** sem o host saber o que é Telegram.
2. **Entrada genérica** (`POST /hooks/bots/{secret}`): valida o segredo do bot (guardado no `kv`, mostrado por `bridgeInfo`), transforma o corpo `{text, from}` em mensagem do usuário e enfileira no ator. Cada plataforma precisa só de um shim de 20 linhas: o webhook do Slack/Discord/Telegram aponta para cá.

Depois disso, adaptadores específicos (Telegram por long polling, Discord por gateway) são otimização: evitam expor um endpoint público, que é o principal custo de operação numa máquina doméstica.

## Riscos

- **Endpoint público**: qualquer bridge aberta é superfície de ataque. Segredo longo, corpo limitado, nenhuma ferramenta de escrita liberada por esse caminho, e o mesmo gate de orçamento/selo dos outros turnos.
- **Laço**: um bot que responde para o canal que o acionou pode se realimentar. Precisa de marca de origem e limite por minuto.
- **Identidade**: mensagem de plataforma não é o usuário do Codync; o turno entra como origem `bridge`, nunca como o usuário, para não mexer na memória "da pessoa" sem querer.

## Primeiro PR

`reply_webhook` apenas: campo no `BotConfig`, envio assinado no fim do turno, teste com servidor local de fixture. Sem endpoint público, sem adaptador de plataforma — é o menor passo que já faz uma integração funcionar de ponta a ponta.
