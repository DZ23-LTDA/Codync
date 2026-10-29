//! Which bot is the computer's lead (`chat::lead`): one at most, picked by the user.

use super::{Hub, validate};
use crate::LockExt;
use crate::agent::bot::Cmd;
use crate::store::BotConfig;
use anyhow::{Result, anyhow, bail};

impl Hub {
    /// Makes `id` the computer's one lead in place of the one before (`chat::lead::set`): checked and saved under one
    /// `emit_lock`, so two picks never leave two leads and a bot deleted or hidden meanwhile never leads. Each bot
    /// whose role changed is then reconfigured, so its next message tells it (`context::profile_update`).
    pub(crate) fn make_lead(&self, id: &str) -> Result<()> {
        let mut changed = vec![];
        let saved = self.save_lead(id, &mut changed);
        // Emitted even when a save failed: a saved row has its rev already, and a client past it must not miss it.
        for cfg in &changed {
            self.emit_bot(&cfg.id);
        }
        for cfg in changed {
            let bot = cfg.id.clone();
            if let Err(error) = self.send_cmd(&bot, Cmd::Reconfigure(Box::new(cfg))) {
                tracing::warn!(bot, error = format!("{error:#}"), "couldn't tell a bot its lead role changed");
            }
        }
        saved
    }

    /// Sets `id`'s lead flag and clears every other live bot's, noting each bot saved in `changed`.
    fn save_lead(&self, id: &str, changed: &mut Vec<BotConfig>) -> Result<()> {
        let _g = self.emit_lock.locked();
        let bots: Vec<BotConfig> = self.store.bots()?.into_iter().filter(|r| !r.deleted).map(|r| r.config).collect();
        let mut target = bots.iter().find(|b| b.id == id).cloned().ok_or_else(|| anyhow!("unknown bot"))?;
        if target.is_group() {
            bail!("a group can't be the lead");
        }
        target.lead = true;
        // Refuses a hidden bot.
        validate(&target)?;
        for mut cfg in bots {
            let lead = cfg.id == id;
            if cfg.lead != lead {
                cfg.lead = lead;
                self.store.save_bot(&cfg)?;
                changed.push(cfg);
            }
        }
        Ok(())
    }
}
