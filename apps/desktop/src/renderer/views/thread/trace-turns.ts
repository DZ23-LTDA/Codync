import type { Entry } from '../../../shared/models.ts'

/** A bot's entries grouped by turn, oldest turn first, each turn's entries in their order. */
export function traceTurns(entries: Entry[]): [number, Entry[]][] {
  const turns = new Map<number, Entry[]>()
  for (const e of entries) turns.set(e.turn, [...(turns.get(e.turn) ?? []), e])
  return [...turns.entries()].sort((a, b) => a[0] - b[0])
}

/** The turn running now while the bot works in this chat; a queued message already holds the next turn. */
export function runningTurn(entries: Entry[], working: boolean): number | undefined {
  if (!working) return undefined
  return entries.reduce((t, e) => (e.data.status === 'queued' ? t : Math.max(t, e.turn)), -1)
}

/** Scrolled to (within a few pixels of) the end: new steps keep it there. */
export function scrolledToEnd(el: { scrollHeight: number; scrollTop: number; clientHeight: number }) {
  return el.scrollHeight - el.scrollTop - el.clientHeight < 8
}

/** A compact live preview never leaks thoughts from a previous turn or a queued request. */
export function latestThought(entries: Entry[]): string | undefined {
  const turn = runningTurn(entries, true)
  return entries.findLast((e) => e.turn === turn && e.kind === 'thought' && e.data.text?.trim())?.data.text
}
