//! A run: one task handed to several bots at once, and the ledger that follows it.
//!
//! The host owns the fan-out (`chat::team`'s request path) and the ledger (`kv`), so the transcript, the cost and the
//! audit stay in one place. The join and the seal are the next slices: `docs/plans/run-orchestration.md`.

use crate::chat::team;
use crate::hub::Hub;
use crate::store::{Store, now_ms};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

/// Below this a run is a message, above it a broadcast nobody joins.
const MIN_BOTS: usize = 2;
const MAX_BOTS: usize = 8;

/// One run, as its ledger keeps it.
#[derive(Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub task: String,
    #[serde(default)]
    pub schema: Option<Value>,
    pub bots: Vec<Asked>,
    pub created_at: i64,
}

/// One bot the run was handed to, and the request that carries it.
#[derive(Clone, Serialize, Deserialize)]
pub struct Asked {
    pub bot: String,
    pub name: String,
    pub request: String,
}

fn key(id: &str) -> String {
    format!("run.{id}")
}

fn save(store: &Store, run: &Run) -> Result<()> {
    store.kv_set(&key(&run.id), &serde_json::to_string(run)?)
}

fn load(store: &Store, id: &str) -> Result<Run> {
    let raw = store.kv_get(&key(id)).with_context(|| format!("no run {id} on this computer"))?;
    serde_json::from_str(&raw).with_context(|| format!("run {id} is unreadable"))
}

/// The bots a run may go to: two to eight, no duplicates, and never the caller itself.
fn bots(from: &str, ids: &Value) -> Result<Vec<String>> {
    let Some(list) = ids.as_array() else { bail!("bots is required and must be a list of bot ids") };
    if list.len() < MIN_BOTS {
        bail!("a run needs at least {MIN_BOTS} bots; use send for one");
    }
    if list.len() > MAX_BOTS {
        bail!("a run takes at most {MAX_BOTS} bots");
    }
    let mut out: Vec<String> = Vec::with_capacity(list.len());
    for id in list {
        let id = id.as_str().with_context(|| format!("bots must be strings, got {id}"))?;
        if id == from {
            bail!("a bot can't ask itself");
        }
        if out.iter().any(|seen| seen == id) {
            bail!("{id} is listed twice");
        }
        out.push(id.to_owned());
    }
    Ok(out)
}

/// `startRun`: one request per bot, and the ledger that follows them.
pub fn start(hub: &Arc<Hub>, from: &str, b: &Value) -> Result<Value> {
    let task = b["task"].as_str().with_context(|| "task is required")?.trim().to_owned();
    if task.is_empty() {
        bail!("task is empty");
    }
    let ids = bots(from, &b["bots"])?;
    let schema = b.get("schema").filter(|s| s.is_object()).cloned();
    let source = team::visible_bot(hub, from)?;
    let mut asked = Vec::with_capacity(ids.len());
    for to in &ids {
        let answer = team::message_bot(hub, &source, to, &task)?;
        asked.push(Asked {
            bot: answer["botId"].as_str().unwrap_or(to).to_owned(),
            name: answer["name"].as_str().unwrap_or_default().to_owned(),
            request: answer["requestId"].as_str().unwrap_or_default().to_owned(),
        });
    }
    let run = Run { id: uuid::Uuid::new_v4().to_string(), task, schema, bots: asked, created_at: now_ms() };
    save(&hub.store, &run)?;
    let named: Vec<Value> = run.bots.iter().map(|a| json!({"bot": a.bot, "name": a.name})).collect();
    Ok(json!({"runId": run.id, "bots": named}))
}

/// `runStatus`: each bot's request as the transcript knows it, and whether the run is still going.
pub fn status(hub: &Arc<Hub>, b: &Value) -> Result<Value> {
    let id = b["runId"].as_str().with_context(|| "runId is required")?;
    let run = load(&hub.store, id)?;
    let mut running = false;
    let mut bots = Vec::with_capacity(run.bots.len());
    for asked in &run.bots {
        let status = hub.store.entry(&asked.request).map_or(Value::Null, |entry| entry.data["status"].clone());
        running |= matches!(status.as_str(), Some("queued" | "sent"));
        bots.push(json!({"bot": asked.bot, "name": asked.name, "request": asked.request, "status": status}));
    }
    Ok(json!({
        "runId": run.id,
        "task": run.task,
        "status": if running { "running" } else { "done" },
        "bots": bots,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &Value) -> Result<Vec<String>> {
        bots("me", v)
    }

    #[test]
    fn a_run_goes_to_between_two_and_eight_other_bots() {
        assert_eq!(ids(&json!(["a", "b"])).expect("two").len(), 2);
        assert!(ids(&json!(["a"])).expect_err("one is a message").to_string().contains("at least 2"));
        let nine: Vec<String> = (0..9).map(|i| format!("b{i}")).collect();
        assert!(ids(&json!(nine)).expect_err("too many").to_string().contains("at most 8"));
        assert!(ids(&json!(["a", "me"])).expect_err("itself").to_string().contains("itself"));
        assert!(ids(&json!(["a", "a"])).expect_err("twice").to_string().contains("twice"));
        assert!(ids(&json!("a")).expect_err("not a list").to_string().contains("list"));
    }
}
