//! The network sheet: this computer's network of the user's computers.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use serde_json::Value;

use super::super::l10n::tr;
use super::super::manage::{CONNECTORS, NetworkSheet, Setup, SetupField, Step};
use super::super::md;
use super::buffer::{centered, frame_box, hline, put, put_line, restyle, rput};
use super::overlay::input_line;
use super::style::theme;
use super::time::clock;

pub(super) fn network(buf: &mut Buffer, area: Rect, n: &NetworkSheet) {
    let t = theme();
    let r = centered(area, 84, area.height.saturating_sub(4).min(26));
    let inner = frame_box(buf, r, t.text, t.panel, Some((tr("Network").as_ref(), t.text)));
    if inner.height < 8 {
        return;
    }
    hline(buf, inner.x, inner.bottom() - 2, inner.width, t.dim.patch(t.panel));
    let body = Rect::new(inner.x, inner.y, inner.width, inner.height - 2);
    let mut y = body.y;
    let keys = match &n.step {
        Step::List => list(buf, body, n),
        Step::Create(s) => setup(buf, body, s, false),
        Step::Join(s) => setup(buf, body, s, true),
        Step::Recovery(code) => {
            let name = n.view.as_ref().and_then(|v| v["computer"].as_str()).unwrap_or("this computer");
            line(buf, body, &mut y, &tr("Save your recovery code"), t.bold);
            y += 1;
            line(buf, body, &mut y, code, t.bold);
            y += 1;
            let note = tr(
                "It makes another computer the owner if {name} is lost. It's shown only once and kept on no computer.",
            );
            wrapped(buf, body, &mut y, &note.replace("{name}", name), t.secondary);
            "y copy · s I saved it"
        }
        Step::Invite(invite) => {
            line(buf, body, &mut y, &tr("Invite a computer"), t.bold);
            y += 1;
            match invite {
                None => line(buf, body, &mut y, &tr("Getting an invite…"), t.dim),
                Some((code, until)) => {
                    wrapped(buf, body, &mut y, code, t.bold);
                    y += 1;
                    let name = n.view.as_ref().and_then(|v| v["computer"].as_str()).unwrap_or("this computer");
                    let note = tr("One use. Works until {time} while {name} stays on.")
                        .replace("{time}", &clock(*until))
                        .replace("{name}", name);
                    line(buf, body, &mut y, &note, t.secondary);
                    line(
                        buf,
                        body,
                        &mut y,
                        &tr("On the other computer, choose Join with invite under Network."),
                        t.secondary,
                    );
                }
            }
            "y copy · n new invite · esc back"
        }
        Step::Recover(code) => {
            line(buf, body, &mut y, &tr("Recover ownership"), t.bold);
            let note = tr("For when the owner is lost: this computer becomes the owner.");
            wrapped(buf, body, &mut y, &note, t.secondary);
            y += 1;
            input_line(buf, Rect::new(body.x, y, body.width, 1), y, code, "Recovery code", true, false);
            "↵ recover · esc back"
        }
        Step::Remove { computer, cascade } => {
            line(buf, body, &mut y, &tr("Remove {name} from the network?").replace("{name}", computer), t.red);
            line(buf, body, &mut y, &tr("It can't come back with its current key."), t.text);
            y += 1;
            let mark = if *cascade { "◉" } else { "○" };
            line(buf, body, &mut y, &format!("{mark} {}", tr("Also remove the computers it invited")), t.text);
            line(buf, body, &mut y, &format!("  {}", tr("For a stolen computer.")), t.secondary);
            "↵ remove · c switches · esc cancel"
        }
    };
    if let Some(e) = &n.error {
        put(buf, inner.x, inner.bottom() - 3, inner.width, e, t.red.patch(t.panel));
    } else if n.busy {
        put(buf, inner.x, inner.bottom() - 3, inner.width, &tr("Working…"), t.dim.patch(t.panel));
    }
    put(buf, inner.x, inner.bottom() - 1, inner.width, &format!(" {}", tr(keys)), t.dim.patch(t.panel));
}

/// The computers, or how to start; then the cloud assistants switch. Returns the key hints.
fn list(buf: &mut Buffer, body: Rect, n: &NetworkSheet) -> &'static str {
    let t = theme();
    let mut y = body.y;
    let Some(view) = &n.view else {
        put(buf, body.x, y, body.width, &tr("Loading…"), t.dim.patch(t.panel));
        return "esc close";
    };
    if n.joined() {
        let me = view["computer"].as_str().unwrap_or_default();
        let role = role_name(n.role());
        let connector = view["connector"].as_str().unwrap_or("public");
        put(buf, body.x, y, body.width, &format!("{me} · {role} · {connector}"), t.secondary.patch(t.panel));
        y += 2;
        let room = usize::from(body.height).saturating_sub(7).max(1);
        let off = n.cursor.saturating_sub(room.saturating_sub(1));
        for (i, c) in n.computers().iter().enumerate().skip(off).take(room) {
            computer_row(buf, body, y, c, i == n.cursor);
            y += 1;
        }
    } else {
        put(buf, body.x, y, body.width, &tr("Not in a network."), t.bold.patch(t.panel));
        y += 1;
        let note = tr("Connect your computers so bots on one can message bots on another.");
        wrapped(buf, body, &mut y, &note, t.secondary);
    }
    y += 1;
    let on = view["assistants"] == true;
    put(buf, body.x, y, body.width, &tr("Cloud assistants"), t.text.patch(t.panel));
    let x = rput(buf, body.right(), y, "  a switches", t.dim.patch(t.panel));
    rput(buf, x, y, if on { "◉ On" } else { "○ Off" }, if on { t.text } else { t.secondary }.patch(t.panel));
    y += 1;
    let about = tr(
        "ChatGPT or Claude in your Codync account can message this computer's bots. Codync's cloud can read these calls.",
    );
    wrapped(buf, body, &mut y, &about, t.secondary);
    match (n.joined(), n.role()) {
        (false, _) => "c create network · J join with invite · esc close",
        (true, "owner") => "i invite · p make admin · x remove · r refresh · esc close",
        (true, "admin") => "i invite · x remove · o recover ownership · r refresh · esc close",
        (true, _) => "o recover ownership · r refresh · esc close",
    }
}

fn computer_row(buf: &mut Buffer, body: Rect, y: u16, c: &Value, sel: bool) {
    let t = theme();
    let base = if sel { t.sel } else { t.panel };
    if sel {
        restyle(buf, Rect::new(body.x - 1, y, body.width + 2, 1), t.sel);
    }
    let name = c["name"].as_str().unwrap_or_default();
    let state = if c["me"] == true {
        tr("This computer")
    } else if c["online"] == true {
        format!("● {}", tr("Online")).into()
    } else {
        format!("○ {}", tr("Away")).into()
    };
    let bots: Vec<&str> = c["agents"].as_array().into_iter().flatten().filter_map(|a| a["name"].as_str()).collect();
    let offered = if bots.is_empty() { tr("No bots").into_owned() } else { bots.join(", ") };
    put(buf, body.x, y, 2, if sel { "▸" } else { " " }, t.secondary.patch(base));
    put(buf, body.x + 2, y, 20, name, t.bold.patch(base));
    put(buf, body.x + 22, y, 8, &role_name(c["role"].as_str().unwrap_or_default()), t.text.patch(base));
    put(buf, body.x + 30, y, 16, &state, t.secondary.patch(base));
    put(buf, body.x + 46, y, body.width.saturating_sub(46), &md::truncate(&offered, 60), t.secondary.patch(base));
}

fn role_name(role: &str) -> String {
    tr(match role {
        "owner" => "Owner",
        "admin" => "Admin",
        _ => "Member",
    })
    .into_owned()
}

fn setup(buf: &mut Buffer, body: Rect, s: &Setup, join: bool) -> &'static str {
    let t = theme();
    let mut y = body.y;
    let title = if join { "Join with invite" } else { "Create network" };
    put(buf, body.x, y, body.width, &tr(title), t.bold.patch(t.panel));
    y += 2;
    for field in s.fields(join) {
        let active = field == s.field;
        let label = match field {
            SetupField::Invite => "Invite",
            SetupField::Name => "Computer name",
            SetupField::Connector => "Connector",
            SetupField::Relay => "Relay address",
        };
        let style = if active { t.bold } else { t.secondary };
        put(buf, body.x, y, 18, &tr(label), style.patch(t.panel));
        let r = Rect::new(body.x + 18, y, body.width.saturating_sub(18), 1);
        match field {
            SetupField::Invite => {
                input_line(buf, r, y, &s.invite, "Paste the invite from the other computer.", active, false);
            }
            SetupField::Name => input_line(buf, r, y, &s.name, "Host name", active, false),
            SetupField::Relay => input_line(buf, r, y, &s.relay, "https://relay.example.com", active, false),
            SetupField::Connector => {
                let shown = format!("‹ {} ›", tr(CONNECTORS[s.connector].0));
                put(buf, r.x, y, r.width, &shown, if active { t.bold } else { t.text }.patch(t.panel));
            }
        }
        y += 1;
    }
    y += 1;
    wrapped(buf, body, &mut y, &tr(CONNECTORS[s.connector].2), t.secondary);
    "↵ done · tab next field · ←→ connector · esc back"
}

fn line(buf: &mut Buffer, body: Rect, y: &mut u16, text: &str, style: Style) {
    if *y < body.bottom() {
        put(buf, body.x, *y, body.width, text, style.patch(theme().panel));
        *y += 1;
    }
}

fn wrapped(buf: &mut Buffer, body: Rect, y: &mut u16, text: &str, style: Style) {
    let t = theme();
    let width = usize::from(body.width);
    for l in md::wrap(&[Span::styled(text.to_owned(), style.patch(t.panel))], width, &[], &[]) {
        if *y >= body.bottom() {
            return;
        }
        put_line(buf, body.x, *y, body.width, &l);
        *y += 1;
    }
}
