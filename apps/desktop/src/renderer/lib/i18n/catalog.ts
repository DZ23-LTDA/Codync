// Every visible string the renderer shows, keyed by what it says rather than where it appears: one key, one meaning,
// so two screens that mean the same thing read the same in every language. `en-US` is the source: a key that exists
// here and has no translation falls back to English instead of showing nothing.
//
// Technical words stay as they are in Portuguese (bot, agent, run, harness, skill, MCP, token, workspace): translating
// them would make the screen harder to read for the people who use them daily.

export const enUS = {
  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.save": "Save",
  "common.retry": "Try again",
  "error.generic": "Something went wrong.",
  "error.offline": "The host isn't reachable.",
  "empty.bots": "No bots yet.",
  "bot.missing": "Bot {name} not found.",
} as const;

export type MessageKey = keyof typeof enUS;

export type Catalog = Record<MessageKey, string>;

export const ptBR: Catalog = {
  "common.cancel": "Cancelar",
  "common.close": "Fechar",
  "common.save": "Salvar",
  "common.retry": "Tentar de novo",
  "error.generic": "Algo deu errado.",
  "error.offline": "O host não está acessível.",
  "empty.bots": "Nenhum bot ainda.",
  "bot.missing": "Bot {name} não encontrado.",
};
