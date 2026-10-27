//! Rules and plugins an agent already has on disk. Names and descriptions only:
//! a plugin manifest's other fields (hooks, MCP, tokens) stay in the file.

use serde_json::Value;
use std::io::Read;
use std::path::Path;

const MAX_ITEMS: usize = 200;
const MAX_DEPTH: usize = 3;
const FILE_BYTES: u64 = 64 * 1024;
const TEXT_CHARS: usize = 180;

pub(super) struct Named {
    pub name: String,
    pub description: String,
}

pub(super) fn list(backend: &str, home: &Path, cwd: Option<&Path>) -> (Vec<Named>, Vec<Named>) {
    let mut rules = Vec::new();
    let mut plugins = Vec::new();
    gather(backend, home, false, &mut rules, &mut plugins);
    if let Some(cwd) = cwd {
        gather(backend, cwd, true, &mut rules, &mut plugins);
    }
    dedupe(&mut plugins);
    (present(rules), plugins)
}

/// `project` is set when `base` is the bot's folder. Home is scanned first, so the same rule path
/// in the project replaces the home one. A different path stays, even when the title matches.
fn gather(backend: &str, base: &Path, project: bool, rules: &mut Vec<Rule>, plugins: &mut Vec<Named>) {
    if base.as_os_str().is_empty() {
        return;
    }
    let (dirs, files) = if project {
        (project_rule_dirs(backend), project_rule_files(backend))
    } else {
        (home_rule_dirs(backend), home_rule_files(backend))
    };
    for rel in dirs {
        walk_rules(&base.join(rel), base, 0, rules);
    }
    for rel in files {
        if let Some(item) = rule_file(&base.join(rel), base) {
            rules.push(item);
        }
    }
    if project {
        for rel in project_plugin_dirs(backend) {
            walk_plugins(&base.join(rel), 0, plugins);
        }
    } else {
        for rel in home_plugin_indexes(backend) {
            read_installed(&base.join(rel), plugins);
        }
        if backend == "codex" {
            read_codex_plugins(base, plugins);
        }
    }
}

fn home_rule_dirs(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/rules"],
        "claude" | "copilot" => &[".claude/rules"],
        "codebuddy" | "workbuddy" => &[".codebuddy/rules"],
        _ => &[],
    }
}

fn project_rule_dirs(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/rules"],
        "claude" | "copilot" => &[".claude/rules"],
        "codebuddy" | "workbuddy" => &[".codebuddy/rules"],
        "gemini" | "antigravity" => &[".gemini/rules"],
        "grok" => &[".grok/rules"],
        "codex" => &[],
        _ => &[".agents/rules"],
    }
}

/// Instruction files the agent reads from the home directory, outside its rules folder.
fn home_rule_files(backend: &str) -> &'static [&'static str] {
    match backend {
        "claude" | "copilot" => &[".claude/CLAUDE.md"],
        "codex" => &[".codex/AGENTS.md"],
        "grok" => &[".grok/AGENTS.md"],
        "codebuddy" | "workbuddy" => &[".codebuddy/CODEBUDDY.md", ".codebuddy/AGENTS.md"],
        "gemini" | "antigravity" => &[".gemini/GEMINI.md"],
        _ => &[],
    }
}

fn project_rule_files(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursorrules", "AGENTS.md", "CLAUDE.md", "CLAUDE.local.md"],
        "claude" | "copilot" => &["CLAUDE.md", "CLAUDE.local.md", "AGENTS.md"],
        "codex" | "grok" => &["AGENTS.md"],
        "codebuddy" | "workbuddy" => &["AGENTS.md", "CODEBUDDY.md"],
        "gemini" | "antigravity" => &["GEMINI.md", "AGENTS.md"],
        _ => &["AGENTS.md", "CLAUDE.md"],
    }
}

fn home_plugin_indexes(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/plugins/installed_plugins.json"],
        "claude" | "copilot" => &[".claude/plugins/installed_plugins.json"],
        "codebuddy" | "workbuddy" => &[".codebuddy/plugins/installed_plugins.json"],
        _ => &[],
    }
}

fn project_plugin_dirs(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/plugins"],
        "claude" | "copilot" => &[".claude/plugins"],
        "codebuddy" | "workbuddy" => &[".codebuddy/plugins"],
        "codex" => &[".codex/plugins"],
        _ => &[],
    }
}

fn walk_rules(dir: &Path, base: &Path, depth: usize, out: &mut Vec<Rule>) {
    if depth > MAX_DEPTH || out.len() >= MAX_ITEMS || skipped(dir) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if out.len() >= MAX_ITEMS {
            return;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || matches!(name.as_ref(), "node_modules" | "dist" | "target") {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            walk_rules(&path, base, depth + 1, out);
        } else if is_rule_file(&path)
            && let Some(item) = rule_file(&path, base)
        {
            out.push(item);
        }
    }
}

fn is_rule_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md" | "mdc" | "markdown"))
}

struct Rule {
    name: String,
    description: String,
    /// Path under the rules folder, or under home/project for a standalone file. Home and project
    /// copies of this path are one rule; `frontend/style.mdc` and `backend/style.mdc` are not.
    key: String,
}

fn rule_file(path: &Path, base: &Path) -> Option<Rule> {
    if !path.is_file() {
        return None;
    }
    let text = read_limited(path, FILE_BYTES)?;
    let (name, description) = front_matter(&text);
    let key = rule_key(path, base);
    let fallback = path.file_name().and_then(|n| n.to_str()).unwrap_or("rule");
    let name = clip(&name.unwrap_or_else(|| fallback.to_owned()), 80);
    if name.is_empty() {
        return None;
    }
    Some(Rule { name, description: clip(&description.unwrap_or_default(), TEXT_CHARS), key })
}

/// The rule's identity: the path inside a rules directory, otherwise the path from `base`.
fn rule_key(path: &Path, base: &Path) -> String {
    const ROOTS: &[&str] =
        &[".cursor/rules/", ".claude/rules/", ".codebuddy/rules/", ".gemini/rules/", ".grok/rules/", ".agents/rules/"];
    let rel = path.strip_prefix(base).unwrap_or(path);
    let text = rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
    for root in ROOTS {
        if let Some(rest) = text.strip_prefix(root)
            && !rest.is_empty()
        {
            return rest.to_owned();
        }
    }
    if text.is_empty() { "rule".to_owned() } else { text }
}

fn front_matter(md: &str) -> (Option<String>, Option<String>) {
    let Some(rest) = md.strip_prefix("---\n").or_else(|| md.strip_prefix("---\r\n")) else {
        return (None, None);
    };
    let Some((head, _)) = rest.split_once("\n---").or_else(|| rest.split_once("\r\n---")) else {
        return (None, None);
    };
    let mut name = None;
    let mut description = None;
    for line in head.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        if key.chars().any(char::is_whitespace) {
            continue;
        }
        let value = value.trim().trim_matches(['"', '\'']).trim();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "name" if name.is_none() => name = Some(value.to_owned()),
            "description" if description.is_none() => description = Some(value.to_owned()),
            _ => {}
        }
    }
    (name, description)
}

const MANIFESTS: &[&str] = &[
    ".cursor-plugin/plugin.json",
    ".claude-plugin/plugin.json",
    ".codebuddy-plugin/plugin.json",
    ".codex-plugin/plugin.json",
    "plugin.json",
];

fn walk_plugins(dir: &Path, depth: usize, out: &mut Vec<Named>) {
    if depth > MAX_DEPTH || out.len() >= MAX_ITEMS || skipped(dir) {
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
        if name.starts_with('.')
            || matches!(name.as_ref(), "node_modules" | "cache" | "marketplaces" | "dist" | "target")
        {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            walk_plugins(&path, depth + 1, out);
        }
    }
}

fn read_installed(path: &Path, out: &mut Vec<Named>) {
    let Some(text) = read_limited(path, FILE_BYTES) else { return };
    let Ok(value) = serde_json::from_str::<Value>(&text) else { return };
    let Some(plugins) = value.get("plugins").and_then(Value::as_object) else { return };
    for (key, entry) in plugins {
        if out.len() >= MAX_ITEMS {
            return;
        }
        let fallback = key.split('@').next().unwrap_or(key);
        let item = entry_path(entry)
            .and_then(|dir| manifest_under(Path::new(dir)))
            .unwrap_or_else(|| Named { name: clip(fallback, 80), description: String::new() });
        if !item.name.is_empty() {
            out.push(item);
        }
    }
}

fn entry_path(value: &Value) -> Option<&str> {
    let obj = value.as_array().and_then(|rows| rows.iter().find_map(Value::as_object)).or_else(|| value.as_object())?;
    obj.get("installPath").and_then(Value::as_str).filter(|path| !path.is_empty())
}

/// `[plugins."name@marketplace"]` in Codex config. Nested tables are not plugins.
fn read_codex_plugins(home: &Path, out: &mut Vec<Named>) {
    let Some(text) = read_limited(&home.join(".codex/config.toml"), FILE_BYTES) else { return };
    let cache = home.join(".codex/plugins/cache");
    for line in text.lines() {
        if out.len() >= MAX_ITEMS {
            return;
        }
        let Some((name, market)) = codex_plugin_header(line.trim()) else { continue };
        let dir = cache.join(&market).join(&name);
        let item = manifest_under(&dir).unwrap_or(Named { name: clip(&name, 80), description: String::new() });
        if !item.name.is_empty() {
            out.push(item);
        }
    }
}

fn codex_plugin_header(line: &str) -> Option<(String, String)> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?.trim();
    let rest = inner.strip_prefix("plugins.")?.trim_matches('"');
    if rest.is_empty() || rest.contains('.') {
        return None;
    }
    let (name, market) = rest.split_once('@')?;
    (!name.is_empty() && !market.is_empty()).then(|| (name.to_owned(), market.to_owned()))
}

fn manifest_here(dir: &Path) -> Option<Named> {
    MANIFESTS.iter().map(|rel| dir.join(rel)).find(|path| path.is_file()).and_then(|path| plugin_meta(&path))
}

fn manifest_under(dir: &Path) -> Option<Named> {
    if let Some(item) = manifest_here(dir) {
        return Some(item);
    }
    let mut children =
        std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect::<Vec<_>>();
    children.sort();
    children.into_iter().rev().find_map(|child| {
        let name = child.file_name()?.to_string_lossy();
        if name.starts_with('.') || name == "node_modules" { None } else { manifest_here(&child) }
    })
}

fn plugin_meta(path: &Path) -> Option<Named> {
    let text = read_limited(path, FILE_BYTES)?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let obj = value.as_object()?;
    let fallback = path.parent().and_then(|dir| dir.file_name()).and_then(|n| n.to_str()).unwrap_or("plugin");
    let name = obj.get("name").and_then(Value::as_str).filter(|n| !n.is_empty()).unwrap_or(fallback);
    let name = clip(name, 80);
    if name.is_empty() {
        return None;
    }
    let description = obj.get("description").and_then(Value::as_str).unwrap_or("");
    Some(Named { name, description: clip(description, TEXT_CHARS) })
}

fn skipped(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

/// One row per rule path. A repeated title keeps the path, so two `style.mdc` files both show.
fn present(mut rules: Vec<Rule>) -> Vec<Named> {
    let mut index = std::collections::HashMap::<String, usize>::new();
    let mut kept = Vec::new();
    for rule in rules.drain(..) {
        if let Some(at) = index.get(&rule.key).copied() {
            kept[at] = rule;
        } else {
            index.insert(rule.key.clone(), kept.len());
            kept.push(rule);
        }
    }
    let mut counts = std::collections::HashMap::<String, usize>::new();
    for rule in &kept {
        *counts.entry(rule.name.clone()).or_insert(0) += 1;
    }
    kept.into_iter()
        .map(|rule| {
            let shared = counts[&rule.name] > 1;
            let name = if shared { clip(&rule.key, 80) } else { rule.name };
            Named { name, description: rule.description }
        })
        .collect()
}

fn dedupe(items: &mut Vec<Named>) {
    let mut index = std::collections::HashMap::<String, usize>::new();
    let mut kept = Vec::new();
    for item in items.drain(..) {
        if let Some(at) = index.get(&item.name).copied() {
            kept[at] = item;
        } else {
            index.insert(item.name.clone(), kept.len());
            kept.push(item);
        }
    }
    *items = kept;
}

fn clip(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max { collapsed } else { collapsed.chars().take(max).collect() }
}

fn read_limited(path: &Path, cap: u64) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?.take(cap);
    let mut text = String::new();
    file.read_to_string(&mut text).ok()?;
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("codync-gear-catalog-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_project_rule_replaces_the_home_one_and_a_plugin_keeps_only_its_name() {
        let root = scratch();
        let home = root.join("home");
        let cwd = root.join("proj");
        let plug = root.join("plug");
        std::fs::create_dir_all(home.join(".cursor/rules")).expect("rules");
        std::fs::create_dir_all(cwd.join(".cursor/rules")).expect("project rules");
        std::fs::create_dir_all(plug.join(".cursor-plugin")).expect("plugin");
        std::fs::write(
            home.join(".cursor/rules/style.mdc"),
            "---\nname: style\ndescription: Home style\n---\n# Style\n",
        )
        .expect("home rule");
        std::fs::write(cwd.join(".cursor/rules/style.mdc"), "---\nname: style\ndescription: Project style\n---\n")
            .expect("project rule");
        std::fs::write(cwd.join("AGENTS.md"), "# Agents\n").expect("agents");
        std::fs::write(
            plug.join(".cursor-plugin/plugin.json"),
            r#"{"name":"browser","description":"Drive a browser","token":"secret-token"}"#,
        )
        .expect("manifest");
        std::fs::create_dir_all(home.join(".cursor/plugins")).expect("index dir");
        let index = serde_json::json!({
            "plugins": { "browser@local": [{ "installPath": plug.display().to_string() }] }
        });
        std::fs::write(
            home.join(".cursor/plugins/installed_plugins.json"),
            serde_json::to_string(&index).expect("index"),
        )
        .expect("index");
        let (rules, plugins) = list("cursor", &home, Some(&cwd));
        assert_eq!(rules.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), ["style", "AGENTS.md"]);
        assert_eq!(rules[0].description, "Project style");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "browser");
        assert_eq!(plugins[0].description, "Drive a browser");
        let dumped = format!("{} {}", plugins[0].name, plugins[0].description);
        assert!(!dumped.contains("secret-token"), "{dumped}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_home_instruction_is_listed_and_nested_rules_stay_separate() {
        let root = scratch();
        let home = root.join("home");
        let cwd = root.join("proj");
        std::fs::create_dir_all(home.join(".claude")).expect("claude home");
        std::fs::create_dir_all(&cwd).expect("project");
        std::fs::write(home.join(".claude/CLAUDE.md"), "# Global\n").expect("home claude");
        std::fs::write(cwd.join("CLAUDE.md"), "# Project\n").expect("project claude");
        let (rules, _) = list("claude", &home, Some(&cwd));
        let names: Vec<_> = rules.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&".claude/CLAUDE.md"), "{names:?}");
        assert!(names.contains(&"CLAUDE.md"), "{names:?}");
        std::fs::create_dir_all(cwd.join(".cursor/rules/frontend")).expect("front");
        std::fs::create_dir_all(cwd.join(".cursor/rules/backend")).expect("back");
        std::fs::write(cwd.join(".cursor/rules/frontend/style.mdc"), "# Front\n").expect("front rule");
        std::fs::write(cwd.join(".cursor/rules/backend/style.mdc"), "# Back\n").expect("back rule");
        let (rules, _) = list("cursor", &home, Some(&cwd));
        let names: Vec<_> = rules.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"frontend/style.mdc"), "{names:?}");
        assert!(names.contains(&"backend/style.mdc"), "{names:?}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn codex_lists_the_plugins_its_config_names() {
        let root = scratch();
        let home = root.join("home");
        let plug = home.join(".codex/plugins/cache/openai-bundled/docs/1/.codex-plugin");
        std::fs::create_dir_all(&plug).expect("cache");
        std::fs::write(plug.join("plugin.json"), r#"{"name":"docs","description":"Write docs"}"#).expect("manifest");
        std::fs::write(
            home.join(".codex/config.toml"),
            "[plugins.\"docs@openai-bundled\"]\n\n[plugins.\"docs@openai-bundled\".env]\nTOKEN = \"secret-token\"\n",
        )
        .expect("config");
        let (_, plugins) = list("codex", &home, None);
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "docs");
        assert_eq!(plugins[0].description, "Write docs");
        assert!(!format!("{:?}", plugins[0].description).contains("secret-token"));
        let _ = std::fs::remove_dir_all(root);
    }
}
