import type { NetworkRule, NetworkView } from '@shared/network'
import type { BotStore } from './bot-store'

/**
 * A managed computer's networks (codync-network) through its host's loopback API: the calls
 * behind the Network section. Membership is decided at a computer, so only loopback works; the
 * `network` event keeps `store.network` current, and each change reads it again in case the
 * event is still on its way. `network` is a network's id.
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

/** An empty field is the host's default. */
const orNull = (text: string) => text.trim() || null

export const network = {
  /** Asks every computer in every network now. */
  peers: (store: BotStore) => changing(store, () => call(store, 'networkPeers', {}, SLOW)),
  /** Keeps the recovery code on the store until the user says they saved it, wherever they went meanwhile. */
  create: async (store: BotStore, networkName: string, name: string, connector: string) => {
    const made = await call<{ recoveryCode: string; computer: string; network: string }>(
      store, 'networkCreate', { network: orNull(networkName), name: orNull(name), connector }, SLOW,
    )
    await refreshNetwork(store)
    // The host may have numbered the name (`home-2`).
    const named = store.network?.networks.find((n) => n.id === made.network)?.name ?? made.network
    store.setRecoveryCode({ code: made.recoveryCode, network: named, computer: made.computer })
  },
  join: (store: BotStore, invite: string, networkName: string, name: string, connector: string) =>
    changing(store, () => call(store, 'networkJoin', { invite: invite.trim(), network: orNull(networkName), name: orNull(name), connector }, SLOW)),
  leave: (store: BotStore, network: string) => changing(store, () => call(store, 'networkLeave', { network })),
  invite: (store: BotStore, network: string) => call<{ invite: string; expiresAt: number }>(store, 'networkInvite', { network }),
  promote: (store: BotStore, network: string, computer: string) => changing(store, () => call(store, 'networkPromote', { network, computer })),
  remove: (store: BotStore, network: string, computer: string, cascade: boolean) =>
    changing(store, () => call(store, 'networkRemove', { network, computer, cascade })),
  recover: (store: BotStore, network: string, code: string) => changing(store, () => call(store, 'networkRecover', { network, code: code.trim() })),
  setRules: (store: BotStore, network: string, rules: NetworkRule[]) => changing(store, () => call(store, 'setNetworkRules', { network, rules })),
}
