import assert from 'node:assert/strict'
import { test } from 'node:test'
import { DROP_GRACE, INITIAL_GRACE, heldGrace } from './connection-grace.ts'

const online = { kind: 'online' } as const
const connecting = { kind: 'connecting' } as const
const offline = { kind: 'offline', message: 'boom' } as const
const computerOffline = { kind: 'computerOffline', lastSeen: null } as const
const unpaired = { kind: 'unpaired' } as const
const unauthorized = { kind: 'unauthorized', message: 'no' } as const

test('a drop from online waits out a wifi blip', () => {
  assert.equal(heldGrace(online, offline), DROP_GRACE)
  assert.equal(heldGrace(online, computerOffline), DROP_GRACE)
  assert.equal(heldGrace(online, connecting), DROP_GRACE)
})

test('a first connection that fails waits only for the host to finish starting', () => {
  assert.equal(heldGrace(connecting, offline), INITIAL_GRACE)
  assert.equal(heldGrace(connecting, computerOffline), INITIAL_GRACE)
})

test('the first connection that is still connecting is shown at once', () => {
  assert.equal(heldGrace(connecting, connecting), null)
})

test('coming back up is never delayed: a person waiting should see it', () => {
  assert.equal(heldGrace(connecting, online), null)
  assert.equal(heldGrace(offline, online), null)
  assert.equal(heldGrace(computerOffline, online), null)
  assert.equal(heldGrace(unpaired, online), null)
})

test('refusals are never delayed: they are an answer, not a blip', () => {
  assert.equal(heldGrace(online, unauthorized), null)
  assert.equal(heldGrace(connecting, unauthorized), null)
  assert.equal(heldGrace(online, unpaired), null)
})

test('a drop after a drop does not restart the grace', () => {
  // `offline` is not `online` and is not a first failure, so the next state applies at once: the screen is
  // already showing the truth, and holding it again would only delay the newest one.
  assert.equal(heldGrace(offline, offline), null)
  assert.equal(heldGrace(offline, computerOffline), null)
  assert.equal(heldGrace(computerOffline, offline), null)
})
