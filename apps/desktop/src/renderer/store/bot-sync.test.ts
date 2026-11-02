import assert from 'node:assert/strict'
import { test } from 'node:test'
import type { Computer } from '@shared/models'
import { HostError } from '../client/host-client.ts'
import type { Connection } from './bot-mirror.ts'
import { BotSync } from './bot-sync.ts'

/** The desktop's bridge, as much of it as a store touches while it is built. */
function bridge() {
  ;(globalThis as { window?: unknown }).window = { codync: { appVersion: '2.34.1' } }
  ;(globalThis as { localStorage?: unknown }).localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
  }
}

const computer = { id: 'c1', name: 'Ana’s Mac' } as Computer

/** A sync store with `connection` already set, so `ready()` has something to read without a socket. */
function sync(connection: Connection, client: unknown = null) {
  bridge()
  const store = new BotSync(computer, true, () => {
    throw new Error('this test never opens a transport')
  })
  const inside = store as unknown as { connection: Connection; client: unknown; retired: boolean }
  inside.connection = connection
  inside.client = client
  return { store, inside }
}

const fakeClient = { call: async () => ({}) } as never

test('an online link hands back its client at once', async () => {
  const { store } = sync({ kind: 'online' }, fakeClient)
  assert.equal(await store.ready(), fakeClient)
})

test('a computer the host says is offline fails with what the person can read', async () => {
  const { store } = sync({ kind: 'computerOffline', lastSeen: null })
  await assert.rejects(store.ready(), (error: unknown) => {
    assert.ok(error instanceof HostError)
    assert.match(error.message, /offline/i)
    return true
  })
})

test('a 401 keeps the host\'s own message instead of inventing one', async () => {
  const { store } = sync({ kind: 'unauthorized', message: 'The token is no longer valid' })
  await assert.rejects(store.ready(), (error: unknown) => {
    assert.ok(error instanceof HostError)
    assert.equal(error.status, 401)
    assert.equal(error.message, 'The token is no longer valid')
    return true
  })
})

test('an unpaired computer reads as unreachable, and pairing later can still fix it', async () => {
  const { store } = sync({ kind: 'unpaired' })
  await assert.rejects(store.ready(), (error: unknown) => {
    assert.ok(error instanceof HostError)
    assert.equal(error.status, 0, 'not a refusal the host sent: there is no link to ask')
    assert.match(error.message, /Can.t reach your computer/)
    assert.equal(error.isTransient, true, 'the person can pair while this waits, so a retry is not pointless')
    return true
  })
})

test('a retired store stops waiting immediately', async () => {
  const { store, inside } = sync({ kind: 'online' }, null)
  inside.retired = true
  await assert.rejects(store.ready(), (error: unknown) => {
    assert.ok(error instanceof HostError)
    assert.equal(error.status, 0)
    return true
  })
})

test('an action that met a dropped link is tried again on the new one', async () => {
  let attempts = 0
  const store = sync({ kind: 'online' }, fakeClient).store
  // `withLink` is the protected path every mutation takes; run one that fails once with a transient error.
  const run = (op: (client: unknown) => Promise<string>) =>
    (store as unknown as { withLink: (o: typeof op, replay?: boolean) => Promise<string> }).withLink(op, true)

  const result = await run(async () => {
    attempts++
    if (attempts === 1) throw HostError.unreachable()
    return 'done'
  })

  assert.equal(result, 'done', 'the second attempt answered')
  assert.equal(attempts, 2, 'the call the link dropped under was retried once')
})

test('an action is not retried when retrying would repeat it', async () => {
  const store = sync({ kind: 'online' }, fakeClient).store
  const run = (op: () => Promise<string>) =>
    (store as unknown as { withLink: (o: typeof op, replay?: boolean) => Promise<string> }).withLink(op, false)

  let attempts = 0
  await assert.rejects(
    run(async () => {
      attempts++
      throw HostError.unreachable()
    }),
  )
  assert.equal(attempts, 1, 'without replay, a failed call stays failed')
})
