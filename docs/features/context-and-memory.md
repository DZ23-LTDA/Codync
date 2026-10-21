# Context and memory

How a bot keeps its instructions, context and long-term memory. Codync owns memory; the ACP harness owns the
conversation and compaction. The design, including what is still to come, is in
[bot memory and compaction](../design/bot-memory.md).

## Transcript, context and memory

| | Where | Owner | Lifetime |
|---|---|---|---|
| Transcript | SQLite `entries` | Codync | One persistent chat per bot; replies quote the message they answer |
| Model context | ACP session (`bots.session_id`) | The harness | Until *New session*, a failed resume, or an agent/command/folder change; each [fallback agent](agent-fallback.md) keeps its own, parked while another answers |
| Long-term memory | SQLite `memory_*` tables | Codync: the user, the bots' memory tools, dreaming | Forever, across sessions |

Codync never replays the transcript into a session. Each turn sends only the new message(s); the harness keeps
its own context and compacts it itself.

## Instruction snapshot (`host/src/chat/context.rs`)

- The bot's instructions are rendered once, in English and kept general: profile (name, description), how to
  message the user, other conversations, workspace, skills, plus the memory section. How to use each tool
  lives with the tool (MCP server instructions, tool descriptions), not here. They're stored as a snapshot in `kv` under `context.<bot>`, keyed by **session + compaction
  epoch**, and don't change until one of those changes. This keeps the prompt byte-identical so the prompt
  cache stays warm.
- How the snapshot reaches the agent (`Instructions` in `agent/bot`):
  - **Claude** (`agentCapabilities._meta.claudeCode` present) receives it as a real system prompt:
    `_meta.systemPrompt.append` on `session/new` and `session/load`. It survives compaction.
  - **An agent advertising ACP `sessionCapabilities.systemPrompt`** receives it as `systemPrompt`
    (`append`) on `session/new`, `session/load` and `session/fork`, and keeps it across compaction.
  - **Every other harness** receives it in the first message of each session (`<bot-profile>…</bot-profile>`),
    and once more, marked as replacing the earlier one, on the first message after a compaction.
- **Compaction epoch:** Codync advertises `clientCapabilities.session.compaction`. Each completed
  `compaction_update` (deduplicated by `compactionId`) increments `context.epoch.<bot>`; for an agent that has
  never sent one, so does its `usage_update.used` falling below half of a previous value of at least 20k. The
  next turn re-renders the snapshot, and Claude's session is loaded again with the new system prompt. The adapter
  rebuilds its query and resumes the same conversation.
- **Profile edits mid-session** (name, description, skills) are sent once as an `<agent_profile_update>` block
  appended to the next message. After that turn reaches the agent, the change is recorded in
  `snapshot.announced`. The next compaction folds it into the snapshot.

## Memory (`host/src/chat/memory/`, crate `host/crates/memory`)

Memory entries record the person, not the work; what a bot learns about a project goes in its own notes. The
[design](../design/bot-memory.md) explains why. The rules live in
the `codync-mem` crate; `chat/memory` runs them in the host.

- **Two layers** in the host database (`memory_entries`): "yours", shared by every bot that uses it, and one per
  bot. Caps count tokens (one per CJK character, one per four others): 400 for "yours", 1,200 per bot. A bot in
  its own workspace uses "yours" by default and one in a project folder doesn't; the user switches it per bot
  (`sharedMemory`).
- **The opening** holds every active entry the bot sees, pinned first, within the caps, with ids so the bot can
  revise or archive one. Archived entries stay searchable (`search_memory`). The opening doesn't change for a
  memory change, so the prompt cache holds: every change (the user's, a bot's, dreaming's) reaches each session of
  the bots that see it on its next message as a `<memory_update>` with the whole memory. The session whose own
  tool call made the change isn't told. A ledger in `kv` keeps, per session, the revision its opening holds and
  the one it has heard; after a compaction the second falls back to the first, so whatever is newer is told again.
- **The bot's tools** (built-in `memory` MCP server): `remember`, `revise_memory`, `archive_memory`,
  `search_memory`, `search_history`. Every bot writes at once, in any turn and to either layer, whatever its
  permission: memory isn't an action on the user's computer. Nothing about memory shows in the chat (no lines,
  cards or working line); the user reviews it in the bot's memory settings and changes it by telling the bot. A
  forgotten memory is never written again by a bot or dreaming, reworded or not: its text stays closed to that
  layer, and the message it came from to both layers the bot uses. Entry ids are never reused, so a stale id can't reach a newer entry. Memory
  lines and cards that older hosts put in the chat are hidden by the apps; `answerMemory` and `undoMemory` still
  answer them for older apps.
- **Checks** on every write but the user's own (`codync_mem::check`): no hidden characters, prompt-carrying
  links, planted instructions, passwords, keys, ID or card numbers, nothing the user forgot, no duplicates, and
  the cap. Entries the user wrote can't be changed by a bot or dreaming.
- **Long-form files** (`chat/memory/files.rs`, rules in `codync_mem::files`): Markdown notes, one topic each, in
  `<data>/shared-memory/` and `<data>/bots/<id>/bot-memory/`. Names are lowercase slugs (`people.md`); each starts
  with frontmatter whose `description:` becomes its line in the opening's index (path, description, tokens). Caps:
  2,000 tokens per file, 20 files per folder, marked in the index when passed, never cut. Links, hidden files and
  folders are ignored.
  - The bot reads and edits them with its own tools: agents that take `additionalDirectories` get both folders on
    every session request. Others use `read_memory` and `edit_memory` (old/new, whole content, or delete), which
    write at once like `remember`.
  - A scan at the start and end of every turn and on the memory screen records each change as a version with its
    full content: mtime and size first, then SHA-256; a file younger than 2 s waits (the turn-end scan looks again
    once it settles). A change from a bot's tool call that changes files (an edit, delete or move, or one with
    diffs, naming the path; a read doesn't count) runs the checks, the cap and the description; one that fails is
    put back (logged, nothing in the chat). Other changes (an editor, another program) are recorded as "this
    computer". The host keeps every version (the apps don't list them); a description that fails the checks is
    withheld from the index.
  - Every host write (the apps, `edit_memory`, dreaming, undo) is compare-and-swap on the content's hash, through a
    temporary file renamed over the note. `search_memory` also returns matching lines of the files.
  - Deleting a bot deletes its folder and versions; "Forget everything" deletes a layer's files and versions; a
    file deleted one at a time stays restorable from its versions.
- **Dreaming** replaces the keeper (`chat/memory/dream.rs`). After five idle minutes or eight user messages
  (sooner on a new session or compaction, and once a day for the time review, which rewrites an entry whose
  date has passed once, never its own rewrite again), it reads the bot's chat since its bookmark on the bot's own harness (Claude: Haiku, no tools). Each change must quote what the
  user typed: pasted text (the apps send `pasted`), fenced code, quoted lines and forwarded messages don't count,
  nor do bare acknowledgements. It only adds and archives, at most 10 changes and never more than a quarter of
  the entries, and drops a run when the user edited memory meanwhile. It waits while the plan's usage is above
  75%. Every run is logged and can be undone; what an undo removes isn't learned again, and an entry the user
  brought back (undoing an archive, or Bring back) isn't archived by dreaming again. It may also append a cited
  line to a long-form file (a new file gets frontmatter with the run's description), and, when a layer is
  nine-tenths full, move an entry the user didn't write out into one. It never rewrites a file; undoing the run
  puts the files back unless they changed since.
- **The harness's own memory** is off for every bot and for background jobs (`agent/own_memory.rs`): Claude Code
  (`autoMemoryEnabled` and `autoDreamEnabled` false, `CLAUDE_CODE_DISABLE_AUTO_MEMORY`), Copilot (its memory tools
  excluded), CodeBuddy, and Qwen Code (workspace settings, so only in a bot's own workspace: a project folder's
  files are the user's). Gemini CLI, Kilo Code and Kiro have no switch Codync can use. Project knowledge goes in
  the bot's own notes, and into the project's `CLAUDE.md` or `AGENTS.md` only when the user asks.
- **Keeping it on.** Where Codync has the switch, the memory screen's "This bot" section offers "<harness>'s own
  memory" (bot field `harnessMemory`, `null` without a switch), off by default, with an info button: the bot may
  remember things twice, the two memories can disagree, and forgetting something in Codync doesn't reach it.
  Changing it restarts the agent, which resumes its session where it can, and moves `memoryRev`. Kept on, the
  harness runs its memory as it sets it, and Claude Code's `MEMORY.md` shows read-only in a "Claude Code" section
  (empty until it saves something). An edit to another harness turns the switch off. Using the harness's memory
  instead of Codync's is planned: [harness memory](../plans/harness-memory.md).
- **The apps** (iPhone, desktop, terminal) show "About you", "This bot", the dream log and Claude Code's notes,
  read-only since 2.21.0: the user changes memory by telling the bot in the chat. Each layer shows its entries with
  who wrote them, then its notes (the terminal: a Notes section) with description, size and date, marked past a
  cap; a note opens to read. The dream log shows added lines green and archived red, beside an info button (the
  terminal: a short paragraph above it) saying what dreaming does. The switches stay: "Use what your bots share"
  and the harness's own memory. They fetch with `memory` and `memoryFile`, and fetch again when the bot's
  `memoryRev` moves. The host still answers the edit methods older apps call: `saveMemory`, `pinMemory`,
  `archiveMemory`, `forgetMemory`, `clearMemory`, `undoDream`, `saveMemoryFile`, `deleteMemoryFile`,
  `restoreMemoryFile`.
- **Moving the old memory**: on the first start of this version, each bot's Engram database (or the Markdown
  memory before it) moves in once. What the old opening showed comes in active as far as the cap allows (the
  profile and pinned facts first, then the newest), the rest archived; then the old files (deleted bots' too) and
  the Engram download go. Dreaming starts from that point in the chat.

## Compaction and rollover (`host/src/agent/bot/compaction.rs`, rules in `codync_mem::compaction`)

- **What the host records**, per session in `kv` (`compaction.<bot>.<session>`): context use and window size
  from `usage_update`, compactions, and when turns end. How long the provider keeps prompts cached is learned
  from turns that came after a pause (`cache.life.<bot>`); until then it is an hour for Claude on a subscription
  and five minutes otherwise. Each turn's cached read and write tokens are logged (`turn usage`).
- **Compacting ahead of time**, in the bot's own chat only (never rooms, routines or bot requests):
  - at the end of a turn past three quarters of the window or 200k tokens, whichever is less;
  - while idle, with a one-hour cache and over 100k in use, five minutes before the cache expires.

  It runs as a hidden turn: dreaming first, then the opening rendered again (a Claude session reloads only when
  it changed), then `/compact <keep-list>` for Claude, or `/compact` for Codex, whose keep-list is its
  `CODEX_CONFIG` `compact_prompt`. Other harnesses compact on their own. The keep-list is also in the opening.
- **Loop guard:** after three host compactions in a day, or one that left 60% of the trigger in use, or one
  fewer than five turns ago, the host rolls over instead of compacting again.
- **Rollover:** also due when a conversation over 100k went cold (its cache expired) or after eight hours away
  with three compactions behind it. A handoff card appears in the chat (`chat/handoff.rs`): the last summary,
  the user's last four messages and the last reply (each cut at 2,000 characters, so a pasted document doesn't fill
  the new conversation), the open plan, and a bookmark for `search_history`. The user can
  edit it (`editHandoff`) or keep the conversation (`keepConversation`); otherwise their next message there starts
  a new session that begins with it.
- **After every compaction** the host records a notice ("Earlier context was summarized to make room.", its
  `compaction` field carrying Claude's summary, which the next handoff starts from). The apps don't show it: the
  same conversation goes on.
- **In the apps** the chat explains only a fresh conversation, in one line, left-aligned like the bot's messages.
  The iPhone and desktop show the draft card with *Keep this conversation* and a pencil that opens the editor
  sheet; once sent it folds to "Started a fresh conversation with this summary.", which unfolds to what it
  carried, and once kept it leaves no line. In the terminal, pick the line (`v`, `j`/`k`), then `o` unfolds it,
  `e` edits and `K` keeps the chat. The user starts a new session from the bot's menu (terminal: `S`), not with a
  `/` command.

## Turns

- Messages sent while the agent works are folded into one next turn, joined by blank lines.
- Bot-to-bot requests occupy separate turns in the same queue; user messages are only folded together up to the next request. Requests never feed dreaming. See [bot collaboration](bot-collaboration.md).
- Delegated turns do not set the restart marker: their waiter disappears on restart. Pending delegation notices become interrupted errors instead of automatically replaying work.
- Stop sends `session/cancel`; an agent still running 15 seconds later is stopped (it resumes its session on
  the next turn). `session/new`, `session/load` and `session/fork` get three minutes before the agent is
  stopped. Connector or computer changes restart an agent that can resume sessions before its next turn; one
  that can't keeps its session, and its next new session gets them.
- `turn.inflight.<bot>` records when the running turn in the bot's chat started. If the host stops mid-turn (update, crash, restart), the next
  start resumes that session with a hidden "you were interrupted, don't repeat finished steps" prompt. If the
  session can't be resumed, or the turn started over an hour ago, it does nothing.

## Limits

- An agent without ACP `systemPrompt` gets the instructions as a user message, which models follow less firmly
  than system text.
- The harness writes every summary. Codync chooses when the bot's own chat compacts and what it asks the summary
  to keep, and re-applies the instructions afterwards; it can't make a harness keep anything.
- A session a harness has deleted (for example, one cleaned up after a month) can't be resumed. The bot
  starts fresh, but its memory remains.
- Memory search is a substring scan, not semantic vectors. Voice transcripts aren't marked yet, so dreaming
  treats them like typed text. Group rooms don't feed dreaming.
- `search_history` covers the bot's own chat, not group chats it took part in. It finds messages by words, or
  reads back from a message id (`from`, then each page's `next`); `search_memory` gives each entry's source
  messages, so a bot can check where a memory came from.

A reply stays in its chat's session, and its quote stays out of what memory takes as the user's words; see [groups and replies](groups-and-replies.md). Group and delegated requests don't feed dreaming.
