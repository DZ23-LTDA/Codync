#!/usr/bin/env node
// Prints the state of the Release 1 gate (goal v3.2 section 18, items 10 and 11 from addendum v3.4) with the numbers
// this repository can compute. The count comes from the file, so adding an item does not need a code change.
// The states come from a file a person edits; the numbers are computed here, so nobody can declare a number that the
// code does not produce.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";

const STATE = "docs/plans/release-state.json";
const ORDER = ["VERIFIED", "TESTED", "IMPLEMENTED", "IN_PROGRESS", "FAILED", "BLOCKED_BY_EXTERNAL_DEPENDENCY", "DEFERRED_BY_OWNER", "NOT_STARTED"];

function measure(command, args, fallback = "could not measure") {
  try {
    return execFileSync(command, args, { encoding: "utf8" }).trim();
  } catch (error) {
    return `${fallback}: ${error.message.split("\n")[0]}`;
  }
}

/** The first line of the unwrapped scanner: `314 visible strings never reach the translation:` or the all-clear. */
function unwrapped() {
  const out = measure("node", ["tools/l10n-unwrapped.mjs"]);
  const match = /^(\d+) visible strings/.exec(out);
  if (match) return Number(match[1]);
  if (out.startsWith("every visible string")) return 0;
  return null;
}

/** Mascots: the count the asset tool reports, or null when the tool could not run. */
function mascots() {
  const out = measure("node", ["tools/mascot-assets.mjs"]);
  const match = /(\d+) with artwork/.exec(out);
  return match ? Number(match[1]) : null;
}

if (!existsSync(STATE)) {
  console.error(`missing ${STATE}`);
  process.exit(1);
}
const { items } = JSON.parse(readFileSync(STATE, "utf8"));
const strings = unwrapped();
const artwork = mascots();

const width = Math.max(...items.map((entry) => entry.item.length));
for (const entry of items.slice().sort((a, b) => a.id - b.id)) {
  const state = ORDER.includes(entry.state) ? entry.state : `UNKNOWN(${entry.state})`;
  const evidence = entry.evidence ? `  ${entry.evidence}` : "";
  console.log(`${String(entry.id).padStart(2)}. ${entry.item.padEnd(width)}  ${state.padEnd(24)}${evidence}`);
}

const counts = items.reduce((tally, entry) => ({ ...tally, [entry.state]: (tally[entry.state] ?? 0) + 1 }), {});
const closed = (counts.VERIFIED ?? 0) + (counts.TESTED ?? 0) + (counts.IMPLEMENTED ?? 0);
console.log("");
console.log(`gate: ${closed}/${items.length} with implementation or better; ${counts.VERIFIED ?? 0} verified`);
// The numbers the gate asks for, from the file the measurements are written to: a report that prints "not measured"
// after it was measured is the same lie as printing a number that was never taken.
const measurementsPath = `${process.cwd()}/docs/plans/measurements.json`;
const measurements = existsSync(measurementsPath) ? JSON.parse(readFileSync(measurementsPath, "utf8")) : {};
const open = measurements.appOpenMs === undefined ? "not measured" : `${measurements.appOpenMs} ms (${measurements.appOpenWhere})`;
const token = measurements.firstTokenP95Ms === undefined
  ? "not measured"
  : `p95 ${measurements.firstTokenP95Ms} ms, median ${measurements.firstTokenMedianMs} ms (${measurements.firstTokenWhere})`;
console.log(`measured: strings outside the translation = ${strings ?? "unmeasurable"}; mascots with artwork = ${artwork ?? "unmeasurable"}`);
console.log(`measured: app open = ${open}; first token = ${token}; screens inspected = ${measurements.screensInspected ?? "not measured"}`);
// The first-use steps, from the same file: the runbook's budget means nothing without them.
const use = measurements.firstUseCreateBotMs === undefined
  ? "not measured"
  : `createBot ${measurements.firstUseCreateBotMs} ms · first answer ${measurements.firstUseFirstAnswerMs} ms`;
const approval = measurements.approvalCardMs === undefined
  ? "not measured"
  : `card ${measurements.approvalCardMs} ms · yes to turn end ${measurements.approvalYesMs} ms`;
const pairing = measurements.pairingQrMs === undefined
  ? "not measured"
  : `QR ${measurements.pairingQrMs} ms · connect ${measurements.pairingConnectMs} ms · redeem ${measurements.pairingRedeemMs} ms`;
console.log(`measured: first use = ${use}; approval = ${approval}; pairing = ${pairing}`);
console.log(`measured: first use numbers are ${measurements.firstUseWhere ?? "of unknown origin"}`);
process.exit(strings === 0 && artwork === 13 ? 0 : 1);
