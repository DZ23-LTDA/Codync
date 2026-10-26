/** What the Network section says about this computer's network (codync-network). */

import type { NetworkComputer, NetworkRole } from '@shared/models'

/** How computers reach each other without a direct path. */
export type ConnectorChoice = 'public' | 'private' | 'relay'

export const connectorChoices: { id: ConnectorChoice; label: string; detail: string }[] = [
  { id: 'public', label: 'Public relays', detail: 'Direct when it can, through a shared relay otherwise.' },
  { id: 'private', label: 'Private: direct only', detail: 'Same network, Tailscale or a VPN.' },
  { id: 'relay', label: 'Your own relay', detail: 'An iroh-relay you run.' },
]

/** The host's `connector` for a choice, or what's missing. An empty URL would quietly mean `public`. */
export function connectorFor(choice: ConnectorChoice, url: string): { connector: string } | { problem: string } {
  if (choice !== 'relay') return { connector: choice }
  const address = url.trim()
  return address ? { connector: address } : { problem: "Enter your relay's address." }
}

export const roleTitle = (role: NetworkRole) => ({ owner: 'Owner', admin: 'Admin', member: 'Member' })[role]

export const presenceTitle = (c: NetworkComputer) => (c.me ? 'This computer' : c.online ? 'Online' : 'Away')

/** "3 bots", untranslated template first: the caller translates, then puts the count in. */
export const botsTemplate = (c: NetworkComputer) => (c.agents.length === 1 ? '1 bot' : '{count} bots')

/** The owner and admins invite; whoever isn't the owner may take ownership with the recovery code. */
export const canInvite = (role: NetworkRole | null | undefined) => role === 'owner' || role === 'admin'
export const canRecover = (role: NetworkRole | null | undefined) => role !== 'owner'

/** "3:27 PM" for an invite's expiry (Unix ms). */
export const expiryTime = (expiresAt: number) => new Date(expiresAt).toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })
