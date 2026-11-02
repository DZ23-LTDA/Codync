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
