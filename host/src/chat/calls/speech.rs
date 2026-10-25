//! Text as a voice says it.

/// Text as speech can carry it: code blocks and links named, not read, markdown marks dropped, capped.
pub(super) fn spoken(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if !in_code {
                out.push_str(" (code) ");
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let words: Vec<String> = line
            .split_whitespace()
            .map(|w| {
                if w.starts_with("http://") || w.starts_with("https://") {
                    "(link)".to_owned()
                } else {
                    // Inline code and emphasis can span words ("`cargo test`."): their marks go wherever they are.
                    w.replace(['`', '*'], "").trim_matches(['#', '>', '_']).to_owned()
                }
            })
            .filter(|w| !w.is_empty())
            .collect();
        out.push_str(&words.join(" "));
        out.push(' ');
    }
    crate::agent::acp::truncate(out.split_whitespace().collect::<Vec<_>>().join(" ").as_str(), max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_names_code_and_links() {
        assert_eq!(
            spoken("**Done.** See https://x.dev\n```\nlet a = 1;\n```\n- `cargo test` passed", 200),
            "Done. See (link) (code) - cargo test passed"
        );
        assert!(spoken(&"word ".repeat(500), 50).chars().count() <= 51);
        assert_eq!(
            spoken("Run `cargo test --release`. **Done**, keep snake_case", 200),
            "Run cargo test --release. Done, keep snake_case"
        );
    }
}
