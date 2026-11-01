#!/usr/bin/env node
// A `const tr = useTr()` written at module scope is legal TypeScript and a broken app: React throws "Invalid hook call"
// the first time the file renders. The compiler does not catch it -- the demo console did. This walks brace depth and
// reports every translator declared outside a function body.
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

/** `[{ file, line }]` for each translator that is not inside a function. Depths are counted per file. */
export function translatorsOutsideFunctions(source, file = "") {
  const found = [];
  let depth = 0;
  source.split("\n").forEach((line, index) => {
    if (line.includes("const tr = useTr()") && depth === 0) {
      found.push({ file, line: index + 1 });
    }
    depth += (line.match(/\{/g) ?? []).length - (line.match(/\}/g) ?? []).length;
    if (depth < 0) depth = 0;
  });
  return found;
}

function main() {
  const root = process.cwd();
  const offenders = [];
  for (const path of globSync(`${root}/apps/desktop/src/renderer/**/*.tsx`)) {
    offenders.push(...translatorsOutsideFunctions(readFileSync(path, "utf8"), path.slice(root.length + 1)));
  }
  if (offenders.length === 0) {
    console.log("every translator is inside a component");
    process.exit(0);
  }
  console.log(`${offenders.length} translator(s) outside a function -- React will throw an invalid hook call:`);
  for (const { file, line } of offenders) console.log(`  ${file}:${line}`);
  process.exit(1);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
