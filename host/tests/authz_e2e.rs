//! The loopback API's front door: every route that touches the computer refuses a caller without the token.
//!
//! The unit tests check the pieces (the token comparison, the loopback rule). This asks the running host, the way
//! anything else on the machine would: no `Authorization` header, a wrong one, and the right one. A route that
//! answers before checking the token is a hole no unit test would notice, because the unit test hands it a caller.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use reqwest::StatusCode;
use serde_json::{Value, json};
use std::process::Child;

/// The routes that can act on the computer. Every one of these exists: the test fails on an unknown method.
const ROUTES: [&str; 6] = ["send", "history", "createBot", "setScreenEnabled", "screenOffer", "setVoiceKey"];

struct Host {
    child: Child,
    base: String,
    token: String,
}

impl Host {
    async fn start() -> Self {
        let home = std::env::temp_dir().join(format!("codync-authz-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&home).unwrap();
        let (child, base) = common::serve(&home).await;
        let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
        Self { child, base, token }
    }

    /// One API call with the given header value: the status, and the body when it was JSON.
    async fn call(&self, method: &str, header: Option<&str>) -> (StatusCode, Option<Value>) {
        let mut request = reqwest::Client::new()
            .post(format!("{}/api/{method}", self.base))
            .json(&json!({"botId": "nobody", "text": "hello"}));
        if let Some(value) = header {
            request = request.header("authorization", value);
        }
        let response = request.send().await.unwrap();
        let status = response.status();
        let body = response.json::<Value>().await.ok();
        (status, body)
    }

    /// The status only, for the cases where the body is not the point.
    async fn status(&self, method: &str, header: Option<&str>) -> StatusCode {
        self.call(method, header).await.0
    }

    async fn events_status(&self, header: Option<&str>) -> StatusCode {
        let mut request = reqwest::Client::new().get(format!("{}/events?since=0&client=authz-test", self.base));
        if let Some(value) = header {
            request = request.header("authorization", value);
        }
        // The stream would never end; only the status line matters here.
        let response = request.send().await.unwrap();
        response.status()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// No token, a foreign token, and the real one, against the routes that can act on the computer.
#[tokio::test(flavor = "multi_thread")]
async fn every_loopback_route_wants_the_token() {
    let host = Host::start().await;
    let wrong = format!("Bearer {}", "0".repeat(43));
    let right = format!("Bearer {}", host.token);

    // Routes a local helper could reach: they read or act on the computer.
    for method in ROUTES {
        assert_eq!(host.status(method, None).await, StatusCode::UNAUTHORIZED, "{method} answered without a token");
        assert_eq!(
            host.status(method, Some(&wrong)).await,
            StatusCode::UNAUTHORIZED,
            "{method} answered with a foreign token"
        );
    }

    // The event stream is the same door: it carries every conversation on this computer.
    assert_eq!(host.events_status(None).await, StatusCode::UNAUTHORIZED, "/events answered without a token");
    assert_eq!(
        host.events_status(Some(&wrong)).await,
        StatusCode::UNAUTHORIZED,
        "/events answered with a foreign token"
    );

    // And the real token is not refused: the gate is the header, not the route. The body above is nonsense
    // (`botId: "nobody"`), so a 4xx is the right answer for most of them; what would be wrong is 401 or a crash.
    //
    // The envelope matters as much as the status: an unknown method answers 404 with `{"error":"unknown method"}`,
    // so a list naming a method this host never had would pass a status-only check. That is not hypothetical --
    // this test used to name `vaultGet`, which does not exist, and it passed.
    for method in ROUTES {
        let (status, body) = host.call(method, Some(&right)).await;
        assert_ne!(status, StatusCode::UNAUTHORIZED, "{method} refused the real token");
        assert!(!status.is_server_error(), "{method} failed with the real token: {status}");
        let body = body.unwrap_or_else(|| panic!("{method} answered {status} with something that is not JSON"));
        assert_ne!(body["error"], "unknown method", "{method} is not a method this host has");
    }

    let events = host.events_status(Some(&right)).await;
    assert_ne!(events, StatusCode::UNAUTHORIZED, "/events refused the real token");
    assert!(!events.is_server_error(), "/events failed with the real token: {events}");
}
