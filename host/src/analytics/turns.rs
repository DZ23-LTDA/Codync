//! `turn_finished`: how each agent turn ended, never its text or the error's.

use super::{Event, capture};
use crate::hub::Hub;
use serde_json::json;

/// How an agent's turn ended (`turn_finished`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnOutcome {
    /// `end_turn`: the agent finished.
    Done,
    /// The user stopped it.
    Stopped,
    /// The agent process or connection failed.
    Failed,
    /// It ended short of `end_turn` (output or step limit, refusal).
    Incomplete,
}

/// What started the turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnSource {
    User,
    Routine,
    Group,
    Bot,
}

/// `turn_finished`: the agent, how the turn ended, why it failed and how long it ran;
/// never its text or the error's.
pub fn turn_finished(
    hub: &Hub,
    agent: &str,
    outcome: TurnOutcome,
    stop_reason: &str,
    failure: Option<&anyhow::Error>,
    source: TurnSource,
    ms: i64,
) {
    let outcome = match outcome {
        TurnOutcome::Done => "done",
        TurnOutcome::Stopped => "stopped",
        TurnOutcome::Failed => "failed",
        TurnOutcome::Incomplete => "incomplete",
    };
    // ACP's stop reasons plus ours; anything else an adapter invents is just `other`.
    let stop_reason = match stop_reason {
        "end_turn" | "cancelled" | "max_tokens" | "max_turn_requests" | "refusal" | "error" | "start_failed" => {
            stop_reason
        }
        _ => "other",
    };
    let source = match source {
        TurnSource::User => "user",
        TurnSource::Routine => "routine",
        TurnSource::Group => "group",
        TurnSource::Bot => "bot",
    };
    let mut p = json!({"agent": agent, "outcome": outcome, "stop_reason": stop_reason, "source": source, "seconds": ms.max(0) / 1000});
    if let Some(e) = failure {
        let (kind, rpc) = failure_kind(e);
        p["error"] = kind.into();
        if let Some(rpc) = rpc {
            p["rpc_code"] = rpc.code.into();
            if let Some(reason) = &rpc.kind {
                p["agent_reason"] = reason.as_str().into();
            }
        }
    }
    capture(hub, Event::TurnFinished, p.as_object().cloned().unwrap_or_default());
}

/// Why a turn failed, as a category (plus the agent's JSON-RPC error, for its code and own
/// category): never the error's text, which can hold paths and agent output.
fn failure_kind(e: &anyhow::Error) -> (&'static str, Option<&crate::agent::acp::RpcError>) {
    use crate::agent::acp::{Failure, FailureKind, RpcError};
    if let Some(f) = e.chain().find_map(|c| c.downcast_ref::<Failure>()) {
        let kind = match f.kind {
            FailureKind::NotSetUp => "not_installed",
            FailureKind::Timeout => "timeout",
            FailureKind::Exited => "exited",
        };
        return (kind, None);
    }
    if crate::agent::acp::needs_sign_in(e) {
        return ("sign_in", None);
    }
    for cause in e.chain() {
        if let Some(r) = cause.downcast_ref::<RpcError>() {
            return ("agent_error", Some(r));
        }
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            return (if io.kind() == std::io::ErrorKind::NotFound { "not_installed" } else { "io" }, None);
        }
    }
    ("other", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_turn_names_its_kind_not_its_text() {
        use crate::agent::acp::{Failure, FailureKind, RpcError};
        use anyhow::Context as _;
        let code = |e: &anyhow::Error| {
            let (kind, rpc) = failure_kind(e);
            (kind, rpc.map(|r| r.code))
        };
        let setup = Failure::error(FailureKind::NotSetUp, "Codex isn't set up on this computer.");
        assert_eq!(code(&setup), ("not_installed", None));
        let hung = Err::<(), _>(Failure::error(FailureKind::Timeout, "no answer")).context("starting");
        assert_eq!(code(&hung.expect_err("failed")), ("timeout", None));
        let missing = std::io::Error::new(std::io::ErrorKind::NotFound, "/Users/me/bin/agent");
        let spawn = Err::<(), _>(missing).context("starting `/Users/me/bin/agent`");
        assert_eq!(code(&spawn.expect_err("failed")), ("not_installed", None));
        let auth = anyhow::Error::new(RpcError { code: -32000, message: "Authentication required".into(), kind: None });
        assert_eq!(code(&auth), ("sign_in", None));
        let rpc =
            anyhow::Error::new(RpcError { code: -32603, message: "Internal error: /Users/me".into(), kind: None });
        assert_eq!(code(&rpc), ("agent_error", Some(-32603)));
        let limited =
            anyhow::Error::new(RpcError { code: -32603, message: "x".into(), kind: Some("rate_limit".into()) });
        assert_eq!(failure_kind(&limited).1.and_then(|r| r.kind.as_deref()), Some("rate_limit"));
        assert_eq!(code(&anyhow::anyhow!("something else")), ("other", None));
    }
}
