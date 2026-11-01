#!/usr/bin/env node
// A screen that can be empty has to say so. This lists the views that render a collection and whether they carry an
// empty state of their own, so the gap is a number before it is a task.
//
// It is a report, not a gate: which views can actually be empty is a judgement, and a scanner that fails on a guess
// teaches people to ignore it.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const root = "apps/desktop/src/renderer/views";

// Something that can render zero of itself: a list, a table, a map over state.
const rendersCollection = /\.map\(|\.filter\(|for \(const .* of /;
// A phrase or component that stands in when there is nothing to show.
const emptyState = /\bEmpty\w*|empty\b|Nothing yet|No \w+ yet|nada ainda|Nenhum|No results|nenhum\b/i;

export function emptyStateOf(source) {
  const list = rendersCollection.test(source);
  const empty = emptyState.test(source);
  return { list, empty, covered: list && empty };
}

function main() {
  const base = process.cwd();
  const rows = [];
  for (const file of globSync(`${base}/${root}/**/*.tsx`)) {
    const relative = file.slice(base.length + 1).replace(`${root}/`, "");
    // A menu does not "become empty": it maps a constant list of choices. Counting it as an uncovered screen made the
    // number bigger and less true.
    if (/Menu\.tsx$/i.test(relative)) continue;
    const state = emptyStateOf(readFileSync(file, "utf8"));
    if (state.list) rows.push({ file: relative, ...state });
  }
  const covered = rows.filter((row) => row.covered).length;
  console.log(`views that render a collection: ${rows.length}`);
  console.log(`with an empty state:             ${covered}`);
  console.log(`without one:                     ${rows.length - covered}`);
  for (const row of rows.filter((entry) => !entry.covered)) console.log(`  ${row.file}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
