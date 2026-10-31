//! Who a bot is, apart from what it runs: the mascot it shows, the role it plays and the team it shows in.
//!
//! The mascot is a catalog id, never a role: a bot can be the same mascot as another one, and any mascot can be any
//! role. The canonical artwork lives at `assets/mascots/<id>.png` (`tools/mascot-assets.mjs` derives the copies each
//! client ships); the images themselves are not needed here, only the list, so a host without the assets still
//! validates what it stores and shows the fallback.

/// Every mascot this build knows, in the order the package lists them.
pub const IDS: [&str; 13] = [
    "tele",
    "steve-jobs",
    "nina",
    "rogerio-ceni",
    "ed",
    "jo-soares",
    "sherlock",
    "flanelinha",
    "ju",
    "joel-santana",
    "pingado",
    "astro",
    "byte",
];

/// The mascot a bot without one shows: deterministic, so an old bot looks the same everywhere.
pub const FALLBACK: &str = "byte";

/// Whether `id` is a mascot this build ships.
pub fn known(id: &str) -> bool {
    IDS.contains(&id)
}

/// A mascot id to show: the bot's own when it is a known one, the fallback otherwise. Missing assets never crash a
/// roster, and a bad id is not silently written back.
pub fn shown(id: Option<&str>) -> &str {
    id.filter(|id| known(id)).unwrap_or(FALLBACK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_mascot_shows_as_itself_and_anything_else_as_the_fallback() {
        assert!(known("tele") && known("byte"));
        assert!(!known("sofia"));
        assert_eq!(shown(Some("nina")), "nina");
        assert_eq!(shown(Some("sofia")), FALLBACK, "an id this build doesn't ship");
        assert_eq!(shown(None), FALLBACK, "an old bot with no mascot");
    }
}
