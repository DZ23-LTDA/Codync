# Bot memory and compaction

Status (2026-10-18): the cards, the move from Engram, writes by permission, the checks, dreaming and all three
apps are built in 2.18.0 (M0); how it works today is in [context and memory](../features/context-and-memory.md).
The plan from there is in [Rollout](#rollout): instructions for every harness (M1), the cache-safe opening and
compaction (M2), long-form memory files (M3), evaluation (M4) and the public mirror (M5). The research behind it
is in [bot memory research](bot-memory-research.md). The numbers are starting values to be tuned against the
[evaluation](#evaluation).

Built differently from the plan below, for now:

- Search is a substring scan, not an FTS5 trigram index. Memory has a few hundred entries; the transcript scan
  took 0.6 s over 180k messages of one bot, so the index waits until a bot's search passes a second.
- No revision history per entry; the dream log and the chat lines carry undo.
- Proposals for a bot that asks first live in their chat card, not in the store, so there is no proposed state.
- Health, religion and politics get no special notice yet; voice transcripts aren't marked, and group rooms
  don't feed dreaming.
- Entries show who wrote them, not a link to the message they were learned from. The bot can follow it:
  `search_memory` returns each entry's source messages, and `search_history` reads back from a message id.
- Until M2, a change the user makes still re-renders the opening (and reloads a Claude session), and the
  notice carries the whole memory rather than a delta.

In one line: **memory records the person, the transcript stays in the host, the opening stays frozen until
compaction, and the host decides when to compact.**

## Why redesign

Reading the current implementation turned up these problems.

- **Two writers for one memory.**
  - The agent writes through Engram's MCP tools, and the keeper extracts facts from the same exchanges.
  - Each keeper batch runs the agent twice.
  - User messages are stored three times: transcript, keeper queue, and Engram's captured prompts.
  - Each turn makes three Engram round trips.
- **The mid-session reload doesn't work.**
  - Claude Code records the system prompt on a session's first request and reuses that record until the session
    is compacted. This is on by default since Claude Code 2.1.265 (SDK 0.3.265) and no longer flag-gated since
    2.1.268. Local transcripts already contain `systemPromptSnapshot` records.
  - When `_meta.systemPrompt` changes, `claude-agent-acp` tears down the query and resumes it, which replays history
    and re-runs SessionStart hooks. The new text still takes effect only at the next compaction.
  - So "memory changed, `session/load` with a new append" restarts the process for nothing.
- **The opening is one compaction late.** The host reacts to `compaction_update` completed by re-rendering and
  loading the new opening. The compaction that just ran already re-recorded the old prompt, so the new one is
  picked up only at the following compaction.
- **Non-Claude harnesses lose the profile.**
  - `<bot-profile>` is sent only in a session's first message (`host/src/agent/bot/turn.rs`).
  - Codex keeps only the last 20k tokens of user messages verbatim when it compacts locally
    (`COMPACT_USER_MESSAGE_MAX_TOKENS`). Remote compaction keeps developer messages only behind a feature flag.
  - `codex-acp` passes developer instructions only on `session/new`, not on load or resume.
  - Codync re-sends the profile in none of these cases.
- **Fork drops the opening.** `session/fork` sends no `_meta`, and the adapter forwards none, so a new thread needs
  an immediate `session/load` (`host/src/agent/bot/session.rs`).
- **Adapter signals are ignored.** Both `claude-agent-acp` and `codex-acp` report:
  - `usage_update{used,size}`;
  - per-prompt `cachedReadTokens` / `cachedWriteTokens`;
  - compaction progress.
  
  Claude also sends `compact_boundary` token counts and the summary. The host reads only
  `_meta["_claude/rateLimit"]` (`host/src/agent/bot/updates.rs`).
- **A second memory runs unseen.** Claude Code's auto memory and auto-dream run inside bot sessions. Auto memory is
  "loaded into the system prompt at session start", and none of it is visible in Codync.
- **Engram has the wrong shape.** The details are under [Engram](#engram).
  - Engram is built for one coding agent remembering its own work: one database per bot, written only by Engram,
    with Codync reading its schema read-only.
  - That rules out the shared "yours" layer, the write gate, provenance with cascade delete, a dream log with undo,
    and memory writes stamped with `rev` in the same transaction.

## Patterns explored

Two research passes on 2026-10-17 compared the following; a third pass on how open-source agents mix files and a
database, embeddings, and harness instructions is in [bot memory research](bot-memory-research.md).

- **Products:** ChatGPT, Claude, Grok (Grok Bot, Grok Build), Gemini, Codex, Claude Code, Cursor, Amp, Factory,
  OpenClaw, Hermes, Letta, Mem0, Zep/Graphiti.
- **Papers** from 2024–2026.
- **Community threads:** Hacker News, Reddit and GitHub issues.
- **Local sources:**
  - `claude-agent-acp` 0.92.0;
  - Claude Agent SDK 0.3.300, which bundles Claude Code 2.1.300;
  - `codex-acp` on GitHub.

### Where memory lives and who writes it

| Pattern | Seen in | What we found | For Codync |
|---|---|---|---|
| Agent-managed memory through tools | Letta/MemGPT core memory, Engram MCP, Hermes | Agents forget to call the tool, or say "remembered" without calling it. Tool writes bypass any review. Letta shared blocks are last-write-wins. | Not the main writer. The bot's writes follow its permission mode, and every write is visible with undo. |
| Background extractor | Mem0, ChatGPT saved memories, our keeper | One background writer is the consensus. Mem0 went add-only in April 2026 because reconciliation "was where context got destroyed". | Kept as **dreaming**, limited to add and archive. |
| Small profile always in context | ChatGPT saved memories, Claude memory summary, Hermes `MEMORY.md`/`USER.md` | Reliable, because nothing depends on the model deciding to look. Needs a size cap. Hermes' 2,200-character cap caused silent fact loss. | **Adopted** for active memory, capped in tokens. |
| Retrieval on demand | Vector RAG, our former "context" section | A community evaluation found about 60% of failures were the agent never searching. | Only for archived memory and the transcript. |
| Raw transcript search | Letta's filesystem benchmark (grep agent ≈74%) | Beats most memory products. Replacing the transcript with summaries or extracted facts lowers accuracy (LongMemEval). | **Adopted**: `search_history` over the host transcript. |
| Knowledge graph / temporal graph | Zep/Graphiti, A-MEM | Graphiti invalidates old facts instead of deleting them. Benchmark gains are small next to benchmark noise: 6.4% of LoCoMo's answer key is wrong. | Not built. We borrow "supersede, don't delete". |
| File-based memory | CLAUDE.md, Claude Code auto memory, Codex memories, OpenClaw `MEMORY.md` | Transparent and editable, but no transactions and no sync to three clients. | SQLite, made fully visible in the UI. Harness file memory stays for project bots only. |
| Sleep-time consolidation ("dreaming") | ChatGPT dreaming, Claude Code auto-dream, OpenClaw, Letta sleep-time, Grok Build `/dream` | Useful, and every product adds guards. OpenClaw rejects a rewrite that removes more than 0.25 of entries and keeps a pre-image. Hermes removed its `replace` action because it wiped other facts, and #5563 traced lost memory to corruption and the cap. Claude Code auto-dream keeps "no log of what was created, modified, or removed" (#38493). | Adopted, with the guards below. |
| External memory service | Engram, Mem0 cloud, Supermemory, Honcho | A separate process owns the writes and has its own failure modes. | Dropped. |

### Scoping across bots

| Product | Scope |
|---|---|
| Grok Bot | Per bot: "Memory and Routines belong to the Bot … context remains with the role" |
| Claude Projects | Per project, with a per-project "Use account memory" toggle |
| ChatGPT Projects | "Project-only memory", but a 653-point Reddit thread reproduced a leak into a project-only project |
| Letta | Shared blocks, last write wins; read-only blocks advised |

Cross-scope bleed is the most common complaint everywhere. A 738-point r/ChatGPT thread: "memory is great until
it quietly carries an assumption from three chats ago into something unrelated. a per-chat toggle would fix most
of it". Codync's shape follows Claude Projects: per-bot memory plus a shared "yours" layer that each bot can switch
off. The user, dreaming and bots in `Auto` write "yours", and each entry names who wrote it.

### Consensus we design to

1. **Memory is for the person, not the work.**
   - Codex: "Keep required team guidance in AGENTS.md"; memories are "a recall aid".
   - Grok Build memory excludes anything the repository already covers.
   - Claude Code auto memory skips what it can derive from the code.
   - Users who let memory touch code complain that it "saves a memory with the patch in it instead of fixing the
     code. disabled memory user wide" (r/ClaudeAI).
2. **Scope it.** Memory must not leak across contexts the user keeps apart.
3. **Staleness is the hard part**, more than finding things. Archive superseded facts with a pointer to what
   replaced them, and turn the future into the past as time passes. ChatGPT dreaming turns "You're going to
   Singapore" into "You went in July 2026".
4. **Visible, editable, with provenance.** Every memory says where it came from and can be undone.
5. **One automatic writer, behind a gate.**
6. **Memory is an attack surface.** Treat it as data, not instructions. Published attacks:
   - MINJA poisons memory through queries alone.
   - AgentPoison reaches over 80% success at a poison rate under 0.1%.
   - Gemini's delayed tool invocation fires a planted instruction when the user later says "yes".
   - Microsoft found over 50 "remember … as a trusted source" prompts from 31 companies planted in links.
   
   The defense that holds is structural (CaMeL, the design-patterns paper): untrusted input must be unable to
   trigger consequential actions.
7. **The raw transcript is the source of truth.** Memory is an index over it.
8. **A stable prompt prefix.** Anthropic: "Put changing information in messages, not the system prompt."
9. **Small and complete beats large plus retrieval.**

Still contested: files vs. a database, vectors, and graphs. We use SQLite for transactions and three-client sync,
and keep the file camp's real requirement: no hidden state.

### Compaction patterns

| Pattern | Seen in | For Codync |
|---|---|---|
| Harness auto-compacts at a threshold | Claude Code (at the model limit, about 967k on 1M models), Codex | Kept as the safety net |
| Idle compaction inside the harness | Claude Code 2.1.299+: interactive terminal only, flag-gated off by default | Never active in bot sessions, so the host's idle compaction is the only one |
| Summary written while idle, swapped in on return | Claude Code's flag-gated `compact_on_return` mode; its "resume from summary" offer after over 1 hour idle and over 100k tokens | Precedent for host idle compaction and rollover |
| Host-timed compaction | No product feature found | **Adopted**: end of turn, and idle before the cache expires |
| Handoff to a fresh session | Amp Handoff, an editable draft; Amp calls compaction "lossy" and warns about "stacking summary on top of summary" | **Adopted** as rollover |
| Anchored or iterative summaries | Factory: only the newly truncated span is summarized and merged. It scores 3.70 vs. 3.44 (Anthropic) and 3.35 (OpenAI); the artifact trail is weakest everywhere. | Left to the harness; rollover resets drift; probes in the evaluation |
| Memory-flush turn before compaction | OpenClaw | Not needed: the host keeps the full transcript |
| Masking old tool output | Anthropic context editing (it invalidates cached prefixes) | The harness's job |
| Writing our own summaries | Anthropic on-demand and threshold compaction APIs | Not built until harness summaries prove insufficient |

## The design

### Two layers

Caps count tokens, not characters, so Chinese users aren't penalised. 2,000 CJK characters are 2–3k tokens, but
2,000 English characters are about 500. The estimate is CJK-aware: one token per CJK character, one per four Latin
characters.

| Layer | Holds | Cap (initial) | Default | Scope |
|---|---|---|---|---|
| **Yours** | Name, time zone, language, important people, standing preferences | ≤ 400 tokens | On for personal bots, off for project bots | Shared by all bots |
| **This bot** | How to work with you, promises, ongoing commitments | ≤ 1,200 tokens | Always on | One bot; wins over "yours" when they conflict |

Active total: ≤ 1,600 tokens per opening. For comparison:

- Hermes: about 800 + 500 tokens;
- the Claude Code `MEMORY.md` index: 200 lines or 25 KB;
- Letta blocks: 5,000 characters by default.

Placement:

- **Active** memory is placed in full in the session opening.
- **Archived** memory is out of the prompt and searchable with FTS5 trigram, which also handles Chinese.
- Past a cap, dreaming moves detail into a long-form file or archives entries.
- Project knowledge stays in the harness's files (`AGENTS.md`, `CLAUDE.md`).

### Long-form memory

Cards hold what a reply goes wrong without. Detail that matters only when its topic comes up lives in Markdown
files the bot reads and edits with its own tools: every system studied keeps a small always-loaded layer and a
larger one read on demand ([research](bot-memory-research.md#lessons-across-systems)).

| | Cards | Long-form files |
|---|---|---|
| Holds | Name, time zone, allergies, "don't …" | One topic's detail: people, health, plans, trips |
| Home | `memory_entries` | `shared-memory/*.md` and `bots/<id>/bot-memory/*.md`; versions in `memory_file_versions` |
| Cap | 400 + 1,200 tokens; 150 per entry | 20 files per folder; 2,000 tokens per file |
| In the opening | In full | An index: path, description, size |
| The bot reads | — | With its own file tools, or `read_memory` |
| The bot writes | Memory tools | With its own file tools, or `edit_memory` |
| Checked | Before the write | Before the write through `edit_memory`; at turn end for its own edits |

**Paths and format.**

- `<data>/shared-memory/` belongs to the shared layer; `<data>/bots/<id>/bot-memory/` to one bot. Not `memory/`:
  the move from the old memory deletes `bots/<id>/memory` on every start.
- File names are lowercase ASCII slugs (`[a-z0-9-]+.md`), one level deep. The title inside can be any language.
  Unicode and case variants of a name would otherwise become duplicate notes (Basic Memory #1275, #1281).
- Every file starts with frontmatter whose `description:` says when to read it. It becomes the file's index line.
- Only regular `.md` files count. Symlinks, hidden files and subfolders are ignored, so a link can't make the host
  read, journal or restore another file.
- An index or a file over its cap is marked in the opening and the apps, never cut.

**The scan.** At the end of every turn, before an opening is rendered and when an app asks, the host scans the
folders the bot can see:

1. Unchanged mtime and size skip a file. One modified in the last 2 s waits for the next scan.
2. A SHA-256 different from the recorded version is a change. A folder that went missing is not "every file
   deleted".
3. The change runs the same checks as every other write (hidden characters, prompt links, planted instructions,
   secrets), plus the caps and the description.
4. It becomes a new version holding the full content. Its author is the bot whose tool call touched that path
   since the last version, otherwise "this computer".
5. A change that fails a check is restored to the previous version and the chat says why. A change that drops more
   than a quarter of the file is kept, flagged in the chat, and can be undone.

**Host writes** (the apps, `edit_memory`, dreaming) are compare-and-swap: re-read the hash, write a temporary file
in the same folder, rename it over the note. A note that changed meanwhile isn't overwritten; the write fails, a
scan records the other change, and the write is retried on top of it. The host never reformats a file
(Basic Memory's own rewrites duplicated frontmatter, #1171).

**Permission.** The bot's own file edits go through the harness's permission requests, which an `Ask` bot shows as
cards and an `Auto` bot approves. `edit_memory` follows `Permission` like `remember`.

**Dreaming** may append to a file, or move detail from a full card layer into one, under the same evidence rules.
It never rewrites a file.

**Scope and lifecycle.** A bot that doesn't use the shared layer sees neither `shared-memory/` in its index nor the
folder itself. Deleting a bot deletes its folder and versions, and "Forget everything" deletes a layer's files
with their versions. Deleting one file through the apps or the bot keeps it as a version that can be restored.

### Writers and the gate

- **The user, in the UI**: writes directly.
- **The bot**, through its memory tool, follows the bot's permission mode (`Permission`):
  - `Auto`, the default for new bots: the write goes straight in, to this bot's layer or to "yours", in any turn.
    The lane where it happened shows a "Remembered" line with undo, and the memory screen lists the entry with
    the bot that wrote it.
  - `Ask`: the write becomes a confirm card in all three clients, and accepting it is a UI write. An explicit
    "remember …" the user typed (not pasted) is written directly in both modes.
- **Dreaming**: proposes. It is the only background writer, and it also records an explicit "remember …" the user
  typed when the bot didn't call its tool. It may merge or archive bot-written entries, never the user's own.

These checks are deterministic code in the core, not another model's judgment. Dreaming proposals pass all of
them. Bot writes, in either mode, pass the last two (instruction-like content and sensitive data) and the cap.

- **Evidence must be user-authored.** Each item quotes the user verbatim and cites at least one message id.
  - The quote must appear in a user-authored span.
  - Code fences, blockquotes, attachments, forwarded text and spans the composer tagged as pasted don't count.
    The iOS, desktop and TUI composers mark pasted spans.
- **Minimum content.** A quote of three content tokens or fewer, or one that is only an acknowledgement (好, yes, ok,
  sure), can't carry a memory. This stops the delayed-trigger attack and "shall I remember X?" loops.
- **Instruction-like content is rejected:**
  - "remember / 記住" imperatives addressed to the assistant that didn't come from the user's typing;
  - invisible Unicode;
  - URLs with prompt-carrying parameters (`?q=`, `prompt=`).
- **Sensitive data.**
  - Passwords, government ID numbers and account numbers are never stored, even when asked.
  - Health, religion and politics are stored only on explicit request, with a visible notice.

Source policy:

| Source | Can create memory |
|---|---|
| UI edit | Directly |
| User text in the main chat or a thread | Yes, through the gate |
| User text in a group room | Only for the bot(s) addressed, tagged with the room id |
| Voice transcript (`source: voice`) | Only after UI confirmation or a second occurrence. Audio-model attacks such as AudioHijack reach 79–96%. |
| Bot-to-bot requests | Never. They also never read another bot's private layer. |
| Bot output, routines, webhooks, files, web pages | Never |

The table governs dreaming. In `Auto`, the bot decides what it writes, in any turn and from anything it read,
including turns a routine, a webhook or another bot started. That is the risk `Auto` accepts, bounded by the
visible line with undo, the hard checks and dreaming's later review. A bot that shouldn't see what others write
can switch "yours" off.

### Dreaming

**Schedule.**

- It runs after five idle minutes or eight queued user messages.
- It also runs before any host-initiated compaction or rollover.
- It is skipped while subscription usage is above 75% of the window. Codex keeps the same 25% headroom with
  `min_rate_limit_remaining_percent`.

**How it runs.**

- It uses a fresh, tool-free context on the bot's cheapest model. The keeper already does this: `tools: []`, Haiku.
- It reads the transcript from a bookmark forward.
- Untrusted and system-provenance spans are stripped before the proposal prompt (OpenClaw does the same).
- It reads the current entries, bot-written ones included, and doesn't add what is already there.

**Pipeline.**

1. Proposal.
2. Audit.
3. The host re-checks the result deterministically.
4. One transaction writes memories, revisions, the dream log and `rev`.

The transaction aborts if the bot's memory `rev` changed since the dream's snapshot, so a UI edit made during the
run is never overwritten (OpenClaw uses optimistic concurrency the same way).

**Rules:**

- **Your words are fixed.** Entries the user wrote, or explicitly asked for, can never be changed by dreaming.
- **Add and archive only.** Dreaming never rewrites or deletes. Archiving names what superseded the entry.
  Deletion happens only when the user presses Forget.
- **Limits.** At most 10 operations per run. A batch that would archive more than ¼ of active entries is rejected
  whole (OpenClaw's default is also 0.25).
- **Tombstones.** A forgotten entry leaves only a content hash, so dreaming can't grow it back. Its revisions are
  deleted too.
- **Cascade.** Deleting a message deletes memories learned only from it. ChatGPT ("Deleting a chat alone does not
  necessarily delete a separate saved memory") and Claude ("related memory entries generated from it won't be
  removed") don't do this.
- **Time review, daily.** Future tense turns into the past once the date passes. Relative dates become absolute at
  write time.
- **Dream log.** Every run's changes are visible and can be undone in one tap.
- **Migrated entries.** Entries imported from Engram or Markdown change only on contrary evidence.

### Frozen opening

The opening, in order:

1. harness tools and preset;
2. identity and rules;
3. yours;
4. this bot.

It stays byte-identical until compaction or a new session.

- **Claude transport.** Send the object form, which the adapter spreads into the SDK preset:
  `_meta: {"systemPrompt": {"append": …, "excludeDynamicSections": true}}`.
  - Not the ACP spec `systemPrompt` field yet: for Claude the adapter rebuilds it as preset plus append and drops
    the other options.
  - `excludeDynamicSections` moves the working directory, git status and similar per-session context into the
    first user message. The opening is then identical across a bot's main chat, threads and routines, and a new
    session reads the cache.
- **Change the opening only at compaction or rollover.** Every change restarts the adapter's query and replays
  history. Resume has had regressions, such as 2.1.288 dropping files a compaction had just restored.
  - The SDK has undocumented mid-session invalidation hooks (`liveAppendSections`,
    `CLAUDE_CODE_HOST_PROMPT_SUPERSEDES_RECORD`).
  - They are unstable and any re-render invalidates the system-prompt cache layer, so the design doesn't depend on
    them.
- **Deltas.** Changes made elsewhere reach the bot as one `<system-reminder>`-style block prefixed to its next
  message. That covers UI edits and what other threads or bots learned.
  - Nothing is sent for what this session learned itself, because the bot already heard it.
- **Ledger.** It records two numbers per session: the memory revision the opening contains, and the revision deltas
  have delivered up to.
- **The handoff summary goes in the first user message**, not the opening, because it differs per session.
- **Fork.** Re-send the parent's `_meta.systemPrompt` on `session/fork`.
- **Settings isolation.** Consider `settingSources: []` or `["project"]`. The adapter default
  `["user","project","local"]` loads the user's own `~/.claude/CLAUDE.md` and settings into every bot.

### Instructions in every harness

Codync runs 27 harnesses. The opening, with the bot's identity and memory, reaches each through the best channel
it offers (`Instructions` in `agent/bot`):

1. **Claude: `_meta.systemPrompt`**, as above. claude-agent-acp 0.92 also advertises ACP's `systemPrompt`, but it
   rebuilds that field without the object options (`excludeDynamicSections`), so Claude stays on `_meta`.
2. **ACP `systemPrompt`** ([Client-Provided System Prompt](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/rfds/client-system-prompt.mdx),
   unstable): when an agent advertises `sessionCapabilities.systemPrompt`, the opening goes as
   `{"mode": "append", "content": [text]}` on `session/new`, `session/load` (the session's current snapshot) and
   `session/fork` (the main session's). The agent keeps it at system level, across compaction.
3. **Every other agent: the first message.** A `<bot-profile>` block precedes the first prompt of a new session.
   After a compaction the next prompt carries a fresh one that says it replaces any earlier profile, so the agent
   never holds two versions that disagree. The compaction is the agent's `compaction_update`, or, for an agent
   that never sent one, its `usage_update.used` falling below half of a previous value of at least 20k.

Measured 2026-10-18 at `initialize`:

| Agent | `systemPrompt` | `additionalDirectories` |
|---|---|---|
| claude-agent-acp 0.92.0 | Yes | Yes |
| codex-acp 2.2.2 | No | Yes |
| OpenCode 1.19 | No | No |
| Copilot CLI | No | No |

codex-acp reports compactions (`compaction_update`). Codex keeps the user's newest ~20k tokens of messages
through a compaction, so a first-message profile survives
short sessions and is the first thing dropped in long ones. In a real run past that budget (two 15k-token
messages, then `/compact`), Codex still knew the bot's name and the user's allergy afterwards, from its summary;
the re-sent profile makes that independent of what the summary keeps. codex-acp also reads
`_meta.jetbrains.air.customInstructions` on `session/new` as Codex developer instructions. It isn't used: it is one
vendor's extension, and it isn't re-sent on `session/load`.

Verified on a real host (2026-10-18): a Codex bot got its profile again on the first message after `/compact`
and still knew its name and the user's allergy; a Copilot bot reported no `store_memory` tool; a Claude bot
answered from its memory as before.

**File access.** The memory folders a bot can see go as `additionalDirectories` on every session request, when
the agent advertises the capability (M3). Agents that can't reach them read and write through `read_memory` and
`edit_memory`.

**The harness's own memory.** For a bot in its own workspace, and for every background job, Codync turns each
harness's own memory off where it can, so the bot has one memory and it is visible (`agent/own_memory.rs`,
surveyed 2026-10-18):

| Harness | Its own memory | What Codync does |
|---|---|---|
| Claude Code | Auto memory, auto-dream | `autoMemoryEnabled: false`, `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`, `autoDreamEnabled: false` |
| Copilot CLI | Copilot Memory, on for individual plans | `--excluded-tools=store_memory,vote_memory` |
| Qwen Code | Auto memory and its own dreaming, on | `.qwen/settings.json` in the workspace: `memory.enableManagedAutoMemory` and `enableManagedAutoDream` false |
| CodeBuddy | Auto memory | `CODEBUDDY_DISABLE_AUTO_MEMORY=1` |
| Codex, Goose, Grok, oh-my-pi, Qoder, Cortex | Off unless the user turns it on | Nothing |
| Gemini CLI | Its system prompt has the model edit `~/.gemini/GEMINI.md` | Nothing: no switch short of replacing its system prompt or its home folder |
| Kilo Code | Memory tools, on per project | Nothing: the only per-process switch replaces the whole config |
| Kiro | Memory with a global setting | Nothing: no per-workspace key |
| OpenCode, Cursor, Kimi, Amp, Cline, Droid, Devin, others | None found | Nothing |

### Compaction

The harness writes the summary, because it knows the work: files read, todos, tool results. The host decides
**when**, because it knows how full the context is, whether the user is present and whether the cache is warm.

Cache economics (Anthropic):

- a cache read costs 0.1× input, or less on newer models (0.05× on Opus and Sonnet 5.5);
- a 5-minute cache write costs 1.25×;
- a 1-hour cache write costs 2×.

Compacting an idle session before its cache expires beats a cold re-read once the context passes roughly 70–90k
tokens at 0.1× reads. Recompute this per model.

The 1-hour TTL applies to SDK turns only "on a Claude subscription within your plan's included usage". On
extra-usage credits or an API key it drops to 5 minutes. The host reads the TTL actually in use (the SDK's
`usage.cache_creation` separates 5-minute and 1-hour writes), or pins `promptCacheTtl: "1h"` for main lanes, which
doubles the write cost on API billing.

| Trigger | When | Why |
|---|---|---|
| **End of turn** | `used ≥ min(0.75 × size, 200k)` when a turn ends, leaving room for the summary (about 20k tokens) | The cache is hot, so reading the history costs ≈1/10. Nothing stalls mid-task. |
| **Idle** | 1-hour TTL: at TTL − 5 min when `used ≥ 100k`, main lane only. 5-minute TTL: no idle compaction. | Cheaper than a cold read on return. The threshold sits above break-even because compaction loses detail. |
| **Rollover** | Any of: (1) returning with a cold cache and `used ≥ 100k`; (2) away ≥ 8 hours with ≥ 3 compactions; (3) the loop guard fires | Summaries of summaries drift. A new session starts with a handoff. |
| **Safety net** | A single turn fills the window | The harness's own auto-compaction |

The `idleCompaction` setting needs no change. The harness never idle-compacts in SDK/ACP sessions, and setting it
to true "does not turn idle compaction on".

**Loop guard.** It allows at most 3 host-initiated compactions per lane per 24 hours. If post-compaction usage is
≥ 60% of the trigger, or two compactions happen within 5 turns, the host rolls over instead.

- Claude Code added a 3-attempt circuit breaker in 2.1.76 and thrash detection in 2.1.89, and still fixed
  "conversations being compacted over and over" in 2.1.301.
- One issue reported 211 compactions in a single session.

Threads, group rooms and routines are never compacted early. Most are not revisited, so the cost isn't spent ahead
of time. When the main chat is nearly full, a new thread starts from a handoff instead of a fork.

**Before a host-initiated compaction:**

1. dream;
2. render a new opening with the latest memory;
3. load it into the session;
4. compact with a keep-list.

This order puts the new opening into the very compaction that runs. Anthropic's docs say "the first compaction
switches to the current prompt".

How the keep-list is passed:

- **Claude:** `/compact <instructions>`, which the SDK turns into `custom_instructions`.
- **Codex:** `/compact` ignores trailing text, so the keep-list goes in as `compact_prompt` through `CODEX_CONFIG`.

The keep-list:

- **Verbatim:** the last user request, and every "don't / 不要".
- **Exact identifiers:** paths, URLs and ids (OpenClaw keeps "exact opaque identifiers").
- **Kept:** decisions with reasons, pending tasks with the next step, who said what in rooms, and the thread intro.
- **Left out:** identity and memory, because the host re-injects them.

The keep-list also lives in the opening, so harness-initiated compactions see it.

**After any compaction.** The signal is `compaction_update` completed, Claude's `compact_boundary`
`pre_tokens`/`post_tokens`, or a sudden drop in `usage_update.used` for harnesses that don't notify. Then:

1. Reset the ledger's delivered number to the opening's, and send one merged delta.
2. Re-send `<bot-profile>` to non-Claude harnesses. Do the same after every resume.
3. Re-send the thread intro inside a thread.
4. Store the summary as a transcript entry and show an "Earlier conversation summarized" card.
   - Codex's `compaction_update` carries no summary. For Codex the host builds the card from its own transcript:
     the boundary plus the latest turns.

**Rollover handoff.** It contains:

- the last compaction summary;
- the last 3–5 user turns verbatim;
- the open plan;
- a bookmark: the id of the old session's last message. The new session reads back from it with
  `search_history`'s `from` instead of guessing words, and pages on with the `next` each result page returns.

The user can edit it as a draft before it is sent (Amp's handoff works this way).

### Claude Code's own memory

| Bot | Auto memory | Auto-dream | Why |
|---|---|---|---|
| Personal (workspace Codync created) | Off: `autoMemoryEnabled: false` + `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1` | Off: `autoDreamEnabled: false` | Only one memory, fully visible in Codync |
| Project (points at a real repo) | On | Off | It is the project's working memory. |

For project bots:

- The auto-memory directory is shared with the user's own Claude Code sessions in the terminal.
- Its `user`/`feedback` notes overlap Codync's person memory.
- Codync shows the project's `MEMORY.md` read-only, so it isn't unseen.
- The opening states the split: people in Codync memory, the project in harness memory and `CLAUDE.md`.

The adapter passes `claudeCode.options.settings` and `.env` through untouched, so these settings reach the SDK.

### Storage

Host SQLite (`codync.db`), one transaction per change, every change stamped with `rev`. The `codync-mem` tables:

- `memory_entries`: the cards, with layer, active/archived, author and writing bot, room, pin, dates and evidence
  message ids;
- `memory_tombstones`: hashes of forgotten text;
- `memory_dreams` and `memory_dream_changes`: the dream log;
- `memory_files` and `memory_file_versions` (M3): each long-form file's path, layer, hash, size, mtime,
  description and tokens, and every version's full content, author and cause;
- `memory_meta`: the schema version.

The transcript is the host's own `entries` table: it is the chat itself, which the apps sync by `rev` and which
changes in place while a reply streams. Search is a substring scan. When one bot's search passes a second, its
text moves to a plain column and an FTS5 trigram index with a CJK bigram column takes over: measured over 180k
messages, the scan took 0.50 s, ripgrep over the same text 0.013 s, and the trigram index 0.002 s after a 26 s
build.

### UI (all three clients)

- **Two sections**, "About you" and the bot's own. Each entry shows where it was learned, with a link to the
  message.
- **A per-bot switch** for "yours".
- **Bot writes:** a "Remembered" line with undo in the chat (`Auto`), or a confirm card (`Ask`). Voice-only memories
  also get a confirm card.
- **The dream log** with undo.
- **Entry actions:** edit, pin, forget, and forget everything.
- **For project bots**, the project `MEMORY.md`, read-only.
- **Long-form memory** under each layer's cards: the files with name, description, size and date, an over-cap
  mark, and a detail view to read, edit, and restore any version. The desktop app can open a file in the user's
  editor.
- **In the chat**, an "Updated people.md" line with the diff and undo, and why a blocked change was restored.
- **In the chat**, a summary card after compaction, and an editable handoff draft at rollover.

### Deliberately not built

Each of these has a trigger for adding it later.

| Not built | Add it when |
|---|---|
| Vectors | Keyword search and the bot's own rephrasing miss real cases; then a local model only, and a model change rebuilds the index |
| Knowledge graph | — |
| Project or team tiers | — |
| Multi-computer sync | — |
| Our own compaction summaries | Harness summaries prove insufficient |
| Per-reply memory citations | — |
| Engram | — |

## Engram

Engram fits its own goal: one coding agent's memory of its own work. The mismatch is shape, not quality.

| Engram | Codync needs |
|---|---|
| One database per bot; a `personal` scope stays inside it | A "yours" layer shared by all bots |
| Engram owns every write; Codync reads the pinned schema read-only | The gate, provenance with cascade delete, revisions, tombstones and dream undo, all inside Codync |
| The agent writes through Engram's tools, unseen and unlabeled | Bot writes follow its permission mode, name the bot, and show with undo |
| A separate process per bot; downloaded on first use; a lost acknowledgement means a write can't be retried safely | Memory that is as reliable as the host |
| Separate from `rev` | Memory and `rev` written in one transaction |

Everything Engram provides that we use, FTS5 search and a few tables, is a few SQLite tables in the host.
`host/src/chat/memory/engram/` (install, process, migration, backup, clear) disappears.

## Packaging

The memory system is built as a self-contained crate, `codync-mem`, so other ACP clients can use it later.

**A core that only decides (sans-IO).**

- The core has no processes, network, model calls or threads. Events and data go in; changes and commands come
  out.
- It contains:
  - the store;
  - the gate;
  - the dreaming rules;
  - the opening, delta, keep-list and handoff renderers;
  - the compaction-timing state machine with its loop guard;
  - the mapping from commands to ACP payloads (`_meta.systemPrompt` for Claude, first message for others,
    `compact_prompt` for Codex).
- It uses the **caller's SQLite connection**, keeps its own tables and migration version, and writes in the
  caller's transaction. Memory and `rev` therefore commit together.
- **Model work goes back to the driver.** The core returns a job (prompt plus answer format). The driver runs it
  on the user's own harness and never calls provider APIs. The core validates the answer deterministically before
  writing: user-authored verbatim evidence, minimum content, the ¼ guard, the operation limit, the format, and the
  `rev` precondition.

**Drivers.**

- **The Codync host** embeds the core. It is the full product: three-client UI, transcript in the host, and a gate
  that sees only what the user typed.
- **An ACP proxy (later)** lets any ACP client use the core without changes: `codync-mem acp -- <agent command>`.
  - It can inject deltas and observe `usage_update` and `compaction_update`.
  - It can't set the opening. The [proxy chains RFD](https://agentclientprotocol.com/rfds/proxy-chains) (Draft)
    says proxies "cannot directly modify an agent's system prompt".
  - That waits for the [Client-Provided System Prompt RFD](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/rfds/client-system-prompt.mdx),
    which adds `systemPrompt {mode, content}` to new, load, resume and fork. It is in the schema behind an
    unstable feature (PR #2361), and claude-agent-acp 0.92 advertises it.
  - It can't separate what the user typed from what the client injected, so its gate is weaker.
  - It has no UI; a CLI covers review and undo.
  - Dreaming and compaction run between turns on the user's quota.

ACP status as of 2026-10:

- Completed: Session Usage, and Session Compaction (2026-10-06).
- Draft: End-Turn Token Usage, Session Fork, Client-Provided System Prompt, Proxy Chains.

**Repository.**

- The crate lives at `host/crates/memory`, as a member of a `[workspace]` added to `host/Cargo.toml`.
- The build output stays in `host/target`, so `npm run dev` and packaging are unaffected.
- CI runs `cargo clippy --workspace`.
- **Codync is the source of truth.** Core and host will change together for months, since memory and `rev`
  commit in one transaction, so the crate is developed only here. No git submodule.
- **A read-only public mirror, `leepokai/codync-mem`,** gives outside developers a repository of their own.
  - On every push to `main` that touches `host/crates/memory`, CI runs
    `git subtree split --prefix=host/crates/memory` and pushes the result to the mirror's `main`.
  - The split is deterministic, so the push always fast-forwards as long as nobody commits to the mirror.
  - Issues are filed on the mirror. PRs go to Codync's `dev`: a bot answers a PR opened on the mirror with that
    pointer and closes it.
- **The crate builds on its own**, because the mirror is a plain copy of its folder:
  - its `Cargo.toml` inherits nothing from the workspace (lints, edition and version are written out);
  - its CI lives in `host/crates/memory/.github/workflows/`. GitHub reads only the root `.github`, so the CI is
    inert in Codync and runs once the folder is the mirror's root;
  - it re-exports `rusqlite` and documents the supported version. Callers pass their own connection, and one build
    can't link two `rusqlite` versions to `sqlite3`;
  - it has its own README and LICENSE (Apache-2.0).
- The crate has its own 0.x versions, separate from Codync's.
- Open the mirror when rollout step 1 ships (the host calls the crate), marked 0.x with an unstable API. Publish
  0.1 to crates.io, with the proxy, only after Codync runs on it.
- **Flip later.** When outside users send PRs, the API stops changing with host changes for a few months, or the
  proxy gains users:
  - stop the sync and develop in the mirror;
  - switch Codync to the crates.io release and delete `host/crates/memory`.

  The mirror already holds the history, so nothing moves.

## Verification status

What the research answered:

1. **Does compaction pick up the latest loaded opening? Yes.**
   - Docs: "The new text takes effect after that compaction, or in a new session."
   - In the SDK binary, the recorded prompt is a hidden `prompt_snapshot` attachment. After a full compaction the
     message list is rebuilt without it, so the next request renders from the current options. This part is
     inferred: no explicit delete was found.
   - Resume reads the record and doesn't re-record.
   - Hence: render, load, then compact.
2. **Can the settings be set through ACP? Yes.**
   - `autoMemoryEnabled`, `autoDreamEnabled` and `idleCompaction` go through `claudeCode.options.settings`, which
     the adapter passes on untouched.
   - `excludeDynamicSections` and `snapshot` go on the `_meta.systemPrompt` object.
3. **What does Codex report?**
   - `codex-acp` reports `usage_update{used,size}` (from `thread/tokenUsage/updated`) and per-turn cached tokens.
   - It sends `compaction_update` in_progress/completed when the client declares `session.compaction`, which
     Codync does. The update carries no summary and no token counts.
   - Its `/compact` takes no instructions.
4. **Is recording on for Codync's sessions? Yes.**
   - Default-on since Claude Code 2.1.265; the feature-flag dependency ended in 2.1.268.
   - In 2.1.300 only `systemPromptSnapshot: false` or `CLAUDE_CODE_SIMPLE` turn it off.
   - Installed versions: Claude Code 2.1.301 and adapter 0.92.0 (SDK 0.3.300).

Still unknown, to check locally before M2:

- **Partial compaction:** how "Summarize from here" treats the recorded prompt.
- **2.1.301:** the installed binary wasn't searched, only the 2.1.300 binary bundled in the SDK.
- **Codex's auto-compact threshold:** about 90% of the window, from a secondary source only.
- **The real flow, end to end:** compaction order, keep-list effect, and cache read/write ratio per turn, all on
  the real host.

## Evaluation

**Property tests** in the crate:

- no memory without a user-authored verbatim span;
- pasted and quoted spans never count;
- acknowledgement-only quotes are rejected;
- the ¼ guard and the 10-operation limit;
- tombstones never resurrect;
- an `Ask` bot's write never lands without a confirmation;
- cascade on message delete;
- the temporal flip;
- the `rev` precondition;
- the loop guard.

**Scripted regression.** About 50 zh-Hant and English conversations covering:

- LongMemEval's five abilities: extraction, multi-session reasoning, temporal reasoning, knowledge update and
  abstention;
- MemoryAgentBench's selective forgetting.

Validate the judge adversarially first: a LoCoMo audit found a judge accepting up to 63% of intentionally wrong
answers.

**Compaction probes** after each compaction, Factory-style:

- recall;
- artifacts (paths, ids);
- continuation;
- decisions.

Also log the cache read/write ratio per turn and assert on it in e2e.

**Product metrics** (PostHog counts only, never content):

- the 7-day regret rate (memories the user edits or deletes);
- the dream-undo rate;
- compactions per session;
- rollovers;
- cache hit ratio before and after compaction;
- "wrong memory" taps.

## Rollout

Each milestone ships as its own minor release, with an e2e pass on all three apps.

- **M0. Cards, dreaming, the move from Engram (2.18.0, built).**
  - `host/crates/memory` in the standalone shape from [Packaging](#packaging); the tables; the move from Engram
    and Markdown memory in one pass.
  - The gate with pasted-span tagging in all three composers, the source policy and the sensitive-data rules.
  - The bot's memory tools: direct writes with undo in `Auto`, confirm cards in `Ask`; the shared "yours" layer.
  - Claude Code's auto memory off for personal bots, auto-dream off for all.
  - Dreaming: proposal, audit, evidence ids, the `rev` precondition, limits, headroom, the time review, the log
    with undo. Automatic naming at turn end.
  - Two layers on the memory screens of all three clients; `minApp` 2.18.0.
- **M1. Instructions for every harness.**
  - ACP `systemPrompt` where advertised; the first-message profile again after a compaction elsewhere, the
    compaction inferred from usage when the agent doesn't report it.
  - Each harness's own memory turned off for personal bots where it can be.
- **M2. Cache-safe opening and compaction.**
  - Remove the reload; send the `_meta.systemPrompt` object with `excludeDynamicSections`; fork re-sends it.
  - The ledger and deltas.
  - Usage and the TTL; the three triggers with the loop guard and the before/after steps.
  - The keep-list: Claude through `/compact`, Codex through `compact_prompt`.
  - The summary card, the editable handoff and its history bookmark.
  - Cache tokens asserted in e2e.
- **M3. Long-form memory.**
  - The folders, the naming and cap rules, `memory_files` and `memory_file_versions`.
  - The turn-end scan, versions, checks and restore; compare-and-swap host writes.
  - The index in the opening; `read_memory`, `edit_memory`, and `search_memory` over files;
    `additionalDirectories`.
  - Dreaming appends and moves detail out of full card layers.
  - The files, versions and "Updated" lines in all three clients.
- **M4. Evaluation and tuning.** The scripted regression below, plus questions answered only by a long-form file,
  short CJK queries and planted instructions; tune the caps and dreaming from it, and decide FTS5 and vectors by
  their triggers.
- **M5. The public mirror.** `leepokai/codync-mem`, synced on every release, as under [Packaging](#packaging).

## Sources

**Products**

- ChatGPT: [memory and dreaming](https://openai.com/index/chatgpt-memory-dreaming/), [Memory FAQ](https://help.openai.com/en/articles/8590148-memory-faq), [Projects](https://help.openai.com/en/articles/10169521-projects-in-chatgpt)
- Claude: [chat search and memory](https://support.claude.com/en/articles/11817273-use-claude-s-chat-search-and-memory-to-build-on-previous-context)
- Claude Code docs: [memory](https://code.claude.com/docs/en/memory), [context window](https://code.claude.com/docs/en/context-window), [prompt caching](https://code.claude.com/docs/en/prompt-caching), [hooks](https://code.claude.com/docs/en/hooks), [settings](https://code.claude.com/docs/en/settings-reference), [modifying system prompts](https://code.claude.com/docs/en/agent-sdk/modifying-system-prompts), [changelog](https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md)
- Claude Code issues: [auto-dream #38493](https://github.com/anthropics/claude-code/issues/38493)
- Anthropic API: [prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching), [compaction](https://platform.claude.com/docs/en/build-with-claude/compaction-on-demand), [context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing), [memory tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)
- Grok: [designing Grok Bot](https://x.ai/news/designing-grok-bot), [Team Bots](https://x.ai/news/team-bots), [Grok Build memory](https://x.ai/news/grok-build-memory)
- Codex: [memories](https://learn.chatgpt.com/docs/customization/memories), [`compact.rs`](https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/src/compact.rs), [`compact_remote_v2.rs`](https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/src/compact_remote_v2.rs)
- Coding agents: [Amp Handoff](https://ampcode.com/news/handoff), [Factory: evaluating compression](https://factory.com/news/evaluating-compression), [Cursor: dynamic context discovery](https://cursor.com/en-US/blog/dynamic-context-discovery)
- OpenClaw: [dreaming](https://docs.openclaw.ai/concepts/dreaming), [compaction](https://docs.openclaw.ai/concepts/compaction)
- Hermes: [memory](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory), [#5563](https://github.com/NousResearch/hermes-agent/issues/5563)
- Letta: [memory blocks](https://docs.letta.com/guides/agents/memory-blocks), [sleep-time compute](https://www.letta.com/blog/sleep-time-compute), [benchmarking agent memory](https://www.letta.com/blog/benchmarking-ai-agent-memory)
- Mem0: [token-efficient memory](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm)
- Zep/Graphiti: [Graphiti](https://github.com/getzep/graphiti), [Zep](https://arxiv.org/abs/2501.13956)

**Research**

- Benchmarks: [LongMemEval](https://arxiv.org/abs/2410.10813), [MemoryAgentBench](https://arxiv.org/abs/2507.05257), [LoCoMo audit](https://penfieldlabs.substack.com/p/we-audited-locomo-64-of-the-answer)
- Memory systems: [A-MEM](https://arxiv.org/abs/2502.12110), [Context Rot](https://www.trychroma.com/research/context-rot)
- Attacks: [MINJA](https://arxiv.org/abs/2503.03704), [AgentPoison](https://arxiv.org/abs/2407.12784), [multi-agent spread](https://arxiv.org/abs/2407.07791), [Gemini delayed tool invocation](https://embracethered.com/blog/posts/2025/gemini-memory-persistence-prompt-injection/), [Microsoft: AI recommendation poisoning](https://www.microsoft.com/en-us/security/blog/2026/02/10/ai-recommendation-poisoning/)
- Defenses: [CaMeL](https://arxiv.org/abs/2503.18813), [design patterns for securing LLM agents](https://arxiv.org/html/2506.08837)

**Community**

- [r/ChatGPT project-only memory leak](https://old.reddit.com/r/ChatGPT/comments/1rdei2p/)
- [r/ChatGPT turning off memory](https://old.reddit.com/r/ChatGPT/comments/1ve7jch/)
- [HN: Claude Memory](https://news.ycombinator.com/item?id=45684134)

**ACP**

- [RFD updates](https://agentclientprotocol.com/rfds/updates), [proxy chains](https://agentclientprotocol.com/rfds/proxy-chains), [client-provided system prompt](https://agentclientprotocol.com/rfds/client-system-prompt)
- [`codex-acp`](https://github.com/agentclientprotocol/codex-acp), [`agent-client-protocol` 2.0 proxies](https://docs.rs/agent-client-protocol/2.0.0/agent_client_protocol/concepts/proxies/index.html)

**Local**

- `@agentclientprotocol/claude-agent-acp` 0.92.0 (`acp-agent.js`, `context-compaction.js`)
- `@anthropic-ai/claude-agent-sdk` 0.3.300 (`sdk.d.ts`, bundled Claude Code 2.1.300)
