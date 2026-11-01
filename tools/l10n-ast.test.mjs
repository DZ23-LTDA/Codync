// Locks what the syntax-tree scanner counts, so its number can be trusted the way the regex ones could not be.
import { execFileSync } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import assert from "node:assert/strict";

const ROOT = "apps/desktop/src/renderer/views/__ast_fixture__";

function scan(body) {
  mkdirSync(ROOT, { recursive: true });
  const file = join(ROOT, "Sample.tsx");
  writeFileSync(file, body, "utf8");
  try {
    return execFileSync(process.execPath, ["tools/l10n-ast.mjs", "--list"], { encoding: "utf8" });
  } catch (error) {
    return error.stdout ?? "";
  } finally {
    rmSync(ROOT, { recursive: true, force: true });
  }
}

const page = (inner) => `export function Sample() {\n  const tr = useTr()\n  return (\n${inner}\n  )\n}\n`;

test("text between tags that never reaches tr is counted", () => {
  assert.match(scan(page("    <p>No matching bots</p>")), /No matching bots/);
});

test("a title built from a name is counted", () => {
  assert.match(scan(page("    <button title={`Remove ${bot.name}`} />")), /Remove/);
});

test("a class name built the same way is not counted", () => {
  assert.doesNotMatch(scan(page("    <div className={`member ${on ? 'on' : ''}`} />")), /member/);
});

test("what is already inside tr is not counted", () => {
  assert.doesNotMatch(scan(page("    <p>{tr('No matching bots')}</p>")), /No matching bots/);
});

test("a comment is not counted", () => {
  assert.doesNotMatch(scan(page("    <p>{/* Nothing (at all) */}</p>")), /Nothing/);
});
