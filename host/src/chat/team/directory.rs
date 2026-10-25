//! `list_bots`: this computer's bots, then those the user's other computers offer, narrowed and
//! paged so a computer with many bots doesn't flood the agent's context.

use crate::hub::Hub;
use anyhow::Result;
use codync_network::Peer;
use serde_json::{Value, json};

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 50;

struct Row {
    value: Value,
    /// Lowercase name and description, for `query`.
    words: String,
    computer: Option<String>,
    online: bool,
}

pub(super) fn list(hub: &Hub, from: &str, args: &Value, remote: &[Peer]) -> Result<Value> {
    let mut rows: Vec<Row> = hub
        .store
        .bots()?
        .into_iter()
        .filter(|b| !b.deleted && !b.config.hidden && !b.config.is_group() && b.config.id != from)
        .map(|b| {
            let c = b.config;
            Row {
                words: format!("{} {}", c.name, c.description).to_lowercase(),
                value: json!({
                    "id": c.id, "name": c.name, "description": c.description,
                    "backend": c.backend, "cwd": c.cwd, "status": hub.runtime(&c.id).status,
                }),
                computer: None,
                online: true,
            }
        })
        .collect();
    let mut others: Vec<Row> = remote
        .iter()
        .flat_map(|p| {
            p.agents.iter().map(|a| Row {
                words: format!("{} {}", a.name, a.about).to_lowercase(),
                value: json!({
                    "id": format!("{}@{}", a.id, p.name), "name": a.name, "description": a.about,
                    "computer": p.name, "status": if p.online { "online" } else { "away" },
                }),
                computer: Some(p.name.clone()),
                online: p.online,
            })
        })
        .collect();
    // Nearest first: this computer's bots, then the ones that can take a message now.
    others.sort_by(|a, b| (!a.online, &a.computer).cmp(&(!b.online, &b.computer)));
    rows.extend(others);

    let query: Vec<String> =
        args["query"].as_str().unwrap_or_default().split_whitespace().map(str::to_lowercase).collect();
    let computer = args["computer"].as_str().map(str::to_lowercase);
    let online = args["online"].as_bool() == Some(true);
    let narrowed = !query.is_empty() || computer.is_some() || online;
    let here = hub.network.node().and_then(|n| n.name()).map(|n| n.to_lowercase());
    rows.retain(|r| {
        let at = r.computer.as_deref().map(str::to_lowercase).or_else(|| here.clone());
        query.iter().all(|w| r.words.contains(w))
            && computer.as_ref().is_none_or(|c| at.as_ref() == Some(c) || (r.computer.is_none() && c == "this"))
            && (!online || r.online)
    });

    let total = rows.len();
    let limit =
        args["limit"].as_u64().map_or(DEFAULT_LIMIT, |n| usize::try_from(n).unwrap_or(MAX_LIMIT)).clamp(1, MAX_LIMIT);
    let start = args["cursor"].as_str().and_then(|c| c.parse::<usize>().ok()).unwrap_or(0).min(total);
    let end = (start + limit).min(total);
    let mut out = json!({"bots": rows[start..end].iter().map(|r| r.value.clone()).collect::<Vec<_>>(), "total": total});
    if end < total {
        out["next"] = end.to_string().into();
        if !narrowed {
            out["summary"] = summary(&rows).into();
        }
    }
    Ok(out)
}

/// How many bots each computer has, so the agent can narrow the list.
fn summary(rows: &[Row]) -> String {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for r in rows {
        let name = r.computer.clone().unwrap_or_else(|| "this computer".into());
        match counts.iter_mut().find(|(n, _)| *n == name) {
            Some((_, n)) => *n += 1,
            None => counts.push((name, 1)),
        }
    }
    let parts: Vec<String> = counts.iter().map(|(name, n)| format!("{name} {n}")).collect();
    format!("{} bots: {}. Narrow with computer or query.", rows.len(), parts.join(", "))
}
