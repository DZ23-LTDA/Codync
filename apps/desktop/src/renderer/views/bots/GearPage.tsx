import { useCallback, useEffect, useState } from 'react'
import { draftOf, type BotDraft } from '@shared/models'
import { CardSection } from '../../components/Controls'
import { VStack } from '../../components/Stack'
import { Text } from '../../components/Text'
import { useTr } from '../../lib/use-language'
import type { AgentGearItem } from '../../store/bot-store'
import { useStore } from '../../store/context'
import { FormSheet } from '../marketplace/pieces'
import { SkillSheet, type SkillRef } from '../marketplace/SkillSheet'
import { PluginToggles } from './BotEditorView'
import { errorText, isValid, normalized, useFollowSaved, usePlugins } from './drafts'
import './bots.css'

/** The agent's own skills or connectors, then the Codync ones this bot passes on. */
export function GearPage({ botId, kind }: { botId: string; kind: 'skills' | 'connectors' }) {
  const tr = useTr()
  const store = useStore()
  const plugins = usePlugins(store)
  const [draft, setDraft] = useState<BotDraft | null>(null)
  const [gear, setGear] = useState<{ skills: AgentGearItem[]; connectors: AgentGearItem[] } | null>(null)
  const [failed, setFailed] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [viewingSkill, setViewingSkill] = useState<SkillRef | null>(null)
  const change = useCallback((update: (d: BotDraft) => BotDraft) => setDraft((d) => (d ? update(d) : d)), [])

  useEffect(() => {
    const bot = store.bots.get(botId)
    setDraft(bot ? draftOf(bot) : null)
  }, [botId])
  useFollowSaved(store.bots.get(botId), change)

  const saved = store.bots.get(botId)
  const cwd = draft?.cwd || saved?.cwd || ''
  const backend = draft?.backend ?? ''
  const agent = store.hello?.backends.find((b) => b.id === backend)?.name ?? backend
  const online = store.connection.kind === 'online'

  useEffect(() => {
    if (!backend || !online) {
      setFailed(!online)
      return
    }
    let live = true
    setFailed(false)
    setGear(null)
    void store
      .agentGear(backend, cwd)
      .then((value) => {
        if (live) setGear(value)
      })
      .catch(() => {
        if (live) setFailed(true)
      })
    return () => {
      live = false
    }
  }, [store, store.client, backend, cwd, online])

  const apply = (next: BotDraft) => {
    setDraft(next)
    if (!isValid(next) || !online) return
    void store
      .save(normalized(next))
      .then(() => setError(null))
      .catch((e: unknown) => setError(errorText(e)))
  }

  if (!draft) return null
  const items = (kind === 'skills' ? gear?.skills : gear?.connectors) ?? []
  const fromAgent = tr('From {agent}').replace('{agent}', agent)
  return (
    <div className="bot-form">
      <div className="bot-form-content">
        <Text size="footnote" tone="secondary">
          {tr('{agent} brings these. Codync adds its own on top, and those are what get passed to the agent.').replace('{agent}', agent)}
        </Text>
        <CardSection title={fromAgent}>
          {failed ? (
            <Text tone="danger">{tr("Couldn't load what this agent brings.")}</Text>
          ) : !gear ? (
            <Text tone="secondary">{tr('Loading…')}</Text>
          ) : items.length === 0 ? (
            <Text tone="secondary">{tr('None from this agent.')}</Text>
          ) : (
            items.map((item) => {
              const note = kind === 'skills' ? item.description : item.detail
              return (
                <VStack key={item.name} gap={2}>
                  <Text>{item.name}</Text>
                  {note ? <Text size="caption" tone="secondary">{note}</Text> : null}
                </VStack>
              )
            })
          )}
        </CardSection>
        {kind === 'skills' ? (
          <PluginToggles
            title="From Codync"
            empty="No skills yet. Get some from Extensions, or write your own."
            items={(plugins?.skills ?? []).map((s) => ({ id: s.id, name: s.name, detail: s.description }))}
            selection={draft.skills ?? []}
            disabled={!online}
            onChange={(ids) => apply({ ...draft, skills: ids })}
            onInfo={(s) => setViewingSkill({ id: s.id, name: s.name, description: s.detail })}
          />
        ) : (
          <PluginToggles
            title="From Codync"
            empty="No connectors yet. Add GitHub, Linear, Notion and more from Extensions."
            items={(plugins?.connectors ?? []).map((c) => ({ id: c.id, name: c.name, detail: c.command ?? c.url ?? '' }))}
            selection={draft.connectors ?? []}
            disabled={!online}
            onChange={(ids) => apply({ ...draft, connectors: ids })}
          />
        )}
        {error ? <Text size="footnote" tone="danger">{error}</Text> : null}
      </div>
      <FormSheet open={viewingSkill !== null} onClose={() => setViewingSkill(null)}>
        {viewingSkill ? <SkillSheet key={viewingSkill.name} skill={viewingSkill} /> : null}
      </FormSheet>
    </div>
  )
}
