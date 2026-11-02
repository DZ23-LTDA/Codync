//! The remote channel's door, from outside: `GET /channel` is the one route that takes no bearer token, because
//! the handshake authenticates instead. That makes its front the most exposed surface the host has, so what it
//! answers to a caller who is not a phone is worth pinning down.
//!
//! What this covers: the protocol version is demanded, a plain HTTP request is not enough to get a socket, and a
//! burst of refusals does not take the host down. What it does not cover: the handshake itself (pairing code,
//! keys, replay), which the channel's own tests drive with the real cryptography.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

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
    for _ in 0..100 {
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
    assert_eq!(refused, 100, "{} of 100 unauthenticated channel requests were answered", 100 - refused);
    assert!(host.healthy().await, "the host stopped answering /health after 100 refused channel requests");
    assert!(host.api_still_works().await, "the loopback API stopped working after the burst");
}
