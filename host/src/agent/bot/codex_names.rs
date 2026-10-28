//! codex-acp leaves out a requested MCP server whose name Codex's own config already uses ("Codex uses its own
//! entry, or no server if that entry is disabled"), and says so only in a notice before the first turn. A connector
//! named like one of the user's Codex servers (`notion`) then never reaches the bot (#217), so it gets a free name.

use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item};

/// Renames each of `servers` whose name a Codex config for `cwd` already defines.
pub(super) async fn avoid_clashes(servers: &mut Value, cwd: &str) {
    let files = config_files(Path::new(cwd));
    match tokio::task::spawn_blocking(move || taken(&files)).await {
        Ok(taken) => rename(servers, &taken),
        Err(e) => tracing::warn!(error = format!("{e:#}"), "couldn't read Codex's MCP server names"),
    }
}

fn rename(servers: &mut Value, taken: &HashSet<String>) {
    for server in servers.as_array_mut().into_iter().flatten() {
        if let Some(name) = server["name"].as_str().filter(|name| taken.contains(*name)) {
            server["name"] = format!("codync-{name}").into();
        }
    }
}

/// The files Codex layers its config from: the user's and the system's, and `.codex/config.toml` in the folders
/// from `cwd` up (projects).
// ponytail: `[profiles.<name>.mcp_servers]` isn't read; add it if a profile's server ever clashes.
fn config_files(cwd: &Path) -> Vec<PathBuf> {
    let user = std::env::var_os("CODEX_HOME").map(PathBuf::from).or_else(|| dirs::home_dir().map(|h| h.join(".codex")));
    let mut files: Vec<PathBuf> = user.into_iter().map(|dir| dir.join("config.toml")).collect();
    if cfg!(unix) {
        files.push("/etc/codex/config.toml".into());
    }
    files.extend(cwd.ancestors().map(|dir| dir.join(".codex/config.toml")));
    files
}

/// Every `[mcp_servers.<name>]` in `files`, disabled ones too: codex-acp skips those as well.
fn taken(files: &[PathBuf]) -> HashSet<String> {
    let mut names = HashSet::new();
    for text in files.iter().filter_map(|file| std::fs::read_to_string(file).ok()) {
        let Ok(doc) = text.parse::<DocumentMut>() else { continue };
        if let Some(servers) = doc.get("mcp_servers").and_then(Item::as_table_like) {
            names.extend(servers.iter().map(|(name, _)| name.to_owned()));
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn servers_codex_already_names_get_a_free_name() {
        let dir = std::env::temp_dir().join(format!("codync-codex-names-{}", uuid::Uuid::new_v4()));
        let project = dir.join("work/app");
        std::fs::create_dir_all(dir.join("work/.codex")).expect("dir");
        std::fs::create_dir_all(&project).expect("dir");
        std::fs::write(
            dir.join("work/.codex/config.toml"),
            "[mcp_servers.notion]\nurl = \"https://mcp.notion.com/mcp\"\n\n[mcp_servers.memory]\ncommand = \"npx\"\nenabled = false\n",
        )
        .expect("config");
        let files: Vec<PathBuf> = project.ancestors().map(|d| d.join(".codex/config.toml")).collect();
        let mut servers = json!([{"name": "notion"}, {"name": "memory"}, {"name": "chat"}]);
        rename(&mut servers, &taken(&files));
        assert_eq!(servers, json!([{"name": "codync-notion"}, {"name": "codync-memory"}, {"name": "chat"}]));
        std::fs::remove_dir_all(dir).expect("cleanup");
    }
}
