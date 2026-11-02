//! A phone talking to the host over the direct E2E channel, for the tests that need the real handshake.
//!
//! Moved out of `e2e.rs` so a second test file can drive it without a copy: the handshake is 80 lines of
//! cryptography that should exist once.

#![allow(dead_code)]

/// The host's wire crypto doubles as this client's device implementation.
#[path = "../../src/remote/crypto.rs"]
pub mod crypto;

use ed25519_dalek::SigningKey;
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::Duration;

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// What came back over the channel: an opened frame, or the close code.
pub enum Frame {
    Inner(Value),
    Closed(u16),
}

pub struct Phone {
    ws: Ws,
    sealer: crypto::Sealer,
    opener: crypto::Opener,
}

impl Phone {
    /// Connects and completes the handshake, verifying the host against the pinned `sk`.
    pub async fn connect(base: &str, key: &SigningKey, sk: &[u8; 32], pair: bool) -> Self {
        let url = format!("{}/channel?v=1", base.replace("http://", "ws://"));
        let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        let cid = crypto::computer_id_raw(sk);
        let dk = key.verifying_key().to_bytes();
        let ek = x25519_dalek::StaticSecret::from(crypto::random::<32>());
        let ek_pub = crypto::x25519_pub(&ek);
        let n = crypto::random::<32>();
        let sig = crypto::sign(key, &crypto::hs1_input(&cid, &dk, &ek_pub, &n));
        let hello = json!({"t": "hello", "v": 1, "dk": crypto::b64(&dk), "ek": crypto::b64(&ek_pub),
            "n": crypto::b64(&n), "sig": crypto::b64(&sig), "pair": pair});
        ws.send(hello.to_string().into()).await.unwrap();
        let welcome: Value = serde_json::from_str(ws.next().await.unwrap().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(welcome["t"], "welcome", "{welcome}");
        let ek_h: [u8; 32] = crypto::unb64_n(welcome["ek"].as_str().unwrap()).unwrap();
        let th = crypto::transcript_hash(&cid, &dk, &ek_pub, &n, &ek_h);
        crypto::verify(sk, &th, &crypto::unb64_n(welcome["sig"].as_str().unwrap()).unwrap()).unwrap();
        let keys = crypto::channel_keys(&crypto::x25519(&ek, &ek_h).unwrap(), &th);
        Self { ws, sealer: crypto::Sealer::new(keys.d2h), opener: crypto::Opener::new(keys.h2d, 16 << 20) }
    }

    pub async fn send(&mut self, v: Value) {
        for f in self.sealer.seal(v.to_string().as_bytes()).unwrap() {
            self.ws.send(f.to_string().into()).await.unwrap();
        }
    }

    pub async fn recv(&mut self) -> Frame {
        use tokio_tungstenite::tungstenite::Message;
        loop {
            let msg = tokio::time::timeout(Duration::from_secs(20), self.ws.next()).await.expect("host answers");
            match msg.unwrap().unwrap() {
                Message::Text(t) => {
                    let f: Value = serde_json::from_str(t.as_str()).unwrap();
                    if let Some(bytes) = self.opener.open(&f).ok().flatten() {
                        return Frame::Inner(serde_json::from_slice(&bytes).unwrap());
                    }
                    // Not sealed: the host answers some refusals (`reject`, before any pairing) in the open, so
                    // the person -- or the client -- can read why without keys. Anything else that will not
                    // open is a frame this client cannot use and is skipped.
                    if f.get("t").is_some() {
                        return Frame::Inner(f);
                    }
                }
                Message::Close(c) => return Frame::Closed(c.map_or(1005, |c| c.code.into())),
                _ => {}
            }
        }
    }

    pub async fn inner(&mut self) -> Value {
        match self.recv().await {
            Frame::Inner(v) => v,
            Frame::Closed(code) => panic!("closed with {code}"),
        }
    }

    pub async fn call(&mut self, id: u64, m: &str, b: Value) -> Value {
        self.send(json!({"id": id, "m": m, "b": b})).await;
        loop {
            let v = self.inner().await;
            if v["id"] == id && v.get("ev").is_none() {
                assert!(v["ok"].is_object(), "{m}: {v}");
                return v["ok"].clone();
            }
        }
    }
}

/// One query parameter of a pairing URL (`...?sk=..&code=..`).
pub fn query_param(url: &str, name: &str) -> String {
    url.split(['?', '&']).find_map(|p| p.strip_prefix(&format!("{name}="))).unwrap().to_owned()
}

/// The host's `sk` from a pairing URL, as the 32 bytes the handshake signs with.
pub fn pairing_key(url: &str) -> [u8; 32] {
    let raw = query_param(url, "sk");
    crypto::unb64_n(&raw).expect("the pairing URL carries a 32-byte sk")
}

/// A fresh key for this side of the channel.
pub fn new_signing_key() -> SigningKey {
    SigningKey::from_bytes(&crypto::random::<32>())
}
