import { CardSection } from '../../components/Controls'
import { Icon } from '../../components/Icon'
import { HStack, IconSlot, VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useModel } from '../../lib/observable'
import { useTr } from '../../lib/use-language'
import type { BotStore } from '../../store/bot-store'
import { useApp } from '../../store/context'

/**
 * The computers a computer can use. This one reaches the computers it manages over SSH; those
 * hold no connection back, so theirs stays empty.
 */
export function ComputersUsedSection({ store }: { store: BotStore }) {
  const tr = useTr()
  const app = useApp()
  const used = store === app.local ? app.managedStores.filter((s) => s !== store) : []
  return (
    <CardSection title={tr('{name} can use').replaceAll('{name}', store.hostName)}>
      {used.length === 0
        ? [<Text key="none" tone="secondary">{tr('None yet.')}</Text>]
        : used.map((computer) => <UsedComputerRow key={computer.computer.id} computer={computer} />)}
    </CardSection>
  )
}

function UsedComputerRow({ computer }: { computer: BotStore }) {
  const tr = useTr()
  useModel(computer)
  return (
    <HStack gap={8} className="settings-row settings-appear">
      <IconSlot width={18} tone="secondary">
        <Icon name="desktopcomputer" size={13} />
      </IconSlot>
      <VStack grow className="settings-stack">
        <span>{computer.hostName}</span>
        <Text size="caption" tone="tertiary">{`${tr('Over SSH')} · ${tr(computer.connectionLabel)}`}</Text>
      </VStack>
    </HStack>
  )
}
