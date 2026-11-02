//! What the terminal client's event stream does when the host goes away and comes back.
//!
//! The command path is proven next door. This is the other half: the stream reads `rev` from the client, asks for
//! `since=<rev>` on every reconnect, and never skips an event it could not read. A duplicate or a gap here is what a
//! person sees as a message that appears twice or never.

use super::reconnect_tests::{host_binary, serve_on, wait_ready};
use super::*;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Messages the stream sent, in the order they arrived.
type Seen = Arc<Mutex<Vec<Msg>>>;

/// Reads the stream's messages into `seen` until the test drops the reader.
fn collect(mut rx: tokio::sync::mpsc::UnboundedReceiver<Msg>) -> Seen {
    let seen: Seen = Arc::default();
    let sink = seen.clone();
    tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            if let Ok(mut v) = sink.lock() {
                v.push(m);
            }
        }
    });
    seen
}

/// Waits for a message the caller is looking for, returning the index it was found at.
async fn wait_for(seen: &Seen, what: &str, f: impl Fn(&Msg) -> bool) -> usize {
    wait_for_from(seen, 0, what, f).await
}

/// The same, but only looking at what arrived after `start` -- the first connection is not the reconnect.
async fn wait_for_from(seen: &Seen, start: usize, what: &str, f: impl Fn(&Msg) -> bool) -> usize {
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        if let Ok(v) = seen.lock()
            && let Some(i) = v.iter().enumerate().skip(start).find(|(_, m)| f(m)).map(|(i, _)| i)
        {
            return i;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the stream never said {what}");
}

/// The highest `rev` in an event, or `None` for anything that is not one.
fn rev_of(m: &Msg) -> Option<i64> {
    match m {
        Msg::Event(v) => v["rev"].as_i64(),
        _ => None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_stream_resumes_from_the_last_rev_without_losing_or_repeating() {
    let Some(bin) = host_binary() else {
        eprintln!("skipping: this run has no codync-host binary; `cargo build --bin codync-host` first");
        return;
    };
    let home = std::env::temp_dir().join(format!("codync-stream-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let base = format!("http://127.0.0.1:{port}");
    let mut host = serve_on(&bin, &home, port);
    wait_ready(&base).await;
    let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
    let client = Client::new(&base, Some(&token));

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    client.spawn_stream(tx);
    let seen = collect(rx);

    // One bot, so there is an event with a rev to resume from.
    client.call("createBot", &json!({"name": "Before", "backend": "claude"})).await.expect("a bot");
    wait_for(&seen, "the first bot's event", |m| rev_of(m).is_some()).await;
    // One mutation sends a few events (the bot, its status); let them land and take the highest rev, which is what
    // the stream will have stored and therefore what it asks for on the way back.
    tokio::time::sleep(Duration::from_secs(2)).await;
    let rev_before = seen.lock().unwrap().iter().filter_map(rev_of).max().expect("at least one event to resume from");

    // The host goes away with the stream open.
    host.kill().unwrap();
    host.wait().unwrap();
    let offline = wait_for(&seen, "offline", |m| matches!(m, Msg::Online(false))).await;

    // And comes back on the same port; the stream is already asking to resume.
    host = serve_on(&bin, &home, port);
    wait_ready(&base).await;

    // The stream says which rev it resumed from: the one it had already seen, not zero.
    let resumed = wait_for_from(&seen, offline + 1, "a reconnect", |m| matches!(m, Msg::Connected { .. })).await;
    let since = match &seen.lock().unwrap()[resumed] {
        Msg::Connected { since } => *since,
        _ => unreachable!(),
    };
    assert_eq!(since, rev_before, "the stream should ask for what it has not seen, not for everything");

    // A new bot after the reconnect arrives, and nothing from before it arrives again.
    client.call("createBot", &json!({"name": "After", "backend": "claude"})).await.expect("a bot");
    wait_for(&seen, "the new bot's event", |m| rev_of(m).is_some_and(|r| r > rev_before)).await;

    // The contract is "everything after `since`", and the host also re-sends the current state of what it holds
    // (`events_stream`: "Duplicates are fine: clients upsert by id/rev"). So the boundary rev may arrive twice; what
    // must never arrive is something *older* than what this client had already seen.
    let messages = seen.lock().unwrap();
    let after_resume: Vec<i64> = messages[resumed..].iter().filter_map(rev_of).collect();
    assert!(
        after_resume.iter().all(|&r| r >= rev_before),
        "an event older than rev {rev_before} came again after the reconnect: {after_resume:?}"
    );

    let _ = host.kill();
    let _ = host.wait();
    let _ = std::fs::remove_dir_all(&home);
}
