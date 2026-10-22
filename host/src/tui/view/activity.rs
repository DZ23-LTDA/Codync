//! A compact activity row with an optional plain-text thinking preview.

use ratatui::text::{Line, Span};

use super::super::app::{App, Bot, Click, Entry, Kind, latest_thought, running_turn};
use super::super::md::{self, truncate};
use super::buffer::u;
use super::style::{spin, theme};
use super::transcript::{Built, gap, name_style};

pub(super) fn activity(out: &mut Built, app: &App, bot: &Bot, entries: &[&Entry], width: usize) {
    let t = theme();
    gap(out);
    out.lines.push(Line::from(Span::styled(format!(" {}", bot.name), name_style(&bot.color))));
    let expanded =
        app.thinking.as_ref().is_some_and(|(id, turn)| id == &bot.id && Some(*turn) == running_turn(entries));
    let text = if bot.activity.is_empty() { "Working…" } else { &bot.activity };
    let row = out.lines.len();
    out.lines.push(Line::from(vec![
        Span::styled(format!(" {} ", spin(app)), t.secondary),
        Span::styled(truncate(text, width.saturating_sub(8)), t.secondary),
        Span::styled(if expanded { "  v" } else { "  >" }, t.dim),
    ]));
    out.buttons.push((row, 0, u(width), Click::Thinking));
    let turn = running_turn(entries);
    let steps = entries.iter().filter(|entry| Some(entry.turn) == turn && entry.kind == Kind::Tool).count();
    let hint = if steps == 0 {
        "   > thinking · t full conversation".to_owned()
    } else {
        format!("   {steps} steps so far · > thinking · t full conversation")
    };
    out.lines.push(Line::from(Span::styled(hint, t.dim)));
    if expanded {
        let text = latest_thought(entries).unwrap_or("No thinking text yet.");
        let prefix = [Span::raw("   ")];
        out.lines.extend(md::wrap(&[Span::styled(text.to_owned(), t.secondary)], width, &prefix, &prefix));
    }
}
