# Context and memory

How a bot keeps its instructions, context and long-term memory. Codync owns memory; the ACP harness owns the
conversation and compaction. The design, including what is still to come, is in
[bot memory and compaction](../design/bot-memory.md).

## Transcript, context and memory

| | Where | Owner | Lifetime |
|---|---|---|---|
| Transcript | SQLite `entries` | Codync | Persistent main chat and flat reply threads per bot |
| Model context | ACP session (`bots.session_id`) | The harness | Until *New session*, a failed resume, or an agent/command/folder change |
| Long-term memory | SQLite `memory_*` tables | Codync: the user, the bots' memory tools, dreaming | Forever, across sessions |

Codync never replays the transcript into a session. Each turn sends only the new message(s); the harness keeps
its own context and compacts it itself.

## Instruction snapshot (`host/src/chat/context.rs`)

- The bot's instructions are rendered once, in English and kept general: profile (name, description), how to
  message the user, other conversations, workspace, skills, plus the memory section. How to use each tool
  lives with the tool (MCP server instructions, tool descriptions), not here. They're stored as a snapshot in `kv` under `context.<bot>`, keyed by **session + compaction
  epoch**, and don't change until one of those changes. This keeps the prompt byte-identical so the prompt
  cache stays warm.
- **Claude** (`agentCapabilities._meta.claudeCode` present) receives the snapshot as a real system prompt:
  `_meta.systemPrompt.append` on `session/new` and `session/load`. It survives compaction.
- **Other harnesses** receive it in the first message of each session (`<bot-profile>…</bot-profile>`).
- **Compaction epoch:** Codync advertises `clientCapabilities.session.compaction`. Each completed
  `compaction_update` (deduplicated by `compactionId`) increments `context.epoch.<bot>`. The next turn
  re-renders the snapshot, and Claude's session is loaded again with the new system prompt. The adapter
  rebuilds its query and resumes the same conversation.
- **Profile edits mid-session** (name, description, skills) are sent once as an `<agent_profile_update>` block
  appended to the next message. After that turn reaches the agent, the change is recorded in
  `snapshot.announced`. The next compaction folds it into the snapshot.

## Memory (`host/src/chat/memory/`, crate `host/crates/memory`)

Memory records the person, not the work; the [design](../design/bot-memory.md) explains why. The rules live in
the `codync-mem` crate; `chat/memory` runs them in the host.

- **Two layers** in the host database (`memory_entries`): "yours", shared by every bot that uses it, and one per
  bot. Caps count tokens (one per CJK character, one per four others): 400 for "yours", 1,200 per bot. A bot in
  its own workspace uses "yours" by default and one in a project folder doesn't; the user switches it per bot
  (`sharedMemory`).
- **The opening** holds every active entry the bot sees, pinned first, within the caps, with ids so the bot can
  revise or archive one. Archived entries stay searchable (`search_memory`). A change the user makes reaches
  each session on its next message as a `<memory_update>` with the whole memory; what a bot or dreaming wrote
  reaches other sessions at their next compaction or new session.
- **The bot's tools** (built-in `memory` MCP server): `remember`, `revise_memory`, `archive_memory`,
  `search_memory`, `search_history`. A bot on automatic approval writes at once, in any turn and to either layer;
  the chat shows a "Remembered" line the user can undo. A bot that asks first shows a card the user approves or
  declines, unless the user's own message in that turn asked it to remember. A declined or undone memory is
  never written again by a bot.
- **Checks** on every write but the user's own (`codync_mem::check`): no hidden characters, prompt-carrying
  links, planted instructions, passwords, keys, ID or card numbers, nothing the user forgot, no duplicates, and
  the cap. Entries the user wrote can't be changed by a bot or dreaming.
- **Dreaming** replaces the keeper (`chat/memory/dream.rs`). After five idle minutes or eight user messages
  (sooner on a new session or compaction, and once a day for the time review), it reads the bot's chat and
  threads since its bookmark on the bot's own harness (Claude: Haiku, no tools). Each change must quote what the
  user typed: pasted text (the apps send `pasted`), fenced code, quoted lines and forwarded messages don't count,
  nor do bare acknowledgements. It only adds and archives, at most 10 changes and never more than a quarter of
  the entries, and drops a run when the user edited memory meanwhile. It waits while the plan's usage is above
  75%. Every run is logged and can be undone; what an undo removes isn't learned again.
- **Claude Code's own memory** is off for bots in their own workspace (`autoMemoryEnabled: false`,
  `CLAUDE_CODE_DISABLE_AUTO_MEMORY`); auto-dream is off for every bot. A project bot's `MEMORY.md` shows
  read-only on its memory screen.
- **The apps** (iPhone, desktop, terminal) show "About you", "This bot", the dream log and the project's notes,
  with add, edit, pin, archive, forget and forget-everything. They fetch with `memory` and fetch again when the
  bot's `memoryRev` moves. Methods: `memory`, `saveMemory`, `pinMemory`, `archiveMemory`, `forgetMemory`,
  `clearMemory`, `undoDream`, `answerMemory`, `undoMemory`.
- **Moving the old memory**: on the first start of this version, each bot's Engram database (or the Markdown
  memory before it) moves in once. What the old opening showed comes in active as far as the cap allows (the
  profile and pinned facts first, then the newest), the rest archived; then the old files (deleted bots' too) and
  the Engram download go. Dreaming starts from that point in the chat.

## Turns

- Messages sent while the agent works are folded into one next turn, joined by blank lines.
- Bot-to-bot requests occupy separate turns in the same queue; user messages are only folded together up to the next request. Requests never feed dreaming. See [bot collaboration](bot-collaboration.md).
- Delegated turns do not set the restart marker: their waiter disappears on restart. Pending delegation notices become interrupted errors instead of automatically replaying work.
- Stop sends `session/cancel`; an agent still running 15 seconds later is stopped (it resumes its session on
  the next turn). `session/new`, `session/load` and `session/fork` get three minutes before the agent is
  stopped. Connector or computer changes restart an agent that can resume sessions before its next turn; one
  that can't keeps its session, and its next new session gets them.
- `turn.inflight.<bot>` records the running turn start and its thread lane. If the host stops mid-turn (update, crash, restart), the next
  start resumes that session with a hidden "you were interrupted, don't repeat finished steps" prompt. If the
  session can't be resumed, or the turn started over an hour ago, it does nothing.

## Limits

- Harnesses other than Claude have no system prompt channel and don't report compaction. For them, the first
  message carries the instructions and the snapshot only refreshes when a new session starts.
- Compaction is the harness's own. Codync can't choose what survives it, only re-apply its instructions
  afterwards.
- A session a harness has deleted (for example, one cleaned up after a month) can't be resumed. The bot
  starts fresh, but its memory remains.
- Memory search is a substring scan, not semantic vectors. Voice transcripts aren't marked yet, so dreaming
  treats them like typed text. Group rooms don't feed dreaming.
- `search_history` covers the bot's own chat, not group chats it took part in.

Reply threads have separate session/context lanes; see [groups and threads](groups-and-threads.md). Group and delegated requests don't feed dreaming.
