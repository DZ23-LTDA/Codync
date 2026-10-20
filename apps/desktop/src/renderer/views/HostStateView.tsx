import type { ReactElement } from 'react'
import { CharacterAvatar } from '../components/Avatar'
import { Button } from '../components/Controls'
import { Icon } from '../components/Icon'
import { Text } from '../components/Text'
import { palette, rgbHex, useDark } from '../lib/theme'
import { useApp } from '../store/context'

const isMac = window.codync.platform === 'darwin'

/** What stands between this computer and the chat, with the one action that fixes it. */
export function HostStateView({ compact = false }: { compact?: boolean }) {
  const app = useApp()
  const state = app.host.state
  switch (state.kind) {
    case 'notInstalled':
      return <EmptyState compact={compact} icon="desktopcomputer" title={`Set up this ${isMac ? 'Mac' : 'computer'}`} message={`Codync runs your coding agents through the host, a small background service on this ${isMac ? 'Mac' : 'computer'}.`} action={['Install host', () => window.codync.host.install()]} />
    case 'missingBinary':
      return <EmptyState compact={compact} icon="desktopcomputer.trianglebadge.exclamationmark" title="The host is missing" message={window.codync.appScheme === 'codync-dev' ? 'Rebuild Codync Dev with its development host, then open it again.' : "This copy of Codync doesn't include codync-host. Download Codync again from codync.dev or GitHub."} />
    case 'failed':
      return <EmptyState compact={compact} icon="desktopcomputer.trianglebadge.exclamationmark" title="The host isn't running" message={state.message} action={['Try again', () => window.codync.host.restart()]} />
    default:
      return <EmptyState compact={compact} icon={<StartingBot size={compact ? 32 : 112} />} title="Starting the host…" message="This takes a few seconds." />
  }
}

/** The app-icon bot looking around while the host starts, like the phones' launch splash. */
function StartingBot({ size }: { size: number }) {
  return (
    <div className="starting-bot">
      <CharacterAvatar shape="squircle" color="" tint={rgbHex(palette(useDark()).text)} size={size} mood="working" />
    </div>
  )
}

/** A quiet placeholder for an empty screen: icon (an SF Symbol name or artwork), title, a line of help and an optional action. */
export function EmptyState({ icon, title, message, action, compact = false }: { icon: string | ReactElement; title: string; message: string; action?: [string, () => void]; compact?: boolean }) {
  return (
    <div className={`empty-state ${compact ? 'local-recovery' : ''}`}>
      {typeof icon === 'string' ? <Icon name={icon} size={compact ? 22 : 36} weight="light" color="var(--tertiary)" /> : icon}
      <Text as="div" size={compact ? 'callout' : 'title3'} weight="semibold" tone="text">{title}</Text>
      <Text as="div" size="callout" tone="secondary" className="empty-state-message">{message}</Text>
      {action ? (
        <Button kind={compact ? 'secondary' : 'primary'} onClick={action[1]} className="empty-state-action">
          {action[0]}
        </Button>
      ) : null}
    </div>
  )
}

