//! The Skills and Connectors pages: the agent's own rows, then Codync's switches.
//! Both groups are one list, so the cursor scrolls agent rows that do not fit.

use super::{App, Buffer, Rect, centered, footer, frame_box, name_of, page_tabs, put, row, theme, truncate, window};
use crate::tui::l10n::tr;
use crate::tui::manage::{Gear, GearKind, GearRow};

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
    let toggles = app.gear_toggles(gear);
    let rows = gear.rows(toggles.len());
    let room = usize::from(bottom.saturating_sub(y));
    let cursor = gear.cursor.min(rows.len().saturating_sub(1));
    for (offset, line) in rows.iter().enumerate().skip(window(cursor, room)).take(room) {
        if y >= bottom {
            break;
        }
        let base = row(buf, inner, y, offset == cursor);
        let (text, style) = match line {
            GearRow::AgentHeading => (tr("From {agent}").replace("{agent}", &agent), t.bold.patch(base)),
            GearRow::AgentNote => {
                let failed = gear.agent().is_some() && gear.failed;
                (agent_note(gear), if failed { t.red.patch(base) } else { t.dim.patch(base) })
            }
            GearRow::Agent(index) => {
                let Some(item) = gear.agent().and_then(|items| items.get(*index)) else { continue };
                let text =
                    if item.detail.is_empty() { item.name.clone() } else { format!("{}  {}", item.name, item.detail) };
                (text, t.text.patch(base))
            }
            GearRow::CodyncHeading => (tr("From Codync").into(), t.bold.patch(base)),
            GearRow::CodyncNote => {
                let empty = match gear.kind {
                    GearKind::Skills => "No skills yet. Get some from Extensions, or write your own.",
                    GearKind::Connectors => "No connectors yet. Add GitHub, Linear, Notion and more from Extensions.",
                };
                (tr(empty).into(), t.dim.patch(base))
            }
            GearRow::Toggle(index) => {
                let Some((_, name, on)) = toggles.get(*index) else { continue };
                (format!("{} {name}", if *on { "☑" } else { "☐" }), t.text.patch(base))
            }
        };
        put(buf, inner.x, y, inner.width, &truncate(&text, usize::from(inner.width)), style);
        y += 1;
    }
    footer(buf, inner, None, "space on / off · ⇧←→ page · esc");
}

fn agent_note(gear: &Gear) -> String {
    if gear.agent().is_none() {
        tr("Loading…").into()
    } else if gear.failed {
        tr("Couldn't load what this agent brings.").into()
    } else {
        tr("None from this agent.").into()
    }
}
