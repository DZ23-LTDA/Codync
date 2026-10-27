//! An agent's own skills, MCP connectors, rules and plugins, read from the config it keeps on disk.
//!
//! Codync's skills and connectors are a separate list: clients show them beside these
//! and pass along the ones the bot has switched on. Rules and plugins are the agent's own;
//! Codync does not add either. This never starts the agent.
//! MCP environment variables, headers and tokens stay in the files.

use serde_json::{Map, Value, json};
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_SKILLS: usize = 200;
const MAX_DEPTH: usize = 3;
const SKILL_BYTES: u64 = 8 * 1024;
const MCP_BYTES: u64 = 256 * 1024;
const NAME_CHARS: usize = 80;
const DETAIL_CHARS: usize = 180;

struct Item {
    name: String,
    description: String,
    detail: String,
}

/// `{ skills, connectors, rules, plugins }` for `backend` working in `cwd`.
/// An empty `cwd` lists only what the agent keeps for the user, not a project.
/// Each entry is `{name, description}` except connectors, which use `{name, detail}`.
pub async fn list(backend: &str, cwd: &str) -> Value {
    let backend = backend.to_owned();
    let cwd = cwd.to_owned();
    match tokio::task::spawn_blocking(move || scan(&backend, &cwd)).await {
        Ok(value) => value,
        Err(e) => {
            tracing::warn!(error = format!("{e:#}"), "listing an agent's skills and connectors");
            json!({"skills": [], "connectors": [], "rules": [], "plugins": []})
        }
    }
}

fn scan(backend: &str, cwd: &str) -> Value {
    let home = dirs::home_dir().unwrap_or_default();
    let project = (!cwd.is_empty()).then(|| Path::new(cwd));
    let (skills, connectors) = collect(backend, &home, project);
    let (rules, plugins) = super::gear_catalog::list(backend, &home, project);
    json!({
        "skills": skills.iter().map(|s| json!({"name": s.name, "description": s.description})).collect::<Vec<_>>(),
        "connectors": connectors.iter().map(|c| json!({"name": c.name, "detail": c.detail})).collect::<Vec<_>>(),
        "rules": rules.iter().map(|s| json!({"name": s.name, "description": s.description})).collect::<Vec<_>>(),
        "plugins": plugins.iter().map(|s| json!({"name": s.name, "description": s.description})).collect::<Vec<_>>(),
    })
}

fn collect(backend: &str, home: &Path, cwd: Option<&Path>) -> (Vec<Item>, Vec<Item>) {
    let mut skills = Vec::new();
    for root in skill_roots(backend, home, cwd) {
        walk_skills(&root, 0, &mut skills);
    }
    let mut connectors = Vec::new();
    for file in mcp_files(backend, home, cwd) {
        read_mcp(&file, &mut connectors);
    }
    dedupe(&mut skills);
    dedupe(&mut connectors);
    (skills, connectors)
}

/// Home directories first, then the project, so a project entry of the same name replaces it.
fn skill_roots(backend: &str, home: &Path, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_rel(&mut roots, home, home_skills(backend));
    if let Some(cwd) = cwd {
        push_rel(&mut roots, cwd, project_skills(backend));
    }
    roots
}

fn home_skills(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[
            ".cursor/skills-cursor",
            ".cursor/skills",
            ".cursor/cloud-skills",
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
            ".grok/skills",
        ],
        "claude" | "copilot" => &[".claude/skills", ".agents/skills"],
        "codex" => &[".codex/skills", ".agents/skills"],
        "grok" => &[".grok/skills", ".agents/skills"],
        "codebuddy" | "workbuddy" => &[".codebuddy/skills", ".claude/skills", ".agents/skills"],
        "gemini" | "antigravity" => &[".gemini/skills", ".agents/skills"],
        _ => &[".agents/skills"],
    }
}

fn project_skills(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[
            ".cursor/skills",
            ".cursor/skills-cursor",
            ".cursor/plugins",
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
            ".grok/skills",
        ],
        "claude" | "copilot" => &[".claude/skills", ".claude/plugins", ".agents/skills"],
        "codex" => &[".codex/skills", ".agents/skills"],
        "grok" => &[".grok/skills", ".agents/skills"],
        "codebuddy" | "workbuddy" => &[".codebuddy/skills", ".claude/skills", ".agents/skills"],
        "gemini" | "antigravity" => &[".gemini/skills", ".agents/skills"],
        _ => &[".agents/skills", ".claude/skills"],
    }
}

fn mcp_files(backend: &str, home: &Path, cwd: Option<&Path>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    push_rel(&mut files, home, home_mcp(backend));
    if let Some(cwd) = cwd {
        push_rel(&mut files, cwd, project_mcp(backend));
    }
    files
}

fn home_mcp(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/mcp.json"],
        "claude" => &[".claude.json", ".claude/settings.json"],
        "codex" => &[".codex/config.toml"],
        "copilot" => &[".copilot/mcp-config.json"],
        "codebuddy" | "workbuddy" => &[".codebuddy/mcp.json", ".codebuddy/settings.json"],
        "gemini" | "antigravity" => &[".gemini/settings.json"],
        "grok" => &[".grok/mcp.json"],
        _ => &[],
    }
}

fn project_mcp(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/mcp.json"],
        "claude" => &[".mcp.json", ".claude/settings.json", ".claude/settings.local.json"],
        "codex" => &[".codex/config.toml"],
        "copilot" => &[".mcp.json", ".vscode/mcp.json"],
        "codebuddy" | "workbuddy" => &[".codebuddy/mcp.json", ".mcp.json"],
        "gemini" | "antigravity" => &[".gemini/settings.json", ".mcp.json"],
        _ => &[".mcp.json"],
    }
}

fn push_rel(out: &mut Vec<PathBuf>, base: &Path, rels: &[&str]) {
    if base.as_os_str().is_empty() {
        return;
    }
    for rel in rels {
        out.push(base.join(rel));
    }
}

fn walk_skills(dir: &Path, depth: usize, out: &mut Vec<Item>) {
    if depth > MAX_DEPTH || out.len() >= MAX_SKILLS {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if out.len() >= MAX_SKILLS {
            return;
        }
        let path = entry.path();
        if skipped_dir(&path, &entry.file_name().to_string_lossy()) {
            continue;
        }
        let skill = path.join("SKILL.md");
        if skill.is_file() {
            if let Some(item) = skill_item(&skill, &entry.file_name().to_string_lossy()) {
                out.push(item);
            }
        } else {
            walk_skills(&path, depth + 1, out);
        }
    }
}

fn skipped_dir(path: &Path, name: &str) -> bool {
    if name.starts_with('.') || matches!(name, "node_modules" | "dist" | "target" | "__pycache__") {
        return true;
    }
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink() || !meta.is_dir())
}

fn skill_item(path: &Path, folder: &str) -> Option<Item> {
    let text = read_limited(path, SKILL_BYTES)?;
    let (name, description) = front_matter(&text);
    let name = clip(&name.unwrap_or_else(|| folder.to_owned()), NAME_CHARS);
    if name.is_empty() {
        return None;
    }
    Some(Item { name, description: clip(&description.unwrap_or_default(), DETAIL_CHARS), detail: String::new() })
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
        let value = bare(value);
        if value.is_empty() || value.starts_with('|') || value.starts_with('>') {
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

fn bare(value: &str) -> &str {
    value.trim().trim_matches(['"', '\'']).trim()
}

fn read_mcp(path: &Path, out: &mut Vec<Item>) {
    if std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir()) {
        return;
    }
    let Some(text) = read_limited(path, MCP_BYTES) else { return };
    if path.extension().is_some_and(|ext| ext == "toml") {
        out.extend(mcp_toml(&text));
        return;
    }
    let Ok(value) = serde_json::from_str::<Value>(&text) else { return };
    let Some(servers) = servers_object(path, &value) else { return };
    for (name, server) in servers {
        let Some(server) = server.as_object() else { continue };
        let name = clip(name, NAME_CHARS);
        if name.is_empty() {
            continue;
        }
        out.push(Item { name, description: String::new(), detail: server_detail(server) });
    }
}

fn servers_object<'a>(path: &Path, value: &'a Value) -> Option<&'a Map<String, Value>> {
    if let Some(servers) = value.get("mcpServers").and_then(Value::as_object) {
        return Some(servers);
    }
    let name = path.file_name()?.to_str()?;
    if !matches!(name, "mcp.json" | "mcp-config.json") {
        return None;
    }
    value.get("servers").and_then(Value::as_object).or_else(|| value.as_object())
}

/// The command or URL a person can recognize. A flag that looks like a token is left out
/// together with the value after it, and a URL loses its user, query and fragment. A URL
/// whose path still looks like a token is omitted.
fn server_detail(server: &Map<String, Value>) -> String {
    if let Some(url) = server.get("url").and_then(Value::as_str).filter(|url| !url.is_empty()) {
        return clip(&visible_url(url), DETAIL_CHARS);
    }
    clip(&command_text(server), DETAIL_CHARS)
}

fn command_text(server: &Map<String, Value>) -> String {
    let mut parts = Vec::new();
    match server.get("command") {
        Some(Value::String(command)) if !command.is_empty() => parts.push(command.clone()),
        Some(Value::Array(command)) => parts.extend(command.iter().filter_map(|v| v.as_str().map(str::to_owned))),
        _ => {}
    }
    if let Some(args) = server.get("args").and_then(Value::as_array) {
        parts.extend(args.iter().filter_map(|v| v.as_str().map(str::to_owned)));
    }
    let mut kept = Vec::new();
    let mut skip_value = false;
    for part in parts {
        if skip_value {
            skip_value = false;
            continue;
        }
        if looks_secret(&part) {
            skip_value = true;
            continue;
        }
        kept.push(part);
    }
    kept.join(" ")
}

/// A URL with its user, query and fragment removed. Empty when the rest still looks like a token.
fn visible_url(url: &str) -> String {
    let url = public_url(url);
    if looks_secret(&url) { String::new() } else { url }
}

fn public_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.split(['?', '#']).next().unwrap_or(url).to_owned();
    };
    let rest = rest.split('@').next_back().unwrap_or(rest);
    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    format!("{scheme}://{rest}")
}

fn looks_secret(part: &str) -> bool {
    let lower = part.to_ascii_lowercase();
    ["token", "secret", "password", "api_key", "apikey", "authorization"].iter().any(|needle| lower.contains(needle))
}

/// `[mcp_servers.name]` tables. A later `url` replaces `command`. Nested tables are skipped.
fn mcp_toml(text: &str) -> Vec<Item> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    let mut detail = String::new();
    let flush = |name: &mut Option<String>, detail: &mut String, out: &mut Vec<Item>| {
        if let Some(name) = name.take() {
            let name = clip(&name, NAME_CHARS);
            if !name.is_empty() {
                out.push(Item { name, description: String::new(), detail: clip(detail, DETAIL_CHARS) });
            }
        }
        detail.clear();
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(next) = toml_header(trimmed) {
            flush(&mut name, &mut detail, &mut out);
            name = match next {
                TomlTable::Server(server) => Some(server),
                TomlTable::Other => None,
            };
            continue;
        }
        if name.is_none() {
            continue;
        }
        if let Some(url) = toml_value(trimmed, "url") {
            detail = visible_url(&url);
        } else if detail.is_empty()
            && let Some(command) = toml_value(trimmed, "command")
            && !looks_secret(&command)
        {
            detail = command;
        }
    }
    flush(&mut name, &mut detail, &mut out);
    out
}

/// A `[table]` line: one MCP server, or any other table. `None` is not a header.
enum TomlTable {
    Server(String),
    Other,
}

fn toml_header(line: &str) -> Option<TomlTable> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?.trim().trim_matches('"');
    let Some(rest) = inner.strip_prefix("mcp_servers.") else {
        return Some(TomlTable::Other);
    };
    if rest.is_empty() || rest.contains('.') {
        return Some(TomlTable::Other);
    }
    Some(TomlTable::Server(rest.to_owned()))
}

fn toml_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    let value = bare(rest);
    (!value.is_empty()).then(|| value.to_owned())
}

fn dedupe(items: &mut Vec<Item>) {
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
    let file = std::fs::File::open(path).ok()?;
    let mut file = file.take(cap);
    let mut text = String::new();
    file.read_to_string(&mut text).ok()?;
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("codync-gear-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_project_server_replaces_the_home_one_and_tokens_stay_out() {
        let root = scratch();
        let home = root.join("home");
        let cwd = root.join("proj");
        std::fs::create_dir_all(home.join(".cursor")).expect("home");
        std::fs::create_dir_all(cwd.join(".cursor/skills/review")).expect("skill");
        std::fs::write(
            home.join(".cursor/mcp.json"),
            r#"{"mcpServers":{"github":{"command":"npx","args":["-y","server"],"env":{"GITHUB_TOKEN":"secret-token"}}}}"#,
        )
        .expect("home mcp");
        std::fs::write(
            cwd.join(".cursor/mcp.json"),
            r#"{"mcpServers":{"github":{"url":"https://user:secret-token@example.test/mcp?token=secret-token"},"linear":{"command":"linear-mcp"},"hidden":{"url":"https://example.test/secret-token"}}}"#,
        )
        .expect("project mcp");
        std::fs::write(
            cwd.join(".cursor/skills/review/SKILL.md"),
            "---\nname: review\ndescription: Review a diff\n---\n# Review\n",
        )
        .expect("skill");
        let (skills, connectors) = collect("cursor", &home, Some(&cwd));
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "review");
        assert_eq!(skills[0].description, "Review a diff");
        assert_eq!(connectors.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["github", "linear", "hidden"]);
        assert_eq!(connectors[0].detail, "https://example.test/mcp");
        assert_eq!(connectors[2].detail, "");
        let dumped =
            serde_json::to_string(&json!(connectors.iter().map(|c| &c.detail).collect::<Vec<_>>())).expect("json");
        assert!(!dumped.contains("secret-token"), "{dumped}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn codex_toml_and_a_generic_project_are_listed() {
        let root = scratch();
        let home = root.join("home");
        let cwd = root.join("proj");
        std::fs::create_dir_all(home.join(".codex")).expect("codex");
        std::fs::create_dir_all(cwd.join(".agents/skills/ship")).expect("skill");
        std::fs::write(
            home.join(".codex/config.toml"),
            "[mcp_servers.github]\ncommand = \"github-mcp\"\n\n[mcp_servers.github.env]\nTOKEN = \"secret-token\"\n",
        )
        .expect("toml");
        std::fs::write(cwd.join(".mcp.json"), r#"{"mcpServers":{"docs":{"command":"docs-mcp"}}}"#).expect("mcp");
        std::fs::write(cwd.join(".agents/skills/ship/SKILL.md"), "---\nname: ship\ndescription: Ship it\n---\n")
            .expect("skill");
        let (codex_skills, codex_connectors) = collect("codex", &home, Some(&cwd));
        assert!(codex_skills.iter().any(|s| s.name == "ship"));
        assert_eq!(codex_connectors.len(), 1);
        assert_eq!(codex_connectors[0].name, "github");
        assert_eq!(codex_connectors[0].detail, "github-mcp");
        assert!(!codex_connectors.iter().any(|c| c.detail.contains("secret-token") || c.name.contains("secret-token")));
        let (generic_skills, generic_connectors) = collect("codebuddy", &home, Some(&cwd));
        assert!(generic_skills.iter().any(|s| s.name == "ship"));
        assert!(generic_connectors.iter().any(|c| c.name == "docs" && c.detail == "docs-mcp"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_token_flag_and_a_toml_url_leave_the_credential_out() {
        let root = scratch();
        let home = root.join("home");
        std::fs::create_dir_all(home.join(".cursor")).expect("cursor");
        std::fs::create_dir_all(home.join(".codex")).expect("codex");
        std::fs::write(
            home.join(".cursor/mcp.json"),
            r#"{"mcpServers":{"github":{"command":"npx","args":["--token","abc123","-y","server"]}}}"#,
        )
        .expect("mcp");
        let (_, connectors) = collect("cursor", &home, None);
        assert_eq!(connectors[0].detail, "npx -y server");
        assert!(!connectors[0].detail.contains("abc123"));
        std::fs::write(
            home.join(".codex/config.toml"),
            "[mcp_servers.github]\nurl = \"https://alice:abc123@example.test/mcp?key=xyz\"\n",
        )
        .expect("toml");
        let (_, connectors) = collect("codex", &home, None);
        assert_eq!(connectors[0].detail, "https://example.test/mcp");
        assert!(!connectors[0].detail.contains("abc123") && !connectors[0].detail.contains("alice"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_missing_config_is_an_empty_list() {
        let root = scratch();
        let (skills, connectors) = collect("cursor", &root.join("no-home"), Some(&root.join("no-proj")));
        assert!(skills.is_empty());
        assert!(connectors.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
