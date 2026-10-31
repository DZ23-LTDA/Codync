//! The outbound side of a chat bridge: a bot's final reply, posted to the URL the user configured.
//!
//! Signed with a secret this computer keeps for the bot, so the receiver can tell a real delivery from anything else
//! on the network. Delivery is fire and forget: a receiver that is slow or down never holds a turn.

use crate::store::{Store, now_ms};
use hmac::{Hmac, Mac, digest::KeyInit};
use sha2::Sha256;
use std::fmt::Write as _;
use std::time::Duration;

/// The bot behind an inbound bridge secret, if this computer issued it.
pub fn bot_for(store: &Store, secret: &str) -> Option<String> {
    store.kv_get(&format!("bri.{secret}"))
}

/// The record that lets `bot_for` answer: called when a secret is made, so the reverse lookup never guesses.
fn remember(store: &Store, bot: &str, secret: &str) {
    if let Err(error) = store.kv_set(&format!("bri.{secret}"), bot) {
        tracing::warn!(bot = %bot, error = format!("{error:#}"), "could not keep the bridge reverse index");
    }
}

/// How long a receiver has to take a delivery before it is given up on.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The secret this computer signs `bot` deliveries with, made on first use.
pub fn secret(store: &Store, bot: &str) -> String {
    let key = format!("bridge.{bot}");
    if let Some(saved) = store.kv_get(&key) {
        return saved;
    }
    let made = uuid::Uuid::new_v4().to_string();
    if let Err(error) = store.kv_set(&key, &made) {
        tracing::warn!(bot = %bot, error = format!("{error:#}"), "could not keep the bridge secret");
    }
    remember(store, bot, &made);
    made
}

/// `HMAC-SHA256` of `body` under `secret`, hex: `sha256=<hex>`, the shape every webhook sender uses.
pub fn signature(secret: &str, body: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC takes any key length");
    mac.update(body.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in mac.finalize().into_bytes() {
        let _ = write!(hex, "{byte:02x}");
    }
    format!("sha256={hex}")
}

/// Posts `text` to `url` in the background. `bot` and the time travel with it, so the receiver can route and dedupe.
pub fn deliver(hub: &crate::hub::Hub, bot: &str, url: &str, text: &str) {
    let body = serde_json::json!({"bot": bot, "text": text, "at": now_ms()}).to_string();
    let signature = signature(&secret(&hub.store, bot), &body);
    let (bot, url) = (bot.to_owned(), url.to_owned());
    tokio::spawn(async move {
        let answered = reqwest::Client::new()
            .post(&url)
            .header("content-type", "application/json")
            .header("x-codync-signature", signature)
            .timeout(TIMEOUT)
            .body(body)
            .send()
            .await;
        match answered {
            Ok(response) if response.status().is_success() => {
                tracing::info!(bot = %bot, url = %url, "bridge delivery accepted");
            }
            Ok(response) => {
                tracing::warn!(bot = %bot, url = %url, status = %response.status(), "bridge receiver refused the delivery");
            }
            Err(error) => {
                tracing::warn!(bot = %bot, url = %url, error = format!("{error:#}"), "bridge delivery failed");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::signature;

    #[test]
    fn the_signature_follows_the_body_and_the_secret() {
        let body = "{\"bot\":\"b1\",\"text\":\"hi\"}";
        let signed = signature("secret", body);
        assert!(signed.starts_with("sha256="));
        assert_eq!(signed, signature("secret", body), "the same input signs the same way");
        assert_ne!(signed, signature("other", body), "another secret signs differently");
        assert_ne!(signed, signature("secret", "{\"text\":\"tampered\"}"), "another body signs differently");
    }
}
