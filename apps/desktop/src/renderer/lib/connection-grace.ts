import type { Connection } from '../store/bot-mirror'

/** Long enough for a wifi blip to come back without the screen flashing "offline". */
export const DROP_GRACE = 5000
/** A first connection that fails is usually the host still starting: half a second of patience. */
export const INITIAL_GRACE = 1000

/**
 * How long `next` should be held back before it replaces `current`, or `null` to apply it at once.
 *
 * A drop from `online` waits out a wifi blip; a first connection that fails waits only for the host to finish
 * starting. Anything else -- coming up, changing meaning, going from offline to offline -- shows immediately,
 * because a person waiting on a screen should not be shown a stale one for no reason.
 */
export function heldGrace(current: Connection, next: Connection): number | null {
  const transient = next.kind === 'connecting' || next.kind === 'offline' || next.kind === 'computerOffline'
  if (!transient) return null
  if (current.kind === 'online') return DROP_GRACE
  const initialFailure = current.kind === 'connecting' && next.kind !== 'connecting'
  return initialFailure ? INITIAL_GRACE : null
}
