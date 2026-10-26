import { useState, type ReactNode } from 'react'
import { CardSection, Reveal } from '../../components/Controls'
import { Icon } from '../../components/Icon'
import { IconSlot, VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useModel } from '../../lib/observable'
import { useTr } from '../../lib/use-language'
import type { BotStore } from '../../store/bot-store'
import { useApp } from '../../store/context'

/** The computers this computer can use (the ones it manages over SSH); one unfolds to its full settings. */
export function ComputersUsedSection({ store, details }: { store: BotStore; details: (computer: BotStore) => ReactNode }) {
  const tr = useTr()
  const app = useApp()
  const [open, setOpen] = useState<string | null>(null)
  const used = app.managedStores.filter((s) => s !== store)
  return (
    <VStack gap={16}>
      <CardSection title={tr('{name} can use').replaceAll('{name}', store.hostName)}>
        {used.length === 0
          ? [<Text key="none" tone="secondary">{tr('None yet.')}</Text>]
          : used.map((computer) => (
              <UsedComputerRow key={computer.computer.id} computer={computer} open={open === computer.computer.id}
                toggle={() => setOpen(open === computer.computer.id ? null : computer.computer.id)} />
            ))}
      </CardSection>
      {used.map((computer) => (
        <Reveal key={computer.computer.id} show={open === computer.computer.id}>{details(computer)}</Reveal>
      ))}
    </VStack>
  )
}

function UsedComputerRow({ computer, open, toggle }: { computer: BotStore; open: boolean; toggle: () => void }) {
  const tr = useTr()
  useModel(computer)
  return (
    <button className="settings-row settings-appear" aria-expanded={open} onClick={toggle}>
      <IconSlot width={18} tone="secondary">
        <Icon name="desktopcomputer" size={13} />
      </IconSlot>
      <VStack grow className="settings-stack">
        <span>{computer.hostName}</span>
        <Text size="caption" tone="tertiary">{`${tr('Over SSH')} · ${tr(computer.connectionLabel)}`}</Text>
      </VStack>
      <Icon name={open ? 'chevron.up' : 'chevron.down'} size={10} color="var(--secondary)" />
    </button>
  )
}
