#!/usr/bin/env node
// Text a person reads that never reaches the translation: JSX text written straight into the markup, and attribute copy
// (placeholder, title, aria-label) that is not built from tr(). Both show in English in every language, so this is the
// list that decides whether a screen is really translated.
//
// The scan reads the whole file, not line by line, so an attribute whose value wraps to the next line is still found.
// A tr(...) call is removed from the text before matching: that way `title="Search"` on the same line as a
// `tr('Cancel')` is still reported, which is exactly what a line-level skip used to hide.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const root = "apps/desktop/src/renderer";
// `tr('...')`, `tr("...")` or `tr(\`...\`)`, any of them possibly spanning lines.
const trCall = /\btr\(\s*(?:'[^']*'|"[^"]*"|`[^`]*`|\{[^}]*\})\s*\)/gs;
const visibleText = />\s*([A-Z][A-Za-z][^<>{}]{2,60})\s*</g;
// `footer`, `detail`, `empty`, `label` and `message` carry copy on the shared components and were invisible to an
// earlier version of this scan: their text showed in English while the report said every string was covered.
const visibleAttribute = /\b(placeholder|title|aria-label|footer|detail|empty|label|message)="([^"]{2,200})"/g;
// Text that is not interface copy: a JSX expression producing a value, or a brand name.
const ignoreText = /^(Promise|Codync|Math\.|B-3PO)/;
const ignoreAttribute = /^(B-3PO|Codync)$/;

/** The text of a file with every tr(...) call blanked out, keeping newlines so positions still map to lines. */
export function withoutTranslations(source) {
  return source.replace(trCall, (call) => call.replace(/[^\n]/g, " "));
}

function lineAt(source, index) {
  return source.slice(0, index).split("\n").length;
}

/**
 * Every visible string that does not go through tr(). `source` is the file's text; returns `{ file, line, kind, text }`
 * so a caller can print it or count it. `file` is the path as given.
 */
export function findUnwrapped(source, file = "") {
  const scanned = withoutTranslations(source);
  const found = [];
  for (const match of scanned.matchAll(visibleText)) {
    const text = match[1].trim();
    if (ignoreText.test(text)) continue;
    // The match starts at the `>`; report the line the words are on, not the line the element opened on.
    const at = match.index + match[0].indexOf(match[1]);
    found.push({ file, line: lineAt(scanned, at), kind: "text", text });
  }
  for (const match of scanned.matchAll(visibleAttribute)) {
    const text = match[2].trim();
    if (ignoreAttribute.test(text)) continue;
    found.push({ file, line: lineAt(scanned, match.index), kind: match[1], text });
  }
  return found.sort((a, b) => a.line - b.line);
}

function main() {
  const root_dir = process.cwd();
  const entries = [];
  for (const file of globSync(`${root_dir}/${root}/**/*.tsx`)) {
    const relative = file.slice(root_dir.length + 1).replace(`${root}/`, "");
    entries.push(...findUnwrapped(readFileSync(file, "utf8"), relative));
  }

  if (entries.length === 0) {
    console.log("every visible string goes through tr()");
    return;
  }
  console.log(`${entries.length} visible strings never reach the translation:`);
  for (const entry of entries) console.log(`  ${entry.file}:${entry.line}: ${entry.kind} "${entry.text}"`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
