//! A run: one task given to several bots at once, and the single result they add up to.
//!
//! `merge` is the join — deterministic, so a run can be audited and the disagreement it found isn't hidden by a
//! summary. `merge_answers` hands it to a bot that already collected the answers; the fan-out, the ledger and the
//! seal are built on top of it (`docs/plans/run-orchestration.md`).

pub mod merge;

use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// `merge_answers` for the team server: several bots' answers to one task, lined up.
pub fn tool(args: &Value) -> Result<Value> {
    let schema = args.get("schema").filter(|s| s.is_object()).ok_or_else(|| {
        anyhow::anyhow!("schema is required and must be a JSON object of the fields each answer carries")
    })?;
    let Some(answers) = args["answers"].as_array() else { bail!("answers is required") };
    let answers: Vec<merge::Answer> = answers
        .iter()
        .map(|a| merge::Answer {
            name: a["name"].as_str().unwrap_or_default().to_owned(),
            text: a["text"].as_str().unwrap_or_default().to_owned(),
        })
        .collect();
    let merged = merge::merge(&answers, schema);
    let agreed: BTreeMap<&String, &Value> = merged.agreed.iter().map(|(field, value)| (field, value)).collect();
    let disagreed: Vec<Value> = merged
        .disagreed
        .iter()
        .map(|d| {
            json!({
                "field": d.field,
                "values": d.values.iter().map(|(name, value)| json!({"name": name, "value": value})).collect::<Vec<_>>(),
            })
        })
        .collect();
    let unparsed: Vec<Value> = merged.unparsed.iter().map(|a| json!({"name": a.name, "text": a.text})).collect();
    Ok(json!({"agreed": agreed, "disagreed": disagreed, "unparsed": unparsed}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_lines_up_the_answers_it_is_given() {
        let args = json!({"schema": {"verdict": "string"}, "answers": [
            {"bot": "b1", "name": "Ravi", "text": "{\"verdict\": \"go\"}"},
            {"bot": "b2", "name": "Ana", "text": "{\"verdict\": \"hold\"}"},
            {"bot": "b3", "name": "Lia", "text": "no idea"},
        ]});
        let out = tool(&args).expect("a merge");
        assert_eq!(out["agreed"], json!({}));
        assert_eq!(out["disagreed"][0]["field"], "verdict");
        assert_eq!(out["disagreed"][0]["values"][1]["value"], "hold");
        assert_eq!(out["unparsed"][0]["name"], "Lia");
    }

    #[test]
    fn a_schema_that_is_not_an_object_is_refused() {
        let err = tool(&json!({"schema": ["verdict"], "answers": []})).expect_err("refused");
        assert!(err.to_string().contains("JSON object"), "{err}");
        assert!(tool(&json!({"answers": []})).is_err(), "no schema at all");
    }
}
