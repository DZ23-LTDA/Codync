import assert from "node:assert/strict";
import { test } from "node:test";

import { humanError } from "./human-error.ts";

test("an error's message is shown without the class prefix", () => {
  assert.equal(humanError(new Error("The host is not reachable")), "The host is not reachable");
  assert.equal(humanError(new TypeError("Error: already prefixed")), "already prefixed");
});

test("only the first line of a chain reaches the screen", () => {
  const chained = new Error("Could not open the database\ncaused by: no such file\nat handler (/Users/ana/x.ts:3)");
  assert.equal(humanError(chained), "Could not open the database");
});

test("a thrown value with no message falls back to the caller's sentence", () => {
  assert.equal(humanError(undefined, "Could not pair"), "Could not pair");
  assert.equal(humanError(null, "Could not pair"), "Could not pair");
  assert.equal(humanError("", "Could not pair"), "Could not pair");
  assert.equal(humanError(new Error(""), "Could not pair"), "Could not pair");
});

test("a plain string is used as it is", () => {
  assert.equal(humanError("The code expired"), "The code expired");
});

test("a long message is cut, with an ellipsis, so a screen does not overflow", () => {
  const long = humanError(new Error("x".repeat(400)));
  assert.equal(long.length, 200);
  assert.ok(long.endsWith("…"));
});

test("the default sentence tells the person what to check, not just that it failed", () => {
  const sentence = humanError(new Error(""));
  assert.ok(sentence.includes("Try again"), "asks for a retry");
  assert.ok(sentence.includes("host is running"), "and names the thing to check");
});
