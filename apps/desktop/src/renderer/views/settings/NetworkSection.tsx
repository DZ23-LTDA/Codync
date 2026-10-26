import { useEffect, useState } from 'react'
import type { NetworkComputer } from '@shared/network'
import { Button, CardSection, IconButton } from '../../components/Controls'
import { Dialog, Sheet } from '../../components/Overlay'
import { HStack, VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useModel } from '../../lib/observable'
import { botsTemplate, presenceTitle, roleTitle } from '../../lib/network-copy'
import { useTr } from '../../lib/use-language'
import type { BotStore } from '../../store/bot-store'
import { network, refreshNetwork } from '../../store/network'
import { InviteSheet, NetworkSetupSheet, RecoverSheet, RecoveryCodeSheet } from './NetworkSheets'
import { errorText } from './parts'
import './network.css'

type Open = 'create' | 'join' | 'invite' | 'recover'

/**
 * A managed computer's network of the user's computers (codync-network): set it up, invite,
 * and who is in. Hidden on hosts without `networkView`. Membership is decided here, at the
 * computer; phones only see it.
 */
export function NetworkSection({ store }: { store: BotStore }) {
  const tr = useTr()
  useModel(store)
  const [open, setOpen] = useState<Open | null>(null)
  const [removing, setRemoving] = useState<NetworkComputer | null>(null)
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

  const accessory = view.joined ? (
    <HStack gap={2}>
      {view.canInvite ? <IconButton title="Invite a computer" icon="person.crop.circle.badge.plus" disabled={!manageable} onClick={() => setOpen('invite')} /> : null}
      {view.canRecover ? <IconButton title="Recover ownership" icon="key" disabled={!manageable} onClick={() => setOpen('recover')} /> : null}
      <IconButton title="Refresh" icon="arrow.clockwise" disabled={busy || !manageable} onClick={() => run(() => network.peers(store))} />
    </HStack>
  ) : null

  return (
    <>
      <CardSection title="Network" accessory={accessory} footer={view.joined ? 'Bots message the bots on these computers as bot@computer.' : null}>
        {view.joined
          ? (view.computers ?? []).map((c) => (
              <ComputerRow
                key={c.name}
                computer={c}
                disabled={busy || !manageable}
                promote={() => run(() => network.promote(store, c.name))}
                remove={() => setRemoving(c)}
              />
            ))
          : [
              <VStack key="setup" gap={10} className="settings-appear">
                <Text size="callout" tone="secondary">{tr('Connect your computers so bots on one can message bots on another.')}</Text>
                <HStack gap={8}>
                  <Button kind="secondary" disabled={!manageable} onClick={() => setOpen('create')}>Create network</Button>
                  <Button kind="secondary" disabled={!manageable} onClick={() => setOpen('join')}>Join with invite</Button>
                </HStack>
              </VStack>,
            ]}
      </CardSection>

      <Sheet open={open === 'create' || open === 'join'} onClose={() => setOpen(null)}>
        <NetworkSetupSheet key={open} store={store} joining={open === 'join'} />
      </Sheet>
      <Sheet open={open === 'invite'} onClose={() => setOpen(null)}>
        <InviteSheet store={store} />
      </Sheet>
      <Sheet open={open === 'recover'} onClose={() => setOpen(null)}>
        <RecoverSheet store={store} />
      </Sheet>
      {/* Escape and the dim don't close it: only "I saved it" does. */}
      <Sheet open={store.recoveryCode !== null} onClose={() => {}}>
        {store.recoveryCode ? <RecoveryCodeSheet code={store.recoveryCode} name={store.network?.computer ?? store.hostName} done={() => store.setRecoveryCode(null)} /> : null}
      </Sheet>
      <Dialog
        open={removing !== null}
        title={tr('Remove {name} from the network?').replaceAll('{name}', removing?.name ?? '')}
        message={tr("It can't come back with its current key. For a stolen computer, also remove the computers it invited.")}
        actions={
          removing
            ? [
                { title: 'Remove', destructive: true, action: () => run(() => network.remove(store, removing.name, false)) },
                { title: 'Remove with its invites', destructive: true, action: () => run(() => network.remove(store, removing.name, true)) },
              ]
            : []
        }
        onClose={() => setRemoving(null)}
      />
    </>
  )
}

function ComputerRow({ computer: c, disabled, promote, remove }: { computer: NetworkComputer; disabled: boolean; promote: () => void; remove: () => void }) {
  const tr = useTr()
  const bots = tr(botsTemplate(c)).replaceAll('{count}', String(c.agents.length))
  const detail = [tr(roleTitle(c.role)), tr(presenceTitle(c)), bots].join(' · ')
  return (
    <HStack gap={8} className="settings-row settings-appear">
      <span className={`network-dot${c.me || c.online ? ' online' : ''}`} aria-hidden />
      <VStack grow className="settings-stack">
        <Text>{c.name}</Text>
        <Text size="caption" tone="secondary" truncate title={c.agents.map((a) => a.name).join(', ') || undefined}>{detail}</Text>
      </VStack>
      {c.canPromote ? <Button kind="secondary" disabled={disabled} onClick={promote}>Make admin</Button> : null}
      {c.canRemove ? <IconButton title="Remove from the network" icon="trash" disabled={disabled} onClick={remove} /> : null}
    </HStack>
  )
}
