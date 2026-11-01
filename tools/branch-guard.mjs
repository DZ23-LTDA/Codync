#!/usr/bin/env node
// Refuses to go on when the work is about to land on `dev` instead of a task branch.
//
// Why a script and not a pre-commit hook: several agents share this checkout and some of them commit to `dev` on
// purpose. A hook that blocks it would break their work, so this guard is run by whoever chooses to run it -- and it
// exists because the same mistake (commit on `dev`, then move it) happened four times in one session.
//
// Usage: node tools/branch-guard.mjs            (checks the current branch)
//        node tools/branch-guard.mjs --allow    (prints and passes, for the deliberate case)
import { execFileSync } from "node:child_process";

const branch = execFileSync("git", ["rev-parse", "--abbrev-ref", "HEAD"], { encoding: "utf8" }).trim();
const staged = execFileSync("git", ["diff", "--cached", "--name-only"], { encoding: "utf8" }).trim();
const allow = process.argv.includes("--allow");

if (branch !== "dev") {
  console.log(`branch-guard: on "${branch}" — a task branch, so a commit here is fine.`);
  process.exit(0);
}

if (!staged) {
  console.log("branch-guard: on \"dev\" with nothing staged. Create the branch before you edit, not after.");
  process.exit(0);
}

if (allow) {
  console.log(`branch-guard: on "dev" with ${staged.split("\n").length} staged file(s), allowed by --allow.`);
  process.exit(0);
}

console.error('branch-guard: you are on "dev" with staged changes.');
console.error(staged.split("\n").map((file) => `  ${file}`).join("\n"));
console.error("");
console.error("A task starts on its own branch:  git switch -c fix/what-you-are-doing");
console.error("If the change is deliberate on dev, say so:  node tools/branch-guard.mjs --allow");
process.exit(1);
