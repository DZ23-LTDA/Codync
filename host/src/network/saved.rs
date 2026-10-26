//! The networks this computer is in, as kept between runs: kv `networks`, each with its node's
//! folder in the data directory.

use crate::hub::Hub;
use anyhow::{Result, bail};
use codync_network::Rule;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const KV: &str = "networks";
/// The folder of the network a computer joined before it could be in several.
const FIRST: &str = "network";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    /// Its node's folder, and its id in the API.
    pub id: String,
    pub name: String,
    /// `public`, `private` or a relay URL.
    pub connector: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

pub fn dir(id: &str) -> PathBuf {
    crate::service::data_dir().join(id)
}

/// The networks to start: kv `networks`, or the one network a computer was in before.
pub fn load(hub: &Hub) -> Vec<Saved> {
    if let Some(saved) = hub.store.kv_get(KV).and_then(|v| serde_json::from_str(&v).ok()) {
        return saved;
    }
    if !dir(FIRST).join("network.json").exists() {
        return Vec::new();
    }
    let first = Saved {
        id: FIRST.into(),
        name: "home".into(),
        connector: hub.store.kv_get("network_connector").unwrap_or_else(|| "public".into()),
        rules: hub.store.kv_get("network_rules").and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_default(),
    };
    if let Err(error) = save(hub, std::slice::from_ref(&first)) {
        tracing::warn!(error = format!("{error:#}"), "couldn't keep the network's settings");
    }
    vec![first]
}

pub fn save(hub: &Hub, all: &[Saved]) -> Result<()> {
    hub.store.kv_set(KV, &serde_json::to_string(all)?)
}

/// A folder no network uses yet: `network`, then `network-<random>`.
pub fn new_id(all: &[Saved]) -> String {
    if all.iter().all(|s| s.id != FIRST) && !dir(FIRST).exists() {
        return FIRST.into();
    }
    loop {
        let id = format!("{FIRST}-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        if !dir(&id).exists() {
            return id;
        }
    }
}

/// `name` as a network name (like a computer's: no spaces, `@` or `.`, which addresses use),
/// numbered when another network here has it (`home-2`).
pub fn free_name(all: &[Saved], name: &str) -> Result<String> {
    let name = super::handle(name.trim());
    if name.len() > 40 {
        bail!("a network's name is at most 40 characters");
    }
    let taken = |n: &str| all.iter().any(|s| s.name.eq_ignore_ascii_case(n));
    // One more number than there are networks: one of them is free.
    Ok(std::iter::once(name.clone())
        .chain((2..=all.len() + 2).map(|i| format!("{name}-{i}")))
        .find(|n| !taken(n))
        .expect("a number is free"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(id: &str, name: &str) -> Saved {
        Saved { id: id.into(), name: name.into(), connector: "public".into(), rules: Vec::new() }
    }

    #[test]
    fn a_taken_network_name_gets_a_number() {
        let all = [saved("network", "home"), saved("network-1", "home-2")];
        assert_eq!(free_name(&all, "Work").unwrap(), "work");
        assert_eq!(free_name(&all, "home").unwrap(), "home-3");
        assert_eq!(free_name(&all, "my.lab").unwrap(), "my-lab", "no dots: addresses use them");
    }
}
