//! A run: one task handed to several bots at once, and the ledger that follows it.
//!
//! The host owns the fan-out (`chat::team`'s request path) and the ledger (`kv`), so the transcript, the cost and the
//! audit stay in one place. A sealed run waits for a person before any bot is asked: the seal is the difference between
//! a bot that proposes and a bot that acts (`docs/plans/run-orchestration.md`).

pub mod merge;

use crate::chat::team;
use crate::hub::Hub;
use crate::store::{Store, now_ms};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::sync::Arc;

/// Below this a run is a message, above it a broadcast nobody joins.
const MIN_BOTS: usize = 2;
const MAX_BOTS: usize = 8;

/// Where a run is in its life.
const AWAITING: &str = "awaitingApproval";
const RUNNING: &str = "running";
const DONE: &str = "done";
const REJECTED: &str = "rejected";
const EXPIRED: &str = "expired";

/// One run, as its ledger keeps it.
#[derive(Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub task: String,
    #[serde(default)]
    pub schema: Option<Value>,
    pub bots: Vec<Asked>,
    pub created_at: i64,
    /// A sealed run is kept here until a person answers it, and no bot is asked before that.
    #[serde(default)]
    pub seal: bool,
    /// The last state the ledger knows; a running run also reads its requests (see `status`).
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approved_at: Option<i64>,
    /// SHA-256 of exactly what will run, so an approval cannot be replayed onto a different payload.
    #[serde(default)]
    pub payload_hash: String,
    /// A sealed run nobody answered by then is expired and never reaches a bot.
    #[serde(default)]
    pub expires_at: Option<i64>,
}

/// How long a sealed run waits for a person.
const SEAL_TTL_MS: i64 = 24 * 60 * 60 * 1000;

/// One bot the run was handed to, and the request that carries it.
#[derive(Clone, Serialize, Deserialize)]
pub struct Asked {
    pub bot: String,
    pub name: String,
    pub request: String,
    /// The bot answer, filed when it reaches the sender (see `record_reply`); empty while the run waits.
    #[serde(default)]
    pub reply: Option<String>,
}

/// What the approval covers: the task, the bots and the schema, canonically serialised and hashed.
fn payload_hash(task: &str, bots: &[Asked], schema: Option<&Value>) -> String {
    let ids: Vec<&str> = bots.iter().map(|asked| asked.bot.as_str()).collect();
    let canonical = serde_json::json!({"task": task, "bots": ids, "schema": schema}).to_string();
    let digest = Sha256::digest(canonical.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
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

/// Asks every bot in the run, in the order the user listed them.
fn fan_out(hub: &Arc<Hub>, from: &str, run: &mut Run) -> Result<()> {
    let source = team::visible_bot(hub, from)?;
    for asked in &mut run.bots {
        let answer = team::message_bot(hub, &source, &asked.bot, &run.task)?;
        let name = answer["name"].as_str().unwrap_or_default();
        if !name.is_empty() {
            name.clone_into(&mut asked.name);
        }
        answer["requestId"].as_str().unwrap_or_default().clone_into(&mut asked.request);
    }
    for asked in &run.bots {
        hub.store.kv_set(&format!("runreq.{}", asked.request), &run.id)?;
    }
    Ok(())
}

/// `startRun`: one request per bot, or a sealed run kept until a person answers it.
pub fn start(hub: &Arc<Hub>, from: &str, by: &str, b: &Value) -> Result<Value> {
    let task = b["task"].as_str().with_context(|| "task is required")?.trim().to_owned();
    if task.is_empty() {
        bail!("task is empty");
    }
    let ids = bots(from, &b["bots"])?;
    let schema = b.get("schema").filter(|s| s.is_object()).cloned();
    let seal = b["seal"].as_bool().unwrap_or(false);
    let asked: Vec<Asked> =
        ids.into_iter().map(|bot| Asked { bot, name: String::new(), request: String::new(), reply: None }).collect();
    let mut run = Run {
        id: uuid::Uuid::new_v4().to_string(),
        payload_hash: payload_hash(&task, &asked, schema.as_ref()),
        task,
        schema,
        bots: asked,
        created_at: now_ms(),
        seal,
        state: if seal { AWAITING } else { RUNNING }.to_owned(),
        approved_by: None,
        approved_at: None,
        expires_at: seal.then(|| now_ms() + SEAL_TTL_MS),
    };
    if seal {
        // A sealed run never reaches a bot before a person answers it.
        save(&hub.store, &run)?;
        return Ok(json!({"runId": run.id, "state": AWAITING, "sealed": true}));
    }
    fan_out(hub, from, &mut run)?;
    let approved_by = by.to_owned();
    run.approved_by = Some(approved_by);
    run.approved_at = Some(now_ms());
    save(&hub.store, &run)?;
    let named: Vec<Value> = run.bots.iter().map(|a| json!({"bot": a.bot, "name": a.name})).collect();
    Ok(json!({"runId": run.id, "state": RUNNING, "bots": named}))
}

/// `approveRun`: the person answers a sealed run, and only now do its bots get asked.
pub fn approve(hub: &Arc<Hub>, from: &str, by: &str, b: &Value) -> Result<Value> {
    let id = b["runId"].as_str().with_context(|| "runId is required")?;
    let mut run = load(&hub.store, id)?;
    if run.state != AWAITING {
        bail!("run {id} isn't waiting for approval (it is {})", run.state);
    }
    if run.expires_at.is_some_and(|at| now_ms() > at) {
        run.state.clear();
        run.state.push_str(EXPIRED);
        save(&hub.store, &run)?;
        bail!("run {id} expired before anyone approved it");
    }
    if run.payload_hash != payload_hash(&run.task, &run.bots, run.schema.as_ref()) {
        bail!("run {id} changed since it was sealed; seal it again");
    }
    fan_out(hub, from, &mut run)?;
    run.state.clear();
    run.state.push_str(RUNNING);
    let approved_by = by.to_owned();
    run.approved_by = Some(approved_by);
    run.approved_at = Some(now_ms());
    save(&hub.store, &run)?;
    Ok(json!({"runId": run.id, "state": RUNNING, "approvedBy": run.approved_by}))
}

/// `rejectRun`: a sealed run nobody approved. It never reaches a bot.
pub fn reject(hub: &Arc<Hub>, by: &str, b: &Value) -> Result<Value> {
    let id = b["runId"].as_str().with_context(|| "runId is required")?;
    let mut run = load(&hub.store, id)?;
    if run.state != AWAITING {
        bail!("run {id} isn't waiting for approval (it is {})", run.state);
    }
    run.state.clear();
    run.state.push_str(REJECTED);
    let approved_by = by.to_owned();
    run.approved_by = Some(approved_by);
    run.approved_at = Some(now_ms());
    save(&hub.store, &run)?;
    Ok(json!({"runId": run.id, "state": REJECTED}))
}

/// Files a reply in its run ledger, when the request belongs to a run.
pub fn record_reply(hub: &Arc<Hub>, request: &str, text: &str) {
    let Some(run_id) = hub.store.kv_get(&format!("runreq.{request}")) else { return };
    let Ok(mut run) = load(&hub.store, &run_id) else { return };
    let mut filed = false;
    for asked in &mut run.bots {
        if asked.request == request {
            asked.reply = Some(text.to_owned());
            filed = true;
        }
    }
    if filed && let Err(error) = save(&hub.store, &run) {
        tracing::warn!(run = %run_id, error = format!("{error:#}"), "could not file a run reply");
    }
}

/// `runStatus`: each bot request as the transcript knows it, and what the run came to.
pub fn status(hub: &Arc<Hub>, b: &Value) -> Result<Value> {
    let id = b["runId"].as_str().with_context(|| "runId is required")?;
    let run = load(&hub.store, id)?;
    let mut pending = false;
    let mut bots = Vec::with_capacity(run.bots.len());
    for asked in &run.bots {
        let status = hub.store.entry(&asked.request).map_or(Value::Null, |entry| entry.data["status"].clone());
        pending |= matches!(status.as_str(), Some("queued" | "sent"));
        bots.push(json!({"bot": asked.bot, "name": asked.name, "request": asked.request, "status": status}));
    }
    let state = match run.state.as_str() {
        RUNNING if pending => RUNNING,
        RUNNING => DONE,
        other => other,
    };
    let answered: Vec<merge::Answer> = run
        .bots
        .iter()
        .filter_map(|a| a.reply.as_ref().map(|text| merge::Answer { name: a.name.clone(), text: text.clone() }))
        .collect();
    let merged = match (run.schema.as_ref(), state) {
        (Some(schema), DONE) => Some(merge::merge(&answered, schema)),
        _ => None,
    };
    Ok(json!({
        "runId": run.id,
        "task": run.task,
        "state": state,
        "sealed": run.seal,
        "approvedBy": run.approved_by,
        "approvedAt": run.approved_at,
        "createdAt": run.created_at,
        "bots": bots,
        "merged": merged.map(|m| json!({
            "agreed": m.agreed.iter().cloned().collect::<std::collections::BTreeMap<String, Value>>(),
            "disagreed": m.disagreed.iter().map(|d| json!({"field": d.field, "values": d.values})).collect::<Vec<_>>(),
            "unparsed": m.unparsed.iter().map(|a| json!({"name": a.name, "text": a.text})).collect::<Vec<_>>(),
        })),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &Value) -> Result<Vec<String>> {
        bots("me", v)
    }

    #[test]
    fn the_approval_covers_exactly_what_was_sealed() {
        let asked = |ids: &[&str]| -> Vec<Asked> {
            ids.iter()
                .map(|id| Asked { bot: (*id).to_owned(), name: String::new(), request: String::new(), reply: None })
                .collect()
        };
        let schema = json!({"verdict": "string"});
        let base = payload_hash("decide", &asked(&["a", "b"]), Some(&schema));
        assert_eq!(base.len(), 64);
        assert_eq!(base, payload_hash("decide", &asked(&["a", "b"]), Some(&schema)), "same payload, same hash");
        assert_ne!(base, payload_hash("decide!", &asked(&["a", "b"]), Some(&schema)), "another task");
        assert_ne!(base, payload_hash("decide", &asked(&["b", "a"]), Some(&schema)), "another roster order");
        assert_ne!(base, payload_hash("decide", &asked(&["a", "b"]), None), "another schema");
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
