//! Chat bridge on a real host: the secret names the bot, an unknown one is refused, a huge body is refused, and the
//! message lands in the bot's chat marked as bridged — never as the person typing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[allow(dead_code)]
mod common;

use common::host::Host;
use serde_json::json;
use std::time::{Duration, Instant};

#[tokio::test]
async fn a_bridged_message_reaches_its_bot_and_bad_secrets_and_bodies_do_not() {
    let host = Host::start().await;
    let bot = host.fixture_bot("Bridged", "instructions_agent.py", "reporting").await;
    let info = host.call("bridgeInfo", json!({"botId": bot.clone()})).await;
    let secret = info["secret"].as_str().unwrap().to_owned();
    let client = reqwest::Client::new();

    let unknown = client
        .post(format!("{}/hooks/bots/not-a-secret", host.base))
        .json(&json!({"text": "hello"}))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), reqwest::StatusCode::UNAUTHORIZED, "an unknown secret names no bot");

    let huge = client
        .post(format!("{}/hooks/bots/{secret}", host.base))
        .json(&json!({"text": "x".repeat(9 * 1024)}))
        .send()
        .await
        .unwrap();
    assert_eq!(huge.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE, "a body past the cap is refused");

    let accepted = client
        .post(format!("{}/hooks/bots/{secret}", host.base))
        .json(&json!({"text": "hello from the bridge", "from": "telegram"}))
        .send()
        .await
        .unwrap();
    assert_eq!(accepted.status(), reqwest::StatusCode::OK, "{}", accepted.text().await.unwrap());

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let history = host.call("history", json!({"botId": bot})).await;
        let found = history["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["data"]["text"] == "hello from the bridge" && e["data"]["from"] == "bridge");
        if found {
            break;
        }
        assert!(Instant::now() < deadline, "the bridged message never landed: {history}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
