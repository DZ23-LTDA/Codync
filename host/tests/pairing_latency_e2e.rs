//! How long the computer takes to show a pairing code.
//!
//! This is a number the first-run script needs and nobody had: between "pair a phone" and the QR on screen. The host
//! gathers its addresses (which may shell out to `tailscale`), issues the code and builds the QR payload, so the cost is
//! real and can regress without anything else noticing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use serde_json::json;
use std::path::PathBuf;
use std::process::Child;
use std::time::Instant;

struct Host {
    child: Child,
    home: PathBuf,
    base: String,
    token: String,
}

impl Drop for Host {
    fn drop(&mut self) {
        common::stop(&mut self.child);
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

impl Host {
    async fn start() -> Self {
        let home = std::env::temp_dir().join(format!("codync-pairing-e2e-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let mut host = Self { child, home, base, token: String::new() };
        std::fs::read_to_string(host.home.join("token")).unwrap().trim().clone_into(&mut host.token);
        host
    }

    async fn call(&self, method: &str, body: serde_json::Value) -> serde_json::Value {
        let response = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let value: serde_json::Value = response.json().await.unwrap();
        assert!(status.is_success(), "{method} failed: {value}");
        value
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_pairing_code_is_shown_and_the_wait_is_measured() {
    let host = Host::start().await;

    let mut samples = Vec::new();
    let mut last = serde_json::Value::Null;
    for _ in 0..10 {
        let started = Instant::now();
        last = host.call("pairing", json!({})).await;
        samples.push(started.elapsed().as_millis());
    }

    // The QR payload exists and carries a code plus at least one address a phone could use: this is what the screen shows.
    let url = last["pairingUrl"].as_str().unwrap_or_default();
    assert!(url.starts_with("codync"), "no pairing url in {last}");
    assert!(url.contains("code="), "the pairing url carries no code: {last}");
    assert!(last["urls"].as_array().is_some_and(|urls| !urls.is_empty()), "no address a phone could reach: {last}");
    assert!(last["expiresAt"].as_i64().is_some_and(|at| at > 0), "no expiry in {last}");

    samples.sort_unstable();
    let fastest = samples[0];
    let slowest = samples[samples.len() - 1];
    println!("pairing code (ms), 10 rounds: {samples:?} · fastest {fastest} · slowest {slowest}");

    // Showing a QR is something a person waits for with the phone in hand, and no call may wait for the background
    // Tailscale read: the whole point is that it never blocks, even before it lands.
    assert!(slowest < 500, "showing the pairing code took {slowest} ms: {samples:?}");
}
