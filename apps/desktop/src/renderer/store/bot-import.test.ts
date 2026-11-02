import assert from 'node:assert/strict'
import { test } from 'node:test'
import type { BotStore } from './bot-store.ts'
import { BotImport, type ImportPlan } from './bot-import.ts'

/** An export file with two bots and a group that needs both. */
const plan = (): ImportPlan =>
  ({
    source: 'grok-bot',
    bots: [
      { key: 'b1', name: 'Ana', description: '', skills: [], routines: [], memories: [] },
      { key: 'b2', name: 'Beto', description: '', skills: [], routines: [], memories: [] },
    ],
    groups: [{ key: 'g1', name: 'Team', members: ['b1', 'b2'] }],
    warnings: [],
  }) as unknown as ImportPlan

/** A store whose host answers `previewImport` with `plan` and records every call it is asked to make. */
function store(answers: (method: string, args: unknown) => unknown = () => undefined) {
  const calls: { method: string; args: unknown }[] = []
  const client = {
    call: async (method: string, args: unknown) => {
      calls.push({ method, args })
      return answers(method, args)
    },
  }
  const botStore = { ready: async () => client } as unknown as BotStore
  return { store: botStore, calls }
}

/** The bridge the class calls: a file picker and a clipboard. */
function bridge(path: string | null) {
  const copied: string[] = []
  ;(globalThis as { window?: unknown }).window = {
    codync: {
      app: {
        pickExport: async () => path,
        copy: (text: string) => copied.push(text),
      },
    },
  }
  return copied
}

test('choosing a file shows its bots and groups for review', async () => {
  bridge('/tmp/export.json')
  const { store: bots, calls } = store((method) => (method === 'previewImport' ? { plan: plan(), skipped: [] } : undefined))
  const importing = new BotImport(bots)

  await importing.choose()

  assert.equal(importing.step.kind, 'review')
  assert.equal(calls[0]?.method, 'previewImport')
  assert.deepEqual(calls[0]?.args, { path: '/tmp/export.json' })
  assert.ok(importing.canImport, 'with both bots picked, importing is allowed')
})

test('closing the picker does nothing at all', async () => {
  bridge(null)
  const { store: bots, calls } = store()
  const importing = new BotImport(bots)

  await importing.choose()

  assert.equal(importing.step.kind, 'start')
  assert.deepEqual(calls, [], 'no host call for a cancelled picker')
})

test('a group comes only with one of its bots', async () => {
  bridge('/tmp/export.json')
  const { store: bots } = store((method) => (method === 'previewImport' ? { plan: plan(), skipped: [] } : undefined))
  const importing = new BotImport(bots)
  await importing.choose()

  assert.equal(importing.groupAvailable(['b1', 'b2']), true)
  importing.toggle('b1')
  importing.toggle('b2')
  assert.equal(importing.groupAvailable(['b1', 'b2']), false, 'both bots left out, so the group cannot come')
  assert.equal(importing.canImport, false, 'nothing picked, nothing to import')
})

test('importing sends only the bots still picked, and only groups that survive', async () => {
  bridge('/tmp/export.json')
  const { store: bots, calls } = store((method) => {
    if (method === 'previewImport') return { plan: plan(), skipped: [] }
    return { bots: [], groups: [], warnings: [] }
  })
  const importing = new BotImport(bots)
  await importing.choose()

  importing.toggle('b2') // leave only b1
  await importing.importPicked()

  const sent = calls.find((c) => c.method === 'importBots')?.args as { bots: string[]; groups: string[] }
  assert.deepEqual(sent.bots, ['b1'])
  assert.deepEqual(sent.groups, ['g1'], 'the group still has b1, so it comes along')
  assert.equal(importing.step.kind, 'done')
})

test('a group whose bots were all left out is not sent', async () => {
  bridge('/tmp/export.json')
  const { store: bots, calls } = store((method) => {
    if (method === 'previewImport') return { plan: plan(), skipped: [] }
    return { bots: [], groups: [], warnings: [] }
  })
  const importing = new BotImport(bots)
  await importing.choose()

  importing.toggle('b1')
  importing.toggle('g1')
  // b1 is out but b2 is still in, so the group is offered; a group the person then unchecks stays out.
  await importing.importPicked()

  const sent = calls.find((c) => c.method === 'importBots')?.args as { bots: string[]; groups: string[] }
  assert.deepEqual(sent.bots, ['b2'])
  assert.deepEqual(sent.groups, [])
})

test('a failing import says one sentence and leaves the review open to try again', async () => {
  bridge('/tmp/export.json')
  const { store: bots } = store((method) => {
    if (method === 'previewImport') return { plan: plan(), skipped: [] }
    throw new Error('connect ECONNREFUSED 127.0.0.1:19222')
  })
  const importing = new BotImport(bots)
  await importing.choose()

  await importing.importPicked()

  assert.equal(importing.step.kind, 'review', 'the person can try again without picking the file anew')
  assert.ok(importing.error, 'the failure is reported')
  assert.ok(!importing.error?.includes('ECONNREFUSED'), 'a socket error is not what the person reads')
  assert.equal(importing.busy, false, 'and the button works again')
})

test('a second click while the first is running does not import twice', async () => {
  bridge('/tmp/export.json')
  let imports = 0
  const { store: bots } = store((method) => {
    if (method === 'previewImport') return { plan: plan(), skipped: [] }
    imports++
    return { bots: [], groups: [], warnings: [] }
  })
  const importing = new BotImport(bots)
  await importing.choose()

  await Promise.all([importing.importPicked(), importing.importPicked()])

  assert.equal(imports, 1)
})

test('the skill text goes to the clipboard', async () => {
  const copied = bridge('/tmp/export.json')
  const { store: bots, calls } = store((method) => (method === 'importSkill' ? { text: 'the skill' } : undefined))
  const importing = new BotImport(bots)

  await importing.copySkill()

  assert.equal(calls[0]?.method, 'importSkill')
  assert.deepEqual(copied, ['the skill'])
  assert.equal(importing.copied, true)
})
