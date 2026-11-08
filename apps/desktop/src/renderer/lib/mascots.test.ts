// The mascot art is derived, never hand-edited, so the thing worth pinning is that the copies are
// actually here and match the host's catalog -- a missing PNG is a bot with no face, and nothing else
// would fail.
import assert from 'node:assert/strict'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'

const here = fileURLToPath(new URL('.', import.meta.url))
const root = join(here, '..', '..', '..', '..', '..')
const art = join(root, 'apps', 'desktop', 'src', 'renderer', 'assets', 'mascots')
const catalog = join(root, 'host', 'src', 'agent', 'avatar.rs')

/** The mascot ids the host resolves a bot to, read from its own catalog. */
function ids(): string[] {
  const code = readFileSync(catalog, 'utf8')
  const start = code.indexOf('pub const IDS')
  const block = code.slice(start, code.indexOf('];', start))
  return [...block.matchAll(/"([a-z0-9-]+)"/g)].map((match) => match[1]!)
}

test('every mascot the host can assign has its picture in the desktop bundle', () => {
  const shipped = readdirSync(art)
    .filter((name) => name.endsWith('.png'))
    .map((name) => name.replace(/\.png$/, ''))
  assert.deepEqual([...ids()].sort(), [...shipped].sort(), 'the derived copies drifted from the host catalog')
})
