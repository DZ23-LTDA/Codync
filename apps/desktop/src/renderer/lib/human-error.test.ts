import assert from "node:assert/strict";
import { test } from "node:test";

import { humanError, humanErrorIn } from "./human-error.ts";

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
  const sentence = humanErrorIn("en", new Error(""));
  assert.ok(sentence.includes("Try again"), "asks for a retry");
  assert.ok(sentence.includes("host is running"), "and names the thing to check");
});

test("a refused connection is the caller's sentence, not the socket's error", () => {
  // The detail a person can do nothing with: the caller knows what it was trying to do and says what to try.
  const fallback = 'The memory list could not be loaded. Try again.'
  for (const plumbing of [
    'connect ECONNREFUSED 127.0.0.1:19222',
    'Error: getaddrinfo ENOTFOUND relay.example',
    'fetch failed',
    'socket hang up',
    'read ECONNRESET',
  ]) {
    assert.equal(humanError(new Error(plumbing), fallback), fallback, plumbing)
  }
})

test("a message that merely mentions a network word is still shown", () => {
  // Only the shapes a failed connection arrives in are replaced; a sentence about being offline is a sentence.
  assert.equal(humanError(new Error('Your computer is offline.')), 'Your computer is offline.')
  assert.equal(humanError(new Error('Reconnect it and try again.')), 'Reconnect it and try again.')
})