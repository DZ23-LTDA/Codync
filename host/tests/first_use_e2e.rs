//! The host half of the first three minutes: create a bot, send a message, see the first answer.
//!
//! The first-run script measures a person's wall clock. Some of that time is this host, and this test says how much: from
//! `createBot` returning to the first entry the bot's answer puts in the transcript, with an agent that answers at once so
//! the model is not blamed for the host's work.

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
        let home = std::env::temp_dir().join(format!("codync-first-use-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let mut host = Self { child, home, base, token: String::new() };
        std::fs::read_to_string(host.home.join("token")).unwrap().trim().clone_into(&mut host.token);
        host
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

    async fn entries(&self, bot: &str) -> Vec<serde_json::Value> {
        let history = self.call("history", json!({"botId": bot})).await;
        history["entries"].as_array().cloned().unwrap_or_default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_three_first_steps_take_this_long_on_the_host() {
    let host = Host::start().await;
    let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fast_reply_agent.py");

    // Step 3 of the runbook: create a bot.
    let created_started = Instant::now();
    let created = host
        .call(
            "createBot",
            json!({"name":"First", "backend":"custom", "cwd":host.home, "command":common::python_agent(&agent), "connectors":[], "notify":false}),
        )
        .await;
    let created_ms = created_started.elapsed().as_millis();
    let bot = created["bot"]["id"].as_str().unwrap().to_owned();

    // Steps 4 and 5: send a message and wait for the first answer to reach the transcript.
    let send_started = Instant::now();
    host.call("send", json!({"botId": bot, "text": "hello"})).await;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let answered = host.entries(&bot).await.iter().any(|e| e["data"]["text"] == json!("pong"));
        if answered {
            break;
        }
        assert!(Instant::now() < deadline, "no answer within 60 s");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let first_answer_ms = send_started.elapsed().as_millis();
    let total_ms = created_ms + first_answer_ms;

    println!(
        "first use on the host: createBot {created_ms} ms · message to first answer {first_answer_ms} ms · total {total_ms} ms"
    );

    // The runbook gives the whole first run three minutes; the host's share has to leave room for the person.
    assert!(total_ms < 20_000, "the host took {total_ms} ms for the first three steps");
}
