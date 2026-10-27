//! Plugins an agent has installed for the user. Claude also lists project installs
//! and plugins its settings turn off; those stay out. A manifest's other fields
//! (hooks, tokens) stay in the file.

use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

use super::{MAX_DEPTH, MAX_ITEMS, MCP_BYTES, NAME_CHARS, TEXT_CHARS, clip, homes, is_dir, read_limited, skipped_name};

use super::Named;

const MANIFESTS: &[&str] = &[
    ".cursor-plugin/plugin.json",
    ".claude-plugin/plugin.json",
    ".codebuddy-plugin/plugin.json",
    ".codex-plugin/plugin.json",
    ".copilot-plugin/plugin.json",
    "plugin.json",
];

pub(super) fn list(backend: &str, home: &Path) -> Vec<Named> {
    let mut out = Vec::new();
    match backend {
        "claude" => {
            read_installed(
                &home.join(".claude/plugins/installed_plugins.json"),
                &mut out,
                true,
                &disabled_claude(home),
            );
        }
        "copilot" => read_copilot(home, &mut out),
        "codex" => read_codex(home, &mut out),
        _ => {
            for file in homes(home, indexes(backend)) {
                read_installed(&file, &mut out, false, &HashSet::new());
            }
        }
    }
    super::dedupe(&mut out);
    out.truncate(MAX_ITEMS);
    out
}

fn indexes(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/plugins/installed_plugins.json"],
        "codebuddy" | "workbuddy" => &[".codebuddy/plugins/installed_plugins.json"],
        _ => &[],
    }
}

fn disabled_claude(home: &Path) -> HashSet<String> {
    let Some(text) = read_limited(&home.join(".claude/settings.json"), MCP_BYTES) else { return HashSet::new() };
    let Ok(value) = serde_json::from_str::<Value>(&text) else { return HashSet::new() };
    value
        .get("enabledPlugins")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, on)| on.as_bool() == Some(false))
        .map(|(key, _)| key.clone())
        .collect()
}

fn read_installed(path: &Path, out: &mut Vec<Named>, user_only: bool, disabled: &HashSet<String>) {
    let Some(text) = read_limited(path, MCP_BYTES) else { return };
    let Ok(value) = serde_json::from_str::<Value>(&text) else { return };
    let Some(plugins) = value.get("plugins").and_then(Value::as_object) else { return };
    for (key, entry) in plugins {
        if out.len() >= MAX_ITEMS || disabled.contains(key) {
            continue;
        }
        let Some(install) = chosen_install(entry, user_only) else { continue };
        let fallback = key.split('@').next().unwrap_or(key);
        let item = install
            .get("installPath")
            .and_then(Value::as_str)
            .filter(|path| !path.is_empty())
            .and_then(|dir| manifest_under(Path::new(dir)))
            .unwrap_or_else(|| Named {
                name: clip(fallback, NAME_CHARS),
                description: String::new(),
                detail: String::new(),
            });
        if !item.name.is_empty() {
            out.push(item);
        }
    }
}

fn chosen_install(entry: &Value, user_only: bool) -> Option<&serde_json::Map<String, Value>> {
    let rows: Vec<&serde_json::Map<String, Value>> = match entry {
        Value::Array(rows) => rows.iter().filter_map(Value::as_object).collect(),
        Value::Object(row) => vec![row],
        _ => return None,
    };
    if user_only {
        rows.into_iter().find(|row| row.get("scope").and_then(Value::as_str) == Some("user"))
    } else {
        rows.into_iter().next()
    }
}

fn read_copilot(home: &Path, out: &mut Vec<Named>) {
    let path = home.join(".copilot/installed-plugins");
    if path.is_file() {
        read_installed(&path, out, false, &HashSet::new());
        return;
    }
    walk_plugins(&path, 0, out);
}

fn read_codex(home: &Path, out: &mut Vec<Named>) {
    let Some(text) = read_limited(&home.join(".codex/config.toml"), MCP_BYTES) else { return };
    let cache = home.join(".codex/plugins/cache");
    let mut current: Option<(String, String)> = None;
    let mut disabled = false;
    let flush = |current: &mut Option<(String, String)>, disabled: &mut bool, out: &mut Vec<Named>| {
        if let Some((name, market)) = current.take()
            && !*disabled
            && out.len() < MAX_ITEMS
        {
            let dir = cache.join(&market).join(&name);
            let item = manifest_under(&dir).unwrap_or(Named {
                name: clip(&name, NAME_CHARS),
                description: String::new(),
                detail: String::new(),
            });
            if !item.name.is_empty() {
                out.push(item);
            }
        }
        *disabled = false;
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some((name, market)) = codex_header(trimmed) {
            flush(&mut current, &mut disabled, out);
            current = Some((name, market));
            continue;
        }
        if current.is_some()
            && trimmed
                .strip_prefix("enabled")
                .is_some_and(|rest| rest.trim().trim_start_matches('=').trim().trim_matches('"') == "false")
        {
            disabled = true;
        }
    }
    flush(&mut current, &mut disabled, out);
}

fn codex_header(line: &str) -> Option<(String, String)> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?.trim();
    let rest = inner.strip_prefix("plugins.")?.trim_matches('"');
    if rest.is_empty() || rest.contains('.') {
        return None;
    }
    let (name, market) = rest.split_once('@')?;
    (!name.is_empty() && !market.is_empty()).then(|| (name.to_owned(), market.to_owned()))
}

fn walk_plugins(dir: &Path, depth: usize, out: &mut Vec<Named>) {
    if depth > MAX_DEPTH || out.len() >= MAX_ITEMS || !is_dir(dir) {
        return;
    }
    if let Some(item) = manifest_here(dir) {
        out.push(item);
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if out.len() >= MAX_ITEMS {
            return;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if skipped_name(&name) {
            continue;
        }
        let path = entry.path();
        if is_dir(&path) {
            walk_plugins(&path, depth + 1, out);
        }
    }
}

fn manifest_here(dir: &Path) -> Option<Named> {
    MANIFESTS.iter().map(|rel| dir.join(rel)).find(|path| path.is_file()).and_then(|path| plugin_meta(&path))
}

fn manifest_under(dir: &Path) -> Option<Named> {
    if let Some(item) = manifest_here(dir) {
        return Some(item);
    }
    let mut children =
        std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| is_dir(p)).collect::<Vec<_>>();
    children.sort();
    children.into_iter().rev().find_map(|child| {
        let name = child.file_name()?.to_string_lossy();
        if skipped_name(&name) { None } else { manifest_here(&child) }
    })
}

fn plugin_meta(path: &Path) -> Option<Named> {
    let text = read_limited(path, MCP_BYTES)?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let obj = value.as_object()?;
    let fallback = path.parent().and_then(|dir| dir.file_name()).and_then(|n| n.to_str()).unwrap_or("plugin");
    let name = obj.get("name").and_then(Value::as_str).filter(|n| !n.is_empty()).unwrap_or(fallback);
    let name = clip(name, NAME_CHARS);
    if name.is_empty() {
        return None;
    }
    let description = obj.get("description").and_then(Value::as_str).unwrap_or("");
    Some(Named { name, description: clip(description, TEXT_CHARS), detail: String::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("codync-gear-plugins-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn claude_keeps_user_plugins_that_are_still_on() {
        let root = scratch();
        let home = root.join("home");
        let plug = root.join("plug");
        std::fs::create_dir_all(plug.join(".claude-plugin")).expect("plug");
        std::fs::create_dir_all(home.join(".claude/plugins")).expect("index");
        std::fs::write(
            plug.join(".claude-plugin/plugin.json"),
            r#"{"name":"browser","description":"Drive a browser","token":"secret-token"}"#,
        )
        .expect("manifest");
        let index = format!(
            r#"{{"plugins":{{"browser@local":[{{"scope":"user","installPath":"{}"}}],"local-only@local":[{{"scope":"project","installPath":"{}"}}],"off@local":[{{"scope":"user","installPath":"{}"}}]}}}}"#,
            plug.display(),
            plug.display(),
            plug.display()
        );
        std::fs::write(home.join(".claude/plugins/installed_plugins.json"), index).expect("index");
        std::fs::write(
            home.join(".claude/settings.json"),
            r#"{"enabledPlugins":{"off@local":false,"browser@local":true}}"#,
        )
        .expect("settings");
        let plugins = list("claude", &home);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "browser");
        assert_eq!(plugins[0].description, "Drive a browser");
        assert!(!format!("{} {}", plugins[0].name, plugins[0].description).contains("secret-token"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_disabled_codex_plugin_is_left_out() {
        let root = scratch();
        let home = root.join("home");
        let plug = home.join(".codex/plugins/cache/openai-bundled/docs/1/.codex-plugin");
        std::fs::create_dir_all(&plug).expect("cache");
        std::fs::write(plug.join("plugin.json"), r#"{"name":"docs","description":"Write docs"}"#).expect("manifest");
        std::fs::write(
            home.join(".codex/config.toml"),
            "[plugins.\"docs@openai-bundled\"]\n\n[plugins.\"browser@openai-bundled\"]\nenabled = false\n\n[plugins.\"docs@openai-bundled\".env]\nTOKEN = \"secret-token\"\n",
        )
        .expect("config");
        let plugins = list("codex", &home);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "docs");
        assert!(!plugins[0].description.contains("secret-token"));
        let _ = std::fs::remove_dir_all(root);
    }
}
