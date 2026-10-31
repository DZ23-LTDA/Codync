import { enUS, ptBR, type Catalog, type MessageKey } from "./catalog.ts";

export type Locale = "en-US" | "pt-BR";

const catalogs: Record<Locale, Catalog> = { "en-US": enUS, "pt-BR": ptBR };

let current: Locale = "en-US";

/** The language the renderer draws in; the desktop app keeps it in settings and calls this once at start-up. */
export function locale(): Locale {
  return current;
}

export function setLocale(next: Locale): void {
  current = next;
}

/** Whether `value` is one of the languages this build can draw in. */
export function isLocale(value: string): value is Locale {
  return Object.hasOwn(catalogs, value);
}

/**
 * The message for `key` in the current language, with `{name}` placeholders filled from `vars`. A missing translation
 * falls back to English rather than to the key: a half-translated screen reads as one language with an accent, not as
 * a screen full of identifiers.
 */
export function t(key: MessageKey, vars: Record<string, string | number> = {}): string {
  const template = catalogs[current][key] ?? enUS[key];
  return template.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in vars ? String(vars[name]) : whole,
  );
}
