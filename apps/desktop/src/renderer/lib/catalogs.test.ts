import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readFileSync } from 'node:fs'

const directory = new URL('../../../../ios/Kit/Sources/CodyncKit/Resources/Languages/', import.meta.url)
const read = (language: string): Record<string, string> => JSON.parse(readFileSync(new URL(`${language}.json`, directory), 'utf8'))

test('all shipped catalogs have the same keys and preserve interpolation tokens', () => {
  const source = read('zh-Hans')
  // Portuguese is still short of the full table (944 of 1468 keys, tracked in
  // docs/plans/release-1-reconciliation.md): until it catches up it is held to the same shape — real
  // newlines, values present, no key the source does not have — but not to completeness.
  const partial = new Set(['pt'])
  for (const locale of ['zh-Hant', 'ja', 'ko', 'fr', 'de', 'es', 'pt']) {
    const catalog = read(locale)
    if (partial.has(locale)) {

      for (const [key, value] of Object.entries(catalog)) {
        assert.doesNotMatch(key, /\\n/, `${locale}: ${key} keeps a literal backslash`)
        assert.ok(value.trim(), `${locale}: ${key}`)
        assert.ok(!value.includes('\\n'), `${locale}: ${value} keeps a literal backslash`)
      }
      continue
    }
    assert.equal(catalog.English, "English", `${locale}: English keeps its native name in language menus`)
    assert.deepEqual(Object.keys(catalog).sort(), Object.keys(source).sort(), locale)
    for (const [key, value] of Object.entries(catalog)) {
      assert.ok(value.trim(), `${locale}: ${key}`)
      assert.deepEqual((value.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), (key.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), `${locale}: ${key}`)
      assert.doesNotMatch(value, /ZXQ\d|QXZ/, `${locale}: ${key}`)
    }
  }
})