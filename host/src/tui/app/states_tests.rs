//! The screen the terminal shows when there is nothing to show.
//!
//! Item 6 of the release gate asks for empty, offline and error states on every client. For the TUI those are three
//! short sentences that exist in `view/roster.rs` and, until now, had no test that renders them: the copy is only
//! true if it reaches the screen. This drives the real `view::draw` into an offscreen buffer, which is as close to a
//! terminal as a test can get without one.

use super::*;
use ratatui::{Terminal, backend::TestBackend};

/// Everything the screen says, as one string, so a test reads like someone looking at it.
fn screen(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
    terminal.draw(|f| crate::tui::view::draw(f, app)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// The screen as a person reads it: the panels wrap and draw their frames, so the text is joined back into
/// sentences with the box-drawing characters taken out.
fn readable(app: &mut App, width: u16, height: u16) -> String {
    let frames = ['│', '─', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼'];
    screen(app, width, height)
        .chars()
        .filter(|c| !frames.contains(c))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[tokio::test]
async fn an_empty_host_says_there_are_no_bots_and_what_makes_one() {
    let mut app = test_app();
    app.online = true;
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("B-3PO"), "the empty roster carries the product's name:\n{text}");
    assert!(text.contains("No bots yet."), "the empty roster should say so:\n{text}");
    assert!(text.contains("n creates one."), "and what to do about it:\n{text}");
}

#[tokio::test]
async fn a_host_that_has_not_answered_yet_says_it_is_connecting_and_shows_offline() {
    let mut app = test_app();
    app.online = false;
    let text = screen(&mut app, 100, 30);
    assert!(text.contains("Connecting…"), "a host that has not answered should say so:\n{text}");
    assert!(text.contains("offline"), "and the status line should agree:\n{text}");
    assert!(!text.contains("No bots yet."), "an unreachable host is not an empty host:\n{text}");
}

#[tokio::test]
async fn a_failed_start_offers_the_way_out_on_this_computer() {
    let mut app = test_app();
    app.online = false;
    // The hint is for the host on this computer; `test_app` leaves the URL empty, so name one.
    app.url = "http://127.0.0.1:19222".into();
    app.error = Some("Could not reach the host: connection refused".into());
    let flat = readable(&mut app, 100, 30);
    assert!(flat.contains("connection refused"), "the host's own words should be what is read:\n{flat}");
    assert!(flat.contains("Press I to install and start it here."), "and the fix, when it runs here:\n{flat}");
}

#[tokio::test]
async fn a_window_too_small_to_use_asks_for_room_instead_of_drawing_over_itself() {
    let mut app = test_app();
    let text = screen(&mut app, 30, 10);
    assert!(text.contains("Make the window bigger"), "a tiny terminal gets one useful line:\n{text}");
}

/// The screen recovers by itself when the host answers: `Connecting…` becomes the empty roster.
///
/// This is the retry half of the same state, and it is worth pinning because a client that needed a restart to
/// find a host it had already lost would look broken on every laptop that sleeps.
#[tokio::test]
async fn the_screen_recovers_when_the_host_answers() {
    let mut app = test_app();
    app.online = false;
    let before = readable(&mut app, 100, 30);
    assert!(before.contains("Connecting…"), "before the host answers:\n{before}");

    app.on_msg(Msg::Online(true));
    let after = readable(&mut app, 100, 30);
    assert!(after.contains("No bots yet."), "once it answers, the empty roster:\n{after}");
    assert!(!after.contains("Connecting…"), "and the waiting line is gone:\n{after}");
    assert!(!after.contains("offline"), "and the status line says online again:\n{after}");
}

/// A host that goes away is not left looking alive.
#[tokio::test]
async fn the_screen_says_offline_again_when_the_host_goes_away() {
    let mut app = test_app();
    app.online = true;
    assert!(readable(&mut app, 100, 30).contains("No bots yet."), "online and empty to start with");

    app.on_msg(Msg::Online(false));
    let after = readable(&mut app, 100, 30);
    assert!(after.contains("offline"), "the status line should say so:\n{after}");
    assert!(!after.contains("No bots yet."), "a host that went away is not an empty host:\n{after}");
}
