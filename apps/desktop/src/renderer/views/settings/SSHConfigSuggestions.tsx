import { useEffect } from 'react'
import { Button, IconButton } from '../../components/Controls'
import { Icon } from '../../components/Icon'
import { HStack, IconSlot, VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useTr } from '../../lib/use-language'
import { thisMac } from './parts'
import { useSSH } from './ssh-model'

/**
 * Under Over SSH: asks whether Codync may read `~/.ssh/config`, then lists the hosts in it that
 * aren't SSH computers yet, each with Add (`add` opens the editor with that host).
 */
export function SSHConfigSuggestions({ add }: { add: (host: string) => void }) {
  const tr = useTr()
  const ssh = useSSH()
  useEffect(() => void ssh.loadConfig(), [ssh])

  if (ssh.config?.access === 'ask') {
    return (
      <div className="settings-card settings-appear">
        <div className="settings-row">
          <IconSlot width={22} tone="secondary">
            <Icon name="magnifyingglass" size={13} />
          </IconSlot>
          <VStack grow className="settings-stack">
            <Text size="callout" weight="medium">{tr('Find computers in your SSH config?')}</Text>
            <Text size="caption" tone="secondary">
              {tr('Codync reads ~/.ssh/config on {name} to list its hosts. Nothing is sent anywhere, and nothing connects until you add one.').replaceAll('{name}', tr(thisMac))}
            </Text>
          </VStack>
        </div>
        <HStack align="stretch" gap={8}>
          <Button onClick={() => void ssh.answerConfig('allowed')}>Allow</Button>
          <Button kind="secondary" onClick={() => void ssh.answerConfig('denied')}>Not now</Button>
        </HStack>
      </div>
    )
  }
  return (
    <>
      {ssh.newConfigHosts.map((h) => (
        <div key={h.alias} className="settings-card settings-appear">
          <div className="settings-row">
            <IconSlot width={22} tone="secondary">
              <Icon name="terminal" size={13} />
            </IconSlot>
            <VStack grow className="settings-stack">
              <Text size="callout" weight="medium">{h.alias}</Text>
              <Text size="caption" tone="secondary">
                {[h.hostname && `${h.user ? `${h.user}@` : ''}${h.hostname}`, tr('In your SSH config')].filter(Boolean).join(' · ')}
              </Text>
            </VStack>
            <IconButton title="Add" icon="plus" onClick={() => add(h.alias)} />
          </div>
        </div>
      ))}
    </>
  )
}
