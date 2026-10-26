import type { NetworkView } from '@shared/network'
import type { BotStore } from './bot-store'

/**
 * A managed computer's network (codync-network) through its host's loopback API: the calls
 * behind the Network section. Membership is decided at a computer, so only loopback works; the
 * `network` event keeps `store.network` current, and each change reads it again in case the
 * event is still on its way.
 */

/** Joining asks the inviting computer; asking every computer waits for the ones that are away. */
const SLOW = 30_000

async function call<T>(store: BotStore, method: string, body: object = {}, timeout?: number) {
  return (await store.ready()).call<T>(method, body, timeout)
}

/** Reads the view again; the host asks the other computers in the background when it's stale. */
export async function refreshNetwork(store: BotStore) {
  try {
    store.setNetwork(await call<NetworkView>(store, 'networkView'))
  } catch {
    // A host without the network screens, or one that went away: the next connection reads it again.
  }
}

async function changing<T>(store: BotStore, work: () => Promise<T>): Promise<T> {
  const result = await work()
  await refreshNetwork(store)
  return result
}

export const network = {
  /** Asks every computer now. */
  peers: (store: BotStore) => changing(store, () => call(store, 'networkPeers', {}, SLOW)),
  /** Keeps the recovery code on the store until the user says they saved it, wherever they went meanwhile. */
  create: (store: BotStore, name: string, connector: string) =>
    changing(store, async () => {
      const { recoveryCode } = await call<{ recoveryCode: string }>(store, 'networkCreate', { name: name.trim() || null, connector }, SLOW)
      store.setRecoveryCode(recoveryCode)
    }),
  join: (store: BotStore, invite: string, name: string, connector: string) =>
    changing(store, () => call(store, 'networkJoin', { invite: invite.trim(), name: name.trim() || null, connector }, SLOW)),
  invite: (store: BotStore) => call<{ invite: string; expiresAt: number }>(store, 'networkInvite'),
  promote: (store: BotStore, computer: string) => changing(store, () => call(store, 'networkPromote', { computer })),
  remove: (store: BotStore, computer: string, cascade: boolean) => changing(store, () => call(store, 'networkRemove', { computer, cascade })),
  recover: (store: BotStore, code: string) => changing(store, () => call(store, 'networkRecover', { code: code.trim() })),
}
