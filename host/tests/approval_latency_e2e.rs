//! How long an approval takes, on both sides: from the message to the card, and from the answer to the turn ending.
//!
//! The approval card is the product's centre (goal §13): nothing sensitive happens without it. A slow card leaves the person
//! looking at a screen that says nothing; a slow answer leaves them unsure whether their yes landed. The agent here asks on
//! every turn and answers at once once allowed, so the model is out of the measurement.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::{Value, json};
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
        let home = std::env::temp_dir().join(format!("codync-approval-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let mut host = Self { child, home, base, token: String::new() };
        std::fs::read_to_string(host.home.join("token")).unwrap().trim().clone_into(&mut host.token);
        host
    }

    async fn call(&self, method: &str, body: Value) -> Value {
        let response = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let value: Value = response.json().await.unwrap();
        assert!(status.is_success(), "{method} failed: {value}");
        value
    }

    async fn entries(&self, bot: &str) -> Vec<Value> {
        self.call("history", json!({"botId": bot})).await["entries"].as_array().cloned().unwrap_or_default()
    }

    /// Waits until an entry matches, returning it; reports how long that took.
    async fn wait_for(&self, bot: &str, what: &str, mut matches: impl FnMut(&Value) -> bool) -> (Value, u128) {
        let started = Instant::now();
        let deadline = started + Duration::from_secs(60);
        loop {
            if let Some(found) = self.entries(bot).await.into_iter().find(|e| matches(e)) {
                return (found, started.elapsed().as_millis());
            }
            assert!(Instant::now() < deadline, "no {what} within 60 s");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_approval_card_shows_and_the_yes_lands_in_this_long() {
    let host = Host::start().await;
    let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fake_agent.py");
    let created = host
        .call(
            "createBot",
            json!({"name":"Asker", "backend":"custom", "cwd":host.home, "command":common::python_agent(&agent),
                "permission":"ask", "connectors":[], "notify":false}),
        )
        .await;
    let bot = created["bot"]["id"].as_str().unwrap().to_owned();

    // The person sends a message; the agent asks before touching anything.
    let asked_started = Instant::now();
    host.call("send", json!({"botId": bot, "text": "edit a.txt"})).await;
    let (card, _) =
        host.wait_for(&bot, "permission card", |e| e["kind"] == "permission" && e["data"]["status"] == "pending").await;
    let card_ms = asked_started.elapsed().as_millis();

    // The person says yes; the turn ends with the agent's answer.
    let answered_started = Instant::now();
    host.call("respondPermission", json!({"entryId": card["id"], "optionId": "allow"})).await;
    host.wait_for(&bot, "final answer", |e| e["kind"] == "agent" && e["data"]["final"] == true).await;
    let done_ms = answered_started.elapsed().as_millis();

    println!("approval: message to card {card_ms} ms · yes to finished turn {done_ms} ms");

    // Both are things a person watches happen with their hand still on the phone.
    assert!(card_ms < 5_000, "the card took {card_ms} ms to appear");
    assert!(done_ms < 5_000, "the turn took {done_ms} ms after the yes");
}
