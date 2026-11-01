/** Um turno que passa disto está demorando mais que o normal: vale avisar a pessoa que ela pode parar. */
export const LONG_TURN_MS = 120_000

/** `true` quando o turno já dura mais que `LONG_TURN_MS`. Puro, para poder ser testado sem UI. */
export function isLongTurn(elapsedMs: number, threshold = LONG_TURN_MS): boolean {
  return elapsedMs >= threshold
}
