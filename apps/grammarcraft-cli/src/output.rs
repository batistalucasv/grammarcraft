use std::collections::BTreeMap;

use serde_json::{Value, json};

const SARIF_SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OutputFormat {
    #[default]
    Json,
    Text,
    Sarif,
}

impl OutputFormat {
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            "sarif" => Ok(Self::Sarif),
            other => Err(format!(
                "unsupported output format `{other}` (expected `text`, `json`, or `sarif`)"
            )),
        }
    }
}

pub(crate) fn render_text(report: &Value, doc_text: &str, artifact_uri: &str) -> String {
    let lang = report
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en-US");
    let score = report
        .get("score")
        .and_then(|s| s.get("score"))
        .and_then(Value::as_u64)
        .unwrap_or(100);
    let issues = report
        .get("issues")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut out = format!(
        "GrammarCraft report ({lang}) — Score: {score}/100 | Issues: {}\n",
        issues.len()
    );
    if issues.is_empty() {
        out.push_str("No issues found.\n");
        return out;
    }

    for issue in issues {
        let start = issue.get("start").and_then(Value::as_u64).unwrap_or(0) as usize;
        let (line, col) = byte_offset_to_line_col(doc_text, start);
        let severity = issue
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("warning");
        let rule_id = issue
            .get("rule_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let title = issue
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Issue");
        let explanation = issue
            .get("explanation")
            .and_then(Value::as_str)
            .unwrap_or("");
        out.push_str(&format!(
            "{artifact_uri}:{line}:{col}: [{severity}] ({rule_id}) {title}: {explanation}\n"
        ));
        if let Some(repls) = issue.get("replacements").and_then(Value::as_array) {
            let texts: Vec<&str> = repls
                .iter()
                .filter_map(|r| r.get("text").and_then(Value::as_str))
                .collect();
            if !texts.is_empty() {
                out.push_str(&format!("  suggestions: {}\n", texts.join(", ")));
            }
        }
    }
    out
}

pub(crate) fn render_sarif(
    report: &Value,
    doc_text: &str,
    artifact_uri: &str,
) -> Result<String, String> {
    let issues = report
        .get("issues")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut rules_map: BTreeMap<&str, &str> = BTreeMap::new();
    let mut results = Vec::with_capacity(issues.len());
    for issue in issues {
        let rule_id = issue
            .get("rule_id")
            .and_then(Value::as_str)
            .unwrap_or("grammarcraft.issue");
        let title = issue
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Issue");
        let explanation = issue
            .get("explanation")
            .and_then(Value::as_str)
            .unwrap_or("");
        let severity = issue
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("warning");
        let start = issue.get("start").and_then(Value::as_u64).unwrap_or(0) as usize;
        let end = issue.get("end").and_then(Value::as_u64).unwrap_or(0) as usize;
        let (start_line, start_col) = byte_offset_to_line_col(doc_text, start);
        let (end_line, end_col) = byte_offset_to_line_col(doc_text, end.max(start));
        let byte_length = end.saturating_sub(start);

        rules_map.entry(rule_id).or_insert(title);
        let message_text = if explanation.is_empty() {
            title.to_owned()
        } else {
            format!("{title}: {explanation}")
        };
        results.push(json!({
            "ruleId": rule_id,
            "level": sarif_level(severity),
            "message": { "text": message_text },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": { "uri": artifact_uri },
                    "region": {
                        "startLine": start_line,
                        "startColumn": start_col,
                        "endLine": end_line,
                        "endColumn": end_col,
                        "byteOffset": start,
                        "byteLength": byte_length,
                    }
                }
            }]
        }));
    }

    let rules: Vec<Value> = rules_map
        .into_iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect();

    let sarif = json!({
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "GrammarCraft",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/batistalucasv/grammarcraft",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    });

    serde_json::to_string_pretty(&sarif)
        .map_err(|error| format!("could not print SARIF JSON: {error}"))
}

fn sarif_level(severity: &str) -> &'static str {
    match severity {
        "error" => "error",
        "warning" => "warning",
        _ => "note",
    }
}

fn byte_offset_to_line_col(text: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, ch) in text.char_indices() {
        if idx >= byte_offset {
            break;
        }
        if ch == '\n' {
            line = line.saturating_add(1);
            col = 1;
        } else {
            col = col.saturating_add(1);
        }
    }
    (line, col)
}
