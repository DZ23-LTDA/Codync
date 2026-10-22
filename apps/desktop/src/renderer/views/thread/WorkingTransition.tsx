import { useLayoutEffect, useRef } from 'react'
import type { Bot } from '@shared/models'
import { usePresence } from '../../components/Overlay'
import { WorkingIndicator } from './ChatRows'

/** Keep the activity and its space alive until the exit finishes, including a quick restart. */
export function WorkingTransition({ bot, visible, thought }: { bot: Bot; visible: boolean; thought?: string }) {
  // Matches --working-duration; initial history appears without an entrance animation.
  const presence = usePresence(visible, 420, false)
  const lastActive = useRef({ bot, thought })
  useLayoutEffect(() => {
    if (visible) lastActive.current = { bot, thought }
  }, [bot, thought, visible])
  const active = visible ? { bot, thought } : lastActive.current
  return (
    <div className={`chat-working ${presence.shown ? 'shown' : ''}`} inert={!visible} aria-hidden={!visible}>
      <div className="chat-working-clip">
        <div className="chat-working-content">
          {presence.mounted ? <WorkingIndicator key={active.bot.startedAt} bot={active.bot} thought={active.thought} /> : null}
        </div>
      </div>
    </div>
  )
}
