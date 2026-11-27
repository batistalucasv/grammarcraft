use grammarcraft_core::{
    Confidence, CoreError, CoreResult, IssueCategory, Replacement, Severity, StyleGuide, StyleRule,
    StyleRuleKind,
};
use regex::{Regex, RegexBuilder};

use crate::RawFinding;

const MAX_REGEX_SIZE_BYTES: usize = 64 * 1024;

/// Validates that any `PatternRule` regex compiles within linear-time bounds.
pub fn validate_style_rule(rule: &StyleRule) -> CoreResult<()> {
    if let StyleRuleKind::PatternRule { pattern, .. } = &rule.kind {
        compile_bounded_regex(pattern)?;
    }
    Ok(())
}

pub(crate) fn check_paragraph(para_text: &str, guide: &StyleGuide) -> Vec<RawFinding> {
    if guide.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for rule in guide.rules() {
        match &rule.kind {
            StyleRuleKind::PreferredTerm {
                discouraged,
                preferred,
                rationale,
            } => eval_preferred_term(para_text, discouraged, preferred, rationale, &mut out),
            StyleRuleKind::BrandCasing { exact, rationale } => {
                eval_brand_casing(para_text, exact, rationale, &mut out);
            }
            StyleRuleKind::PatternRule {
                pattern,
                replacement,
                rationale,
            } => eval_pattern_rule(
                para_text,
                pattern,
                replacement.as_deref(),
                rationale,
                &mut out,
            ),
        }
    }
    out
}

fn eval_preferred_term(
    para_text: &str,
    discouraged: &str,
    preferred: &str,
    rationale: &str,
    out: &mut Vec<RawFinding>,
) {
    let lower = para_text.to_lowercase();
    let needle = discouraged.to_lowercase();
    if lower.len() != para_text.len() || needle.is_empty() {
        return;
    }
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(&needle)) {
        let start = cursor + rel;
        let end = start + needle.len();
        if word_boundaries(&lower, start, end)
            && let Some(orig) = para_text.get(start..end)
        {
            let suggested = match_initial_case(orig, preferred);
            out.push(RawFinding {
                rule_id: "style_guide.preferred_term",
                category: IssueCategory::StyleGuide,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: start,
                rel_end: end,
                title: "Preferred term".to_owned(),
                explanation: rationale.to_owned(),
                replacements: vec![Replacement::new(suggested)],
            });
        }
        cursor = end;
    }
}

fn eval_brand_casing(para_text: &str, exact: &str, rationale: &str, out: &mut Vec<RawFinding>) {
    let lower = para_text.to_lowercase();
    let needle = exact.to_lowercase();
    if lower.len() != para_text.len() || needle.is_empty() {
        return;
    }
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(&needle)) {
        let start = cursor + rel;
        let end = start + needle.len();
        if word_boundaries(&lower, start, end)
            && let Some(orig) = para_text.get(start..end)
            && orig != exact
        {
            out.push(RawFinding {
                rule_id: "style_guide.brand_casing",
                category: IssueCategory::StyleGuide,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: start,
                rel_end: end,
                title: "Brand capitalization".to_owned(),
                explanation: rationale.to_owned(),
                replacements: vec![Replacement::new(exact.to_owned())],
            });
        }
        cursor = end;
    }
}

fn eval_pattern_rule(
    para_text: &str,
    pattern: &str,
    replacement: Option<&str>,
    rationale: &str,
    out: &mut Vec<RawFinding>,
) {
    let Ok(regex) = compile_bounded_regex(pattern) else {
        return;
    };
    for matched in regex.find_iter(para_text) {
        if matched.start() == matched.end() {
            continue;
        }
        let replacements = replacement
            .map(|rep| vec![Replacement::new(rep.to_owned())])
            .unwrap_or_default();
        out.push(RawFinding {
            rule_id: "style_guide.pattern",
            category: IssueCategory::StyleGuide,
            severity: Severity::Warning,
            confidence: Confidence::High,
            rel_start: matched.start(),
            rel_end: matched.end(),
            title: "Style rule".to_owned(),
            explanation: rationale.to_owned(),
            replacements,
        });
    }
}

fn compile_bounded_regex(pattern: &str) -> CoreResult<Regex> {
    RegexBuilder::new(pattern)
        .size_limit(MAX_REGEX_SIZE_BYTES)
        .dfa_size_limit(MAX_REGEX_SIZE_BYTES)
        .build()
        .map_err(|err| CoreError::InvalidStyleRule(format!("invalid regex `{pattern}`: {err}")))
}

fn word_boundaries(text: &str, start: usize, end: usize) -> bool {
    let left_ok = text
        .get(..start)
        .and_then(|s| s.chars().next_back())
        .is_none_or(|c| !c.is_alphanumeric());
    let right_ok = text
        .get(end..)
        .and_then(|s| s.chars().next())
        .is_none_or(|c| !c.is_alphanumeric());
    left_ok && right_ok
}

fn match_initial_case(original: &str, replacement: &str) -> String {
    if original.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = replacement.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    } else {
        replacement.to_owned()
    }
}
