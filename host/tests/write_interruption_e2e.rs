//! Killing the host while a database write is still in flight.
//!
//! The release gate's database section asks for an interrupted write, not only an interrupted process:
//! a laptop that shuts down between "the row was updated" and "the transaction is committed" must come
//! back with a database that opens, not with a message about corruption and not with half a bot.
//!
//! Two properties, both checked after an outright kill (no shutdown, no flush):
//!
//! 1. While another process holds the database's write lock, the host's write cannot commit; killing it
//!    there must leave the file untouched and readable.
//! 2. Every write the client saw succeed must be there after the restart, and no write the client never
//!    saw succeed may appear as a half-built bot.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

struct Host {
    child: Child,
    base: String,
    token: String,
}

impl Host {
    async fn start(home: PathBuf) -> Self {
        let (child, base) = common::serve(&home).await;
        let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
        Self { child, base, token }
    }

    async fn call(&self, method: &str, body: serde_json::Value) -> serde_json::Value {
        let response = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let value: serde_json::Value = response.json().await.unwrap();
        assert!(status.is_success(), "{method} failed: {value}");
        value
    }

    async fn try_call(&self, method: &str, body: serde_json::Value) -> Option<serde_json::Value> {
        let response = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .ok()?;
        response.json::<serde_json::Value>().await.ok()
    }

    fn kill_outright(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The database file in `home`, whatever the host named it.
fn database(home: &Path) -> PathBuf {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(home).unwrap().flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| ["db", "sqlite", "sqlite3"].iter().any(|e| ext.eq_ignore_ascii_case(e)))
        {
            found.push(path);
        }
    }
    found.sort();
    assert!(!found.is_empty(), "no database file under {}", home.display());
    found.remove(0)
}

/// Anything the host set aside because it could not read it.
fn set_aside(home: &Path) -> Vec<String> {
    std::fs::read_dir(home)
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            let lower = name.to_ascii_lowercase();
            lower.contains("corrupt")
                || lower.contains("damaged")
                || Path::new(name).extension().is_some_and(|e| e.eq_ignore_ascii_case("bak"))
        })
        .collect()
}

/// What SQLite itself says about the file: `ok` is the only acceptable answer.
fn integrity(home: &Path) -> String {
    let db = rusqlite::Connection::open(database(home)).expect("open the database to check it");
    db.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)).expect("integrity check")
}

fn bot_names(home: &Path) -> Vec<String> {
    let db = rusqlite::Connection::open(database(home)).expect("open the database");
    let mut st = db.prepare("SELECT config FROM bots WHERE deleted = 0").expect("read bots");
    let rows = st.query_map([], |row| row.get::<_, String>(0)).expect("bots");
    let mut names: Vec<String> = rows
        .map(|config| {
            let config: serde_json::Value = serde_json::from_str(&config.expect("config")).expect("bot config is JSON");
            config["name"].as_str().unwrap_or_default().to_owned()
        })
        .collect();
    names.sort();
    names
}

fn bot_body(name: &str, home: &Path) -> serde_json::Value {
    let command =
        common::python_agent(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent_agent.py"));
    json!({"name": name, "backend": "custom", "cwd": home, "command": command, "connectors": [], "notify": false})
}

/// The host is killed while another process holds the write lock: its write never commits, and the
/// database it leaves behind opens cleanly with everything written before.
#[tokio::test(flavor = "multi_thread")]
async fn a_kill_while_the_store_cannot_write_leaves_a_database_that_opens() {
    let home = std::env::temp_dir().join(format!("codync-locked-write-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();

    let host = Host::start(home.clone()).await;
    host.call("createBot", bot_body("Before the lock", &home)).await;
    assert_eq!(integrity(&home), "ok");

    // Hold the write lock from the test, so the host's next write cannot reach a commit.
    let lock = rusqlite::Connection::open(database(&home)).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    lock.execute("INSERT INTO kv(k, v) VALUES('lock-probe', 'held')", []).unwrap();

    // Ask for a write and kill the host while that request is still outstanding.
    let pending = tokio::spawn({
        let base = host.base.clone();
        let token = host.token.clone();
        let body = bot_body("Never committed", &home);
        async move {
            let _ = reqwest::Client::new()
                .post(format!("{base}/api/createBot"))
                .bearer_auth(token)
                .json(&body)
                .timeout(Duration::from_secs(20))
                .send()
                .await;
        }
    });
    tokio::time::sleep(Duration::from_millis(400)).await;
    host.kill_outright();
    pending.abort();
    lock.execute_batch("ROLLBACK").unwrap();
    drop(lock);

    // The file is still a valid database with what was committed, and nothing was set aside.
    assert_eq!(integrity(&home), "ok", "the database is damaged after a write interrupted by the lock");
    assert_eq!(bot_names(&home), vec!["Before the lock".to_owned()], "the uncommitted bot reached the file");

    // And the host comes back on it.
    let host = Host::start(home.clone()).await;
    let sync = common::snapshot(&host.base, &host.token).await;
    let names: Vec<&str> = sync["bots"].as_array().unwrap().iter().filter_map(|b| b["name"].as_str()).collect();
    assert!(names.contains(&"Before the lock"), "the committed bot is gone: {names:?}");
    assert!(!names.contains(&"Never committed"), "the write that never committed came back: {names:?}");
    assert!(set_aside(&home).is_empty(), "the host set files aside: {:?}", set_aside(&home));
    host.kill_outright();
    let _ = std::fs::remove_dir_all(&home);
}

/// Writes are fired as fast as the host takes them and the process is killed in the middle of the
/// burst: every write the client saw succeed must survive, and no half-built bot may exist.
#[tokio::test(flavor = "multi_thread")]
async fn every_answered_write_survives_the_kill_and_no_half_write_does() {
    let home = std::env::temp_dir().join(format!("codync-burst-write-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();

    let host = Host::start(home.clone()).await;
    host.call("createBot", bot_body("Before the burst", &home)).await;
    // The host makes a lead bot of its own (Codync), on the first start or on a restart that
    // finds the home empty: it is the host's own row, not a write this test asked for.
    let mut baseline = bot_names(&home);
    baseline.push("Codync".to_owned());

    let mut answered: Vec<String> = Vec::new();
    for index in 0..40 {
        let name = format!("Burst {index}");
        match host.try_call("createBot", bot_body(&name, &home)).await {
            Some(value) if value.get("bot").and_then(|b| b["name"].as_str()) == Some(name.as_str()) => {
                answered.push(name);
            }
            _ => break,
        }
        if index == 8 {
            // Kill right after an answered write: the transaction for the next one may be open.
            host.kill_outright();
            break;
        }
    }
    let host = Host::start(home.clone()).await;
    // A second burst on the restarted host, then the kill that ends the test.
    let mut after_restart: Vec<String> = Vec::new();
    for index in 0..40 {
        let name = format!("Again {index}");
        match host.try_call("createBot", bot_body(&name, &home)).await {
            Some(_) => after_restart.push(name),
            None => break,
        }
    }
    host.kill_outright();

    assert_eq!(integrity(&home), "ok", "the database is damaged after a burst of writes and a kill");
    let names = bot_names(&home);
    for name in answered.iter().chain(after_restart.iter()) {
        assert!(names.contains(name), "{name} was answered but is not in the file: {names:?}");
    }
    // Every row is a full bot: a half-written transaction would leave a row whose config does not parse
    // (it would have failed above) or a name nobody asked for.
    let known: std::collections::HashSet<&String> = answered.iter().chain(after_restart.iter()).collect();
    let unexpected: Vec<&String> = names.iter().filter(|n| !baseline.contains(n) && !known.contains(n)).collect();
    assert!(unexpected.is_empty(), "the file holds bots nobody wrote: {unexpected:?}");
    assert!(set_aside(&home).is_empty(), "the host set files aside: {:?}", set_aside(&home));

    let host = Host::start(home.clone()).await;
    let sync = common::snapshot(&host.base, &host.token).await;
    assert_eq!(sync["bots"].as_array().unwrap().len(), names.len(), "the host lists a different set than the file");
    host.kill_outright();
    let _ = std::fs::remove_dir_all(&home);
}
