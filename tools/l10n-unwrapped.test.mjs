import assert from "node:assert/strict";
import { test } from "node:test";

import { findUnwrapped, withoutTranslations } from "./l10n-unwrapped.mjs";

const kinds = (source) => findUnwrapped(source).map((entry) => `${entry.kind}:${entry.text}`);

test("direct JSX text is reported", () => {
  assert.deepEqual(kinds("<p>No bots yet</p>"), ["text:No bots yet"]);
});

test("a wrapped string is not reported", () => {
  assert.deepEqual(kinds("<p>{tr('No bots yet')}</p>"), []);
});

test("placeholder, title and aria-label are reported", () => {
  const source = '<input placeholder="Search" title="Search" aria-label="Open menu" />';
  assert.deepEqual(kinds(source), ["placeholder:Search", "title:Search", "aria-label:Open menu"]);
});

test("an attribute built from tr() is not reported", () => {
  assert.deepEqual(kinds('<Icon title={tr(\'Search\')} aria-label={tr("Open menu")} />'), []);
});

test("a literal on the same line as a tr() call is still reported", () => {
  // The line-level skip this tool used to have hid exactly this case.
  const source = '<div><span>{tr(\'Cancel\')}</span><Icon title="Delete" /></div>';
  assert.deepEqual(kinds(source), ['title:Delete']);
});

test("an attribute that wraps to the next line is reported", () => {
  const source = '<Icon\n  title="Open the\n  account menu"\n/>';
  assert.deepEqual(kinds(source), ["title:Open the\n  account menu"]);
});

test("a template literal or a conditional inside tr() is not reported", () => {
  const source = "<p>{tr(`Loading…`)} {tr(ready ? 'Done' : 'Waiting')}</p>";
  assert.deepEqual(kinds(source), []);
});

test("a string built dynamically is left alone: it has no literal to translate", () => {
  assert.deepEqual(kinds("<p>{errorMessage}</p>"), []);
});

test("brand names that must not be translated are left alone", () => {
  assert.deepEqual(kinds("<b>Codync</b><span>B-3PO</span>"), []);
});

test("withoutTranslations keeps the file's line count", () => {
  const source = "<p>{tr('a')}</p>\n<p>{tr(\n'b'\n)}</p>\n";
  const blanked = withoutTranslations(source);
  assert.equal(blanked.split("\n").length, source.split("\n").length);
});
