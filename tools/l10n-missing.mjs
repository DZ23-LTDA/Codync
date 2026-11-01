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
  for (const pattern of patterns) {
    for (const match of source.matchAll(pattern)) {
      const phrase = match[1].trim();
      // A phrase made only of a variable name is not a phrase; neither is an empty string.
      if (phrase !== "" && !/^[A-Za-z_$][\w$]*$/.test(phrase)) found.add(phrase);
    }
  }
  return found;
}

/** The phrases used in the renderer with no key in `table`. Pure, so the rule is testable on its own. */
export function missingFrom(phrases, table) {
  return [...phrases].filter((phrase) => !(phrase in table)).sort();
}

function main() {
  const base = process.cwd();
  const table = JSON.parse(readFileSync(`${base}/${PT}`, "utf8"));
  const used = new Set();
  for (const file of globSync(`${base}/${RENDERER}/**/*.{ts,tsx}`)) {
    if (file.includes(".test.") || file.includes("/demo/")) continue;
    for (const phrase of phrasesIn(readFileSync(file, "utf8"))) used.add(phrase);
  }
  const missing = missingFrom(used, table);
  console.log(`${used.size} phrases reach tr() in the renderer`);
  console.log(`${missing.length} of them have no pt.json entry`);
  for (const phrase of missing) console.log(`  ${phrase}`);
  process.exit(0);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
