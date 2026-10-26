/** This computer's network of the user's computers (codync-network), as the host describes it. */

export type NetworkRole = 'owner' | 'admin' | 'member'

/** A computer in this computer's network, as last heard. */
export interface NetworkComputer {
  name: string
  role: NetworkRole
  /** This computer. */
  me: boolean
  online: boolean
  /** The bots it offers, by address name and display name. */
  agents: { id: string; name: string }[]
  /** What this computer may do to it; the host decides. */
  canRemove: boolean
  canPromote: boolean
}

/** `networkView`, the `network` event and the stream's hello: this computer's network, and cloud assistants. */
export interface NetworkView {
  joined: boolean
  computer?: string | null
  role?: NetworkRole | null
  /** What this computer may do; the host decides. */
  canInvite?: boolean
  canRecover?: boolean
  /** `public`, `private` or a relay URL. */
  connector?: string | null
  computers?: NetworkComputer[]
  assistants: boolean
  /** Cloud assistants can reach this computer at all: it's in a Codync account. */
  assistantsAvailable: boolean
}
