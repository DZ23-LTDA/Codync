import assert from 'node:assert/strict'
import test from 'node:test'
import type { NetworkComputer } from '@shared/models'
import { botsTemplate, canInvite, canRecover, connectorFor, presenceTitle } from './network-copy.ts'

const computer = (c: Partial<NetworkComputer>): NetworkComputer => ({
  name: 'laptop', role: 'member', me: false, online: false, agents: [], canRemove: false, canPromote: false, ...c,
})

test('a relay connector needs an address; an empty one would quietly mean public', () => {
  assert.deepEqual(connectorFor('public', 'ignored'), { connector: 'public' })
  assert.deepEqual(connectorFor('private', ''), { connector: 'private' })
  assert.deepEqual(connectorFor('relay', ' https://relay.example.com '), { connector: 'https://relay.example.com' })
  assert.ok('problem' in connectorFor('relay', '  '))
})

test('rows say where each computer is and how many bots it offers', () => {
  assert.equal(presenceTitle(computer({ me: true })), 'This computer')
  assert.equal(presenceTitle(computer({ online: true })), 'Online')
  assert.equal(presenceTitle(computer({})), 'Away')
  assert.equal(botsTemplate(computer({ agents: [{ id: 'a', name: 'A' }] })), '1 bot')
  assert.equal(botsTemplate(computer({})), '{count} bots')
})

test('the owner and admins invite; everyone but the owner may recover', () => {
  assert.deepEqual([canInvite('owner'), canInvite('admin'), canInvite('member')], [true, true, false])
  assert.deepEqual([canRecover('owner'), canRecover('admin'), canRecover(null)], [false, true, true])
})
