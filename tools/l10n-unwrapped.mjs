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
// `tr('…')`, `tr("…")`, `tr(`…`)`, or a ternary of literals: `tr(a ? 'One' : 'Two')` is translated too.
const trCall = /\btr\(\s*(?:'[^']*'|"[^"]*"|`[^`]*`|\{[^}]*\}|[^()]*\?[^()]*)\s*\)/gs;
const visibleText = />\s*([A-Z][A-Za-z][^<>{}]{2,60})\s*</g;
// `footer`, `detail`, `empty`, `label` and `message` carry copy on the shared components and were invisible to an
// earlier version of this scan: their text showed in English while the report said every string was covered.
const visibleAttribute = /\b(placeholder|title|aria-label|footer|detail|empty|label|message)="([^"]{2,200})"/g;
// Text that is not interface copy: a JSX expression producing a value, or a brand name.
const ignoreText = /^(Promise|Codync|Math\.|B-3PO)/;
// A string inside a JSX expression: `{cond ? 'Message a bot' : 'New bot'}` or `{'Nothing yet.'}`. This used to be
// invisible to every counter -- the text is not between `>` and `<`, so the markup scan walked past it.
// The block first, then every literal inside it: a ternary holds two phrases and both are copy.
const expressionBlock = /\{[^{}\n]{0,200}\}/g;
const quoted = /(['"])([^'"\n]{2,120})\1/g;
// Prose, not an id or a class: capital letter, and either a space or a full stop.
const isProse = (text) => /^[A-Z]/.test(text) && /[\s.!?…]/.test(text);
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
  // A JSX comment is not copy: `{/* the heading still reads "Needs update" */}` was reported as if it were.
  const scanned = withoutTranslations(source).replace(/\{\/\*[\s\S]*?\*\/\}/g, (comment) => comment.replace(/[^\n]/g, " "));
  const found = [];
  // Copy at module scope cannot call a hook: by convention it is announced by the component that renders it
  // (`tr(item.title)`). Those are counted apart, so the number that matters -- copy inside a component -- stays honest.
  const depthAt = (index) => {
    let depth = 0;
    for (const char of scanned.slice(0, index)) {
      if (char === "{") depth += 1;
      else if (char === "}") depth = Math.max(0, depth - 1);
    }
    return depth;
  };

  for (const match of scanned.matchAll(visibleText)) {
    const text = match[1].trim();
    if (ignoreText.test(text)) continue;
    if (depthAt(match.index) === 0) {
      found.push({ file, line: lineAt(scanned, match.index), kind: "module", text });
      continue;
    }
    // The match starts at the `>`; report the line the words are on, not the line the element opened on.
    const at = match.index + match[0].indexOf(match[1]);
    found.push({ file, line: lineAt(scanned, at), kind: "text", text });
  }
  for (const match of scanned.matchAll(visibleAttribute)) {
    const text = match[2].trim();
    if (ignoreAttribute.test(text)) continue;
    found.push({ file, line: lineAt(scanned, match.index), kind: match[1], text });
  }
  for (const block of scanned.matchAll(expressionBlock)) {
    const atModule = depthAt(block.index) === 0;
    for (const match of block[0].matchAll(quoted)) {
      const text = match[2].trim();
      if (!isProse(text) || ignoreText.test(text)) continue;
      if (atModule) {
        found.push({ file, line: lineAt(scanned, block.index + block[0].indexOf(match[2])), kind: "module", text });
        continue;
      }
      const at = block.index + block[0].indexOf(match[2], match.index);
      found.push({ file, line: lineAt(scanned, at), kind: "expression", text });
    }
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

  const moduleLevel = entries.filter((entry) => entry.kind === "module");
  const inComponents = entries.filter((entry) => entry.kind !== "module");
  if (entries.length === 0) {
    console.log("every visible string goes through tr()");
    return;
  }
  // Two numbers, because they mean different things: copy inside a component must call tr(); copy at module scope
  // cannot, and is announced by the component that renders it (`tr(item.title)`).
  console.log(`${inComponents.length} visible strings inside a component never reach the translation:`);
  for (const entry of inComponents) console.log(`  ${entry.file}:${entry.line}: ${entry.kind} "${entry.text}"`);
  console.log(`${moduleLevel.length} at module scope, announced by the component that renders them.`);
  if (process.argv.includes("--module")) {
    for (const entry of moduleLevel) console.log(`  ${entry.file}:${entry.line}: ${entry.kind} "${entry.text}"`);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
