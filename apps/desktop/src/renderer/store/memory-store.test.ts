import assert from 'node:assert/strict'
import { test } from 'node:test'
import type { HostClient } from '../client/host-client'
import { MemoryStore, type MemoryView } from './memory-store.ts'

/** A view the host answers with, in the shape this app knows. */
const view = (entries = 0): MemoryView =>
  ({
    yours: { entries: [], used: entries, cap: 100 },
    bot: { entries: [], used: 0, cap: 100 },
    dreams: [],
  }) as unknown as MemoryView

/** A store whose client answers with `answer` for `memory`, counting the calls. */
function store(answer: () => Promise<unknown>) {
  let calls = 0
  const client = {
    call: async (method: string) => {
      assert.equal(method, 'memory')
      calls++
      return await answer()
    },
  } as unknown as HostClient
  const memory = new MemoryStore('bot', async () => client)
  return { memory, calls: () => calls }
}

test('a good answer fills the view and clears any earlier failure', async () => {
  const { memory } = store(async () => view(7))
  await memory.load()
  assert.equal(memory.error, null)
  assert.equal(memory.view?.yours.used, 7)
})

test('a failure keeps what was on screen and says one sentence', async () => {
  const { memory } = store(async () => view(3))
  await memory.load()
  const shown = memory.view

  const failing = store(async () => {
    throw new Error('connect ECONNREFUSED 127.0.0.1:19222')
  })
  await failing.memory.load()
  assert.equal(failing.memory.view, null)
  assert.ok(failing.memory.error, 'the failure is reported')
  assert.ok(
    !String(failing.memory.error).includes('ECONNREFUSED'),
    'a socket detail is not what a person reads',
  )
  assert.equal(shown?.yours.used, 3, 'the store that already had a view keeps it')
})

test('loading again after a failure recovers, with the error gone', async () => {
  let fail = true
  const { memory } = store(async () => {
    if (fail) throw new Error('offline')
    return view(2)
  })
  await memory.load()
  assert.ok(memory.error)

  fail = false
  await memory.load()
  assert.equal(memory.error, null, 'a retry that works clears the error')
  assert.equal(memory.view?.yours.used, 2)
})

test('a slow answer that lands after a newer one is dropped', async () => {
  // The first `load` is held open until the second has finished: the older answer must not win.
  const held: { release?: (v: MemoryView) => void } = {}
  let calls = 0
  const client = {
    call: async () => {
      calls++
      if (calls === 1) return await new Promise<MemoryView>((resolve) => (held.release = resolve))
      return view(9)
    },
  } as unknown as HostClient
  const memory = new MemoryStore('bot', async () => client)

  const first = memory.load()
  const second = memory.load()
  await second
  held.release?.(view(1))
  await first

  assert.equal(memory.view?.yours.used, 9, 'the newest answer is the one on screen')
})

test('a host too old for this screen says to update, not to retry', async () => {
  // Before 2.18.0 the host answers with its old listing: no `dreams` array.
  const { memory } = store(async () => ({ yours: { entries: [] }, bot: { entries: [] } }))
  await memory.load()
  assert.match(String(memory.error), /Update Codync on your computer/)
})
