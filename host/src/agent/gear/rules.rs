//! Instruction files an agent keeps in the user's home. Copilot's file is
//! `~/.copilot/copilot-instructions.md`, not Claude's rules.

use std::path::Path;

use super::{MAX_DEPTH, MAX_ITEMS, NAME_CHARS, TEXT_CHARS, clip, homes, is_dir, read_limited, skipped_name};
use crate::market::front_matter;

use super::Named;

const RULE_BYTES: u64 = 64 * 1024;

pub(super) fn list(backend: &str, home: &Path) -> Vec<Named> {
    let mut out = Vec::new();
    for dir in homes(home, dirs(backend)) {
        walk(&dir, 0, &mut out);
    }
    for file in homes(home, files(backend)) {
        if let Some(item) = rule_file(&file) {
            out.push(item);
        }
    }
    super::dedupe(&mut out);
    out
}

fn dirs(backend: &str) -> &'static [&'static str] {
    match backend {
        "cursor" => &[".cursor/rules"],
        "claude" => &[".claude/rules"],
        "codebuddy" | "workbuddy" => &[".codebuddy/rules"],
        "gemini" | "antigravity" => &[".gemini/rules"],
        "grok" => &[".grok/rules"],
        _ => &[],
    }
}

fn files(backend: &str) -> &'static [&'static str] {
    match backend {
        "copilot" => &[".copilot/copilot-instructions.md"],
        _ => &[],
    }
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<Named>) {
    if depth > MAX_DEPTH || out.len() >= MAX_ITEMS || !is_dir(dir) {
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
            walk(&path, depth + 1, out);
        } else if is_rule(&path)
            && let Some(item) = rule_file(&path)
        {
            out.push(item);
        }
    }
}

fn is_rule(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md" | "mdc" | "markdown"))
}

fn rule_file(path: &Path) -> Option<Named> {
    if !path.is_file() {
        return None;
    }
    let text = read_limited(path, RULE_BYTES)?;
    let (name, description) = front_matter(&text);
    let fallback = path.file_name().and_then(|n| n.to_str()).unwrap_or("rule");
    let name = clip(&name.unwrap_or_else(|| fallback.to_owned()), NAME_CHARS);
    if name.is_empty() {
        return None;
    }
    Some(Named { name, description: clip(&description.unwrap_or_default(), TEXT_CHARS), detail: String::new() })
}
