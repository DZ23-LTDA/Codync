//! How a run's answers become one result: what every bot agreed on, and where they didn't.
//!
//! Deliberately deterministic. Asking a model to summarize the models would hide disagreement and make the run
//! impossible to audit; here every conclusion keeps the bot that produced it, and a field only counts as agreed when
//! every answering bot gave the same value. Answers that ignored the schema are kept verbatim.

use serde_json::{Map, Value};

/// One bot's answer in a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub name: String,
    pub text: String,
}

/// A field the bots answered differently, with each bot's value.
#[derive(Clone, Debug, PartialEq)]
pub struct Disagreement {
    pub field: String,
    pub values: Vec<(String, Value)>,
}

/// What a run concluded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Merged {
    /// Fields every answering bot gave the same value for.
    pub agreed: Vec<(String, Value)>,
    /// Fields that differ, each with who said what.
    pub disagreed: Vec<Disagreement>,
    /// Answers that didn't follow the schema, kept as they came.
    pub unparsed: Vec<Answer>,
}

/// Merges `answers` against `schema`, a JSON object whose keys every answer must carry.
pub fn merge(answers: &[Answer], schema: &Value) -> Merged {
    let Some(fields) = schema.as_object() else { return Merged::default() };
    let mut merged = Merged::default();
    let mut parsed: Vec<(&Answer, Map<String, Value>)> = Vec::new();
    for answer in answers {
        match serde_json::from_str::<Value>(&answer.text) {
            Ok(Value::Object(map)) if fields.keys().all(|k| map.contains_key(k)) => parsed.push((answer, map)),
            _ => merged.unparsed.push(answer.clone()),
        }
    }
    for field in fields.keys() {
        let values: Vec<(String, Value)> =
            parsed.iter().map(|(answer, map)| (answer.name.clone(), map[field].clone())).collect();
        let Some((_, first)) = values.first() else { continue };
        if values.iter().all(|(_, value)| value == first) {
            merged.agreed.push((field.clone(), first.clone()));
        } else {
            merged.disagreed.push(Disagreement { field: field.clone(), values });
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn answer(name: &str, text: &str) -> Answer {
        Answer { name: name.to_owned(), text: text.to_owned() }
    }

    fn schema() -> Value {
        json!({"verdict": "string", "confidence": "number"})
    }

    #[test]
    fn the_same_answer_from_every_bot_is_agreed() {
        let answers = [
            answer("ravi", r#"{"verdict": "go", "confidence": 0.8}"#),
            answer("ana", r#"{"verdict": "go", "confidence": 0.8}"#),
        ];
        let merged = merge(&answers, &schema());
        assert_eq!(merged.agreed, vec![("verdict".to_owned(), json!("go")), ("confidence".to_owned(), json!(0.8))]);
        assert!(merged.disagreed.is_empty() && merged.unparsed.is_empty());
    }

    #[test]
    fn a_field_the_bots_split_on_keeps_every_value() {
        let answers = [
            answer("ravi", r#"{"verdict": "go", "confidence": 0.8}"#),
            answer("ana", r#"{"verdict": "hold", "confidence": 0.8}"#),
        ];
        let merged = merge(&answers, &schema());
        assert_eq!(merged.agreed, vec![("confidence".to_owned(), json!(0.8))]);
        assert_eq!(merged.disagreed.len(), 1);
        assert_eq!(merged.disagreed[0].field, "verdict");
        assert_eq!(
            merged.disagreed[0].values,
            vec![("ravi".to_owned(), json!("go")), ("ana".to_owned(), json!("hold"))]
        );
    }

    #[test]
    fn an_answer_that_ignores_the_contract_is_kept_verbatim() {
        let answers = [answer("ravi", r#"{"verdict": "go", "confidence": 0.8}"#), answer("ana", "go, trust me")];
        let merged = merge(&answers, &schema());
        assert_eq!(merged.agreed.len(), 2, "the one that followed the contract still counts");
        assert_eq!(merged.unparsed, vec![answer("ana", "go, trust me")]);
    }

    #[test]
    fn an_answer_missing_a_field_is_unparsed() {
        let answers =
            [answer("ravi", r#"{"verdict": "go", "confidence": 0.8}"#), answer("ana", r#"{"verdict": "go"}"#)];
        let merged = merge(&answers, &schema());
        assert_eq!(merged.unparsed.len(), 1);
        // Only the bot that answered counts: with one parsed answer, both fields are agreed with itself.
        assert_eq!(merged.agreed, vec![("verdict".to_owned(), json!("go")), ("confidence".to_owned(), json!(0.8))]);
    }

    #[test]
    fn no_answers_is_an_empty_conclusion() {
        let merged = merge(&[], &schema());
        assert!(merged.agreed.is_empty() && merged.disagreed.is_empty() && merged.unparsed.is_empty());
    }
}
