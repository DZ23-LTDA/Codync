//! What a connector process may inherit from the host.
//!
//! A connector is third-party code started for a bot: npm packages, a Python package, an https endpoint's companion
//! server. Its `env` comes from the connector's own definition (resolved from the vault); everything else it inherits
//! from the host process we started it from would be the host's whole environment — agent API keys, cloud tokens,
//! `CODYNC_*` internals, whatever the user's shell had. That is not something a connector has any business reading,
//! so it inherits a short list instead of everything.
use std::ffi::{OsStr, OsString};

/// The variables a process needs to start at all: where its binaries are, which home to write to, a temp folder, and
/// the platform's own system paths. Compared case-insensitively, because Windows environment names are.
const KEEP: &[&str] = &[
    "PATH",
    "HOME",
    "USERPROFILE",
    "SystemRoot",
    "windir",
    "TEMP",
    "TMP",
    "TMPDIR",
    "ComSpec",
    "PATHEXT",
    "LANG",
    "LC_ALL",
    "TERM",
];

/// Whether `key` is one a connector inherits.
fn keep(key: &OsStr) -> bool {
    let Some(key) = key.to_str() else { return false };
    KEEP.iter().any(|allowed| key.eq_ignore_ascii_case(allowed))
}

/// The pairs a connector inherits from `vars`. Pure, so the policy can be tested without touching this process's
/// environment (which `std::env::set_var` cannot do safely in edition 2024 anyway).
pub fn inherited(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<(OsString, OsString)> {
    vars.filter(|(key, _)| keep(key)).collect()
}

/// Clears everything a `Command` would inherit and sets back only what `inherited` allows.
pub fn apply(command: &mut tokio::process::Command) {
    command.env_clear();
    for (key, value) in inherited(std::env::vars_os()) {
        command.env(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
        pairs.iter().map(|(k, v)| (OsString::from(*k), OsString::from(*v))).collect()
    }

    fn keys(pairs: Vec<(OsString, OsString)>) -> Vec<String> {
        pairs.into_iter().map(|(k, _)| k.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn a_connector_keeps_what_it_needs_to_start() {
        let kept = keys(inherited(
            vars(&[("PATH", "/usr/bin"), ("HOME", "/Users/ana"), ("TMPDIR", "/tmp"), ("SystemRoot", "C:\\Windows")])
                .into_iter(),
        ));
        assert_eq!(kept, vec!["PATH", "HOME", "TMPDIR", "SystemRoot"]);
    }

    #[test]
    fn a_connector_does_not_get_the_hosts_secrets() {
        // Every one of these would be a real leak if it reached third-party code.
        let kept = keys(inherited(
            vars(&[
                ("PATH", "/usr/bin"),
                ("OPENAI_API_KEY", "secret"),
                ("ANTHROPIC_API_KEY", "secret"),
                ("GITHUB_TOKEN", "secret"),
                ("AWS_SECRET_ACCESS_KEY", "secret"),
                ("CODYNC_TOKEN", "secret"),
                ("POSTHOG_API_KEY", "secret"),
                ("SSH_AUTH_SOCK", "/tmp/agent"),
            ])
            .into_iter(),
        ));
        assert_eq!(kept, vec!["PATH"], "only the allowlist survives");
    }

    #[test]
    fn the_names_are_matched_the_way_windows_matches_them() {
        let kept = keys(inherited(vars(&[("path", "/usr/bin"), ("systemroot", "C:\\Windows")]).into_iter()));
        assert_eq!(kept, vec!["path", "systemroot"]);
    }

    #[test]
    fn a_variable_that_is_not_text_is_left_out_instead_of_guessed() {
        let odd = [(OsString::from("PATH"), OsString::from("/usr/bin"))];
        let kept = inherited(odd.into_iter());
        assert_eq!(kept.len(), 1);

        #[cfg(unix)]
        let weird = {
            use std::os::unix::ffi::OsStringExt;
            vec![
                (OsString::from("PATH"), OsString::from("/usr/bin")),
                (OsString::from_vec(vec![0x70, 0x41, 0xff]), OsString::from("x")),
            ]
        };
        #[cfg(not(unix))]
        let weird = vec![(OsString::from("PATH"), OsString::from("/usr/bin"))];
        assert_eq!(inherited(weird.into_iter()).len(), 1, "an unreadable name is not an allowlisted one");
    }
}
