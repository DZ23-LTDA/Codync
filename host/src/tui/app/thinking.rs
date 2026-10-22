//! The inline activity disclosure is presentation state, scoped to one bot and turn.

use super::{App, Entry, Kind, Status};

pub(in crate::tui) fn running_turn(entries: &[&Entry]) -> Option<i64> {
    entries.iter().filter(|e| e.data["status"] != "queued").map(|e| e.turn).max()
}

pub(in crate::tui) fn latest_thought<'a>(entries: &[&'a Entry]) -> Option<&'a str> {
    let turn = running_turn(entries)?;
    entries
        .iter()
        .rev()
        .find(|e| e.turn == turn && e.kind == Kind::Thought && !e.text().trim().is_empty())
        .map(|e| e.text())
}

impl App {
    pub(super) fn toggle_thinking(&mut self) {
        let Some(bot) = self
            .bot()
            .filter(|b| self.online && b.works_here() && matches!(b.status, Status::Working | Status::NeedsInput))
        else {
            return;
        };
        let Some(turn) = running_turn(&self.lane(&bot.id)) else { return };
        let key = (bot.id.clone(), turn);
        self.thinking = if self.thinking.as_ref() == Some(&key) { None } else { Some(key) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(turn: i64, kind: Kind, text: &str, status: &str) -> Entry {
        Entry { id: text.into(), seq: 0, turn, kind, data: json!({"text": text, "status": status}), created_at: 0 }
    }

    #[test]
    fn preview_uses_latest_thought_without_queued_or_tool_text() {
        let entries = [
            entry(1, Kind::Thought, "earlier", ""),
            entry(1, Kind::Thought, "latest", ""),
            entry(1, Kind::Tool, "tool output", ""),
            entry(2, Kind::User, "next", "queued"),
        ];
        assert_eq!(latest_thought(&entries.iter().collect::<Vec<_>>()), Some("latest"));
    }

    #[test]
    fn new_turn_does_not_reuse_previous_thinking() {
        let entries =
            [entry(1, Kind::Thought, "old", ""), entry(2, Kind::User, "new", ""), entry(2, Kind::Thought, "  ", "")];
        assert_eq!(latest_thought(&entries.iter().collect::<Vec<_>>()), None);
    }
}
