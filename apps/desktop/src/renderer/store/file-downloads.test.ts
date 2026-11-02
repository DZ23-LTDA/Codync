import assert from 'node:assert/strict'
import { test } from 'node:test'
import type { SharedFile } from '@shared/models'
import type { HostClient } from '../client/host-client'
import { FileDownloads } from './file-downloads.ts'

/** A shared file of `size` bytes whose content is a run of one letter; `sha256` only has to look valid. */
const file = (size: number, id = 'f1'): SharedFile =>
  ({ id, name: 'notes.txt', size, sha256: 'a'.repeat(64) }) as SharedFile

/** Bytes as the host sends them: base64 of a chunk that is exactly as long as it should be. */
const chunk = (count: number, size: number) => ({ data: btoa('x'.repeat(count)), size })

/** The bridge the store calls: records what the native side was asked to do. */
function bridge() {
  const calls: string[] = []
  let handle = 0
  const window = {
    codync: {
      files: {
        begin: async () => {
          calls.push('begin')
          return `h${++handle}`
        },
        write: async (_h: string, offset: number) => {
          calls.push(`write@${offset}`)
        },
        finish: async () => {
          calls.push('finish')
        },
        cancel: async () => {
          calls.push('cancel')
        },
      },
    },
  }
  ;(globalThis as { window?: unknown }).window = window
  return calls
}

/** A host client answering `readFile` with bytes until the file is complete. */
function client(size: number, options: { failAt?: number } = {}) {
  let reads = 0
  return {
    async readFile(_entry: string, _file: string, offset: number) {
      reads++
      if (options.failAt !== undefined && reads === options.failAt) throw new Error('connect ECONNREFUSED 127.0.0.1:19222')
      return chunk(Math.min(size - offset, 384 * 1024), size)
    },
  } as unknown as HostClient
}

test('a download runs to saved, reporting progress on the way', async () => {
  const calls = bridge()
  const states: string[] = []
  const downloads = new FileDownloads(() => {
    for (const [id, state] of downloads.states) states.push(`${id}:${state.kind}`)
  })

  await downloads.save(client(10), 'entry', file(10))

  assert.equal(downloads.states.get('f1')?.kind, 'saved')
  assert.ok(calls.includes('begin') && calls.includes('finish'), 'the native side was asked to begin and finish')
  assert.ok(states.includes('f1:downloading'), 'the screen was told it was downloading')
  assert.ok(states.includes('f1:saved'), 'and then that it was saved')
})

test('a failed read says one sentence and cancels the handle it opened', async () => {
  const calls = bridge()
  const downloads = new FileDownloads(() => {})
  await downloads.save(client(10, { failAt: 1 }), 'entry', file(10))

  const state = downloads.states.get('f1')
  assert.equal(state?.kind, 'failed')
  const shown = state && state.kind === 'failed' ? state.message : ''
  assert.ok(shown.length > 0, 'the failure is reported')
  assert.ok(!shown.includes('ECONNREFUSED'), 'a refused socket is not what the person reads')
  assert.ok(!shown.includes('127.0.0.1'), 'and neither is the address it tried')
  assert.ok(calls.includes('cancel'), 'the half-written file is not left behind')
})

test('a cancelled download leaves no state and no handle open', async () => {
  const calls = bridge()
  const downloads = new FileDownloads(() => {})
  const running = downloads.save(client(10), 'entry', file(10))
  downloads.cancel('f1')

  assert.equal(downloads.states.get('f1'), undefined, 'a cancelled download is not on the screen')
  await running
  assert.ok(calls.includes('cancel'))
})

test('a retired store saves nothing', async () => {
  const calls = bridge()
  const downloads = new FileDownloads(() => {})
  downloads.retire()
  await downloads.save(client(10), 'entry', file(10))

  assert.equal(downloads.states.get('f1'), undefined)
  assert.deepEqual(calls, [], 'the native side was never asked to start')
})

test('the same file is not downloaded twice at once', async () => {
  const calls = bridge()
  const downloads = new FileDownloads(() => {})
  await Promise.all([
    downloads.save(client(10), 'entry', file(10)),
    downloads.save(client(10), 'entry', file(10)),
  ])

  assert.equal(calls.filter((c) => c === 'begin').length, 1, 'one job, one begin')
})

test('with no computer connected, the failure names what to do', async () => {
  bridge()
  const downloads = new FileDownloads(() => {})
  await downloads.save(null, 'entry', file(10))

  const state = downloads.states.get('f1')
  assert.equal(state?.kind, 'failed')
  assert.match(state && state.kind === 'failed' ? state.message : '', /Connect to this computer/)
})

test('a file the host cannot have described is refused before anything is written', async () => {
  const calls = bridge()
  const downloads = new FileDownloads(() => {})
  // A name with a path separator is the shape a hostile host would send.
  await downloads.save(client(10), 'entry', { ...file(10), name: '../escape.txt' } as SharedFile)

  assert.equal(downloads.states.get('f1')?.kind, 'failed')
  assert.deepEqual(calls, [], 'nothing was opened on disk')
})
