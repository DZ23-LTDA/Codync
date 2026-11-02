# Release 1 — where the evidence is

This is the index, not the story. **`release-1-reconciliation.md` says where each gate item stands and why;**
this file says how to check it again: the command to run, what it should print, and where the artifact lives.

Canonical source per kind of fact:

| Fact | Canonical source |
|---|---|
| Gate item state | [`release-state.json`](release-state.json) |
| Why an item sits where it sits, and what is missing | [`release-1-reconciliation.md`](release-1-reconciliation.md) |
| Numbers (latency, memory, first token, CPU) | [`release-1-reconciliation.md`](release-1-reconciliation.md) §Performance — the only place these numbers are written down |
| Security findings and their classification | [`security-findings.md`](security-findings.md) |
| What was recovered from the ZIP and when | [`B3PO-CHECKPOINT.md`](B3PO-CHECKPOINT.md) |
| Screenshots | `reports/` in the working copy (not committed: they are session evidence, described by path below) |

## Re-running everything

| Component | Command | Expected |
|---|---|---|
| Host | `cd host && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace --profile ci` | 866 tests, 0 failed |
| Host, interrupted write | `cd host && cargo test --test write_interruption_e2e -- --test-threads=1` | 2 passed |
| Host, damaged database | `cd host && cargo test --test database_damage_e2e -- --test-threads=1` | 3 passed |
| Host, first word | `cd host && cargo test --test first_token_e2e -- --ignored --nocapture` | three tests; the numbers below |
| Host, first word under load | `cd host && cargo test --test first_token_e2e first_word_with_eight -- --ignored --nocapture` | `… · p95 1212` on this machine |
| Desktop | `cd apps/desktop && npm ci && npm run typecheck && npm test && npm run build` | typecheck clean, tests pass, build ok |
| iOS package | `cd apps/ios/Kit && xcodebuild test -scheme CodyncKit-Package -destination 'platform=iOS Simulator,name=iPhone 16'` | macOS only |
| Android | `cd apps/android && ./gradlew :core:test :app:testDebugUnitTest :app:assembleDebug :app:lintDebug` | BUILD SUCCESSFUL |
| Android instrumented | `cd apps/android && ./gradlew :app:connectedDebugAndroidTest` | 127 tests, 0 failed (24 skipped: need a live host, camera or speech) |
| Cloud / relay / private relay | `cd cloud && npm ci && npm test && npm run typecheck` (and the same in `relay`, `private-relay`) | tests pass |
| Scanners (i18n) | `node tools/l10n-missing.mjs && node tools/l10n-unwrapped.mjs && node tools/l10n-coverage.mjs` | 815 phrases / 0 missing; 0 unwrapped; pt 1673 keys, 100% |
| Tools' own tests | `node --test tools/*.test.mjs && python tools/release-gate-test.py` | 41 passed; `ok` |

## The 11 items

`VERIFIED` means the behaviour was demonstrated and the evidence below is on the exact commit named.
`IN_PROGRESS`/`NOT_STARTED`/`BLOCKED` mean what they say — no item is marked VERIFIED on a merged PR alone.

| # | Item | State | Evidence | Where it is |
|---|---|---|---|---|
| 1 | i18n scanner corrected and recounted | **VERIFIED** | #184 `fix/tools` (decodes escapes; a translated phrase no longer reads as missing), #185 `ci/tools` (the scanners now have a workflow). Output: 815 phrases through `tr()`, **0** without a pt.json entry; **0** visible strings outside `tr()`; pt **1673 keys / 100%** | `tools/l10n-missing.test.mjs` (11 tests), `tools/l10n-missing.mjs`, the `tools` workflow run on #185 |
| 2 | fork release separated from upstream | **VERIFIED** | #164 kept the fork out of upstream URLs/tap; #184 pins the tap too. Probes: `RELEASE_GATE_REPO=leepokai/Codync` → exit 1; `RELEASE_GATE_TAP=leepokai/homebrew-codync` → exit 1; defaults proceed | `tools/release-gate.py` (two guards), `tools/release-gate-test.py`, `.github/workflows/release-gate.yml` (`if:` names the fork and `vars.RELEASE_GATE_ENABLED`) |
| 3 | database: missing, empty, valid, damaged, migrated, interrupted, recovered | **VERIFIED** | #43/#48/#50/#62 store tests; #181 `write_interruption_e2e.rs` (kill with the write locked, kill mid-burst); #182 `database_damage_e2e.rs` (process refuses newer schema, destroyed header, truncated file — file byte for byte intact, nothing put in its place) | `host/src/store/schema.rs` tests, `host/tests/write_interruption_e2e.rs`, `host/tests/database_damage_e2e.rs` |
| 4 | clean install, update and rollback on three systems | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | owner: publish and install; see the v2.34.0 gap below |
| 5 | first use under 3 minutes, QR, reconnect, uninstall | `BLOCKED_BY_EXTERNAL_DEPENDENCY` for the phone half | Desktop onboarding walked twice (#186 #189): two steps, Portuguese, empty state, error state, offline; Android onboarding walked on the emulator (`reports/android-e2e/pt-install-step.png`) | phone pairing needs the firewall rule (`release-1-reconciliation.md` §"Pareamento de celular") |
| 6 | human errors, empty and offline states | `IN_PROGRESS` | Desktop proved (#186): "Iniciando o host…", "O host não está rodando" + "Tentar de novo" at ~30 s, empty roster, "Offline" on the bot row after a click; nothing leaks a stack trace or an internal path | phone half and TUI untested |
| 7 | pt-BR in the released scope, visually inspected | `TESTED` | #166 #178 #179 (catalog complete) + #186 #189 (screens): onboarding, empty, error, offline, Extensions ×3 tabs, bot menu, delete dialog, chat, Stop — all Portuguese; third-party registry content stays English on purpose | phone inner screens blocked by pairing; iOS only in CI |
| 8 | performance measured | **VERIFIED** | #187 (first word p95 17 ms; 8 bots at once 1211–1212 ms), #188 (host CPU 28.6% of one core, 54 MB; a 62 KB answer in 0.68 s, opened in the window in 39 ms) | `docs/plans/release-1-reconciliation.md` §Performance is the canonical table; `tools/measure-host-cpu.ps1`, `tools/measure-heavy.ps1` re-run it |
| 9 | checkpoint, version bump, WhatsNew, tag | `IN_PROGRESS` | #190 bumped to **2.34.1** (host + desktop + `apps/project.yml`), wrote `## 2.34.1` notes for both phone apps, and refreshed `B3PO-CHECKPOINT.md` | the **tag** is the release trigger and needs the owner's authorization; the committed `project.pbxproj` needs `xcodegen` on a Mac to match 2.34.1 |
| 10 | delivery cycle: bump, tag, publish, update in app | `BLOCKED_BY_EXTERNAL_DEPENDENCY` | — | owner: tag `v2.34.1`, decide `TAP_TOKEN` + `RELEASE_GATE_ENABLED` |
| 11 | cancel a stuck turn | **VERIFIED** | #86/#87/#88 (180 s watchdog, 6 tests, E2E with a silent agent) + #183 (the **Stop** button exercised in a running window: the turn closes with "Stopped."; 17 s, the host's cancel grace) | `host/tests/stall_e2e.rs`, `tools/e2e-desktop/e2e-stop.cjs`, `reports/desktop-e2e/stop/` |

## What a release cut still needs (2.34.1)

1. **`xcodegen generate --spec apps/project.yml` on a Mac** so the committed Xcode project carries 2.34.1. The
   desktop release workflow regenerates it before building, so that path is fine without this; Xcode Cloud is
   not verified either way.
2. **The tag**: `git tag v2.34.1 && git push origin v2.34.1` from `main` after the dev → main merge. That is
   the only release trigger, and it needs the owner's authorization (goal §H).
3. **The Homebrew tap and the gate**: `TAP_TOKEN` and `vars.RELEASE_GATE_ENABLED` exist only when the owner
   creates them; without them the gate stays skipped by design and no tap is pushed.

## Known gaps, stated rather than hidden

- **v2.34.0's upload is incomplete**: `tools/release-gate.py` in a dry run reports the release is missing
  `latest.yml`, `latest-mac.yml` and the four `codync-host-*.update.json`. That is why in-app update has never
  been demonstrated, and it is a release-side repair the owner must make.
- **The phone's inner screens** (memory, connectors, settings inside the app) have not been inspected in any
  round: pairing needs two Windows Firewall rules the owner must allow.
- **iOS** has only ever been exercised by CI (`kit` workflow on macOS runners); no simulator or device run
  happened here, because this machine is Windows.
- **Item 8**: twenty turns in one conversation were not measured; the first attempt stopped on a bug in the
  measurement loop (recorded in the reconciliation doc).
