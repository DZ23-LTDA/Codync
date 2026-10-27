//! The Skills and Connectors pages: the agent's own rows, then Codync's switches.

use super::{App, Buffer, Rect, centered, footer, frame_box, name_of, page_tabs, put, row, theme, truncate, window};
use crate::tui::l10n::tr;
use crate::tui::manage::{Gear, GearKind};

pub fn draw(buf: &mut Buffer, area: Rect, app: &App, gear: &Gear) {
    let t = theme();
    let frame = centered(area, 90, 28);
    let title = name_of(app, &gear.bot);
    let inner = frame_box(buf, frame, t.text, t.panel, Some((&title, t.text)));
    let inner = page_tabs(buf, inner, gear.kind.page());
    let agent = app.bots.get(&gear.bot).map(|bot| app.agent_name(&bot.backend)).unwrap_or_default();
    let intro = tr("{agent} brings these. Codync adds its own on top, and those are what get passed to the agent.")
        .replace("{agent}", &agent);
    let mut y = inner.y;
    let bottom = inner.bottom().saturating_sub(2);
    if y < bottom {
        put(buf, inner.x, y, inner.width, &intro, t.secondary.patch(t.panel));
        y += 2;
    }
    let from_agent = tr("From {agent}").replace("{agent}", &agent);
    if y < bottom {
        put(buf, inner.x, y, inner.width, &from_agent, t.bold.patch(t.panel));
        y += 1;
    }
    y = agent_rows(buf, inner, y, bottom, gear);
    if y < bottom {
        y += 1;
        put(buf, inner.x, y, inner.width, &tr("From Codync"), t.bold.patch(t.panel));
        y += 1;
    }
    let toggles = app.gear_toggles(gear);
    let room = usize::from(bottom.saturating_sub(y));
    if toggles.is_empty() {
        if y < bottom {
            let empty = match gear.kind {
                GearKind::Skills => "No skills yet. Get some from Extensions, or write your own.",
                GearKind::Connectors => "No connectors yet. Add GitHub, Linear, Notion and more from Extensions.",
            };
            put(buf, inner.x, y, inner.width, &tr(empty), t.dim.patch(t.panel));
        }
    } else {
        let cursor = gear.cursor.min(toggles.len().saturating_sub(1));
        for (offset, (_id, name, on)) in toggles.iter().enumerate().skip(window(cursor, room)).take(room) {
            let base = row(buf, inner, y, offset == cursor);
            let mark = if *on { "☑" } else { "☐" };
            put(buf, inner.x, y, inner.width, &format!("{mark} {name}"), t.text.patch(base));
            y += 1;
        }
    }
    footer(buf, inner, None, "space on / off · ⇧←→ page · esc");
}

fn agent_rows(buf: &mut Buffer, inner: Rect, mut y: u16, bottom: u16, gear: &Gear) -> u16 {
    let t = theme();
    match gear.agent() {
        None => {
            if y < bottom {
                put(buf, inner.x, y, inner.width, &tr("Loading…"), t.dim.patch(t.panel));
                y += 1;
            }
        }
        Some(_) if gear.failed => {
            if y < bottom {
                put(buf, inner.x, y, inner.width, &tr("Couldn't load what this agent brings."), t.red.patch(t.panel));
                y += 1;
            }
        }
        Some([]) => {
            if y < bottom {
                put(buf, inner.x, y, inner.width, &tr("None from this agent."), t.dim.patch(t.panel));
                y += 1;
            }
        }
        Some(items) => {
            let room = usize::from(bottom.saturating_sub(y).saturating_sub(3));
            for item in items.iter().take(room.max(1)) {
                if y >= bottom {
                    break;
                }
                let line =
                    if item.detail.is_empty() { item.name.clone() } else { format!("{}  {}", item.name, item.detail) };
                put(buf, inner.x, y, inner.width, &truncate(&line, usize::from(inner.width)), t.text.patch(t.panel));
                y += 1;
            }
        }
    }
    y
}
