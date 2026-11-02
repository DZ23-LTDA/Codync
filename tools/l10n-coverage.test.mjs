// The coverage percentage is the number a person quotes when asked "is the desktop translated?", so it gets the same
// escape handling the missing-key scanner has: a literal written as `'a\nb'` is the key with a real newline in the
// table, and comparing the raw `\n` reported a fully translated screen as 99.8%.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const script = fileURLToPath(new URL("./l10n-coverage.mjs", import.meta.url));

/** The lines the scanner prints for `language` on the screen-coverage table. */
function screenLine(language) {
  const output = execFileSync(process.execPath, [script], { encoding: "utf8" });
  const line = output.split("\n").find((row) => row.trimStart().startsWith(`${language} `) && row.includes("desktop screen"));
  assert.ok(line, `the scanner printed a screen line for ${language}`);
  return line;
}

test("pt covers the whole desktop screen, with nothing counted missing by accident", () => {
  const line = screenLine("pt");
  assert.match(line, /100\.0% of the desktop screen/, line);
  assert.ok(line.trimEnd().endsWith("0 missing"), line);
});

test("every other language is measured on the same screen, not a shorter one", () => {
  // All the others come from the same source scan, so they must agree with each other and be below pt.
  const others = ["es", "fr", "ja", "ko", "zh-Hans", "zh-Hant"].map((language) => ({ language, line: screenLine(language) }));
  const missing = others.map(({ line }) => Number(/(\d+) missing/.exec(line)?.[1] ?? "-1"));
  assert.ok(missing.every((count) => count > 0), "the other languages are not complete");
  assert.equal(new Set(missing).size, 1, `they share the same missing count, got ${missing.join("/")}`);
});
