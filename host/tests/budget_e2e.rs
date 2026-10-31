//! A bot's daily token budget: what its turns spent, the gate before the next one, and the line the chat shows. The
//! count comes from what the agent reports per turn (`usage` on the `session/prompt` answer), not from a provider API.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[allow(dead_code)]
mod common;

use common::host::Host;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

async fn bot_value(host: &Host, bot: &str) -> Value {
    host.snapshot().await["bots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == bot)
        .cloned()
        .expect("the bot is in the snapshot")
}

/// Waits for a transcript entry whose text carries `needle`.
async fn line_with(host: &Host, bot: &str, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let history = host.call("history", json!({"botId": bot})).await;
        if let Some(text) = history["entries"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .filter_map(|e| e["data"]["text"].as_str())
            .find(|text| text.contains(needle))
        {
            return text.to_owned();
        }
        assert!(Instant::now() < deadline, "no line with {needle:?}: {history}");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn a_bot_past_its_daily_budget_starts_no_further_turn() {
    let host = Host::start().await;
    let bot = host.fixture_bot("Spender", "instructions_agent.py", "reporting").await;
    host.call("updateBot", json!({"id": bot, "budget": 1000})).await;
    assert_eq!(bot_value(&host, &bot).await["budget"], 1000);

    host.reply(&bot, "SPEND 600").await;
    assert_eq!(bot_value(&host, &bot).await["budgetSpentToday"], 600);
    host.reply(&bot, "SPEND 600").await;
    assert_eq!(bot_value(&host, &bot).await["budgetSpentToday"], 1200);

    // Over the budget: the third message never reaches the agent.
    host.call("send", json!({"botId": bot, "text": "one more", "clientNonce": "n-budget"})).await;
    let line = line_with(&host, &bot, "daily budget").await;
    assert!(line.contains("1000 tokens"), "{line}");
    let prompts = host.prompts(&bot);
    assert_eq!(prompts.len(), 2, "the agent saw only the two paid turns: {prompts:?}");

    // A raised budget lets it carry on.
    host.call("updateBot", json!({"id": bot, "budget": 5000})).await;
    host.reply(&bot, "SPEND 10").await;
    assert_eq!(host.prompts(&bot).len(), 3);
    // Clearing it removes the limit.
    host.call("updateBot", json!({"id": bot, "budget": null})).await;
    assert!(bot_value(&host, &bot).await["budget"].is_null());
}
