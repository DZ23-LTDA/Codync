//! What the terminal client does when the host goes away and comes back: a laptop that sleeps.
//!
//! `call` retries a refused connection for twenty seconds. That behaviour is what makes a slept laptop resume
//! without the person restarting anything, and nothing outside the client proved it — the screen tests drive the
//! state machine with messages, not with a socket. This kills a real host, brings it back on the same port, and asks
//! the same client to do something while that happens.

use super::*;
use serde_json::json;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The host binary next to this test's own, when it has been built (`cargo build --bin codync-host`).
///
/// `cargo test --bin` builds the test binary and not this one, so a run that never built the host cannot drive this
/// test; it says so and returns instead of failing on an absent file. A workspace run (`cargo nextest run
/// --workspace`, which is what CI does) builds every binary, so there it does run.
pub(super) fn host_binary() -> Option<PathBuf> {
    let mut dir = std::env::current_exe().expect("this test binary");
    dir.pop(); // the test binary lives in `deps/`
    if dir.ends_with("deps") {
        dir.pop();
    }
    let name = if cfg!(windows) { "codync-host.exe" } else { "codync-host" };
    let path = dir.join(name);
    path.is_file().then_some(path)
}

/// A host on `port` for `home`: the same start `common::serve` does, but with the port chosen here so a restart can
/// use the same one.
pub(super) fn serve_on(bin: &Path, home: &Path, port: u16) -> Child {
    Command::new(bin)
        .args(["serve", "--bind", "127.0.0.1", "--port", &port.to_string()])
        .env("CODYNC_HOME", home)
        .env("CODYNC_CLOUD", "off")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start the host")
}

pub(super) async fn wait_ready(base: &str) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if crate::http().get(format!("{base}/health")).send().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the host never answered at {base}");
}

/// A call that reads what is on this computer and changes nothing.
const READ: &str = "devices";

#[tokio::test(flavor = "multi_thread")]
async fn the_client_reaches_the_host_that_comes_back() {
    let Some(bin) = host_binary() else {
        eprintln!("skipping: this run has no codync-host binary; `cargo build --bin codync-host` first");
        return;
    };
    let home = std::env::temp_dir().join(format!("codync-reconnect-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&home).unwrap();
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let base = format!("http://127.0.0.1:{port}");
    let mut host = serve_on(&bin, &home, port);
    wait_ready(&base).await;
    let token = std::fs::read_to_string(home.join("token")).unwrap().trim().to_owned();
    let client = Client::new(&base, Some(&token));

    let first = client.call(READ, &json!({})).await.expect("the host answers while it is up");
    assert!(first["devices"].is_array(), "a host with nothing paired answers with no devices: {first}");

    // The host goes away, as it does when the laptop sleeps or the process is upgraded.
    host.kill().unwrap();
    host.wait().unwrap();

    // And comes back on the same port two seconds later, while the client is already waiting for it.
    let home_for_restart = home.clone();
    let bin_for_restart = bin.clone();
    let restart = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let child = serve_on(&bin_for_restart, &home_for_restart, port);
        wait_ready(&format!("http://127.0.0.1:{port}")).await;
        child
    });

    // No new client, no restart of the app: the same call, begun while the host was down, ends in an answer.
    let after = tokio::time::timeout(Duration::from_secs(40), client.call(READ, &json!({})))
        .await
        .expect("the call should not outlast the client's patience by much")
        .expect("the same client reaches the host that came back");
    assert!(after["devices"].is_array(), "and gets a real answer: {after}");

    let mut host = restart.await.unwrap();
    let _ = host.kill();
    let _ = host.wait();
    let _ = std::fs::remove_dir_all(&home);
}
