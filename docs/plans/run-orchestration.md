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
