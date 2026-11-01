import assert from 'node:assert/strict'
import { test } from 'node:test'

import { isLongTurn, LONG_TURN_MS } from './long-turn.ts'

test('a turn below the threshold is not pointed at', () => {
  assert.equal(isLongTurn(0), false)
  assert.equal(isLongTurn(LONG_TURN_MS - 1), false)
})

test('a turn at or past the threshold is pointed at', () => {
  assert.equal(isLongTurn(LONG_TURN_MS), true)
  assert.equal(isLongTurn(LONG_TURN_MS * 3), true)
})

test('the threshold can be given, so callers are not locked to two minutes', () => {
  assert.equal(isLongTurn(5_000, 5_000), true)
  assert.equal(isLongTurn(4_999, 5_000), false)
})

