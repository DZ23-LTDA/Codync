// The derived file name is per client, not per mascot id: Android refuses what the others accept, and getting that
// wrong fails the resource merger rather than a test — which is how it was found.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { clientFile } from "./mascot-assets.mjs";

const android = { rename: "res" };
const desktop = {};

test("a client that takes file names gets the id as it is", () => {
  assert.equal(clientFile("astro", desktop), "astro.png");
  assert.equal(clientFile("steve-jobs", desktop), "steve-jobs.png");
});

test("Android gets a resource name: no hyphen, no keyword", () => {
  assert.equal(clientFile("astro", android), "astro.png");
  assert.equal(clientFile("steve-jobs", android), "steve_jobs.png");
  assert.equal(clientFile("joel-santana", android), "joel_santana.png");
  assert.equal(clientFile("rogerio-ceni", android), "rogerio_ceni.png");
});

test("byte is a Java keyword, so the Android drawable is not called byte", () => {
  // `byte` is a primitive type name: the merger refuses a drawable called that.
  assert.equal(clientFile("byte", android), "byte_avatar.png");
  assert.equal(clientFile("byte", desktop), "byte.png");
});

test("a name that starts with a digit is not a valid resource either", () => {
  assert.equal(clientFile("3po", android), "_3po.png");
});

test("the script still runs on its own, and importing it derives nothing", () => {
  // It used to call process.exit at import time, which would take a test runner down with it.
  const script = fileURLToPath(new URL("./mascot-assets.mjs", import.meta.url));
  const output = execFileSync(process.execPath, [script], { encoding: "utf8" });
  assert.match(output, /mascots in the catalog/);
});
