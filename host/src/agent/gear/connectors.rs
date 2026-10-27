//! MCP servers an agent configures in the user's home. A detail is the command, or
//! only a URL's origin: a path can carry a key (`/api/mcp/s/<key>/mcp`).

use serde_json::{Map, Value};
use std::path::Path;

use super::{MAX_ITEMS, MCP_BYTES, NAME_CHARS, TEXT_CHARS, clip, homes, read_limited};

use super::Named;

pub(super) fn list(backend: &str, home: &Path) -> Vec<Named> {
    let mut out = Vec::new();
    for file in homes(home, files(backend)) {
        read_file(&file, &mut out);
    }
    super::dedupe(&mut out);
    out.truncate(MAX_ITEMS);
    out
}

fn files(backend: &str) -> &'static [&'static str] {
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

fn read_file(path: &Path, out: &mut Vec<Named>) {
    if std::fs::metadata(path).is_ok_and(|meta| meta.is_dir()) {
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
        if server.get("enabled").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        let name = clip(name, NAME_CHARS);
        if name.is_empty() {
            continue;
        }
        out.push(Named { name, description: String::new(), detail: server_detail(server) });
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

fn server_detail(server: &Map<String, Value>) -> String {
    if let Some(url) = server.get("url").and_then(Value::as_str).filter(|url| !url.is_empty()) {
        return clip(&visible_url(url), TEXT_CHARS);
    }
    clip(&command_text(server), TEXT_CHARS)
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
    keep_public(&parts)
}

/// A flag named like a credential, the value after it, and a long letter-and-digit run.
fn keep_public(parts: &[String]) -> String {
    let mut kept = Vec::new();
    let mut skip_value = false;
    for part in parts {
        if skip_value {
            skip_value = false;
            continue;
        }
        if credential_flag(part) {
            skip_value = true;
            continue;
        }
        if token_run(part) {
            continue;
        }
        kept.push(part.as_str());
    }
    kept.join(" ")
}

fn credential_flag(part: &str) -> bool {
    let raw = part.trim().trim_matches(['"', '\'']);
    if !raw.starts_with('-') {
        return false;
    }
    let name = raw.trim_start_matches('-').to_ascii_lowercase().replace('-', "_");
    name.split('_')
        .filter(|seg| !seg.is_empty())
        .any(|seg| matches!(seg, "token" | "secret" | "password" | "authorization" | "apikey" | "auth" | "key"))
        || name.contains("api_key")
}

/// A bare argument that is a long mix of letters and digits, such as an API key.
fn token_run(part: &str) -> bool {
    let s = part.trim().trim_matches(['"', '\'']);
    if s.starts_with('-') || s.chars().count() < 20 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        && s.chars().any(|c| c.is_ascii_alphabetic())
        && s.chars().any(|c| c.is_ascii_digit())
}

/// Scheme and host only. The path, user and query stay out.
fn visible_url(raw: &str) -> String {
    let Ok(url) = url::Url::parse(raw) else { return String::new() };
    match url.origin() {
        url::Origin::Tuple(..) => url.origin().ascii_serialization(),
        url::Origin::Opaque(_) => String::new(),
    }
}

/// `[mcp_servers.name]` tables. `enabled = false` leaves the server out. Nested tables are skipped.
fn mcp_toml(text: &str) -> Vec<Named> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    let mut detail = String::new();
    let mut disabled = false;
    let flush = |name: &mut Option<String>, detail: &mut String, disabled: &mut bool, out: &mut Vec<Named>| {
        if let Some(name) = name.take()
            && !*disabled
        {
            let name = clip(&name, NAME_CHARS);
            if !name.is_empty() {
                out.push(Named { name, description: String::new(), detail: clip(detail, TEXT_CHARS) });
            }
        }
        detail.clear();
        *disabled = false;
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(next) = toml_header(trimmed) {
            flush(&mut name, &mut detail, &mut disabled, &mut out);
            name = match next {
                TomlTable::Server(server) => Some(server),
                TomlTable::Other => None,
            };
            continue;
        }
        if name.is_none() {
            continue;
        }
        if toml_value(trimmed, "enabled").as_deref() == Some("false") {
            disabled = true;
        } else if let Some(url) = toml_value(trimmed, "url") {
            detail = visible_url(&url);
        } else if detail.is_empty()
            && let Some(command) = toml_value(trimmed, "command")
        {
            detail = keep_public(&split_command(&command));
        }
    }
    flush(&mut name, &mut detail, &mut disabled, &mut out);
    out
}

fn split_command(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_owned).collect()
}

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
    let value = rest.trim().trim_matches(['"', '\'']).trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("codync-gear-mcp-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_large_claude_file_and_a_secret_argument_stay_readable() {
        let root = scratch();
        let home = root.join("home");
        std::fs::create_dir_all(home.join(".claude")).expect("claude");
        let pad = "x".repeat(300 * 1024);
        let body = format!(
            r#"{{"history":"{pad}","mcpServers":{{"github":{{"command":"npx","args":["--api-key","abc123","-y","server","sk-abc123def456ghij78"]}},"off":{{"command":"nope","enabled":false}},"zap":{{"url":"https://user:secret@hooks.zapier.com/api/mcp/s/secret-token/mcp?token=1"}}}}}}"#
        );
        std::fs::write(home.join(".claude.json"), body).expect("claude.json");
        let connectors = list("claude", &home);
        assert_eq!(connectors.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["github", "zap"]);
        assert_eq!(connectors[0].detail, "npx -y server");
        assert_eq!(connectors[1].detail, "https://hooks.zapier.com");
        let dumped = format!("{} {}", connectors[0].detail, connectors[1].detail);
        assert!(!dumped.contains("abc123") && !dumped.contains("secret"), "{dumped}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_disabled_codex_server_is_left_out() {
        let root = scratch();
        let home = root.join("home");
        std::fs::create_dir_all(home.join(".codex")).expect("codex");
        std::fs::write(
            home.join(".codex/config.toml"),
            "[mcp_servers.github]\ncommand = \"github-mcp\"\nenabled = false\n\n[mcp_servers.docs]\nurl = \"https://alice:abc123@example.test/mcp?key=xyz\"\n\n[mcp_servers.docs.env]\nTOKEN = \"secret-token\"\n",
        )
        .expect("toml");
        let connectors = list("codex", &home);
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].name, "docs");
        assert_eq!(connectors[0].detail, "https://example.test");
        assert!(!connectors[0].detail.contains("abc123") && !connectors[0].detail.contains("alice"));
        let _ = std::fs::remove_dir_all(root);
    }
}
