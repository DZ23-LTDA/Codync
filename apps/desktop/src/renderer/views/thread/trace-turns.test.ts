import assert from 'node:assert/strict'
import { test } from 'node:test'
import { latestThought, runningTurn, scrolledToEnd, traceTurns } from './trace-turns.ts'
import type { Entry } from '../../../shared/models.ts'

const entry = (id: string, turn: number, kind = 'agent', status?: string): Entry => ({
  id, seq: 0, botId: 'b', rev: 1, kind, turn, data: { text: id, status }, createdAt: 0, updatedAt: 0,
})

test('turns group in turn order and keep their entries in order', () => {
  const turns = traceTurns([entry('u2', 2, 'user'), entry('u1', 1, 'user'), entry('a1', 1), entry('a2', 2)])
  assert.deepEqual(turns.map(([turn, es]) => [turn, es.map((e) => e.id)]), [[1, ['u1', 'a1']], [2, ['u2', 'a2']]])
})

test('the running turn is the newest one, not a queued message waiting for the next', () => {
  const entries = [entry('u1', 1, 'user'), entry('t1', 1, 'thought'), entry('q', 2, 'user', 'queued')]
  assert.equal(runningTurn(entries, true), 1)
  assert.equal(runningTurn(entries, false), undefined)
})

test('the trace follows new steps only while scrolled to the end', () => {
  assert.equal(scrolledToEnd({ scrollHeight: 1000, scrollTop: 600, clientHeight: 400 }), true)
  assert.equal(scrolledToEnd({ scrollHeight: 1000, scrollTop: 595, clientHeight: 400 }), true)
  assert.equal(scrolledToEnd({ scrollHeight: 1000, scrollTop: 500, clientHeight: 400 }), false)
})

test('thinking preview uses the latest nonempty thought in the active turn', () => {
  const entries = [entry('old', 1, 'thought'), entry('first', 2, 'thought'), entry('latest', 2, 'thought'),
    entry('  ', 2, 'thought'), entry('tool output', 2, 'tool'), entry('queued', 3, 'user', 'queued')]
  assert.equal(latestThought(entries), 'latest')
  assert.equal(latestThought([...entries, entry('new turn', 3, 'user')]), undefined)
  assert.equal(latestThought([]), undefined)
})
