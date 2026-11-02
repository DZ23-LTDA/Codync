import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readFileSync } from 'node:fs'

const directory = new URL('../../../../ios/Kit/Sources/CodyncKit/Resources/Languages/', import.meta.url)
const read = (language: string): Record<string, string> => JSON.parse(readFileSync(new URL(`${language}.json`, directory), 'utf8'))

test('all shipped catalogs have the same keys and preserve interpolation tokens', () => {
  const source = read('zh-Hans')
  // Portuguese covers every key the source has, plus 205 the source itself lacks: strings the apps
  // ask for that zh-Hans never picked up, which is why the source reads short in English. It is held
  // to the shape (real newlines, values present, tokens preserved) without the "same keys" rule,
  // which would delete those translations to please the comparison.
  const partial = new Set(['pt'])
  for (const locale of ['zh-Hant', 'ja', 'ko', 'fr', 'de', 'es', 'pt']) {
    const catalog = read(locale)
    assert.equal(catalog.English, 'English', `${locale}: English keeps its native name in language menus`)
    if (partial.has(locale)) {
      const absent = Object.keys(source).filter((key) => !(key in catalog))
      assert.deepEqual(absent, [], `${locale}: keys the source has that this catalog lacks`)
      for (const [key, value] of Object.entries(catalog)) {
        assert.ok(value.trim(), `${locale}: ${key}`)
        assert.ok(!key.includes('\\n') && !value.includes('\\n'), `${locale}: ${key} keeps a literal backslash`)
        assert.deepEqual((value.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), (key.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), `${locale}: ${key}`)
      }
      continue
    }
    assert.deepEqual(Object.keys(catalog).sort(), Object.keys(source).sort(), locale)
    for (const [key, value] of Object.entries(catalog)) {
      assert.ok(value.trim(), `${locale}: ${key}`)
      assert.deepEqual((value.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), (key.match(/\{[A-Za-z0-9]+\}/g) ?? []).sort(), `${locale}: ${key}`)
      assert.doesNotMatch(value, /ZXQ\d|QXZ/, `${locale}: ${key}`)
    }
  }
})
