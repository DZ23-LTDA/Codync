#!/usr/bin/env node
// How complete each interface language is, against the largest table: what is missing says how much a screen would fall
// back to English, which is the number that decides whether a language is ready to show.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

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
