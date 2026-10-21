import { useEffect, useState } from 'react'
import { Icon } from './Icon'
import { LevelDots, type LevelSample } from './LevelDots'
import { Text } from './Text'
import './recording-pill.css'

/** m:ss since `startedAt`, updated every second. */
function useElapsed(startedAt: number) {
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(timer)
  }, [])
  const seconds = Math.max(0, Math.floor((now - startedAt) / 1000))
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}

/**
 * Dictation in progress, in place of the mic: stop, the time since it started, and dots that
 * follow the microphone's level. `color` null is the text color.
 */
export function RecordingPill({ startedAt, sample, onStop, color = null, className = '' }: {
  startedAt: number
  sample: () => LevelSample
  onStop: () => void
  color?: string | null
  className?: string
}) {
  const elapsed = useElapsed(startedAt)
  return (
    <span className={`recording-pill ${className}`}>
      <button type="button" className="recording-stop" aria-label="Stop dictating" title="Stop dictating" onClick={onStop}>
        <Icon name="stop.fill" size={11} scaled={false} />
      </button>
      <Text tabular tone="text">{elapsed}</Text>
      <LevelDots sample={sample} color={color} width={32} height={12} label="Listening" />
    </span>
  )
}
