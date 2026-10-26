import { useEffect, useState } from 'react'
import type { Network, NetworkComputer } from '@shared/network'
import { Button } from '../../components/Controls'
import { Dialog, Sheet } from '../../components/Overlay'
import { HStack, VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useModel } from '../../lib/observable'
import { useTr } from '../../lib/use-language'
import type { BotStore } from '../../store/bot-store'
import { network, refreshNetwork } from '../../store/network'
import { NetworkCard } from './NetworkCard'
import { InviteSheet, NetworkSetupSheet, RecoverSheet, RecoveryCodeSheet } from './NetworkSheets'
import { errorText } from './parts'
import { RuleSheet } from './RuleSheet'
import './network.css'

/** A sheet or dialog open for one network. */
type Open =
  | { kind: 'create' | 'join' }
  | { kind: 'invite' | 'recover' | 'rule' | 'leave'; net: Network }
  | { kind: 'remove'; net: Network; computer: NetworkComputer }

/**
 * A managed computer's networks of the user's computers (codync-network): one card per
 * network, with who is in it and who may message the bots here, and making or joining another.
 * Hidden on hosts without `networkView`. Membership is decided here, at the computer; phones
 * only see it.
 */
export function NetworkSection({ store }: { store: BotStore }) {
  const tr = useTr()
  useModel(store)
  const [open, setOpen] = useState<Open | null>(null)
  const [busy, setBusy] = useState(false)
  const view = store.network
  const manageable = store.connection.kind === 'online' && !store.mismatch

  // Opening the page reads it again: the host asks the other computers when what it heard is stale.
  useEffect(() => {
    if (manageable) void refreshNetwork(store)
  }, [store, manageable])

  if (!view) return null

  const run = (work: () => Promise<unknown>) => {
    setBusy(true)
    work()
      .catch((e: unknown) => store.setError(errorText(e)))
      .finally(() => setBusy(false))
  }
  const close = () => setOpen(null)
  const several = view.networks.length > 1
  const sheetFor = (kind: 'invite' | 'recover' | 'rule') => (open && open.kind === kind ? open.net : null)
  const invite = sheetFor('invite')
  const recover = sheetFor('recover')
  const rule = sheetFor('rule')
  const removing = open?.kind === 'remove' ? open : null
  const leaving = open?.kind === 'leave' ? open.net : null

  return (
    <VStack gap={14}>
      <HStack gap={6} className="settings-section-header">
        <Text as="h3" size="body" weight="semibold" tone="text">{tr('Network')}</Text>
        <Text as="span" size="caption" weight="semibold" className="beta-tag">{tr('Beta')}</Text>
      </HStack>
      {view.networks.map((net) => (
        <NetworkCard
          key={net.id}
          net={net}
          several={several}
          disabled={busy || !manageable}
          act={{
            invite: () => setOpen({ kind: 'invite', net }),
            recover: () => setOpen({ kind: 'recover', net }),
            refresh: () => run(() => network.peers(store)),
            leave: () => setOpen({ kind: 'leave', net }),
            promote: (c) => run(() => network.promote(store, net.id, c.name)),
            remove: (computer) => setOpen({ kind: 'remove', net, computer }),
            addRule: () => setOpen({ kind: 'rule', net }),
            setRules: (rules) => run(() => network.setRules(store, net.id, rules)),
          }}
        />
      ))}
      <VStack gap={10} className="settings-appear">
        {view.joined ? null : <Text size="callout" tone="secondary">{tr('Connect your computers so bots on one can message bots on another.')}</Text>}
        <HStack gap={8}>
          <Button kind="secondary" disabled={!manageable} onClick={() => setOpen({ kind: 'create' })}>Create network</Button>
          <Button kind="secondary" disabled={!manageable} onClick={() => setOpen({ kind: 'join' })}>Join with invite</Button>
        </HStack>
      </VStack>

      <Sheet open={open?.kind === 'create' || open?.kind === 'join'} onClose={close}>
        <NetworkSetupSheet key={open?.kind} store={store} joining={open?.kind === 'join'} />
      </Sheet>
      <Sheet open={invite !== null} onClose={close}>
        {invite ? <InviteSheet key={invite.id} store={store} net={invite} /> : null}
      </Sheet>
      <Sheet open={recover !== null} onClose={close}>
        {recover ? <RecoverSheet key={recover.id} store={store} net={recover} /> : null}
      </Sheet>
      <Sheet open={rule !== null} onClose={close}>
        {rule ? <RuleSheet key={rule.id} store={store} net={rule} /> : null}
      </Sheet>
      {/* Escape and the dim don't close it: only "I saved it" does. */}
      <Sheet open={store.recoveryCode !== null} onClose={() => {}}>
        {store.recoveryCode ? <RecoveryCodeSheet recovery={store.recoveryCode} done={() => store.setRecoveryCode(null)} /> : null}
      </Sheet>
      <Dialog
        open={removing !== null}
        title={tr('Remove {name} from the network?').replaceAll('{name}', removing?.computer.name ?? '')}
        message={tr("It can't come back with its current key. For a stolen computer, also remove the computers it invited.")}
        actions={
          removing
            ? [
                { title: 'Remove', destructive: true, action: () => run(() => network.remove(store, removing.net.id, removing.computer.name, false)) },
                { title: 'Remove with its invites', destructive: true, action: () => run(() => network.remove(store, removing.net.id, removing.computer.name, true)) },
              ]
            : []
        }
        onClose={close}
      />
      <Dialog
        open={leaving !== null}
        title={tr('Leave {network}?').replaceAll('{network}', leaving?.name ?? '')}
        message={tr('This computer stops being in {network}. The others keep listing it until the owner or an admin removes it there; rejoining needs a new invite.').replaceAll(
          '{network}',
          leaving?.name ?? '',
        )}
        actions={leaving ? [{ title: 'Leave', destructive: true, action: () => run(() => network.leave(store, leaving.id)) }] : []}
        onClose={close}
      />
    </VStack>
  )
}
