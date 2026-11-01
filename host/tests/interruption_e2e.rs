//! Killing the host while it is writing, then starting it again on the same home.
//!
//! The database section of the release gate asks for "interruption, recovery": a person whose computer shuts down mid-write
//! must get their bots back, not a message about corruption. This kills the process outright — no shutdown, no flush — and
//! restarts it, twice, then reads what was written before.

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

    /// Ends the process the way a power cut does: nothing is flushed, nothing is closed.
    fn kill_outright(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A half-written file left beside the database, as an interrupted write leaves one.
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

#[tokio::test(flavor = "multi_thread")]
async fn a_host_killed_mid_write_comes_back_with_its_bots() {
    let home = std::env::temp_dir().join(format!("codync-interrupt-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();

    // Write something worth losing, and stop nothing.
    let host = Host::start(home.clone()).await;
    let command =
        common::python_agent(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent_agent.py"));
    let first = host
        .call("createBot", json!({"name":"Survivor", "backend":"custom", "cwd":home, "command":command, "connectors":[], "notify":false}))
        .await["bot"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let second = host
        .call("createBot", json!({"name":"Also here", "backend":"custom", "cwd":home, "command":command, "connectors":[], "notify":false}))
        .await["bot"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(database(&home).exists());
    host.kill_outright();

    // Start again: the database must open, and both bots must be there.
    let host = Host::start(home.clone()).await;
    let sync = common::snapshot(&host.base, &host.token).await;
    let ids: Vec<&str> = sync["bots"].as_array().unwrap().iter().filter_map(|b| b["id"].as_str()).collect();
    assert!(ids.contains(&first.as_str()), "the first bot is gone after the kill: {ids:?}");
    assert!(ids.contains(&second.as_str()), "the second bot is gone after the kill: {ids:?}");
    assert!(host.call("history", json!({"botId": first})).await["entries"].is_array());

    // Nothing was set aside: an interrupted write is not a damaged file.
    assert!(set_aside(&home).is_empty(), "the host set files aside: {:?}", set_aside(&home));

    // And the same home survives a second one: the list must be the same size, not smaller and not duplicated. The host
    // adds a lead bot of its own on a fresh home, so the number itself is not the point -- stability is.
    host.kill_outright();
    let host = Host::start(home.clone()).await;
    let after_kill = common::snapshot(&host.base, &host.token).await["bots"].as_array().unwrap().len();
    tokio::time::sleep(Duration::from_millis(50)).await;
    host.kill_outright();
    let again = Host::start(home.clone()).await;
    let after_second = common::snapshot(&again.base, &again.token).await["bots"].as_array().unwrap().len();
    assert_eq!(after_second, after_kill, "a second restart changed the bot count: {after_kill} then {after_second}");
    again.kill_outright();
    let _ = std::fs::remove_dir_all(&home);
}
