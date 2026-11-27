use craft_proof::ProofLang;
use grammarcraft_core::{
    Confidence, Formality, IssueCategory, Replacement, Severity, WritingGoals,
};

use crate::RawFinding;
use crate::clarity::{is_portuguese, is_spanish, is_word_boundary, match_initial_case};

const EN_FORMAL_CONTRACTIONS: &[(&str, &str)] = &[
    ("don't", "do not"),
    ("can't", "cannot"),
    ("won't", "will not"),
    ("isn't", "is not"),
    ("aren't", "are not"),
    ("didn't", "did not"),
    ("doesn't", "does not"),
    ("wasn't", "was not"),
    ("weren't", "were not"),
    ("it's", "it is"),
    ("i'm", "I am"),
    ("you're", "you are"),
    ("we're", "we are"),
    ("they're", "they are"),
    ("gonna", "going to"),
    ("wanna", "want to"),
    ("a lot of", "many"),
];

const PT_FORMAL_TERMS: &[(&str, &str)] = &[
    ("pra", "para"),
    ("pro", "para o"),
    ("tá", "está"),
    ("né", "não é"),
];

const ES_FORMAL_TERMS: &[(&str, &str)] = &[("pa'", "para"), ("pa", "para")];

const EN_INCLUSIVE_TERMS: &[(&str, &str, &str)] = &[
    (
        "chairman",
        "chairperson",
        "Use gender-neutral `chairperson` or `chair`.",
    ),
    (
        "chairmen",
        "chairpersons",
        "Use gender-neutral `chairpersons`.",
    ),
    ("mankind", "humanity", "Consider gender-neutral `humanity`."),
    (
        "manpower",
        "workforce",
        "Consider gender-neutral `workforce` or `staffing`.",
    ),
    (
        "fireman",
        "firefighter",
        "Use gender-neutral `firefighter`.",
    ),
    (
        "policeman",
        "police officer",
        "Use gender-neutral `police officer`.",
    ),
    (
        "blacklist",
        "denylist",
        "Prefer clearer, inclusive `denylist` or `blocklist`.",
    ),
    (
        "whitelist",
        "allowlist",
        "Prefer clearer, inclusive `allowlist`.",
    ),
    (
        "sanity check",
        "consistency check",
        "Prefer `consistency check` or `quick check`.",
    ),
];

const PT_INCLUSIVE_TERMS: &[(&str, &str, &str)] = &[
    (
        "lista negra",
        "lista de bloqueio",
        "Prefira o termo inclusivo e direto `lista de bloqueio`.",
    ),
    (
        "lista branca",
        "lista de permissão",
        "Prefira o termo inclusivo e direto `lista de permissão`.",
    ),
];

const ES_INCLUSIVE_TERMS: &[(&str, &str, &str)] = &[
    (
        "lista negra",
        "lista de bloqueo",
        "Prefiera el término inclusivo y claro `lista de bloqueo`.",
    ),
    (
        "lista blanca",
        "lista de permitidos",
        "Prefiera el término inclusivo y claro `lista de permitidos`.",
    ),
];

pub(crate) fn check_paragraph(
    para_text: &str,
    lang: ProofLang,
    goals: WritingGoals,
) -> Vec<RawFinding> {
    let mut out = Vec::new();
    if goals.formality == Formality::Formal {
        check_formality(para_text, lang, &mut out);
    }
    if goals.inclusive_language {
        check_inclusive(para_text, lang, &mut out);
    }
    out
}

fn check_formality(para_text: &str, lang: ProofLang, out: &mut Vec<RawFinding>) {
    let lower = para_text.to_lowercase();
    if lower.len() != para_text.len() {
        return;
    }
    let is_pt = is_portuguese(lang);
    let is_es = is_spanish(lang);
    let table = if is_pt {
        PT_FORMAL_TERMS
    } else if is_es {
        ES_FORMAL_TERMS
    } else {
        EN_FORMAL_CONTRACTIONS
    };
    for &(informal, formal) in table {
        find_phrase_matches(para_text, &lower, informal, |start, end, original| {
            let suggestion = match_initial_case(original, formal);
            out.push(RawFinding {
                rule_id: "clarity.formality",
                category: IssueCategory::ToneAndAppropriateness,
                severity: Severity::Suggestion,
                confidence: Confidence::High,
                rel_start: start,
                rel_end: end,
                title: if is_pt {
                    "Registro informal em meta formal".to_owned()
                } else if is_es {
                    "Registro coloquial en texto formal".to_owned()
                } else {
                    "Informal wording in formal context".to_owned()
                },
                explanation: format!(
                    "In formal writing, replace `{original}` with `{suggestion}`."
                ),
                replacements: vec![Replacement::new(suggestion)],
            });
        });
    }
}

fn check_inclusive(para_text: &str, lang: ProofLang, out: &mut Vec<RawFinding>) {
    let lower = para_text.to_lowercase();
    if lower.len() != para_text.len() {
        return;
    }
    let table = if is_portuguese(lang) {
        PT_INCLUSIVE_TERMS
    } else if is_spanish(lang) {
        ES_INCLUSIVE_TERMS
    } else {
        EN_INCLUSIVE_TERMS
    };
    for &(term, preferred, note) in table {
        find_phrase_matches(para_text, &lower, term, |start, end, original| {
            let suggestion = match_initial_case(original, preferred);
            out.push(RawFinding {
                rule_id: "style.inclusive",
                category: IssueCategory::StyleAndVocabulary,
                severity: Severity::Suggestion,
                confidence: Confidence::Medium,
                rel_start: start,
                rel_end: end,
                title: "Inclusive language suggestion".to_owned(),
                explanation: (*note).to_owned(),
                replacements: vec![Replacement::new(suggestion)],
            });
        });
    }
}

fn find_phrase_matches(
    para_text: &str,
    lower: &str,
    phrase: &str,
    mut on_match: impl FnMut(usize, usize, &str),
) {
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(phrase)) {
        let start = cursor + rel;
        let end = start + phrase.len();
        if is_word_boundary(lower, start, end)
            && let Some(original) = para_text.get(start..end)
        {
            on_match(start, end, original);
        }
        cursor = end;
    }
}
