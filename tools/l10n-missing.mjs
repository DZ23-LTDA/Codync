#!/usr/bin/env node
// Which phrases the desktop passes to `tr()` have no entry in pt.json. The coverage tool answers a different question:
// it compares tables against the German one, so a phrase that only exists in the code is invisible to it. This one starts
// from the code.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const RENDERER = "apps/desktop/src/renderer";
const PT = "apps/ios/Kit/Sources/CodyncKit/Resources/Languages/pt.json";

/** Every literal handed to `tr(...)`: single quotes, double quotes and template literals without placeholders. */
export function phrasesIn(source) {
  const found = new Set();
  const patterns = [
    /\btr\(\s*'((?:[^'\\]|\\.)+)'/g,
    /\btr\(\s*"((?:[^"\\]|\\.)+)"/g,
    /\btr\(\s*`([^`$]+)`/g,
  ];
  // Copy that reaches the screen through a list of UI items: `{ title: 'Support', … }` rendered by `tr(item.title)`.
  // The literal never passes through tr() itself, so a missing key here showed as English while every counter said zero.
  for (const match of source.matchAll(/\b(?:title|label|detail|empty):\s*(['"])([^'"\n]{2,80})\1/g)) {
    const phrase = match[2].trim();
    // A field that repeats its own name (`title: 'title'`) is metadata, not copy.
    if (phrase !== "" && phrase !== "title" && phrase !== "label") found.add(phrase);
  }
  for (const pattern of patterns) {
    for (const match of source.matchAll(pattern)) {
      // A single-quoted string in the source keeps its escapes; the table has the text a reader sees.
      const phrase = match[1].replaceAll("\\'", "'").replaceAll('\\"', '"').trim();
      // An empty literal is not a phrase. A bare identifier is not matched at all: every pattern here requires a quote,
      // so `tr(label)` never reaches this line -- filtering single words here would drop "Skills" and "Installed".
      if (phrase !== "") found.add(phrase);
    }
  }
  return found;
}

/**
 * A value shown as an example of a format, not copy: a placeholder UUID, a key with its assignment, a header syntax,
 * a scheme prefix, a plain number used as a default, a cron expression shown as an example. Translating those would be wrong -- "Authorization: Bearer" stays as
 * it is in every language and 3600 is 3600.
 */
const example = /^(?:[0-9a-f]{8}-|API_KEY=|Authorization: Bearer|codync-network:|https?:\/\/|[a-z-]+:|\d+$|name@example\.com|\S+ \S+ \S+ \S+ \S+)/;

/** The phrases used in the renderer with no key in `table`. Pure, so the rule is testable on its own. */
export function missingFrom(phrases, table) {
  return [...phrases].filter((phrase) => !(phrase in table) && !example.test(phrase)).sort();
}

function main() {
  const base = process.cwd();
  const table = JSON.parse(readFileSync(`${base}/${PT}`, "utf8"));
  const used = new Set();
  for (const file of globSync(`${base}/${RENDERER}/**/*.{ts,tsx}`)) {
    // Windows paths come back with backslashes: normalise before the demo/test filter, or the demo story reads as copy.
    const normalized = file.replaceAll("\\", "/");
    if (normalized.includes(".test.") || normalized.includes("/demo/")) continue;
    for (const phrase of phrasesIn(readFileSync(file, "utf8"))) used.add(phrase);
  }
  const missing = missingFrom(used, table);
  console.log(`${used.size} phrases reach tr() in the renderer`);
  console.log(`${missing.length} of them have no pt.json entry`);
  for (const phrase of missing) console.log(`  ${phrase}`);
  process.exit(0);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
