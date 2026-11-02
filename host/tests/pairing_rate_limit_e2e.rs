//! Pairing attempts over the real channel: three wrong codes a minute go through, the fourth is refused.
//!
//! The limiter has a unit test (`attempts_are_limited_per_source_and_in_total`), and that test hands `attempt` a
//! string. What it cannot show is that the socket path calls it, or that the refusal a person sees arrives before
//! the code is even looked at. This drives a phone through the real handshake until the host says stop.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::phone::{Frame, Phone, new_signing_key, pairing_key};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Child;

struct Host {
    child: Child,
    base: String,
    token: String,
    home: PathBuf,
}

impl Host {
    async fn start() -> Self {
        let home = std::env::temp_dir().join(format!("codync-rate-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
        Self { child, base, token, home }
    }

    /// One loopback call: what the computer itself would do, e.g. show a pairing QR code.
    async fn call(&self, method: &str, body: Value) -> Value {
        let response = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let text = response.text().await.unwrap();
        assert!(status.is_success(), "{method} answered {status}: {text}");
        serde_json::from_str(&text).unwrap()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// Three wrong codes are answered 403; the fourth is refused for the limit, not for the code.
#[tokio::test(flavor = "multi_thread")]
async fn the_fourth_wrong_pairing_code_is_refused_by_the_channel() {
    let host = Host::start().await;
    let qr = host.call("pairing", json!({})).await;
    let url = qr["pairingUrl"].as_str().unwrap().to_owned();
    let sk = pairing_key(&url);
    let wrong = json!({"code": "not-the-code", "name": "Tester", "platform": "ios"});

    // Three tries, three channels: each wrong code answers 403 and ends its own channel (4001), because the
    // pairing channel lives only long enough to redeem one code. A fresh device key per try, like a new phone.
    for attempt in 0..3 {
        let key = new_signing_key();
        let mut phone = Phone::connect(&host.base, &key, &sk, true).await;
        phone.send(json!({"id": attempt + 1, "m": "pair", "b": wrong})).await;
        let answer = phone.inner().await;
        assert_eq!(answer["err"]["status"], 403, "attempt {attempt} should have been a bad code: {answer}");
        assert!(matches!(phone.recv().await, Frame::Closed(4001)), "attempt {attempt} should have ended its channel");
    }

    // The fourth is refused before the code is looked at: it is not a 403 about the code, it is the limit.
    let key = new_signing_key();
    let mut phone = Phone::connect(&host.base, &key, &sk, true).await;
    phone.send(json!({"id": 9, "m": "pair", "b": wrong})).await;
    let answer = phone.inner().await;
    assert_eq!(answer["t"], "reject", "the fourth attempt should be refused before the code is checked: {answer}");
    // The frame carries the symbolic reason; the socket carries the number (4008). Both are the same refusal.\n    assert_eq!(answer["code"], "rateLimited", "rate limited names itself in the frame: {answer}");
    assert!(
        answer["message"].as_str().is_some_and(|m| m.contains("Too many pairing attempts")),
        "the refusal should tell the person what to do: {answer}"
    );
    assert!(matches!(phone.recv().await, Frame::Closed(4008)), "the channel closes with the rate-limit code");
}
