import assert from 'node:assert/strict'
import { test } from 'node:test'
import type { Computer } from '@shared/models'
import { BotMirror, builtInBackendName, type Connection } from './bot-mirror.ts'

/** The desktop's bridge, as much of it as the mirror touches while it is built. */
function bridge(appVersion = '2.34.1') {
  ;(globalThis as { window?: unknown }).window = { codync: { appVersion } }
  ;(globalThis as { localStorage?: unknown }).localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
  }
}

const computer = { id: 'c1', name: 'Ana’s Mac' } as Computer

/** A mirror for `computer`, with the host version it has heard of (none until `hello`). */
function mirror(options: { appVersion?: string; hostVersion?: string | null } = {}) {
  bridge(options.appVersion)
  const store = new BotMirror(computer, true, () => {
    throw new Error('this test never opens a transport')
  })
  if (options.hostVersion !== undefined) {
    ;(store as unknown as { hostVersion: { version: string; minApp?: string } | null }).hostVersion =
      options.hostVersion === null ? null : { version: options.hostVersion }
  }
  return store
}

/** Puts the store in one connection state, the way the sync layer would. */
function connecting(store: BotMirror, connection: Connection) {
  ;(store as unknown as { connection: Connection }).connection = connection
}

/** The header state as a name, so a `never` narrowing in the type checker cannot hide the comparison. */
function shownKind(store: BotMirror): string {
  return (store.shownConnection as Connection).kind
}

test('a computer that dropped, refused or answered 401 counts as offline; connecting does not', () => {
  const store = mirror()
  for (const [state, offline] of [
    [{ kind: 'offline', message: 'boom' }, true],
    [{ kind: 'computerOffline', lastSeen: null }, true],
    [{ kind: 'unauthorized', message: 'no' }, true],
    [{ kind: 'unpaired' }, false],
    [{ kind: 'connecting' }, false],
    [{ kind: 'online' }, false],
  ] as const) {
    connecting(store, state as Connection)
    assert.equal(store.isOffline, offline, state.kind)
  }
})

test('a header shows "connecting" while an action waits on an online link', () => {
  const store = mirror()
  connecting(store, { kind: 'online' })
  assert.equal(shownKind(store), 'online', 'idle: no reason to say anything else')

  ;(store as unknown as { waiting: number }).waiting = 1
  assert.equal(shownKind(store), 'connecting', 'an action is waiting, so say so')

  ;(store as unknown as { waiting: number }).waiting = 0
  connecting(store, { kind: 'offline', message: 'boom' })
  assert.equal(shownKind(store), 'offline', 'an offline link is reported as itself, waiting or not')
})

test('version gates read the host version, and nothing is offered before it is known', () => {
  assert.equal(mirror().hasRosterSections, false, 'no hello yet: no sections')
  assert.equal(mirror().hasLeadChoice, false, 'no hello yet: no lead choice')

  assert.equal(mirror({ hostVersion: '2.34.1' }).hasRosterSections, true)
  assert.equal(mirror({ hostVersion: '2.34.1' }).hasLeadChoice, true)

  // The thresholds differ: sections from 2.32.0, the lead choice from 2.33.0. A host between them keeps sections
  // and still refuses `setLead` and the delete that comes with it.
  assert.equal(mirror({ hostVersion: '2.31.9' }).hasRosterSections, false)
  assert.equal(mirror({ hostVersion: '2.32.0' }).hasRosterSections, true)
  assert.equal(mirror({ hostVersion: '2.32.9' }).hasLeadChoice, false)
  assert.equal(mirror({ hostVersion: '2.33.0' }).hasLeadChoice, true)
  assert.equal(mirror({ hostVersion: '2.32.9' }).hasRosterSections, true, '2.32.9 is past the sections gate')
})

test('the computer name is what the hello says, until a hello arrives', () => {
  const store = mirror()
  assert.equal(store.hostName, 'Ana’s Mac')
  ;(store as unknown as { hello: { name: string } }).hello = { name: 'mac-mini' }
  assert.equal(store.hostName, 'mac-mini')
})

test('each built-in backend has its own name; an unknown one is called a custom agent', () => {
  assert.equal(builtInBackendName('claude'), 'Claude Code')
  assert.equal(builtInBackendName('codex'), 'Codex')
  assert.equal(builtInBackendName('opencode'), 'OpenCode')
  assert.equal(builtInBackendName('grok'), 'Grok Build')
  assert.equal(builtInBackendName('gemini'), 'Gemini CLI')
  // An adapter this app has never heard of still needs something to show a person.
  assert.equal(builtInBackendName('my-own-adapter'), 'Custom agent')
  assert.equal(builtInBackendName(''), 'Custom agent')
})
