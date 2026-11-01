import assert from "node:assert/strict";
import { test } from "node:test";

import { missingFrom, phrasesIn } from "./l10n-missing.mjs";

test("literals handed to tr are collected, in every quote style", () => {
  const source = `const a = tr('First'); const b = tr("Second"); const c = tr(\`Third\`)`;
  assert.deepEqual([...phrasesIn(source)].sort(), ["First", "Second", "Third"]);
});

test("a phrase built with a placeholder is skipped, and so is a variable name", () => {
  const source = "tr(`Hello ${name}`); tr(label); tr('')";
  assert.deepEqual([...phrasesIn(source)], []);
});

test("what is missing is what the table has no key for", () => {
  const phrases = new Set(["Present", "Absent"]);
  assert.deepEqual(missingFrom(phrases, { Present: "Presente" }), ["Absent"]);
});

test("a table with every key reports nothing missing", () => {
  assert.deepEqual(missingFrom(new Set(["A"]), { A: "A" }), []);
});

test("a value shown as an example of a format is not copy to translate", () => {
  const phrases = new Set(["https://", "API_KEY=", "Authorization: Bearer", "Connect"]);
  assert.deepEqual(missingFrom(phrases, {}), ["Connect"]);
});

test("a plain number is a value, not copy", () => {
  assert.deepEqual(missingFrom(new Set(["3600", "Save"]), {}), ["Save"]);
});

test("a cron expression is an example, not copy", () => {
  assert.deepEqual(missingFrom(new Set(["0 9 * * 1-5", "Repeat"]), {}), ["Repeat"]);
});

test("copy announced through a list of items is collected too", () => {
  const source = "const items = [{ title: 'Support', icon: 'book' }, { label: 'Allow' }]";
  assert.deepEqual([...phrasesIn(source)].sort(), ["Allow", "Support"]);
});

