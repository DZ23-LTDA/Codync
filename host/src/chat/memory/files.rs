//! Memory on disk ([design](../../../../docs/design/bot-memory.md#memory-is-files)): the folders, the scan that
//! turns what changed in them into versions, and the host's own compare-and-swap writes. The rules (names, checks,
//! who wrote each fact, what a bot may change, the index) are `codync_mem::files`.

use crate::LockExt;
use crate::hub::Hub;
use crate::store::{BotConfig, Store};
use anyhow::{Context, Result, anyhow, bail};
use codync_mem::files::{self as mf, Author, Cause, Change, Version};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

/// A file changed more recently than this may still be mid-write: the next scan takes it.
const SETTLE_MS: i64 = 2_000;
/// How long a file's earlier versions are kept, and with them what a bot forgot.
const KEEP_MS: i64 = 30 * 24 * 60 * 60 * 1000;

/// A layer's folder: the shared one (`None`), or a bot's own.
pub fn folder(bot: Option<&str>) -> PathBuf {
    let data = crate::service::data_dir();
    match bot {
        Some(id) => data.join("bots").join(id).join("bot-memory"),
        None => data.join("shared-memory"),
    }
}

/// The layers a bot sees: its own, and the shared one when it uses it.
pub fn layers(cfg: &BotConfig) -> Vec<Option<String>> {
    let mut layers = vec![Some(cfg.id.clone())];
    if super::shared(cfg) {
        layers.push(None);
    }
    layers
}

/// The folders a bot sees, created so an agent can be given them.
pub fn folders(cfg: &BotConfig) -> Vec<PathBuf> {
    layers(cfg)
        .iter()
        .map(|layer| {
            let dir = folder(layer.as_deref());
            if let Err(error) = std::fs::create_dir_all(&dir) {
                tracing::warn!(dir = %dir.display(), error = %error, "couldn't create a memory folder");
            }
            dir
        })
        .collect()
}

/// The index of the notes a bot sees, for its opening.
pub fn index(store: &Store, cfg: &BotConfig) -> String {
    match store.memory_read(|c| mf::index(c, &cfg.id, super::shared(cfg))) {
        Ok(index) => index.render(|bot| folder(bot).display().to_string()),
        Err(error) => {
            tracing::warn!(bot = %cfg.id, error = format!("{error:#}"), "couldn't read the memory files");
            String::new()
        }
    }
}

/// A bot's turn, from its prompt until the scan after it: a change found meanwhile in a folder the bot sees is the
/// bot's, even one no tool call of it named (a shell command, say), once the turn ran a tool that can write files.
#[derive(Clone, Debug)]
struct Turn {
    bot: String,
    /// When it began, Unix ms; also its id.
    start: i64,
    /// The session it runs in: what of its changes was put back is told there.
    session: String,
    /// As in `Runtime::message`: only then may the bot change or take out what memory says.
    message: Option<String>,
    /// When it ended, Unix ms; `None` while it runs.
    ended: Option<i64>,
    /// It ran a tool that can write files (an edit, delete, move or command).
    wrote: bool,
}

// ponytail: in memory, so a host restart forgets turns; a change made just before one is then the computer's.
static TURNS: Mutex<Vec<Turn>> = Mutex::new(Vec::new());

/// A bot is about to prompt its agent. A turn of it that never ended (a fallback took over) is this one now.
pub fn begin_turn(bot: &str, session: &str, message: Option<String>) {
    let mut turns = TURNS.locked();
    turns.retain(|t| t.bot != bot || t.ended.is_some());
    turns.push(Turn {
        bot: bot.to_owned(),
        start: crate::store::now_ms(),
        session: session.to_owned(),
        message,
        ended: None,
        wrote: false,
    });
}

/// A bot's turn didn't start after all: nothing of it is left to scan.
pub fn drop_turn(bot: &str) {
    TURNS.locked().retain(|t| t.bot != bot || t.ended.is_some());
}

/// Notes that the bot's running turn ran a tool that can write files; the turn's start.
fn wrote(bot: &str) -> Option<i64> {
    let mut turns = TURNS.locked();
    let turn = turns.iter_mut().find(|t| t.bot == bot && t.ended.is_none())?;
    turn.wrote = true;
    Some(turn.start)
}

/// The tool call of a bot that last touched a memory file, and in which turn, so the scan credits the change to it.
#[derive(Serialize, Deserialize)]
struct Touch {
    bot: String,
    /// The [`Turn::start`] of the turn it was made in.
    #[serde(default)]
    turn: i64,
}

/// Notes the memory files a tool call names (its `locations` and diffs), so the scan knows which bot changed them,
/// and that its turn ran a tool that can write files. Only calls that change files count: a read left behind
/// would credit the next outside edit to the bot.
pub fn touched(hub: &Hub, cfg: &BotConfig, tool: &serde_json::Value) {
    let kind = tool["toolKind"].as_str();
    let changes =
        matches!(kind, Some("edit" | "delete" | "move")) || tool["diffs"].as_array().is_some_and(|d| !d.is_empty());
    if !changes && kind != Some("execute") {
        return;
    }
    let Some(turn) = wrote(&cfg.id).filter(|_| changes) else { return };
    let paths = tool["locations"]
        .as_array()
        .into_iter()
        .chain(tool["diffs"].as_array())
        .flatten()
        .filter_map(|item| item["path"].as_str());
    let folders: Vec<PathBuf> = layers(cfg).iter().map(|layer| folder(layer.as_deref())).collect();
    for path in paths {
        let path = Path::new(path);
        if path.parent().is_some_and(|dir| folders.iter().any(|f| f == dir)) {
            let touch = Touch { bot: cfg.id.clone(), turn };
            let value = serde_json::to_string(&touch).expect("a touch is plain data");
            if let Err(error) = hub.store.kv_set(&touch_key(path), &value) {
                tracing::warn!(error = format!("{error:#}"), "couldn't note a memory file edit");
            }
        }
    }
}

fn touch_key(path: &Path) -> String {
    format!("memory.touched.{}", path.display())
}

/// One scan or host write at a time, so two can't record the same change, nor two writes pass the same
/// compare-and-swap.
// ponytail: one lock for every folder; per-folder locks if scans ever queue behind each other.
static SCANNING: Mutex<()> = Mutex::new(());

/// Records what changed in the folders a bot sees since the last scan. Returns whether a file was too fresh to
/// take yet.
pub fn scan(hub: &Hub, cfg: &BotConfig) -> bool {
    let _one = SCANNING.locked();
    prune(&hub.store);
    let mut deferred = false;
    for layer in layers(cfg) {
        match scan_layer(hub, layer.as_deref()) {
            Ok(fresh) => deferred |= fresh,
            Err(error) => tracing::warn!(bot = %cfg.id, error = format!("{error:#}"), "couldn't scan the memory files"),
        }
    }
    deferred
}

/// Once a day: versions older than [`KEEP_MS`] go.
fn prune(store: &Store) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if store.kv_get("memory.pruned").as_deref() == Some(today.as_str()) {
        return;
    }
    match store.memory_write(|c, now, _| mf::prune(c, now - KEEP_MS)) {
        Ok(()) => {
            let _ = store.kv_set("memory.pruned", &today);
        }
        Err(error) => tracing::warn!(error = format!("{error:#}"), "couldn't prune old memory versions"),
    }
}

/// At the end of a turn: the bot's edits usually come right before its reply, so a file still settling is taken
/// once it has. The turn stays the bot's until then.
pub fn scan_after_turn(hub: Arc<Hub>, cfg: BotConfig) {
    let mut start = None;
    for turn in TURNS.locked().iter_mut().filter(|t| t.bot == cfg.id && t.ended.is_none()) {
        turn.ended = Some(crate::store::now_ms());
        start = Some(turn.start);
    }
    tokio::spawn(async move {
        let (h, c) = (hub.clone(), cfg.clone());
        if tokio::task::spawn_blocking(move || scan(&h, &c)).await.unwrap_or(false) {
            tokio::time::sleep(std::time::Duration::from_millis(u64::try_from(SETTLE_MS).unwrap_or(2_000) + 100)).await;
            let c = cfg.clone();
            let _ = tokio::task::spawn_blocking(move || scan(&hub, &c)).await;
        }
        TURNS.locked().retain(|t| t.bot != cfg.id || Some(t.start) != start);
    });
}

/// What is on disk for one file.
struct OnDisk {
    content: String,
    mtime: i64,
    size: u64,
}

/// Scans one folder; true when a file was too fresh to take yet.
fn scan_layer(hub: &Hub, layer: Option<&str>) -> Result<bool> {
    let dir = folder(layer);
    // A folder that went missing is not every file deleted.
    let Ok(listing) = std::fs::read_dir(&dir) else { return Ok(false) };
    let mut deferred = false;
    let recorded: HashMap<String, mf::File> =
        hub.store.memory_read(|c| mf::layer(c, layer))?.into_iter().map(|f| (f.name.clone(), f)).collect();
    let now = crate::store::now_ms();
    let mut present = HashSet::new();
    let mut items: Vec<(String, PathBuf)> =
        listing.flatten().filter_map(|item| Some((item.file_name().to_str()?.to_owned(), item.path()))).collect();
    // MEMORY.md last: a fact moved from it into a new note is then found in the note, not taken as forgotten.
    items.sort_by_key(|(name, _)| name == mf::MAIN);
    for (name, path) in items {
        let Some((mtime, size)) = regular(&path, &name) else { continue };
        present.insert(name.clone());
        let known = recorded.get(&name);
        if known.is_some_and(|f| !f.deleted && f.mtime == mtime && f.size == size) {
            continue;
        }
        // A time in the future isn't waited for: it would outlast the turn that set it.
        if (0..SETTLE_MS).contains(&(now - mtime)) {
            deferred = true;
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => {
                tracing::warn!(file = %name, error = %error, "couldn't read a memory file");
                continue;
            }
        };
        if known.is_some_and(|f| f.hash == mf::hash(&content)) {
            hub.store.memory_write(|c, _, _| mf::observed(c, layer, &name, mtime, size))?;
            continue;
        }
        found(hub, layer, &name, Some(&OnDisk { content, mtime, size }))?;
    }
    for name in recorded.values().filter(|f| !f.deleted && !present.contains(&f.name)).map(|f| &f.name) {
        found(hub, layer, name, None)?;
    }
    Ok(deferred)
}

/// A regular `.md` file with a valid name: its modification time and size. Links, hidden files and folders are
/// left alone, so nothing can make the host read, journal or restore another file.
fn regular(path: &Path, name: &str) -> Option<(i64, u64)> {
    if !mf::valid_name(name) {
        return None;
    }
    let meta = std::fs::symlink_metadata(path).ok().filter(std::fs::Metadata::is_file)?;
    let mtime = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_millis();
    Some((i64::try_from(mtime).ok()?, meta.len()))
}

/// The bot's turn a change found in a layer's folder is credited to: the one whose tool call named the file, else
/// the turn of a bot that sees the folder and ran a tool that can write files, one the user didn't start first, so
/// the stricter rules apply. A turn that ended before the file was last modified didn't make it, unless that time
/// is in the future. Any other change is the computer's: the user's own, in an editor say.
fn turn_of(hub: &Hub, layer: Option<&str>, touch: Option<&Touch>, mtime: Option<i64>) -> Option<(BotConfig, Turn)> {
    let now = crate::store::now_ms();
    let turns: Vec<Turn> = TURNS
        .locked()
        .iter()
        .filter(|t| match (t.ended, mtime) {
            (Some(ended), Some(at)) => at <= ended || at > now,
            _ => true,
        })
        .cloned()
        .collect();
    let config = |bot: &str| hub.store.bot(bot).ok().flatten().filter(|row| !row.deleted).map(|row| row.config);
    let named = touch.and_then(|touch| turns.iter().find(|t| t.bot == touch.bot && t.start == touch.turn));
    if let Some(turn) = named
        && let Some(cfg) = config(&turn.bot)
    {
        return Some((cfg, turn.clone()));
    }
    turns
        .into_iter()
        .filter(|turn| turn.wrote)
        .filter_map(|turn| Some((config(&turn.bot)?, turn)))
        .filter(|(cfg, _)| layers(cfg).iter().any(|l| l.as_deref() == layer))
        .min_by_key(|(_, turn)| turn.message.is_some())
}

/// A change the scan found: made in a bot's turn, checked, and put back when it may not stand; otherwise just
/// recorded as the computer's.
fn found(hub: &Hub, layer: Option<&str>, name: &str, after: Option<&OnDisk>) -> Result<()> {
    let path = folder(layer).join(name);
    let touch: Option<Touch> = hub.store.kv_get(&touch_key(&path)).and_then(|v| serde_json::from_str(&v).ok());
    hub.store.kv_delete(&touch_key(&path))?;
    let before = hub.store.memory_read(|c| mf::current(c, layer, name))?;
    let (mtime, size) = after.map_or((0, 0), |a| (a.mtime, a.size));
    let content = after.map(|a| a.content.as_str());
    let change = Change {
        bot: layer,
        name,
        content,
        author: Author::Computer,
        author_bot: None,
        cause: if content.is_some() { Cause::Edit } else { Cause::Delete },
        message: None,
        mtime,
        size,
    };
    let Some((cfg, turn)) = turn_of(hub, layer, touch.as_ref(), after.map(|a| a.mtime)) else {
        record(hub, &change)?;
        return Ok(());
    };
    let asked = turn.message.is_some();
    if let Err(error) = admit(hub, layer, name, before.as_deref(), content, asked) {
        // Put back what was there, as it was recorded, and tell the bot why.
        put(&path, before.as_deref())?;
        let (mtime, size) = regular(&path, name).unwrap_or_default();
        if before.is_some() {
            hub.store.memory_write(|c, _, _| mf::observed(c, layer, name, mtime, size))?;
        }
        tracing::info!(bot = %cfg.id, file = name, reason = %error, "put back a bot's memory change");
        super::put_back(&hub.store, &cfg.id, &turn.session, name, &error.to_string());
        return Ok(());
    }
    let change = Change { author: Author::Bot, author_bot: Some(&cfg.id), message: turn.message.as_deref(), ..change };
    record_bot_change(hub, &cfg, Some(&turn.session), before.as_deref(), &change, asked)?;
    Ok(())
}

/// Whether a bot's change to a file may stand: what it brings in may be memory, and a bot may change it outside a
/// turn the user started in its own chat (`asked`).
fn admit(
    hub: &Hub,
    layer: Option<&str>,
    name: &str,
    before: Option<&str>,
    after: Option<&str>,
    asked: bool,
) -> Result<()> {
    if let Some(content) = after {
        mf::check_change(layer, name, before, content)?;
    }
    hub.store.memory_read(|c| mf::admit(c, layer, before, after, asked))?;
    Ok(())
}

/// Records a bot's change, with what it had forgotten when the user asked for it, and tells the other sessions;
/// the one it was made in (`session`) already knows.
fn record_bot_change(
    hub: &Hub,
    cfg: &BotConfig,
    session: Option<&str>,
    before: Option<&str>,
    change: &Change<'_>,
    asked: bool,
) -> Result<Version> {
    let heard = super::current_revision(&hub.store, &cfg.id);
    let version = hub.store.memory_write(|c, now, rev| {
        mf::settle(c, change.bot, change.name, before, change.content, asked)?;
        mf::record(c, change, now, rev)
    })?;
    super::changed(hub, change.bot);
    if let Some(session) = session
        && let Err(error) = super::heard_own_write(&hub.store, &cfg.id, session, &heard)
    {
        tracing::warn!(bot = %cfg.id, error = format!("{error:#}"), "couldn't record the bot's own edit");
    }
    Ok(version)
}

/// The file's content on disk now, `None` when there is none. Refuses names the folders don't hold.
pub fn read(layer: Option<&str>, name: &str) -> Result<Option<String>> {
    if !mf::valid_name(name) {
        bail!(
            "Memory files are MEMORY.md, or notes named in lowercase letters, digits and dashes, ending in .md (like people.md)."
        );
    }
    let path = folder(layer).join(name);
    if regular(&path, name).is_none() {
        return Ok(None);
    }
    std::fs::read_to_string(&path).map(Some).with_context(|| format!("reading {name}"))
}

/// Who a host write is for.
#[derive(Clone, Copy)]
pub struct Writer<'a> {
    pub author: Author,
    pub bot: Option<&'a BotConfig>,
    pub cause: Cause,
    /// A bot's write: the user's message that started the turn, when it was one the user started in the bot's own
    /// chat. Only then may it change or take out what memory says.
    pub message: Option<&'a str>,
}

/// Writes a file the way every host writer does: only over the content whose hash is `expected` (`None`: no file
/// yet), through a temporary file in the same folder renamed over it, or by deleting it (`content` `None`). A file
/// that changed meanwhile isn't overwritten: its change is recorded and the write fails. A bot's write passes the
/// checks and the rules on what it may change first.
pub fn write(
    hub: &Hub,
    layer: Option<&str>,
    name: &str,
    content: Option<&str>,
    expected: Option<&str>,
    writer: Writer<'_>,
) -> Result<Version> {
    // Held until the change is recorded, so another writer can't pass the same compare-and-swap meanwhile.
    let _one = SCANNING.locked();
    let now = read(layer, name)?;
    if now.as_deref().map(mf::hash).as_deref() != expected {
        // The other change is recorded, so it isn't lost.
        scan_layer(hub, layer)?;
        bail!("{name} changed meanwhile. Read it again and redo the change on top of it.");
    }
    let asked = writer.message.is_some();
    if writer.author == Author::Bot {
        admit(hub, layer, name, now.as_deref(), content, asked)?;
    }
    if now.as_deref() == content {
        return hub
            .store
            .memory_read(|c| {
                let file = mf::get(c, layer, name)?.ok_or(codync_mem::store::Error::NotFound)?;
                mf::version(c, file.version)?.ok_or(codync_mem::store::Error::NotFound)
            })
            .context("the file has no recorded version");
    }
    let path = folder(layer).join(name);
    std::fs::create_dir_all(folder(layer)).context("creating the memory folder")?;
    put(&path, content)?;
    let (mtime, size) = regular(&path, name).unwrap_or_default();
    let change = Change {
        bot: layer,
        name,
        content,
        author: writer.author,
        author_bot: writer.bot.map(|cfg| cfg.id.as_str()),
        cause: writer.cause,
        message: writer.message,
        mtime,
        size,
    };
    match writer.bot {
        Some(cfg) if writer.author == Author::Bot => {
            let session = hub.runtime(&cfg.id).session;
            record_bot_change(hub, cfg, session.as_deref(), now.as_deref(), &change, asked)
        }
        _ => record(hub, &change),
    }
}

/// Records a change no rule on a bot's changes applies to, and tells the apps and sessions.
fn record(hub: &Hub, change: &Change<'_>) -> Result<Version> {
    let version = hub.store.memory_write(|c, now, rev| mf::record(c, change, now, rev))?;
    super::changed(hub, change.bot);
    Ok(version)
}

/// Replaces or deletes a file on disk: a temporary sibling renamed over it, so a reader never sees half of it.
fn put(path: &Path, content: Option<&str>) -> Result<()> {
    let Some(content) = content else {
        return match std::fs::remove_file(path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        };
    };
    let dir = path.parent().ok_or_else(|| anyhow!("a memory file has a folder"))?;
    let temp = dir.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temp, content).context("writing the memory file")?;
    if let Err(error) = std::fs::rename(&temp, path) {
        let _ = std::fs::remove_file(&temp);
        return Err(error).context("replacing the memory file");
    }
    Ok(())
}

/// A deleted bot's folder.
pub fn remove_bot(id: &str) {
    if let Err(error) = std::fs::remove_dir_all(folder(Some(id)))
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %error, "couldn't remove a deleted bot's memory files");
    }
}
