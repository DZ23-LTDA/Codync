//! Today's tokens and cost from Mistral Vibe's session logs. There is no plan
//! percent here — that lives behind a browser cookie — so these are notes,
//! not limit bars.

use std::path::Path;

use super::Window;

pub(super) fn collect() -> Option<Vec<Window>> {
    let root = dirs::home_dir()?.join(".vibe/logs/session");
    if !root.is_dir() {
        return None;
    }
    let now = crate::store::now_ms();
    let start = local_midnight(now)?;
    let mut tokens = 0.0;
    let mut cost = 0.0;
    let mut any = false;
    let entries = std::fs::read_dir(&root).ok()?;
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(at) = session_ms(name) else { continue };
        if at < start || at > now.saturating_add(86_400_000) {
            continue;
        }
        let meta = entry.path().join("meta.json");
        let Some((session_tokens, session_cost)) = read_meta(&meta) else { continue };
        any = true;
        tokens += session_tokens;
        cost += session_cost;
    }
    any.then(|| notes(tokens, cost))
}

fn notes(tokens: f64, cost: f64) -> Vec<Window> {
    let mut windows = Vec::new();
    if tokens > 0.0 {
        windows.push(Window::note("tokens", "Tokens today", token_text(tokens)));
    }
    if cost > 0.0 {
        windows.push(Window::note("cost", "Cost today", format!("${cost:.2}")));
    }
    windows
}

fn read_meta(path: &Path) -> Option<(f64, f64)> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let stats = &value["stats"];
    let tokens = stats["session_total_llm_tokens"].as_f64().unwrap_or(0.0);
    let cost = stats["session_cost"].as_f64().unwrap_or(0.0);
    Some((tokens, cost))
}

/// `session_YYYYMMDD_HHMMSS`, the stamp Vibe puts on the folder, in UTC.
fn session_ms(name: &str) -> Option<i64> {
    let stamp = name.strip_prefix("session_")?;
    let (date, time) = stamp.split_once('_')?;
    if date.len() != 8 || time.len() < 6 {
        return None;
    }
    let year: i32 = date[0..4].parse().ok()?;
    let month: u32 = date[4..6].parse().ok()?;
    let day: u32 = date[6..8].parse().ok()?;
    let hour: u32 = time[0..2].parse().ok()?;
    let minute: u32 = time[2..4].parse().ok()?;
    let second: u32 = time[4..6].parse().ok()?;
    let at = chrono::NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    Some(at.and_utc().timestamp_millis())
}

fn local_midnight(now_ms: i64) -> Option<i64> {
    let now = chrono::DateTime::from_timestamp_millis(now_ms)?.with_timezone(&chrono::Local);
    let day = now.date_naive();
    Some(day.and_hms_opt(0, 0, 0)?.and_local_timezone(chrono::Local).single()?.timestamp_millis())
}

fn token_text(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M tokens", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}k tokens", n / 1_000.0)
    } else {
        format!("{n:.0} tokens")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_stamp_is_utc_and_today_sums() {
        let at = session_ms("session_20261025_153045").unwrap();
        assert_eq!(
            at,
            chrono::NaiveDate::from_ymd_opt(2026, 10, 25)
                .unwrap()
                .and_hms_opt(15, 30, 45)
                .unwrap()
                .and_utc()
                .timestamp_millis()
        );
        let notes = notes(1_500_000.0, 1.5);
        assert_eq!(notes[0].text.as_deref(), Some("1.5M tokens"));
        assert_eq!(notes[1].text.as_deref(), Some("$1.50"));
    }
}
