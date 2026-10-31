//! An edit to a bot's settings: the fields a client sends, laid over what is saved.

use crate::agent::avatar;
use crate::store::BotConfig;
use anyhow::{Result, bail};
use serde_json::Value;

/// `stored` with `patch` laid over it: a field the patch leaves out keeps its saved value, a `null`
/// clears it. The id, creation and lead fields never change here.
pub(super) fn patched(stored: &BotConfig, patch: &Value) -> Result<BotConfig> {
    let mut merged = serde_json::to_value(stored)?;
    for (k, v) in patch.as_object().into_iter().flatten() {
        if !matches!(k.as_str(), "id" | "createdAt" | "createdBy" | "lead") {
            merged[k] = v.clone();
        }
    }
    let mut cfg: BotConfig = serde_json::from_value(merged)?;
    if cfg.name.trim().is_empty() {
        cfg.name.clone_from(&stored.name);
    }
    // A name the user picked ends automatic naming; the namer passes `autoName` itself.
    if cfg.name != stored.name && patch.get("autoName").is_none() {
        cfg.auto_name = false;
    }
    // Options belong to one model; an app that doesn't know them can't clear them itself.
    if (cfg.model != stored.model || cfg.backend != stored.backend) && patch.get("modelOptions").is_none() {
        cfg.model_options.clear();
    }
    if let Some(avatar) = cfg.avatar_id.as_deref()
        && !avatar::known(avatar)
    {
        bail!("unknown mascot: {avatar}");
    }
    if cfg.kind != stored.kind {
        bail!("a bot can't become a group, or a group a bot");
    }
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::patched;
    use crate::store::BotConfig;
    use serde_json::json;

    #[test]
    fn a_mascot_must_be_one_the_build_ships() {
        let stored = |avatar: serde_json::Value| -> BotConfig {
            serde_json::from_value(serde_json::json!({"id": "a", "name": "a", "backend": "claude", "avatarId": avatar}))
                .expect("a bot config")
        };
        let patched = patched(&stored(serde_json::Value::Null), &serde_json::json!({"avatarId": "nina"}))
            .expect("a known mascot");
        assert_eq!(patched.avatar_id.as_deref(), Some("nina"));
        let refused =
            patched_unknown(&stored(serde_json::Value::Null), &serde_json::json!({"avatarId": "not-a-mascot"}));
        assert!(refused.to_string().contains("unknown mascot"), "{refused}");
    }

    fn patched_unknown(stored: &BotConfig, patch: &serde_json::Value) -> anyhow::Error {
        patched(stored, patch).expect_err("an unknown mascot is refused")
    }

    #[test]
    fn a_patch_keeps_the_saved_section_unless_it_names_one() {
        let stored: BotConfig = serde_json::from_value(
            json!({"id": "a", "name": "a", "backend": "claude", "cwd": "/tmp", "section": "Play"}),
        )
        .expect("a bot config");
        let section = |patch| patched(&stored, &patch).expect("a valid patch").section;
        assert_eq!(section(json!({"id": "a", "name": "b"})).as_deref(), Some("Play"), "left out: kept");
        assert_eq!(section(json!({"id": "a", "section": null})), None, "null: out of its section");
        assert_eq!(section(json!({"id": "a", "section": "Work"})).as_deref(), Some("Work"));
    }
}
