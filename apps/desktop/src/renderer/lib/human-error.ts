/**
 * Text a person can read when something fails. A thrown value is not a message: `String(e)` shows "Error: ..." and a
 * host error can carry a path, a code or a chain of contexts. What a screen shows is one sentence, and what it keeps
 * for the log is the detail.
 *
 * Callers that need the detail for a log keep the original value; this is only for the string on screen.
 */
import { translate, type AppLanguage } from "./l10n.ts";

const MAX = 200;

const GENERIC = "Something went wrong. Try again — if it keeps happening, check that the host is running.";

/** The language saved in Settings (`prefs` cannot be imported here: it touches `window` when it loads). */
function savedLanguage(): AppLanguage {
  try {
    const saved = globalThis.localStorage?.getItem("appLanguage");
    return saved === null || saved === undefined ? "system" : (JSON.parse(saved) as AppLanguage);
  } catch {
    return "system";
  }
}

export function humanError(error: unknown, fallback = GENERIC): string {
  return humanErrorIn(savedLanguage(), error, fallback);
}

/** The same, for a caller that knows the language — a test, a background worker without a saved preference. */
export function humanErrorIn(language: AppLanguage, error: unknown, fallback = GENERIC): string {
  const raw = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  // The first line only: an anyhow chain or a Node stack arrives with the detail on the lines below.
  const first = raw.split("\n")[0]?.trim() ?? "";
  // A thrown value with no message tells a person nothing, so the caller's sentence is better.
  if (first === "" || first === "undefined" || first === "null") return translate(fallback, language, navigator?.language ?? "en");
  // Strip the "Error: " prefix a wrapped value keeps.
  const withoutPrefix = first.replace(/^(?:Error|TypeError|RangeError|SyntaxError):\s*/i, "");
  return withoutPrefix.length > MAX ? `${withoutPrefix.slice(0, MAX - 1)}…` : withoutPrefix;
}


