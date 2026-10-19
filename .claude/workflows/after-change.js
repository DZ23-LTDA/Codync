export const meta = {
  name: 'after-change',
  description: 'Review, fix, end-to-end test, then clean up and refactor a finished code change (docs/guides/clean-code.md)',
  whenToUse: 'After every code change once its tests pass, before commit. Run it from the change\'s worktree. args: {change: what changed and which flows to walk, base?: git ref (default origin/dev), e2e?: "minimal" | "full"}',
  phases: [
    { title: 'Review', detail: 'two reviewers, then one verifier that refutes what it can' },
    { title: 'Fix', detail: 'confirmed findings, then e2e failures' },
    { title: 'E2E', detail: 'the changed flows on the real apps' },
    { title: 'Cleanup', detail: 'cleanup edits and a read-only refactor review, then the refactors worth doing' },
    { title: 'Finish', detail: 'stop the worktree\'s npm run dev, sandbox host and Electron' },
  ],
}

const input = typeof args === 'string' ? { change: args } : (args ?? {})
const change = input.change ?? 'the uncommitted and unpushed change in this worktree'
const base = input.base ?? 'origin/dev'
const e2e = input.e2e === 'full' ? 'full' : 'minimal'
const scope = `The change: ${change}
Its diff: \`git diff ${base}\` (committed and uncommitted) plus the new files \`git status --short\` lists, in the current directory (the change's git worktree). Touch nothing outside that worktree and no other agent's work.`

const FINDINGS = {
  type: 'object',
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          file: { type: 'string' },
          line: { type: 'integer' },
          severity: { type: 'string', enum: ['high', 'medium', 'low'] },
          summary: { type: 'string' },
          failure_scenario: { type: 'string', description: 'concrete inputs or state and the wrong result' },
          fix: { type: 'string' },
        },
        required: ['file', 'severity', 'summary', 'failure_scenario', 'fix'],
      },
    },
  },
  required: ['findings'],
}

const VERDICTS = {
  type: 'object',
  properties: {
    confirmed: { type: 'array', items: FINDINGS.properties.findings.items },
    refuted: { type: 'array', items: { type: 'object', properties: { summary: { type: 'string' }, why: { type: 'string' } }, required: ['summary', 'why'] } },
  },
  required: ['confirmed', 'refuted'],
}

const FIXED = {
  type: 'object',
  properties: {
    applied: { type: 'array', items: { type: 'string' } },
    skipped: { type: 'array', items: { type: 'object', properties: { item: { type: 'string' }, why: { type: 'string' } }, required: ['item', 'why'] } },
    tests: { type: 'string', description: 'which builds and tests ran and their result' },
  },
  required: ['applied', 'skipped', 'tests'],
}

const CHECKS = {
  type: 'object',
  properties: {
    checks: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          platform: { type: 'string' },
          flow: { type: 'string' },
          result: { type: 'string', enum: ['pass', 'fail', 'not_run'] },
          detail: { type: 'string', description: 'what was seen; for fail the symptom, for not_run why' },
        },
        required: ['platform', 'flow', 'result', 'detail'],
      },
    },
  },
  required: ['checks'],
}

const LENSES = [
  { name: 'correctness', text: 'correctness: bugs, regressions, broken edge cases, error handling, data loss, security, and compatibility with older clients and hosts (docs/reference/compatibility.md)' },
  { name: 'completeness', text: 'completeness: every client the change should reach (iPhone, Android, desktop, terminal UI, per CLAUDE.md cross-platform rules), docs made stale by it, missing or weak tests, and the version bump and What\'s New rules' },
]

phase('Review')
const reviews = await parallel(LENSES.map((lens) => () =>
  agent(`Review a code change, read-only (edit nothing). Lens: ${lens.text}.
${scope}
Report only real defects with a concrete failure scenario, most severe first; style nits are not findings. Return an empty list when there are none.`,
    { label: `review:${lens.name}`, phase: 'Review', schema: FINDINGS })))
const found = reviews.filter(Boolean).flatMap((r) => r.findings)
let confirmed = []
let refuted = []
if (found.length) {
  const verdict = await agent(`Verify review findings on a code change, read-only. For each finding, read the code and try to refute it: is the failure scenario real in this code? Default to refuted when you can't reproduce the reasoning. Merge duplicates.
${scope}
Findings:
${JSON.stringify(found, null, 2)}`, { label: 'verify', phase: 'Review', schema: VERDICTS })
  confirmed = verdict?.confirmed ?? found
  refuted = verdict?.refuted ?? []
}
log(`Review: ${found.length} found, ${confirmed.length} confirmed`)

const fix = (what, items) => agent(`Fix these ${what} in the change, the smallest correct fix at the root cause, then rerun the builds and tests the fix touches (CLAUDE.md lists each part's checks).
${scope}
${JSON.stringify(items, null, 2)}`, { label: `fix:${what}`, phase: 'Fix', schema: FIXED })

phase('Fix')
const reviewFix = confirmed.length ? await fix('review findings', confirmed) : null

const walk = (round) => agent(`End-to-end test a code change on the real apps, ${e2e} e2e as CLAUDE.md "End-to-end test every change" defines it (minimal: only this computer's platform; full: also Lima Linux and the UTM Windows VM, skipping and reporting a VM that isn't installed).
${scope}
Walk the flows the change touches, as a person would, and note what you saw. Stop the old build before installing the new one. In this worktree \`npm run dev\` in apps/desktop runs an isolated sandbox (docs/guides/dev-app.md#worktree-sandboxes); never stop or replace the shared Codync Dev, the production Codync app or another session's sandbox, and stop everything you started when done. A change with nothing to walk (CI, tools, docs) reports one not_run check saying so.${round > 1 ? ' This is the second walk, after fixes for the failures of the first.' : ''}`,
  { label: `e2e:${round}`, phase: 'E2E', schema: CHECKS })

phase('E2E')
let walked = await walk(1)
let e2eFix = null
const failures = (walked?.checks ?? []).filter((c) => c.result === 'fail')
if (failures.length) {
  e2eFix = await fix('e2e failures', failures)
  walked = await walk(2)
}

phase('Cleanup')
const [cleanup, refactor] = await parallel([
  () => agent(`Cleanup pass on a code change (docs/guides/clean-code.md "After every change"): edit directly. Remove dead code, duplication, unclear names, comments that restate the code or no longer match it, unused imports; reuse existing helpers. Keep behavior identical and match the surrounding code. Rerun the affected builds and tests.
${scope}
Return what you changed, one line each, and the test result.`,
    { label: 'cleanup', phase: 'Cleanup', agentType: 'code-simplifier:code-simplifier' }),
  () => agent(`Refactor review of a code change (docs/guides/clean-code.md "After every change"), read-only: responsibility splits, files past ~500 lines, long functions, layer leaks, missed reuse, language best practices (Swift 6, Rust rust-skills, strict TypeScript, Compose), and any bug you notice. Only suggestions worth their diff.
${scope}`, { label: 'refactor-review', phase: 'Cleanup', schema: FINDINGS }),
])
const refactors = refactor?.findings ?? []
const applied = refactors.length
  ? await agent(`Apply the refactor suggestions worth doing to the change, skip the rest with one line why each, then rerun the affected builds and tests. A cleanup agent just edited the same files: work on the current state.
${scope}
${JSON.stringify(refactors, null, 2)}`, { label: 'apply-refactors', phase: 'Cleanup', schema: FIXED })
  : null

phase('Finish')
const stopped = await agent(`Stop every process this worktree started: \`npm run dev\` (apps/desktop tools/dev.mjs), its Vite, its Electron app and its sandbox host. Find them with \`ps\`/\`lsof\` by this worktree's path in their command line or cwd, stop them, then check that none remain. Never stop processes of the main checkout, another worktree, the shared Codync Dev or the production Codync app.
${scope}
Return what you stopped, one line each, or that nothing was running.`, { label: 'stop-dev', phase: 'Finish', effort: 'low' })

return {
  review: { found: found.length, confirmed, refuted, fix: reviewFix },
  e2e: { checks: walked?.checks ?? [], fix: e2eFix },
  cleanup,
  refactor: { suggested: refactors, applied },
  stopped,
  next: 'Unless an e2e check failed or a skipped fix matters: commit in this worktree, rebase on origin/dev, fast-forward dev, push origin dev, pull dev in the main checkout. Then leave the worktree and, from the main checkout, delete it: `git worktree remove --force <worktree>`, `git branch -D <branch>`, then `npm run dev -- --prune` in apps/desktop. Check that no `npm run dev` from it is still running.',
}
