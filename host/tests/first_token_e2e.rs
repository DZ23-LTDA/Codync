//! How long the host takes to put an agent's first word into the transcript, with an agent that answers at once.
//!
//! The gate asks for "first token" and a p95. Done with a real model those numbers include the model, which says nothing
//! about this program; with `fast_reply_agent.py` the only remaining cost is the host's own path -- process start, ACP
//! handshake, the update being turned into an entry -- which is the part a change here can make worse.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::json;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
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
        let home = std::env::temp_dir().join(format!("codync-first-token-{}", uuid::Uuid::new_v4()));
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

/// Milliseconds from `send` to the first entry whose text is the agent's, `rounds` times on one bot, sorted.
async fn samples(rounds: usize) -> Vec<u128> {
    let host = Host::start().await;
    let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fast_reply_agent.py");
    let created = host
        .call(
            "createBot",
            json!({"name":"Quick", "backend":"custom", "cwd":host.home, "command":common::python_agent(&agent), "connectors":[], "notify":false}),
        )
        .await;
    let bot = created["bot"]["id"].as_str().unwrap().to_owned();

    let mut samples = Vec::new();
    for round in 0..rounds {
        let before = host.entries(&bot).await.len();
        let started = Instant::now();
        host.call("send", json!({"botId": bot, "text": format!("round {round}")})).await;
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let entries = host.entries(&bot).await;
            if entries.len() > before && entries.iter().any(|e| e["data"]["text"] == json!("pong")) {
                break;
            }
            assert!(Instant::now() < deadline, "no answer within 30 s on round {round}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        samples.push(started.elapsed().as_millis());
    }

    samples.sort_unstable();
    samples
}

/// The ceiling a change must not cross, on a busy machine.
#[tokio::test(flavor = "multi_thread")]
async fn first_word_latency_stays_under_a_ceiling() {
    let all = samples(5).await;
    let worst = all[all.len() - 1];
    println!("first word (ms), 5 rounds: {all:?}");
    assert!(worst < 5_000, "first word took {worst} ms: {all:?}");
}

/// The p95 the gate asks for. Twenty rounds is slow for CI, so it runs on demand:
/// `cargo test --test first_token_e2e -- --ignored --nocapture`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "twenty rounds: run it to publish the baseline"]
async fn first_word_p95_over_twenty_rounds() {
    let all = samples(20).await;
    // Integer arithmetic: a float-to-usize cast here is a lint the crate refuses, and it was right to.
    let index = (all.len() * 95).div_ceil(100).saturating_sub(1);
    let p95 = all[index.min(all.len() - 1)];
    println!("first word (ms), 20 rounds: {all:?} · p95 {p95}");
}

/// How the first word behaves when eight bots answer at the same time.
///
/// One bot's latency says nothing about a machine running a team: every bot is its own process and its own
/// actor, and they all write into the same store. The gate asks for load, so this measures the moment the first
/// word of eight concurrent turns reaches the transcript:
/// `cargo test --test first_token_e2e -- --ignored --nocapture`.
///
/// No ceiling is asserted: the numbers belong in the report, and a slow CI runner is not a regression. What must
/// hold is that every one of them answers.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "eight bots at once: run it to publish the baseline"]
async fn first_word_with_eight_bots_answering_at_once() {
    let host = Arc::new(Host::start().await);
    let agent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fast_reply_agent.py");
    let mut bots = Vec::new();
    for index in 0..8 {
        let created = host
            .call(
                "createBot",
                json!({
                    "name": format!("Twin {index}"),
                    "backend": "custom",
                    "cwd": host.home,
                    "command": common::python_agent(&agent),
                    "connectors": [],
                    "notify": false,
                }),
            )
            .await;
        bots.push(created["bot"]["id"].as_str().unwrap().to_owned());
    }

    let started = Instant::now();
    let first_words = futures::future::join_all(bots.into_iter().map(|bot| {
        let host = Arc::clone(&host);
        async move {
            host.call("send", json!({"botId": bot, "text": "at once"})).await;
            let deadline = Instant::now() + Duration::from_secs(120);
            loop {
                let entries = host.entries(&bot).await;
                if entries.iter().any(|e| e["data"]["text"] == json!("pong")) {
                    return started.elapsed().as_millis();
                }
                assert!(Instant::now() < deadline, "bot {bot} never answered under load");
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }))
    .await;

    let mut all = first_words;
    all.sort_unstable();
    let index = (all.len() * 95).div_ceil(100).saturating_sub(1);
    let p95 = all[index.min(all.len() - 1)];
    println!("first word under load (ms), 8 bots at once: {all:?} · p95 {p95}");
}
