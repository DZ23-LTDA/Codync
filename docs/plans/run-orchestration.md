# Run: one task, several bots, one result

A bot asked for a second opinion today opens a request per bot and gets a reply per bot: it has to line them up
itself, and nothing records what it concluded. A **run** does that in the host, where the transcript, the cost and the
audit already live.

## Contract

```text
startRun { bots: [id…], task: string, schema?: object, timeoutSec?: number } -> { runId }
runStatus { runId } -> { runId, status, answers: [{ bot, name, status, text }], merged }
```

- `bots`: 2–8 bots on this computer, visible to the caller. Duplicates and the caller itself are refused.
- `schema`: the JSON object every answer must carry (keys only; types are the bots' problem). Without it a run is a
  plain fan-out: answers come back verbatim and nothing is merged.
- `status`: `running` → `done` | `timedOut` | `cancelled`.

## Shape of the work

1. **Fan-out.** One request per bot through the existing bot-to-bot path (`chat::team`), each carrying `runId` so the
   ledger can follow it. Requests go out at once; each bot's own queue serializes its turns as usual.
2. **Join** (`chat::run::merge`). Deterministic: a field counts as agreed only when every answering bot gave the same
   value; a split is recorded with each bot's value, and an answer that ignores the schema is kept verbatim. No model
   summarizes the models — the run has to stay auditable, and disagreement is the useful part.
3. **Ledger.** One `run` entry in the chat, updated in place: status, per-bot state, tokens spent (from
   `agent::bot::budget`), and the merged result. `runId` is on every child request and reply.
4. **Seal** (next slice). A run whose `task` is marked as an action ends `awaitingApproval`; only a human answer
   moves it on, and the approval is recorded once with who answered and when. Until then a bot may propose, never act.

## Refusals

- A bot that is not visible, is a group, or is the caller itself.
- Fewer than two bots, more than eight, or the same bot twice.
- A second run from the same bot while its first is still running (one queue, one ledger).
- A `schema` that isn't a JSON object.

## What this is not

- Not a subagent mechanism: native subagents stay with the harness.
- Not a route around permissions: each request runs in the bot's own folder and permission, as `message_bot` does.
- Not multi-computer: bots here are on this computer; `codync-network` bots are a separate step.

## Implementation seams (where the code goes)

Found while wiring stage 1; recorded so the next commit doesn’t rediscover them:

- **Fan-out.** `chat::team::message_bot(hub, source, to, message)` already creates one request and returns
  `{requestId, botId, name, status}`; it only needs `pub(crate)`. A run calls it once per bot, in the order the
  user listed them, with the task plus the schema the answer must carry.
- **Who may run.** `chat::team::visible_bot(hub, from)` is the visibility check every team tool uses; the caller
  itself, a group and a bot that isn’t visible are refused from there.
- **Reply capture.** A recipient’s answer comes back as `Cmd::BotReply` carrying `request_id` and the outcome;
  the sender’s actor turns it into the next turn’s prompt (`chat::team::reply_prompt`). The ledger records the text
  there, keyed by `request_id`, so a reply that arrives while the sender is busy still lands in the run.
- **Statuses.** A request’s state is the `RequestStatus` in its entry’s data, read with `store.entry(id)`; the
  run doesn’t keep a second copy that can drift.
- **Ledger.** One `kv` key per run (`run.<id>`): task, schema, the bots and their request ids, created-at. Small
  and rewrite-on-change; no schema migration, same choice as the daily budget.
- **API.** A new method needs an arm in `api::dispatch` and a line in the permission table
  (`api::devices::permit`): a run is control-plane, not `LOCAL_ONLY`.
- **No tool.** Registering anything in the `team` server’s `tools()` changes what delegated turns see and broke
  `files_e2e`/`network_e2e`; the run is driven by the host API and, later, by an explicit tool only with tests
  that exercise a delegated turn.
