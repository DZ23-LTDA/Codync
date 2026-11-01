import assert from "node:assert/strict";
import { test } from "node:test";

import { emptyStateOf } from "./empty-states.mjs";

test("a list with no empty state is reported as uncovered", () => {
  assert.deepEqual(emptyStateOf("bots.map((b) => <Row bot={b} />)"), { list: true, empty: false, covered: false });
});

test("a list with an empty state is covered", () => {
  const source = "if (bots.length === 0) return <p>No bots yet</p>\nbots.map((b) => <Row bot={b} />)";
  assert.deepEqual(emptyStateOf(source), { list: true, empty: true, covered: true });
});

test("a screen with no collection is not counted at all", () => {
  assert.deepEqual(emptyStateOf("<p>Settings</p>"), { list: false, empty: false, covered: false });
});
