//! A bot whose agent accepts the turn and never answers stops on its own, and says why in the conversation.
//! `eventually` is here instead of a fixed sleep so a slow machine is not a failure.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::json;
use std::path::PathBuf;
use std::process::Child;
use std::time::{Duration, Instant};

struct Host {
    child: Child,
    home: PathBuf,
    base: String,
    token: String,
}

impl Drop for Host {
    fn drop(&mut self) {
        common::stop(&mut self.child);
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

impl Host {
    async fn start() -> Self {
        let home = std::env::temp_dir().join(format!("codync-stall-e2e-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let mut host = Self { child, home, base, token: String::new() };
        std::fs::read_to_string(host.home.join("token")).unwrap().trim().clone_into(&mut host.token);
        host
    }

    /// Every entry of a bot's main chat.
    async fn entries(&self, bot: &str) -> Vec<serde_json::Value> {
        let history = self.call("history", json!({"botId": bot})).await;
        history["entries"].as_array().cloned().unwrap_or_default()
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
}

#[tokio::test(flavor = "multi_thread")]
async fn a_silent_agent_is_stopped_and_the_chat_says_why() {
    // One second instead of three minutes: the watchdog reads the window from here so the test can watch it act.
    // SAFETY: this test binary has one test, so nobody else reads or writes the environment while it runs.
    unsafe { std::env::set_var("CODYNC_STALL_AFTER_SECS", "1") };
    let host = Host::start().await;
    let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent_agent.py");
    let created = host
        .call(
            "createBot",
            json!({"name":"Stalls", "backend":"custom", "cwd":host.home, "command":common::python_agent(&agent), "connectors":[], "notify":false}),
        )
        .await;
    let bot = created["bot"]["id"].as_str().unwrap().to_owned();

    host.call("send", json!({"botId":bot,"text":"say something"})).await;

    let deadline = Instant::now() + Duration::from_secs(45);
    let notice = loop {
        let found = host
            .entries(&bot)
            .await
            .into_iter()
            .find(|e| e["data"]["text"].as_str().is_some_and(|t| t.contains("stopped answering")));
        if let Some(found) = found {
            break found;
        }
        assert!(Instant::now() < deadline, "timed out waiting for the stalled turn to be stopped");
        tokio::time::sleep(Duration::from_millis(250)).await;
    };
    assert_eq!(notice["data"]["stopReason"], "cancelled", "the turn is recorded as cancelled: {notice}");

    // And the bot is free again: it is not left Working forever.
    let sync = common::snapshot(&host.base, &host.token).await;
    let status = common::bot(&sync, &bot)["status"].clone();
    assert_ne!(status, json!("working"), "the bot is still Working after the watchdog stopped its turn");
}
