/**
 * Text a person can read when something fails. A thrown value is not a message: `String(e)` shows "Error: ..." and a
 * host error can carry a path, a code or a chain of contexts. What a screen shows is one sentence, and what it keeps
 * for the log is the detail.
 *
 * Callers that need the detail for a log keep the original value; this is only for the string on screen.
 */
const MAX = 200;

export function humanError(error: unknown, fallback = "Something went wrong. Try again."): string {
  const raw = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  // The first line only: an anyhow chain or a Node stack arrives with the detail on the lines below.
  const first = raw.split("\n")[0]?.trim() ?? "";
  // A thrown value with no message tells a person nothing, so the caller's sentence is better.
  if (first === "" || first === "undefined" || first === "null") return fallback;
  // Strip the "Error: " prefix a wrapped value keeps.
  const withoutPrefix = first.replace(/^(?:Error|TypeError|RangeError|SyntaxError):\s*/i, "");
  return withoutPrefix.length > MAX ? `${withoutPrefix.slice(0, MAX - 1)}…` : withoutPrefix;
}
