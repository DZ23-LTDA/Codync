import type { BotDraft } from '@shared/models'
import { useStore } from '../../store/context'
import { SwitchRow } from './parts'
import { useTr } from '../../lib/use-language'

/**
 * Who beyond this computer reaches the bot: the user's other computers (while this one is in a
 * network) and the cloud assistant (while it's on). Each shows only from a host that has it.
 */
export function NetworkSwitches({ draft, set }: { draft: BotDraft; set: <K extends keyof BotDraft>(key: K, value: BotDraft[K]) => void }) {
  const tr = useTr()
  const network = useStore().network
  return (
    <>
      {draft.offeredToNetwork !== undefined && network?.joined ? (
        <SwitchRow
          title={tr('Offered to the network')}
          detail={tr('Bots on your other computers can see and message this bot.')}
          on={draft.offeredToNetwork}
          onChange={(on) => set('offeredToNetwork', on)}
        />
      ) : null}
      {draft.reachableByAssistants !== undefined && network?.assistants ? (
        <SwitchRow
          title={tr('Reachable by cloud assistants')}
          detail={tr('ChatGPT or Claude in your Codync account can see and message this bot.')}
          on={draft.reachableByAssistants}
          onChange={(on) => set('reachableByAssistants', on)}
        />
      ) : null}
    </>
  )
}
