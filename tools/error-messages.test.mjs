import assert from "node:assert/strict";
import { test } from "node:test";

import { findErrorLeaks } from "./error-messages.mjs";

const kinds = (source) => findErrorLeaks(source).map((leak) => leak.kind);

test("printing a stack is flagged", () => {
  assert.deepEqual(kinds("setError(error.stack)"), ["stack"]);
});

test("a message from an error is not flagged as a stack", () => {
  assert.deepEqual(kinds("setError(error.message)"), []);
});

test("turning a thrown value into text is flagged for review", () => {
  assert.deepEqual(kinds("setError(e instanceof Error ? e.message : String(e))"), ["raw value"]);
});

test("the same line is counted once, as the worse problem", () => {
  assert.deepEqual(kinds("setError(String(e) + error.stack)"), ["stack"]);
});
