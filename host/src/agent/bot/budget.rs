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
    fn yesterdays_spend_does_not_count_against_today() {
        let (_dir, store) = TempDir::store();
        store.kv_set(&key("b1", "2000-01-01"), "900").expect("write yesterday");
        assert_eq!(spent(&store, "b1"), 0, "today starts at zero");
        record(&store, "b1", 100);
        assert_eq!(spent(&store, "b1"), 100);
        assert!(store.kv_get(&key("b1", "2000-01-01")).is_none(), "the day before is dropped");
    }
}
