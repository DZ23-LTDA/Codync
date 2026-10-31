#!/usr/bin/env node
// One place holds the mascot artwork; every client gets a copy derived from it. Run without flags to see what is
// missing or outside the contract, with --derive to write the per-client copies.
//
// The id list is read from the host's own catalog, so adding a mascot is: add the id in avatar.rs, drop the PNG here.
import { copyFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

const source = "assets/mascots";
const catalog = "host/src/agent/avatar.rs";

// 8-byte signature, then IHDR: width and height are big-endian 32-bit at bytes 16 and 20.
function pngSize(file) {
  const bytes = readFileSync(file);
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (bytes.length < 24 || signature.some((byte, index) => bytes[index] !== byte)) return null;
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

function ids() {
  const code = readFileSync(catalog, "utf8");
  const start = code.indexOf("pub const IDS");
  if (start === -1) throw new Error(`no IDS array in ${catalog}`);
  const block = code.slice(start, code.indexOf("];", start));
  return [...block.matchAll(/"([a-z0-9-]+)"/g)].map((match) => match[1]);
}

// Each client keeps its own folder; the copies are build output, never edited by hand.
const clients = [
  { file: "apps/desktop/resources/mascots", note: "desktop (Electron resources)" },
  { file: "apps/ios/Kit/Sources/CodyncKit/Resources/Mascots", note: "iOS package resources" },
  { file: "apps/android/design/src/main/res/drawable-nodpi", note: "Android drawables" },
  { file: "host/assets/mascots", note: "host (bundled with the daemon)" },
];

const mascots = ids();
const derive = process.argv.includes("--derive");
const missing = [];
const wrong = [];

for (const id of mascots) {
  const file = join(source, `${id}.png`);
  if (!existsSync(file)) {
    missing.push(id);
    continue;
  }
  const size = pngSize(file);
  if (size === null) {
    wrong.push(`${file}: not a PNG`);
    continue;
  }
  const longest = Math.max(size.width, size.height);
  if (longest < 256 || longest > 512) {
    wrong.push(`${file}: ${size.width}x${size.height}, the longest side must be 256-512 px`);
  }
}

console.log(`${mascots.length} mascots in the catalog, ${mascots.length - missing.length} with artwork`);
if (missing.length > 0) console.log(`missing: ${missing.join(", ")}`);
for (const problem of wrong) console.log(`out of contract: ${problem}`);

if (derive && mascots.length > missing.length) {
  const present = mascots.filter((id) => !missing.includes(id));
  for (const client of clients) {
    mkdirSync(client.file, { recursive: true });
    for (const id of present) {
      copyFileSync(join(source, `${id}.png`), join(client.file, `${id}.png`));
    }
    console.log(`derived ${present.length} into ${client.file} — ${client.note}`);
  }
} else if (derive) {
  console.log("nothing to derive: no artwork is in the repository yet");
}

// Missing artwork is not a failure: a bot with an id and no picture shows the fallback. Artwork that exists but breaks
// the contract is, because it would ship wrong to four clients at once.
process.exit(wrong.length > 0 ? 1 : 0);
