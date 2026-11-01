#!/usr/bin/env node
// How complete each interface language is, against the largest table: what is missing says how much a screen would fall
// back to English, which is the number that decides whether a language is ready to show.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { globSync } from "node:fs";

import { isExample } from "./l10n-missing.mjs";

const dir = "apps/ios/Kit/Sources/CodyncKit/Resources/Languages";
const tables = new Map();
for (const name of readdirSync(dir).filter((file) => file.endsWith(".json")).sort()) {
  tables.set(name.replace(".json", ""), Object.keys(JSON.parse(readFileSync(join(dir, name), "utf8"))));
}
const widest = [...tables.entries()].sort((a, b) => b[1].length - a[1].length)[0];
const [reference, keys] = widest;
const known = new Set(keys);

for (const [language, list] of tables) {
  const have = new Set(list);
  const missing = keys.filter((key) => !have.has(key)).length;
  const extra = list.filter((key) => !known.has(key)).length;
  const percent = ((list.length - extra) / keys.length) * 100;
  console.log(
    `${language.padEnd(8)} ${String(list.length).padStart(5)} keys  ${percent.toFixed(1).padStart(5)}% of ${reference}  ` +
      `${String(missing).padStart(4)} missing  ${String(extra).padStart(3)} not in ${reference}`,
  );
}

// What the desktop app actually shows: the phrases it passes through tr(). A language can cover the whole table and
// still leave this screen in English, or cover this screen and leave the iPhone app mostly English, so both numbers
// are reported and both are decisions, not accidents.
//
// Values shown as an example of a format (a cron expression, a placeholder UUID, `https://`) are not copy and are
// counted out here with the same rule the missing-key scanner uses, so the percentage describes the screen instead of
// the placeholders on it.
const renderer = "apps/desktop/src/renderer";
const used = new Set();
for (const file of globSync(`${renderer}/**/*.tsx`)) {
  const source = readFileSync(file, "utf8");
  for (const match of source.matchAll(/tr\(\s*[`'"]([^`'"]{2,60})[`'"]\s*\)/g)) {
    if (!isExample(match[1])) used.add(match[1]);
  }
}
console.log(`\ndesktop shows ${used.size} distinct phrases through tr()`);
for (const [language, list] of tables) {
  const have = new Set(list);
  const missing = [...used].filter((phrase) => !have.has(phrase)).length;
  const percent = ((used.size - missing) / used.size) * 100;
  console.log(`${language.padEnd(8)} ${percent.toFixed(1).padStart(5)}% of the desktop screen  ${String(missing).padStart(4)} missing`);
}
