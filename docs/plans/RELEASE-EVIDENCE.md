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
| Android instrumented | `cd apps/android && ./gradlew :app:connectedDebugAndroidTest` | **127 tests, 0 real failures**, 24 *assumption* skips (need a live host, camera or speech). The XML says `failures="24"`: every one of those is `org.junit.AssumptionViolatedException`, an `assumeTrue` skip that AGP counts under failures. Re-run on `b3905bca` (02/11): BUILD SUCCESSFUL in 3m15s |
| Cloud / relay / private relay | `cd cloud && npm ci && npm test && npm run typecheck` (and the same in `relay`, `private-relay`) | tests pass |
| Website | `cd web && npm ci && npm run build`, then serve `out/` | build 200 in 6.5 s; `/`, `/download/mac.html`, `/download/windows.html`, `/docs.html`, `/compare/grok-bot.html`, `/privacy.html` all 200 (02/11, `9a48e7bb`, next 16.4.3) |
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

## The remote channel's security tests — what exists, and where

`GET /channel` is the one route with no bearer token: the handshake authenticates. Before adding a test there,
read this list — **the handshake is already covered from both sides**, and a new negative test may only
duplicate one of these.

| What | Test | Where |
|---|---|---|
| An unknown device cannot connect | `unknown_device_is_rejected` | `host/src/remote/channel.rs:719` |
| A `pair` hello without a code is refused before any crypto | `pair_hello_without_a_code_is_refused_before_any_crypto` | `channel.rs:763` |
| The pairing channel only takes `pair`, and a code works once | `pairing_channel_only_takes_pair_and_codes_are_single_use` | `channel.rs:781` |
| Replayed or malformed frames close the channel | `replayed_or_bad_frames_close_the_channel` | `channel.rs:892` |
| Revoking a device closes the channel and its subscription | `revoking_closes_the_channel_and_its_subscription` | `channel.rs:803` |
| A relay link's hello must match its open | `relay_link_hello_must_match_open` | `channel.rs:878` |
| A cancelled subscription ends | `cancelled_subscription_ends` | `channel.rs:905` |
| A device that stops reading ends the channel cleanly | `a_device_that_stops_reading_ends_the_channel_cleanly` | `channel.rs:941` |
| Account devices need a synced live lease | `account_devices_need_a_synced_live_lease` | `channel.rs:849` |
| An unknown protocol version is refused over a real socket (426) | `phone_pairs_and_chats_over_the_direct_channel` | `host/tests/e2e.rs:346` |
| A real phone pairs over the socket, then chats and subscribes | same test — `Phone::connect` drives the real X25519/signature handshake | `host/tests/e2e.rs:340` |
| A plain HTTP request never gets a socket; 25 refusals leave the host healthy; a 2 MiB frame costs only the sender his channel | `channel_surface_e2e` (3 tests) | `host/tests/channel_surface_e2e.rs` |

## Channel limits: what is capped, and what is not (measured 02/11)

Two different things get called "rate limiting" on the channel, and they are not in the same state:

**Pairing attempts are capped.** `PER_SOURCE_PER_MINUTE = 3`, `TOTAL_PER_MINUTE = 10` (`host/src/api/devices.rs:172`),
counted per source IP or relay link, pruned to a one-minute window, and wired into the `pair` command at
`host/src/remote/channel.rs:535` — over the limit the channel answers `RejectCode::RateLimited` (close code 4008)
with "Too many pairing attempts. Wait a minute and try again." The limiter itself has a unit test
(`attempts_are_limited_per_source_and_in_total`); **what has no test is the wiring** — nothing outside sends four
`pair` hellos and watches the fourth be refused.

**Concurrent connections are not capped at all.** Measured against a scratch host on 02/11:

| Connections opened at once | Refused | Host afterwards |
|---|---|---|
| 100 | **0** | 63 MB RSS, 62 threads, `/health` 200 |

Each unauthenticated connection holds a socket and a task until the handshake completes or the peer goes away,
and the pairing limit does not count them — a caller who never sends `hello` never touches it. That is a
resource-exhaustion surface (E3: *resource exhaustion, DoS*), not a known breach: nothing behind it can be
reached without the handshake. Putting a cap on **unauthenticated** connections while leaving established ones
alone is the shape of the fix, and it changes behaviour for anyone reconnecting, so it is the owner's call.
Still **not covered at the socket level**: the pairing-attempt limiter (its wiring is called at
`channel.rs:535`, but nothing outside sends four `pair` hellos and watches the fourth be refused).

## Windows: what an install actually does (measured 03/11, at `cc078c15`)

Item 4 asks for "clean install, update and rollback on three systems". Only one of the three can be measured here —
this machine is Windows and there is no Mac or Linux — so this is the Windows half, and it is **half done**: the
installer builds, installs, registers and uninstalls cleanly; **the installed app does not stay running**.

### It builds

```
cd apps/desktop && npm ci && npm run dist:win:dev
  • packaging  platform=win32 arch=x64 electron=44.10.1 appOutDir=dist\win-unpacked
  • building   target=nsis file=dist\codync-dev-windows-x64.exe archs=x64 oneClick=true perMachine=false
  • no code signing certificate configured, signing is skipped
exit 0 · dist\codync-dev-windows-x64.exe 140.5 MB · .blockmap 0.1 MB
```

`latest.yml` is **not** produced, so there is no update feed for the app to check — the Windows build cannot
update itself as packaged today.

### It installs

```
codync-dev-windows-x64.exe /S        exit 0
%LOCALAPPDATA%\Programs\codync-desktop-dev\
  Codync Dev.exe                     product 2.34.1.0, file 2.34.1, Authenticode NotSigned (expected: no certificate)
  resources\codync-host.exe          43.6 MB — the host travels with the app
Uninstall entry: "Codync Dev 2.34.1"
```

### The app exits immediately

| Started | Result |
|---|---|
| `Codync Dev.exe` (installed) | exits at once, code 0, no window, no process, nothing written |
| `dist\win-unpacked\Codync Dev.exe` | same — so it is not the installer's layout |
| with `--user-data-dir=<empty dir>` | exits at once, **code 9**, still writes nothing |
| with `--enable-logging --log-file=…` | no log file is produced at all |

The sources give two early exits — `yieldsToInstall()` (`app.exit(0)`, an update marker in prefs) and
`!app.requestSingleInstanceLock()` (`app.quit()`) — and neither leaves anything behind to inspect, because the
app's `userData` folder is never created. `npm run dev` in this same checkout does start the app, so the difference
is packaging, not the code.

That is a **defect to chase, not a conclusion**: the next step is to run the packaged `out/main/index.js` under the
bundled Electron with stdout attached, or add a one-line marker at the top of `main`, and see which exit is taken.

### It uninstalls — and leaves 140 MB behind

```
"Uninstall Codync Dev.exe" /S     exit 0
%LOCALAPPDATA%\Programs\codync-desktop-dev\   gone
Uninstall entry                                gone
processes                                      none
%LOCALAPPDATA%\codync-desktop-dev-updater\installer.exe   140.5 MB, still there
```

The updater's cache survives. electron-updater keeps its downloads there by design; an uninstaller that leaves a
140 MB installer behind is the kind of "orphan" item 4 names. Removed by hand here (it came from this test), and
recorded so it is not mistaken for the machine's own data.

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

## Android instrumented suite — re-run on 02/11

`./gradlew :app:connectedDebugAndroidTest` on the emulator `dsh_pixel` (Android 15, x86_64, software GPU) at
`b3905bca`, the tree that already carries the keyboard fix (#175) and the list-arrangement change (#173):

```
tests = 127 · failures = 24 · errors = 0 · BUILD SUCCESSFUL in 3m 15s
```

**All 24 are `org.junit.AssumptionViolatedException`** — `assumeTrue` guards for tests that need a live host,
a camera or speech on the device — and **not one is an assertion failure**:

| Class | Skips |
|---|---|
| LiveRosterTest | 3 |
| LiveTerminalTest, LiveCredentialsTest, CameraScannerTest, WidgetSystemTest | 2 each |
| LiveDraft, LiveManagement, LiveMemory, LivePlugins, LiveRoutines, LiveRouting, LiveSettings, LiveChat, LiveVoice, LiveVoiceSpeech, PlatformSpeech, NativeVoice, ComputerNotifications | 1 each |

AGP writes those under `<failure>` in `TEST-*.xml`, so `failures="24"` reads like a red suite when it is not.
Anyone re-running this should count `<failure>` blocks that are **not** `AssumptionViolatedException`: that
number is 0.

The suite exists in CI as a nightly job (`.github/workflows/android-instrumented.yml`, #176) because the
Android workflow only compiled the APK; this run is the local proof that it passes on the current tree.

## Website build and smoke (02/11)

`web/` had never been built in this session, and the dependency work in #159 (Next.js off 16.2.1) had never been
proven to install and build. It now has:

```
cd web && npm ci            # next 16.4.3, from the lockfile
npm run build               # ✓ Compiled successfully in 6.5 s · 43 static pages
python -m http.server       # serving out/
```

| Path | Status |
|---|---|
| `/` | 200, 1 200 979 bytes |
| `/download/mac.html` | 200 |
| `/download/windows.html` | 200 |
| `/docs.html` | 200 |
| `/compare/grok-bot.html` | 200 |
| `/privacy.html` | 200 |

**A trap worth writing down:** before `npm ci`, this checkout's `web/node_modules` still held **next 16.2.1**
while `package.json` and `package-lock.json` said 16.4.3 — the first build here ran the vulnerable version and
reported "Compiled successfully". Nothing on screen says which version ran; the log's `▲ Next.js 16.4.3` line
is the only tell. Anyone verifying a dependency fix locally must `npm ci` first, or they are verifying the
previous tree.