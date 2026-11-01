import { ProgressBar } from '../../components/Controls'
import { useTr } from '../../lib/use-language'
import './usage.css'

/** A capsule filled to `percent`: the provider's tint below 70%, then amber, red from 90%. */
export function UsageBar({ percent, tint }: { percent: number; tint: string }) {
  const tr = useTr()
  const fill = percent >= 90 ? 'var(--danger)' : percent >= 70 ? 'var(--warning)' : tint
  return (
    <ProgressBar
      value={Math.min(1, Math.max(0.02, percent / 100))}
      label={tr('{percent}% used').replaceAll('{percent}', String(Math.round(percent)))}
      fill={fill}
      className="usage-bar"
    />
  )
}
