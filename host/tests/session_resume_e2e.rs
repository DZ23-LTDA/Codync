//! Reopening a session with `session/resume` instead of `session/load`: an agent that advertises only
//! `sessionCapabilities.resume` and no `loadSession` keeps its conversation across a host restart, and the host
//! never sends it a request it doesn't take.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[allow(dead_code)]
mod common;

use common::host::Host;

#[tokio::test]
async fn a_session_comes_back_with_session_resume_when_the_agent_has_no_load() {
    let mut host = Host::start().await;
    let bot = host.fixture_bot("Resumer", "instructions_agent.py", "resume").await;
    host.reply(&bot, "hello").await;
    assert_eq!(host.requests(&bot, "session/new").len(), 1);

    host.restart(|_| {}).await;
    host.reply(&bot, "again").await;

    assert_eq!(host.requests(&bot, "session/resume").len(), 1, "it reopens with the only request it advertised");
    assert!(host.requests(&bot, "session/load").is_empty(), "it was never asked for session/load");
    assert_eq!(host.requests(&bot, "session/new").len(), 1, "the same conversation carries on");
}
