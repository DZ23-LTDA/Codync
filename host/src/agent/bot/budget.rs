//! A bot's daily token budget: what its turns spent since midnight, and the gate the next turn passes before it
//! costs anything. Counted from what the agents report for each turn (`usage` on the `session/prompt` answer), kept
//! in `kv` so no schema change is needed, and reset by the calendar rather than by a job.
//!
//! Cost in money would need a price per model; tokens are what every harness reports, so a budget is in tokens.

use crate::store::{BotConfig, Store};
use chrono::Local;

/// The day a spend belongs to, in the computer's time zone: a budget is "per day as the user lives it".
fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn key(bot: &str, day: &str) -> String {
    format!("spend.{bot}.{day}")
}

/// Tokens the bot spent today.
pub fn spent(store: &Store, bot: &str) -> u64 {
    store.kv_get(&key(bot, &today())).and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// Adds a turn's tokens to today's spend, and drops the days before it.
pub fn record(store: &Store, bot: &str, tokens: u64) {
    if tokens == 0 {
        return;
    }
    let day = today();
    let total = spent(store, bot).saturating_add(tokens);
    if let Err(error) = store.kv_set(&key(bot, &day), &total.to_string()) {
        tracing::warn!(bot = %bot, error = format!("{error:#}"), "couldn't record the turn's spend");
        return;
    }
    // Yesterday's lines are of no use to the gate and would pile up for the life of the bot.
    let prefix = format!("spend.{bot}.");
    for old in store.kv_prefix(&prefix).into_iter().filter(|k| !k.ends_with(&day)) {
        if let Err(error) = store.kv_delete(&old) {
            tracing::warn!(bot = %bot, error = format!("{error:#}"), "couldn't drop an old spend line");
        }
    }
}

/// `Some(budget)` when today's spend has reached the bot's budget: the next turn would go over it.
pub fn exhausted(store: &Store, cfg: &BotConfig) -> Option<u64> {
    let budget = cfg.budget?;
    (spent(store, &cfg.id) >= budget).then_some(budget)
}

fn cost_key(bot: &str, day: &str) -> String {
    format!("spend-usd.{bot}.{day}")
}

/// Micro-dollars the bot spent today, from the price the user declared for its plan.
pub fn spent_usd(store: &Store, bot: &str) -> u64 {
    store.kv_get(&cost_key(bot, &today())).and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// Adds a turn cost in micro-dollars: tokens times the price of a million tokens (the same number, by definition).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "a day spend stays far below 2^53 micro-dollars"
)]
pub fn record_cost(store: &Store, bot: &str, tokens: u64, price_per_mtok: f64) {
    if tokens == 0 || price_per_mtok <= 0.0 {
        return;
    }
    let micro = (tokens as f64 * price_per_mtok).round().max(0.0) as u64;
    let total = spent_usd(store, bot).saturating_add(micro);
    if let Err(error) = store.kv_set(&cost_key(bot, &today()), &total.to_string()) {
        tracing::warn!(bot = %bot, error = format!("{error:#}"), "could not record what the turn cost");
    }
}

/// `Some(limit)` when today spend has reached the bot money limit: the next turn would go over it.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a daily limit stays far below 2^53 micro-dollars"
)]
pub fn exhausted_usd(store: &Store, cfg: &BotConfig) -> Option<f64> {
    let budget = cfg.budget_usd?;
    let limit = (budget * 1_000_000.0).round().max(0.0) as u64;
    (spent_usd(store, &cfg.id) >= limit).then_some(budget)
}

fn usage_key(bot: &str, day: &str) -> String {
    format!("usage.{bot}.{day}")
}

/// Tokens the bot spent today, split as the agents report them: fresh input, cache read, cache written.
pub fn usage(store: &Store, bot: &str) -> (u64, u64, u64) {
    let saved =
        store.kv_get(&usage_key(bot, &today())).and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok());
    let part = |name: &str| saved.as_ref().and_then(|v| v[name].as_u64()).unwrap_or(0);
    (part("input"), part("cacheRead"), part("cacheWrite"))
}

/// Adds a turn usage to today, by kind.
pub fn record_usage(store: &Store, bot: &str, input: u64, cache_read: u64, cache_write: u64) {
    if input == 0 && cache_read == 0 && cache_write == 0 {
        return;
    }
    let (i, r, w) = usage(store, bot);
    let saved = serde_json::json!({
        "input": i.saturating_add(input),
        "cacheRead": r.saturating_add(cache_read),
        "cacheWrite": w.saturating_add(cache_write),
    });
    if let Err(error) = store.kv_set(&usage_key(bot, &today()), &saved.to_string()) {
        tracing::warn!(bot = %bot, error = format!("{error:#}"), "could not record the turn usage");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn store() -> (Self, Store) {
            let dir = std::env::temp_dir().join(format!("codync-budget-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).expect("create the fixture directory");
            let store = Store::open(&dir.join("t.db")).expect("open the store");
            (Self(dir), store)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn bot(budget: Option<u64>) -> BotConfig {
        let mut cfg: BotConfig =
            serde_json::from_value(serde_json::json!({"id": "b1", "name": "B", "backend": "claude"}))
                .expect("a bot config");
        cfg.budget = budget;
        cfg
    }

    #[test]
    fn a_day_spent_to_its_budget_stops_the_next_turn() {
        let (_dir, store) = TempDir::store();
        let cfg = bot(Some(1000));
        assert_eq!(exhausted(&store, &cfg), None, "a new bot may spend");
        record(&store, &cfg.id, 400);
        record(&store, &cfg.id, 600);
        assert_eq!(spent(&store, &cfg.id), 1000);
        assert_eq!(exhausted(&store, &cfg), Some(1000), "the budget is reached, not exceeded");
        // A bot with no budget never stops, whatever it spent.
        let unlimited = bot(None);
        assert_eq!(exhausted(&store, &unlimited), None);
        assert_eq!(exhausted(&store, &bot(Some(2000))), None, "a raised budget lets it carry on");
    }

    #[test]
    fn usage_keeps_the_kinds_apart() {
        let (_dir, store) = TempDir::store();
        record_usage(&store, "b1", 100, 90_000, 400);
        record_usage(&store, "b1", 50, 1_000, 0);
        assert_eq!(usage(&store, "b1"), (150, 91_000, 400));
        assert_eq!(usage(&store, "other"), (0, 0, 0), "a bot that never ran");
    }

    #[test]
    fn a_turn_costs_tokens_times_the_price_of_a_million() {
        let (_dir, store) = TempDir::store();
        // Three dollars per million tokens: a thousand tokens cost three thousand micro-dollars.
        record_cost(&store, "b1", 1000, 3.0);
        assert_eq!(spent_usd(&store, "b1"), 3_000);
        let mut cfg = bot(None);
        cfg.budget_usd = Some(0.002);
        assert_eq!(exhausted_usd(&store, &cfg), Some(0.002), "already over the limit");
        cfg.budget_usd = Some(1.0);
        assert_eq!(exhausted_usd(&store, &cfg), None, "a limit above the spend lets it carry on");
        assert_eq!(exhausted_usd(&store, &bot(None)), None, "a bot with no money limit never stops");
    }

    #[test]
    fn yesterdays_spend_does_not_count_against_today() {
        let (_dir, store) = TempDir::store();
        store.kv_set(&key("b1", "2000-01-01"), "900").expect("write yesterday");
        assert_eq!(spent(&store, "b1"), 0, "today starts at zero");
        record(&store, "b1", 100);
        assert_eq!(spent(&store, "b1"), 100);
        assert!(store.kv_get(&key("b1", "2000-01-01")).is_none(), "the day before is dropped");
    }
}
