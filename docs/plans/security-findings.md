# Security findings — reconciling the GitGuard scan against `dev`

Source: GitGuard scan `cmvy6nwnz08ehctqpcb5j23s3`, 2026-11-01, branch **`main` @ `7f33f3cb419a`**,
156 findings (2 CRITICAL, 35 HIGH, 112 MEDIUM, 7 LOW; Trivy 35, Semgrep 115, Gitleaks 6).

The free report carries **no file or line**, so everything here was reproduced on `dev` by hand:
package versions from the lockfiles, `npm audit` in every JS workspace, and a directed search for
each Semgrep rule. Where a pattern no longer exists, the finding is `NOT_REPRODUCED_ON_DEV`.

## Dependencies — 35 findings

| Package | In the scanned `main` | In `dev` today | Findings | Status |
|---|---|---|---|---|
| `next` | 16.2.1 | **16.4.3** | 25 (2 CRITICAL) | fixed on `dev` (#159) |
| `sharp` | 0.34.5 | **^0.35.5** | 2 HIGH | fixed on `dev` |
| `nanoid` | 3.3.11 | **^3.3.16** | 3 HIGH | fixed on `dev` |
| `postcss` | 8.4.31 | **8.5.23** | 4 (2 HIGH) | fixed on `dev` |
| `baseline-browser-mapping` | 2.10.10 | **^2.11.33** | 1 MEDIUM | fixed on `dev` |

Independent evidence (the `npm audit` database, not the scan):

```
web            critical=0 high=0 moderate=16 low=0
apps/desktop   critical=0 high=0 moderate=15 low=0
cloud          critical=0 high=0 moderate=2  low=0
relay          critical=0 high=0 moderate=0  low=0
private-relay  critical=0 high=0 moderate=0  low=0
```

The 33 moderates are build/test tooling (`eslint-*`, `electron-builder` and its
`app-builder-lib`/`dmg-builder`/`ejs`, `vitest`), not code that ships to a user. **No known CRITICAL
or HIGH on `dev` today.** Status: `VERIFIED_FIXED` for CRITICAL/HIGH.

## Secrets — 6 `generic-api-key` findings

Directed scan over every tracked file (`sk_live_`, `sk_test_`, `AKIA`, `ghp_`, `github_pat_`,
`xox*-`, PEM private key, `phc_`, `AIza`):

| Where | What it is | Status |
|---|---|---|
| `cloud/test/dev-sandbox.test.ts` | test strings that check "a sandbox only takes a development key" | false positive (fixture) |
| `.env.development.example` | `sk_test_…` placeholder in the template | false positive (placeholder) |
| `host/src/*/check.rs` | the repo's own secret-detection patterns | false positive (pattern) |
| `cloud/src/fcm.ts` | `private_key.includes("BEGIN PRIVATE KEY")` | false positive (a check) |
| `Analytics.kt`, `analytics.ts`, `Analytics.swift` | PostHog project key (`phc_…`) | public by design |

No value was printed, logged or committed. Status: `FALSE_POSITIVE_WITH_EVIDENCE`.

## SAST — 115 Semgrep findings

| Rule | Reproduced on `dev` | Status |
|---|---|---|
| `python.use-defused-xml(-parse)` ×4 | `apps/android/tools/agent_icons.py`, `packaging/updates/verify-appcast.py` | build-time input from this repo/pipeline → false positive, documented in both files |
| `javascript.spawn-shell-true` | no | `NOT_REPRODUCED_ON_DEV` |
| `javascript.detect-insecure-websocket` ×5 | only test fixtures and the `http://`→`ws://` upgrade in `host/src/remote/relay.rs` | false positive |
| `yaml.github-actions-mutable-action-tag` ×63 | no | fixed by #169 (actions pinned by SHA) |
| `html.missing-integrity` | no | `NOT_REPRODUCED_ON_DEV` |
| `java.android.exported_activity` ×2 | the widget receivers, the tile service and `WidgetConfigurationActivity` | by design: the launcher must reach them; they carry no sensitive data |
| `rust.reqwest-accept-invalid` | `host/src/usage/antigravity.rs` | **accepted**: a loopback-only client talking to a local language server's self-signed certificate (documented in the file) |
| `kotlin.gcm-detection` ×3 | detection code that refuses GCM | false positive |
| `javascript.detect-non-literal-regexp`, `regex_dos` | regexes built from constants | reviewed, not user input |
| `crypto.node_insecure_random_generator`, `node_sha1` | UI ids / a dev-only screenshot tool | not used for tokens |
| `javascript.insecure-object-assign`, `unsafe-formatstring` | desktop state merge, log formatting | reviewed |
| `python.dynamic-urllib-use` ×6 | `tools/` build scripts | build-time |

## Limits of this reconciliation

- Semgrep, Trivy and Gitleaks were not installed locally; the reproduction is by equivalent pattern
  search plus `npm audit`. A real rescan on the merge commit is the next step.
- `cargo audit` did not run (the tool is not installed here).
- Counts per file are this document's reproduction, not the scanner's.
