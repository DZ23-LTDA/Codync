#!/usr/bin/env node
// Text a person reads that never reaches the translation: JSX text written straight into the markup, and attribute copy
// (placeholder, title, aria-label) that is not built from t(). Both show in English in every language, so this is the
// list that decides whether a screen is really translated.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";

const root = "apps/desktop/src/renderer";
const text = />\s*([A-Z][A-Za-z][^<>{}]{2,60})\s*</g;
const attribute = /\b(placeholder|title|aria-label)="([A-Z][^"]{2,60})"/g;
const found = [];

for (const file of globSync(`${root}/**/*.tsx`)) {
  const lines = readFileSync(file, "utf8").split("\n");
  for (const [index, line] of lines.entries()) {
    if (line.includes("tr(")) continue;
    for (const match of line.matchAll(text)) {
      const phrase = match[1].trim();
      if (/^(Promise|Codync|Math\.)/.test(phrase)) continue;
      found.push(`${file.replace(root + "/", "")}:${index + 1}: text "${phrase}"`);
    }
    if (!line.includes("tr(")) {
      for (const match of line.matchAll(attribute)) {
        found.push(`${file.replace(root + "/", "")}:${index + 1}: ${match[1]} "${match[2]}"`);
      }
    }
  }
}

if (found.length === 0) {
  console.log("every visible string goes through tr()");
} else {
  console.log(`${found.length} visible strings never reach the translation:`);
  for (const entry of found) console.log(`  ${entry}`);
}
