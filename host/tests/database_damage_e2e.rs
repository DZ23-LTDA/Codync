//! Starting the host on a home whose database it cannot use: a newer schema and a damaged file.
//!
//! The store already refuses both (its own unit tests cover the message and the untouched file). What
//! these tests add is the process: a person double-clicks the app, the host comes up on a home it cannot
//! read, and what they get must be a sentence they can act on — not a silently empty roster, not a
//! rewritten database, not a process that keeps restarting forever.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
        assert!(response.status().is_success(), "{method} failed");
        response.json().await.unwrap()
    }

    fn kill_outright(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// What the host said and how it ended when it was started on a home it cannot use.
struct Refusal {
    exit: Option<std::process::ExitStatus>,
    log: String,
}

/// Starts `codync-host serve` for `home` and waits for it to end: a host that answers instead of
/// ending is killed and reported as "it started", which is the failure these tests guard against.
async fn start_and_wait(home: &Path) -> Refusal {
    let log = home.join("refusal.log");
    let file = std::fs::File::create(&log).expect("log");
    let errors = file.try_clone().expect("log clone");
    let mut child = Command::new(env!("CARGO_BIN_EXE_codync-host"))
        .args(["serve", "--bind", "127.0.0.1", "--port", "0"])
        .env("CODYNC_HOME", home)
        .env("CODYNC_CLOUD", "off")
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(errors))
        .spawn()
        .expect("start the host");
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(exit) = child.try_wait().expect("host state") {
            return Refusal { exit: Some(exit), log: std::fs::read_to_string(&log).unwrap_or_default() };
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Refusal { exit: None, log: std::fs::read_to_string(&log).unwrap_or_default() };
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn database(home: &Path) -> PathBuf {
    home.join("codync.db")
}

fn bot_body(name: &str, home: &Path) -> serde_json::Value {
    let command =
        common::python_agent(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent_agent.py"));
    json!({"name": name, "backend": "custom", "cwd": home, "command": command, "connectors": [], "notify": false})
}

/// A home with one bot of its own, ready to be broken on purpose.
async fn home_with_a_bot(tag: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("codync-{tag}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();
    let host = Host::start(home.clone()).await;
    host.call("createBot", bot_body("Written by this build", &home)).await;
    host.kill_outright();
    // A killed host leaves a WAL that still holds the database: folding it back in makes the file
    // on disk the only copy, which is the thing the tests below damage.
    {
        let connection = rusqlite::Connection::open(database(&home)).unwrap();
        connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").unwrap();
    }
    assert!(!home.join("codync.db-wal").exists() || std::fs::metadata(home.join("codync.db-wal")).unwrap().len() == 0);
    home
}

/// A database written by a newer build is refused by the process: it exits, says what happened, and
/// leaves the file exactly as it found it.
#[tokio::test(flavor = "multi_thread")]
async fn the_process_refuses_a_database_from_a_newer_build() {
    let home = home_with_a_bot("newer-schema").await;
    let db = database(&home);
    {
        let connection = rusqlite::Connection::open(&db).unwrap();
        connection.execute("UPDATE kv SET v = '99' WHERE k = 'schema'", []).unwrap();
        assert_eq!(
            connection.query_row("SELECT v FROM kv WHERE k = 'schema'", [], |r| r.get::<_, String>(0)).unwrap(),
            "99"
        );
    }
    let before = std::fs::read(&db).unwrap();

    let refusal = start_and_wait(&home).await;
    assert!(refusal.exit.is_some(), "the host served a home written by a newer build:\n{}", refusal.log);
    assert!(!refusal.exit.unwrap().success(), "the host exited successfully on a newer schema:\n{}", refusal.log);
    assert!(refusal.log.contains("newer version"), "the message doesn't say it is newer:\n{}", refusal.log);
    assert!(
        refusal.log.contains("Nothing was changed"),
        "the message doesn't say nothing was touched:\n{}",
        refusal.log
    );
    assert_eq!(std::fs::read(&db).unwrap(), before, "the refused database was rewritten");
    let _ = std::fs::remove_dir_all(&home);
}

/// A database whose header is gone is refused the same way, byte for byte intact, and no empty
/// database is put in its place.
#[tokio::test(flavor = "multi_thread")]
async fn the_process_refuses_a_damaged_database_and_puts_nothing_in_its_place() {
    let home = home_with_a_bot("damaged-header").await;
    let db = database(&home);
    std::fs::write(&db, b"this is not a database").unwrap();
    let before = std::fs::read(&db).unwrap();

    let refusal = start_and_wait(&home).await;
    assert!(refusal.exit.is_some(), "the host served a damaged database:\n{}", refusal.log);
    assert!(!refusal.exit.unwrap().success(), "the host exited successfully on a damaged file:\n{}", refusal.log);
    assert!(refusal.log.contains("may be damaged"), "the message doesn't name the damage:\n{}", refusal.log);
    assert!(
        refusal.log.contains("Nothing was deleted"),
        "the message doesn't say nothing was deleted:\n{}",
        refusal.log
    );
    assert_eq!(std::fs::read(&db).unwrap(), before, "the damaged file was rewritten instead of left alone");
    assert!(!home.join("codync.db-wal").exists() || std::fs::metadata(home.join("codync.db-wal")).unwrap().len() == 0);

    // The person's next step, from the message, still works: move the damaged file away and the host
    // starts on a fresh database rather than on top of the old one.
    std::fs::rename(&db, home.join("codync.db.damaged")).unwrap();
    let host = Host::start(home.clone()).await;
    let sync = common::snapshot(&host.base, &host.token).await;
    assert!(sync["bots"].is_array(), "the host didn't come up on a fresh home");
    host.kill_outright();
    assert!(home.join("codync.db.damaged").exists(), "the damaged file is gone");
    let _ = std::fs::remove_dir_all(&home);
}

/// A truncated database (the file cut in half, as a full disk or a bad copy leaves it) is refused, not
/// silently repaired into an empty one.
#[tokio::test(flavor = "multi_thread")]
async fn the_process_refuses_a_truncated_database() {
    let home = home_with_a_bot("truncated").await;
    let db = database(&home);
    let bytes = std::fs::read(&db).unwrap();
    assert!(bytes.len() >= 4096, "the database is too small to truncate meaningfully: {}", bytes.len());
    std::fs::write(&db, &bytes[..bytes.len() / 2]).unwrap();
    let before = std::fs::read(&db).unwrap();

    let refusal = start_and_wait(&home).await;
    assert!(refusal.exit.is_some(), "the host served a truncated database:\n{}", refusal.log);
    assert!(!refusal.exit.unwrap().success(), "the host exited successfully on a truncated file:\n{}", refusal.log);
    assert!(
        refusal.log.contains("may be damaged") || refusal.log.contains("malformed"),
        "the message doesn't name the damage:\n{}",
        refusal.log
    );
    assert_eq!(std::fs::read(&db).unwrap(), before, "the truncated file was rewritten");
    let _ = std::fs::remove_dir_all(&home);
}
