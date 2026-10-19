//! What a user message asks the agent: its text with a `/skill …` command expanded, then the
//! paths of the files attached to it.

use crate::chat::uploads;
use crate::store::Store;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// `skills`: the asked bot's skills on. `root`: the chat's uploads (`uploads::root`; a group's
/// own in a group). `attachments`: the message's `{id, name}` upload metadata, as saved in the chat.
pub fn of_message(store: &Store, skills: &[String], root: &Path, text: &str, attachments: &[Value]) -> String {
    let asked = crate::market::expand_command(store, skills, text);
    let asked = asked.as_deref().unwrap_or(text);
    let paths: Vec<PathBuf> = attachments.iter().filter_map(|meta| uploads::path_of(root, meta)).collect();
    if paths.is_empty() { asked.to_owned() } else { format!("{asked}{}", uploads::prompt_suffix(&paths)) }
}
