#!/usr/bin/env node
// What a person can be shown when something fails. A stack trace, an internal path or an error object printed whole is
// a bug in the message, not a detail: the release gate asks for errors that say what happened and what to do.
//
// This scans the renderer for the two ways that leaks happen: printing a stack, and turning an arbitrary thrown value
// into text. It fails only on the first — a thrown value is reviewed by a person, a stack is always wrong.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const root = "apps/desktop/src/renderer";

// Any read of `.stack`, the one thing that must never reach a screen.
const stackRead = /\.stack\b/;
// Turning a caught value into text for the screen: allowed, but a person should look at it once.
const rawValue = /String\(\s*(error|e|err|reason)\s*\)/;

export function findErrorLeaks(source, file = "") {
  const found = [];
  for (const [index, line] of source.split("\n").entries()) {
    // Prose about errors is not an error shown to anyone: a doc comment naming String(e) must not count as one.
    const trimmed = line.trim();
    if (trimmed.startsWith("//") || trimmed.startsWith("*") || trimmed.startsWith("/*")) continue;
    if (stackRead.test(line)) found.push({ file, line: index + 1, kind: "stack", text: line.trim() });
    // humanError() is the reviewed way to turn a thrown value into a sentence: it strips the class prefix, keeps one
    // line and falls back to a sentence the caller chose, so it is not a leak to review.
    else if (rawValue.test(line) && !line.includes("humanError("))
      found.push({ file, line: index + 1, kind: "raw value", text: line.trim() });
  }
  return found;
}

function main() {
  const base = process.cwd();
  const leaks = [];
  for (const file of globSync(`${base}/${root}/**/*.{ts,tsx}`)) {
    const relative = file.slice(base.length + 1).replace(`${root}/`, "");
    leaks.push(...findErrorLeaks(readFileSync(file, "utf8"), relative));
  }
  const stacks = leaks.filter((leak) => leak.kind === "stack");
  const review = leaks.filter((leak) => leak.kind !== "stack");

  console.log(`stack traces printed by the renderer: ${stacks.length}`);
  for (const leak of stacks) console.log(`  ${leak.file}:${leak.line}: ${leak.text}`);
  console.log(`thrown values turned into text (for review): ${review.length}`);
  for (const leak of review) console.log(`  ${leak.file}:${leak.line}: ${leak.kind} — ${leak.text}`);

  process.exit(stacks.length === 0 ? 0 : 1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
