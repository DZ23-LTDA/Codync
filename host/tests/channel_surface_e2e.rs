//! The remote channel's door, from outside: `GET /channel` is the one route that takes no bearer token, because
//! the handshake authenticates instead. That makes its front the most exposed surface the host has, so what it
//! answers to a caller who is not a phone is worth pinning down.
//!
//! What this covers: a plain HTTP request never gets a socket, a burst of refusals does not take the host down, and
//! a frame bigger than the 1 MiB limit costs only the sender his channel. What it does not cover: the handshake
//! itself (pairing code, keys, replay), which the channel's own tests drive with the real cryptography.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use futures::{SinkExt, StreamExt};
use reqwest::StatusCode;
use std::process::Child;

struct Host {
    child: Child,
    base: String,
    token: String,
}

impl Host {
    async fn start() -> Self {
        let home = std::env::temp_dir().join(format!("codync-channel-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
        Self { child, base, token }
    }

    /// The channel URL with the given query, requested as plain HTTP (no upgrade headers).
    async fn channel_status(&self, query: &str) -> StatusCode {
        reqwest::Client::new()
            .get(format!("{}/channel{query}", self.base))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .unwrap()
            .status()
    }

    async fn healthy(&self) -> bool {
        reqwest::Client::new()
            .get(format!("{}/health", self.base))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    /// The loopback API still answers with its token after the channel was abused.
    async fn api_still_works(&self) -> bool {
        reqwest::Client::new()
            .post(format!("{}/api/history", self.base))
            .bearer_auth(&self.token)
            .json(&serde_json::json!({"botId": "none"}))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A plain HTTP request never gets a socket, whatever version it claims.
///
/// The version demand (`426 upgradeRequired`) only answers *after* a successful WebSocket upgrade, because the
/// upgrade itself is an extractor: a caller without the upgrade headers is refused by that first, with 400. So
/// from plain HTTP the only thing to check is that no version of the request opens the channel -- the version
/// error itself belongs to the handshake tests, which have a socket.
#[tokio::test(flavor = "multi_thread")]
async fn a_plain_request_never_gets_a_socket() {
    let host = Host::start().await;

    for query in ["", "?v=1", "?v=2", "?v=0", "?v=1&junk=1"] {
        let status = host.channel_status(query).await;
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UPGRADE_REQUIRED,
            "a plain GET {query} answered {status}, which is neither a refusal nor an upgrade demand"
        );
        assert_ne!(status, StatusCode::OK, "a plain GET {query} opened the channel");
    }

    // None of that touched the host.
    assert!(host.healthy().await, "the host stopped answering /health after the channel was probed");
    assert!(host.api_still_works().await, "the loopback API stopped working after the channel was probed");
}

/// A caller who is not a phone can hammer the door; the host is still the host afterwards.
#[tokio::test(flavor = "multi_thread")]
async fn a_burst_against_the_channel_does_not_take_the_host_down() {
    let host = Host::start().await;
    let client = reqwest::Client::new();
    let mut refused = 0;
    // Twenty-five, not a hundred: the property is "refusals do not take the host down", and every request costs a
    // fresh connection on a cold CI runner -- a hundred of them turned a fast test into a slow one for nothing.
    for _ in 0..25 {
        let status = client
            .get(format!("{}/channel?v=1", host.base))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .unwrap()
            .status();
        if !status.is_success() {
            refused += 1;
        }
    }
    assert_eq!(refused, 25, "{} of 25 unauthenticated channel requests were answered", 25 - refused);
    assert!(host.healthy().await, "the host stopped answering /health after 25 refused channel requests");
    assert!(host.api_still_works().await, "the loopback API stopped working after the burst");
}

/// A frame bigger than the host accepts closes that one connection, and only that one.
///
/// The upgrade sets `max_message_size` to `MAX_WS_MESSAGE` (1 MiB). A caller that ignores it must lose its own
/// socket -- not the host, and not anyone else's channel.
#[tokio::test(flavor = "multi_thread")]
async fn a_frame_over_the_limit_costs_only_the_sender_his_channel() {
    let host = Host::start().await;
    let url = format!("{}/channel?v=1", host.base.replace("http://", "ws://"));
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();

    // 2 MiB of text: twice what the upgrade allows, in one message so it cannot be split into legal frames.
    let oversized = "x".repeat(2 << 20);
    let sent = ws.send(tokio_tungstenite::tungstenite::Message::text(oversized)).await;

    // Either the send itself fails, or the host closes on us: a silent acceptance would be the bug.
    let closed = if sent.is_err() {
        true
    } else {
        match tokio::time::timeout(std::time::Duration::from_secs(15), ws.next()).await {
            // The stream ended, the transport broke, or the host closed: all refusals.
            Ok(None | Some(Err(_) | Ok(tokio_tungstenite::tungstenite::Message::Close(_)))) => true,
            // No answer at all is a hang, not a refusal; any other frame is an answer that should have been one. A
            // timeout counted as "closed" would have made this test pass against a host that simply hung.
            Err(_) | Ok(Some(Ok(_))) => false,
        }
    };
    assert!(closed, "the host accepted a 2 MiB frame after setting a 1 MiB limit");
    drop(ws);

    // The host, and everyone else, carries on.
    assert!(host.healthy().await, "the host stopped answering /health after an oversized frame");
    assert!(host.api_still_works().await, "the loopback API stopped working after an oversized frame");
}
