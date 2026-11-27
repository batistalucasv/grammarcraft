use craft_proof::{IssueKind, Options, ProofLang};
use grammarcraft_core::{Confidence, IssueCategory, Replacement, Severity};

use crate::RawFinding;

const MAX_SPELLING_SUGGESTIONS: usize = 5;

pub(crate) fn resolve_lang(tag: Option<&str>) -> (ProofLang, String) {
    let proof_lang = ProofLang::from_tag(tag);
    let canonical = tag
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "en-US".to_owned());
    (proof_lang, canonical)
}

pub(crate) fn check_paragraph(para_text: &str, lang: ProofLang) -> Vec<RawFinding> {
    let raw_issues = craft_proof::check_text_with(
        para_text,
        &[(0..para_text.len(), lang)],
        &Options::default(),
        true,
        false,
    );
    raw_issues
        .into_iter()
        .filter_map(|issue| convert_proof_issue(para_text, issue))
        .collect()
}

fn convert_proof_issue(para_text: &str, issue: craft_proof::Issue) -> Option<RawFinding> {
    let word = para_text.get(issue.start..issue.end)?;
    let (rule_id, severity, title, raw_suggestions) = match issue.kind {
        IssueKind::Spelling => {
            let suggestions = if issue.suggestions.is_empty() {
                craft_proof::suggest_in(word, MAX_SPELLING_SUGGESTIONS, issue.lang)
            } else {
                issue.suggestions
            };
            ("proof.spelling", Severity::Error, "Spelling", suggestions)
        }
        IssueKind::Grammar => (
            "proof.grammar",
            Severity::Warning,
            "Grammar",
            issue.suggestions,
        ),
    };
    let replacements = raw_suggestions
        .into_iter()
        .take(MAX_SPELLING_SUGGESTIONS)
        .map(Replacement::new)
        .collect();
    Some(RawFinding {
        rule_id,
        category: IssueCategory::SpellingAndGrammar,
        severity,
        confidence: Confidence::High,
        rel_start: issue.start,
        rel_end: issue.end,
        title: title.to_owned(),
        explanation: issue.message,
        replacements,
    })
}
