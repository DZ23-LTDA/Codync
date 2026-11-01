import assert from "node:assert/strict";
import { test } from "node:test";

import { findUnwrapped, withoutTranslations } from "./l10n-unwrapped.mjs";

const kinds = (source) => findUnwrapped(source).map((entry) => `${entry.kind}:${entry.text}`);

test("direct JSX text is reported", () => {
  assert.deepEqual(kinds("export function A() { return <p>No bots yet</p> }"), ["text:No bots yet"]);
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


test("copy carried by footer, detail, empty, label and message is reported", () => {
  const source = [
    "const a = (",
    '  <CardSection footer="Lets you unfold a working bot\'s thinking.">',
    '    <Row detail="Runs in the background." />',
    '    <Row empty="Nothing here yet." />',
    "  </CardSection>",
    ")",
  ].join("\n");
  const found = findUnwrapped(source, "card.tsx").map((entry) => entry.text);
  assert.ok(found.some((text) => text.startsWith("Lets you unfold")), "footer: " + found);
  assert.ok(found.includes("Runs in the background."), "detail: " + found);
  assert.ok(found.includes("Nothing here yet."), "empty: " + found);
});

test("a text node is reported on the line the words are, not the line the element opened", () => {
  const source = ["export function A() {", "  return <Button", "    kind=\"secondary\"", "  >", "    Apple", "  </Button>", "}"].join("\n");
  const [found] = findUnwrapped(source, "button.tsx");
  assert.equal(found.text, "Apple");
  assert.equal(found.line, 5, "the words are on line 5, the element opened on line 2");
});

test("copy inside a JSX expression is reported, which the markup scan cannot see", () => {
  const source = "<span>{focus ? 'Your bots are idle.' : 'Create a bot to start.'}</span>";
  const found = findUnwrapped(source, "island.tsx").map((entry) => entry.text);
  assert.deepEqual(found.sort(), ["Create a bot to start.", "Your bots are idle."]);
});

test("a value inside an expression is not copy: an id, a class, a single word", () => {
  const source = "<div className={active ? 'card on' : 'card'}>{tab}</div>";
  assert.deepEqual(findUnwrapped(source, "row.tsx"), []);
});

test("a label in a module-level list is reported apart from copy inside a component", () => {
  const source = [
    "const tabs = [{ title: 'Computers & devices' }]",
    "export function Row() {",
    "  return <span>Update now</span>",
    "}",
  ].join("\n");
  const kinds = findUnwrapped(source, "tabs.tsx").map((entry) => `${entry.kind}:${entry.text}`);
  assert.deepEqual(kinds, ["module:Computers & devices", "text:Update now"]);
});

