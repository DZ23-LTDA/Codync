//! A bot's Skills and Connectors pages: what its agent already has, then the Codync ones it passes on.

use crossterm::event::{KeyCode, KeyEvent};
use serde_json::{Value, json};

use super::super::app::{After, App, Overlay};
use super::{Page, Reply};

/// Which of the two pages is open.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GearKind {
    Skills,
    Connectors,
}

impl GearKind {
    pub fn page(self) -> Page {
        match self {
            Self::Skills => Page::Skills,
            Self::Connectors => Page::Connectors,
        }
    }

    fn field(self) -> &'static str {
        match self {
            Self::Skills => "skills",
            Self::Connectors => "connectors",
        }
    }
}

/// One skill or connector the agent keeps for itself.
#[derive(Clone, Debug)]
pub struct GearItem {
    pub name: String,
    pub detail: String,
}

/// The Skills or Connectors page of one bot.
pub struct Gear {
    pub bot: String,
    pub kind: GearKind,
    /// `None` until `agentGear` answers.
    pub agent_skills: Option<Vec<GearItem>>,
    pub agent_connectors: Option<Vec<GearItem>>,
    pub failed: bool,
    pub cursor: usize,
}

impl Gear {
    pub fn agent(&self) -> Option<&[GearItem]> {
        match self.kind {
            GearKind::Skills => self.agent_skills.as_deref(),
            GearKind::Connectors => self.agent_connectors.as_deref(),
        }
    }

    /// One scrollable list: the agent's rows, then Codync's switches. Headings scroll with them.
    pub fn rows(&self, toggles: usize) -> Vec<GearRow> {
        let mut rows = vec![GearRow::AgentHeading];
        match self.agent() {
            Some(items) if !self.failed && !items.is_empty() => rows.extend((0..items.len()).map(GearRow::Agent)),
            _ => rows.push(GearRow::AgentNote),
        }
        rows.push(GearRow::CodyncHeading);
        if toggles == 0 {
            rows.push(GearRow::CodyncNote);
        } else {
            rows.extend((0..toggles).map(GearRow::Toggle));
        }
        rows
    }
}

/// A line on the Skills or Connectors page. `Toggle` is an index into the installed Codync list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearRow {
    AgentHeading,
    AgentNote,
    Agent(usize),
    CodyncHeading,
    CodyncNote,
    Toggle(usize),
}

pub(super) fn items(value: &Value) -> Vec<GearItem> {
    value
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let name = row["name"].as_str()?.trim();
                    if name.is_empty() {
                        return None;
                    }
                    let detail = row["description"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .or_else(|| row["detail"].as_str())
                        .unwrap_or("")
                        .to_owned();
                    Some(GearItem { name: name.to_owned(), detail })
                })
                .collect()
        })
        .unwrap_or_default()
}

impl App {
    pub(in crate::tui) fn open_gear(&mut self, kind: GearKind) {
        let Some(bot) = self.bot().filter(|bot| !bot.group) else {
            self.flash("Skills and connectors belong to a bot, not a group");
            return;
        };
        let id = bot.id.clone();
        let backend = bot.backend.clone();
        let cwd = bot.cwd.clone();
        self.sheet("agentGear", json!({"backend": backend, "cwd": cwd}), Reply::Gear(id.clone()));
        self.call("connectors", json!({}), After::Connectors);
        self.call("skills", json!({}), After::Skills);
        self.overlays.push(Overlay::Gear(Gear {
            bot: id,
            kind,
            agent_skills: None,
            agent_connectors: None,
            failed: false,
            cursor: 1,
        }));
    }

    pub(in crate::tui) fn gear_key(&mut self, mut gear: Gear, key: KeyEvent) -> Option<Overlay> {
        if let Some(page) = gear.kind.page().step(key) {
            self.open_page(page);
            return None;
        }
        let rows = gear.rows(self.gear_toggles(&gear).len());
        let last = rows.len().saturating_sub(1);
        gear.cursor = gear.cursor.min(last);
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return None,
            KeyCode::Down | KeyCode::Char('j') => gear.cursor = (gear.cursor + 1).min(last),
            KeyCode::Up | KeyCode::Char('k') => gear.cursor = gear.cursor.saturating_sub(1),
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(GearRow::Toggle(index)) = rows.get(gear.cursor) {
                    self.toggle_gear(&gear, *index);
                }
            }
            _ => {}
        }
        Some(Overlay::Gear(gear))
    }

    /// Installed Codync connectors or skills, and whether this bot passes each one on.
    pub(in crate::tui) fn gear_toggles(&self, gear: &Gear) -> Vec<(String, String, bool)> {
        let Some(bot) = self.bots.get(&gear.bot) else { return Vec::new() };
        let (installed, enabled) = match gear.kind {
            GearKind::Skills => (&self.market.1, &bot.skills),
            GearKind::Connectors => (&self.market.0, &bot.connectors),
        };
        installed
            .iter()
            .map(|item| (item.id.clone(), item.name.clone(), enabled.iter().any(|id| id == &item.id)))
            .collect()
    }

    fn toggle_gear(&mut self, gear: &Gear, index: usize) {
        let (bot_id, field, ids) = {
            let Some(bot) = self.bots.get(&gear.bot) else { return };
            let installed = match gear.kind {
                GearKind::Skills => &self.market.1,
                GearKind::Connectors => &self.market.0,
            };
            let Some(item) = installed.get(index) else { return };
            let id = item.id.clone();
            let mut ids = match gear.kind {
                GearKind::Skills => bot.skills.clone(),
                GearKind::Connectors => bot.connectors.clone(),
            };
            if ids.iter().any(|have| have == &id) {
                ids.retain(|have| have != &id);
            } else {
                ids.push(id);
            }
            (gear.bot.clone(), gear.kind.field(), ids)
        };
        let mut body = json!({"id": bot_id});
        body[field] = json!(ids);
        self.call("updateBot", body, After::Nothing);
    }
}
