#!/usr/bin/env node
// Measures the iPhone and Android clients, which the desktop numbers said nothing about.
//
// Two different questions, kept apart on purpose:
//   1. how many visible literals go straight to Text(...) instead of the client's translator (coarse: a regex, so it
//      counts what it can see and says so);
//   2. which keys the clients *do* translate have no entry in the shared pt.json (exact for a literal key, because the
//      table either has it or it does not).
//
// Usage: node tools/l10n-clients.mjs [--list]
import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

const LIST = process.argv.includes("--list");
const TABLE = "apps/ios/Kit/Sources/CodyncKit/Resources/Languages/pt.json";
const pt = JSON.parse(readFileSync(TABLE, "utf8"));

function files(dir, extension) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (["node_modules", "build", "DerivedData", ".gradle", "test", "Tests"].includes(entry.name)) continue;
      found.push(...files(full, extension));
    } else if (entry.name.endsWith(extension) && !entry.name.includes("Test")) {
      found.push(full);
    }
  }
  return found;
}

// `${…}` and `\(…)` are values, not words: "\${(top.percent).roundToInt()}%" is a number on screen, not copy.
const withoutValues = (text) => text.replace(/\$\{[^}]*\}/g, " ").replace(/\\\([^)]*\)/g, " ");
const hasWords = (text) => /[A-Za-zÀ-ÿ]{2,}/.test(withoutValues(text)) && withoutValues(text).trim().split(/\s+/).length >= 1;
const decoded = (raw) => raw.replace(/\\"/g, '"');

/** Literal text handed to a view constructor without going through the translator. */
function untranslatedLiteral(line, call) {
  const pattern = new RegExp(call + '\\(\\s*"((?:[^"\\\\]|\\\\.)*)"', "g");
  const translator = call === "Text" && /L10n\.tr\(|\btr\(/.test(line) ? true : /L10n\.tr\(|\btr\(/.test(line);
  const found = [];
  let match;
  while ((match = pattern.exec(line)) !== null) found.push(decoded(match[1]));
  return translator ? [] : found.filter(hasWords);
}

/** Every key passed to the client's translator, with the file it came from. */
function translatedKeys(source, file) {
  const keys = [];
  const pattern = /(?:L10n\.tr\(|\btr\()\s*"((?:[^"\\]|\\.)*)"/g;
  let match;
  while ((match = pattern.exec(source)) !== null) keys.push({ key: decoded(match[1]), file });
  return keys;
}

const report = { ios: { literals: [], keys: [] }, android: { literals: [], keys: [] } };

for (const file of [...files("apps/ios/Kit/Sources", ".swift"), ...files("apps/ios/Widgets", ".swift")]) {
  const source = readFileSync(file, "utf8");
  source.split("\n").forEach((line, index) => {
    const literals = ["Text", "Button", "Label"].flatMap((call) => untranslatedLiteral(line, call));
    for (const text of literals) report.ios.literals.push({ file, line: index + 1, text });
  });
  report.ios.keys.push(...translatedKeys(source, file));
}

for (const file of files("apps/android", ".kt")) {
  const source = readFileSync(file, "utf8");
  source.split("\n").forEach((line, index) => {
    const literals = ["Text", "Button"].flatMap((call) => untranslatedLiteral(line, call));
    for (const text of literals) report.android.literals.push({ file, line: index + 1, text });
  });
  report.android.keys.push(...translatedKeys(source, file));
}

// A key with an interpolation or a printf slot is built at runtime: no table can hold it, so counting it as missing
// would inflate the number with things that can never be there. They are counted apart and named.
// A cron expression is a value a person edits, not copy.
const cronKey = (key) => /^[\d*/,\- ]+$/.test(key.trim()) && key.trim().split(/\s+/).length === 5;
const composedKey = (key) =>
  /\\\(|%\d|%s|\$[A-Za-z_]/.test(key) ||
  cronKey(key) ||
  key.trim().replace(/[\s·%+\-–—.,:;/()]/g, "").length === 0;

const missing = (keys) => {
  const seen = new Map();
  for (const { key, file } of keys) if (!composedKey(key) && !(key in pt) && !seen.has(key)) seen.set(key, file);
  return seen;
};

const composed = (keys) => new Set(keys.filter(({ key }) => composedKey(key)).map(({ key }) => key)).size;

for (const client of ["ios", "android"]) {
  const literals = report[client].literals;
  const keys = report[client].keys;
  const gaps = missing(keys);
  console.log(`${client}: ${keys.length} translated keys · ${gaps.size} without a pt.json entry · ${composed(keys)} built at runtime · ${literals.length} literals straight to a view`);
  if (LIST) {
    for (const [key, file] of gaps) console.log(`  missing  ${key}   (${relative(".", file).split(sep).join("/")})`);
    for (const { file, line, text } of literals) {
      console.log(`  literal  ${relative(".", file).split(sep).join("/")}:${line}  ${text.slice(0, 70)}`);
    }
  }
}

const totalGaps = missing(report.ios.keys).size + missing(report.android.keys).size;
process.exit(totalGaps === 0 ? 0 : 1);
