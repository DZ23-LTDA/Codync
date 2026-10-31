//! Driving coding agents: harness discovery, the ACP registry, sign-in, setup
//! terminals and the per-bot actor that runs turns over ACP.

pub mod acp;
pub mod auth;
pub mod backends;
pub mod bot;
pub mod config;
pub mod fallback;
pub mod gear;
pub mod new_bot;
pub mod node;
pub mod own_memory;
pub mod registry;
pub mod term;
pub mod workspace;

mod process;
#[cfg(windows)]
mod process_windows;

/// Which ACP adapter answers, where Codync treats it apart from the others. Known twice: declared per
/// harness (`backends::catalog`), for the decisions taken before an agent runs, and confirmed by the
/// `initialize` answer (`bot/launch.rs`), which wins.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Adapter {
    /// claude-agent-acp: takes `_meta.systemPrompt` and `_meta.claudeCode.options`, and `/compact <instructions>`.
    Claude,
    /// codex-acp: `/compact` (its keep-list is in `CODEX_CONFIG`).
    Codex,
    /// Anything else: ACP as advertised, nothing treated apart.
    #[default]
    Other,
}
