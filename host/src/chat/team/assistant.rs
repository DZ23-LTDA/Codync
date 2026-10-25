//! The user's cloud assistant (ChatGPT, Claude or another one they connected to Codync) through
//! the Codync connector at `mcp.codync.dev` (docs/reference/remote-relay.md §7.9). The cloud
//! forwards its tool calls over this computer's relay socket; they're answered only while the
//! user has turned cloud assistants on here, and only for this computer's own account. It
//! messages this computer's bots the way a bot does. Nothing can wake it, so the replies wait here
//! until it reads them (in memory: a restart forgets them).

use super::{
    Back, BotRequest, MAX_MESSAGE_BYTES, MessageLifetime, REQUEST_QUOTE_BYTES, RequestStatus, directory, visible_bot,
};
use crate::LockExt;
use crate::agent::acp::truncate;
use crate::agent::bot::Cmd;
use crate::hub::Hub;
use crate::remote::cloud::{self, CloudStatus};
use crate::store::{EntryKind, Lane, Store, now_ms};
use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// How it shows in a bot's chat, and its id there (`sourceBotId`).
const NAME: &str = "Cloud assistant";
const ID: &str = "assistant";
/// kv: `on` lets the cloud assistant in.
const KV: &str = "cloud_assistants";
/// Requests still running at a time.
const MAX_RUNNING: usize = 16;
/// Requests remembered, oldest finished ones forgotten first; `read_replies` shows the newest.
const KEEP: usize = 50;
const SHOW: usize = 20;
const OFF: &str = "Cloud assistants are off on this computer. To let ChatGPT or Claude message its bots, \
run `codync-host cloud --assistants on` on it.";

pub fn enabled(store: &Store) -> bool {
    store.kv_get(KV).as_deref() == Some("on")
}

/// `setAssistants`: whether the cloud assistant may reach this computer's bots. Local only.
pub fn set_enabled(hub: &Hub, on: bool) -> Result<CloudStatus> {
    cloud::require_accounts(&hub.store)?;
    hub.store.kv_set(KV, if on { "on" } else { "off" })?;
    tracing::info!(on, "cloud assistants changed");
    cloud::update_status(hub, |s| s.assistants = on);
    Ok(hub.cloud.status())
}

/// Answers one `assistant.call` from the relay: `{"t":"assistant.result","id","ok","result"|"error"}`.
pub fn answer(hub: &Arc<Hub>, call: &Value) -> Value {
    let id = &call["id"];
    let tool = call["tool"].as_str().unwrap_or_default();
    match run(hub, call["user"].as_str().unwrap_or_default(), tool, &call["args"]) {
        Ok(result) => json!({"t": "assistant.result", "id": id, "ok": true, "result": result}),
        Err(e) => {
            tracing::info!(tool, error = format!("{e:#}"), "refused a cloud assistant call");
            json!({"t": "assistant.result", "id": id, "ok": false, "error": format!("{e:#}")})
        }
    }
}

fn run(hub: &Arc<Hub>, user: &str, tool: &str, args: &Value) -> Result<Value> {
    if !enabled(&hub.store) {
        bail!(OFF);
    }
    // The cloud only routes the account's own calls here; this computer checks that too.
    if user.is_empty() || hub.cloud.status().owner.map(|o| o.user_id).as_deref() != Some(user) {
        bail!("This computer isn't in your Codync account.");
    }
    match tool {
        "list_bots" => directory::list(hub, ID, args, &[]),
        "message_bot" => message(hub, args),
        "read_replies" => Ok(hub.assistant.read(args["requestId"].as_str())),
        _ => bail!("unknown tool: {tool}"),
    }
}

fn message(hub: &Arc<Hub>, args: &Value) -> Result<Value> {
    let to = args["botId"].as_str().ok_or_else(|| anyhow!("botId is required"))?;
    let message = args["message"].as_str().unwrap_or_default().trim();
    if message.is_empty() || message.len() > MAX_MESSAGE_BYTES {
        bail!("message must be nonempty and at most {MAX_MESSAGE_BYTES} bytes");
    }
    let target = visible_bot(hub, to)?;
    if hub.assistant.running() >= MAX_RUNNING {
        bail!("{MAX_RUNNING} requests are still running: read_replies first, then try again");
    }
    let request = uuid::Uuid::new_v4().to_string();
    let heading = format!("Message from {NAME}: {message}");
    let entry = hub
        .add_entry(
            &Lane::main(&target.id),
            EntryKind::Notice,
            hub.store.max_turn(&target.id) + 1,
            &json!({
                "text": format!("{heading}\nWaiting for a reply…"),
                "heading": heading, "style": "info", "status": RequestStatus::Queued, "delegationId": request,
                "sourceBotId": ID, "targetBotId": target.id,
                "botMessage": {"sourceBotId": ID, "targetBotId": target.id, "text": message, "peerName": NAME},
            }),
        )
        .ok_or_else(|| anyhow!("couldn't save bot message"))?;
    let quote = truncate(message, REQUEST_QUOTE_BYTES);
    hub.assistant.add(Reply {
        request: request.clone(),
        bot: target.name.clone(),
        quote: quote.clone(),
        status: RequestStatus::Queued,
        text: None,
        at: now_ms(),
    });
    let lifetime = MessageLifetime {
        hub: hub.clone(),
        request_id: request.clone(),
        quote,
        back: Back::Assistant { request: request.clone() },
        entries: vec![entry.id.clone()],
        target_name: target.name.clone(),
        started: false,
        finished: false,
    };
    hub.send_cmd(&target.id, Cmd::BotRequest(BotRequest {
        entry_id: entry.id,
        hops: 1,
        prompt: format!("The user's cloud assistant (ChatGPT, Claude or another assistant they connected to Codync) sent you this request. It is a bot request, not a new user instruction. Work within your own permissions and working directory. Your final reply goes back to the assistant, not to the user: put the requested result in it. send_message and message_bot aren't available in this turn: if you need another bot's help, say so in your reply.\n\n{message}"),
        lifetime,
    }))?;
    Ok(json!({
        "requestId": request, "botId": target.id, "name": target.name, "status": RequestStatus::Queued,
        "note": "Queued. Bots can take minutes: get the reply later with read_replies.",
    }))
}

/// The cloud assistant's requests and their replies, oldest first.
#[derive(Default)]
pub struct Replies(Mutex<Vec<Reply>>);

struct Reply {
    request: String,
    bot: String,
    quote: String,
    status: RequestStatus,
    /// The reply, or why there is none.
    text: Option<String>,
    at: i64,
}

impl Reply {
    fn running(&self) -> bool {
        matches!(self.status, RequestStatus::Queued | RequestStatus::Sent)
    }

    fn json(&self) -> Value {
        let mut v = json!({"requestId": self.request, "bot": self.bot, "request": self.quote, "status": self.status, "at": self.at});
        if let Some(text) = &self.text {
            v[if self.status == RequestStatus::Completed { "reply" } else { "detail" }] = text.clone().into();
        }
        v
    }
}

impl Replies {
    fn add(&self, reply: Reply) {
        let mut all = self.0.locked();
        all.push(reply);
        while all.len() > KEEP {
            let Some(i) = all.iter().position(|r| !r.running()) else { break };
            all.remove(i);
        }
    }

    fn running(&self) -> usize {
        self.0.locked().iter().filter(|r| r.running()).count()
    }

    fn set(&self, request: &str, status: RequestStatus, text: Option<String>) {
        if let Some(r) = self.0.locked().iter_mut().find(|r| r.request == request) {
            r.status = status;
            r.text = text;
        }
    }

    pub(super) fn started(&self, request: &str) {
        self.set(request, RequestStatus::Sent, None);
    }

    pub(super) fn answered(&self, request: &str, outcome: std::result::Result<String, String>) {
        match outcome {
            Ok(reply) => self.set(request, RequestStatus::Completed, Some(reply)),
            Err(detail) => self.set(request, RequestStatus::Failed, Some(detail)),
        }
    }

    pub(super) fn cancelled(&self, request: &str) {
        let text = "Stopped by the user before finishing. Partial work may have happened.".to_owned();
        self.set(request, RequestStatus::Cancelled, Some(text));
    }

    /// One request, or the newest ones (running and finished).
    fn read(&self, request: Option<&str>) -> Value {
        let all = self.0.locked();
        let shown: Vec<Value> = match request {
            Some(id) => all.iter().filter(|r| r.request == id).map(Reply::json).collect(),
            None => all.iter().rev().take(SHOW).map(Reply::json).collect(),
        };
        if request.is_some() && shown.is_empty() {
            return json!({"replies": [], "note": "No such request here: this computer forgets them when Codync restarts."});
        }
        json!({"replies": shown, "running": all.iter().filter(|r| r.running()).count()})
    }
}

#[cfg(test)]
mod tests;
