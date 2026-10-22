import { useLayoutEffect, useRef } from 'react'
import type { Bot } from '@shared/models'
import { usePresence } from '../../components/Overlay'
import { WorkingIndicator } from './ChatRows'

/** Keep the activity and its space alive until the exit finishes, including a quick restart. */
export function WorkingTransition({ bot, visible }: { bot: Bot; visible: boolean }) {
  // Matches --working-duration; initial history appears without an entrance animation.
  const presence = usePresence(visible, 420, false)
  const lastActive = useRef(bot)
  useLayoutEffect(() => {
    if (visible) lastActive.current = bot
  }, [bot, visible])
  const active = visible ? bot : lastActive.current
  return (
    <div className={`chat-working ${presence.shown ? 'shown' : ''}`} inert={!visible} aria-hidden={!visible}>
      <div className="chat-working-clip">
        <div className="chat-working-content">
          {presence.mounted ? <WorkingIndicator bot={active} /> : null}
        </div>
      </div>
    </div>
  )
}
