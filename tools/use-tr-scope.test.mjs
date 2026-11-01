import assert from "node:assert/strict";
import { test } from "node:test";

import { translatorsOutsideFunctions } from "./use-tr-scope.mjs";

test("a translator inside a component is fine", () => {
  const source = [
    "export function Row() {",
    "  const tr = useTr()",
    "  return <span>{tr('Hi there')}</span>",
    "}",
  ].join("\n");
  assert.deepEqual(translatorsOutsideFunctions(source, "row.tsx"), []);
});

test("a translator at module scope is reported with its line, because React throws an invalid hook call", () => {
  const source = ["import { useTr } from './use-language'", "  const tr = useTr()", "const list = [tr('One')]"].join("\n");
  assert.deepEqual(translatorsOutsideFunctions(source, "settings.tsx"), [{ file: "settings.tsx", line: 2 }]);
});

test("a translator inside a nested function is still inside a function", () => {
  const source = [
    "function outer() {",
    "  const inner = () => {",
    "    const tr = useTr()",
    "    return tr('Nested phrase')",
    "  }",
    "}",
  ].join("\n");
  assert.deepEqual(translatorsOutsideFunctions(source, "nested.tsx"), []);
});
