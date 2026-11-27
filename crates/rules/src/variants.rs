use craft_proof::ProofLang;
use grammarcraft_core::{Confidence, IssueCategory, Replacement, Severity};

use crate::RawFinding;
use crate::clarity::word_spans;

const EN_US_GB_PAIRS: &[(&str, &str)] = &[
    ("color", "colour"),
    ("favor", "favour"),
    ("honor", "honour"),
    ("humor", "humour"),
    ("labor", "labour"),
    ("neighbor", "neighbour"),
    ("behavior", "behaviour"),
    ("center", "centre"),
    ("meter", "metre"),
    ("theater", "theatre"),
    ("analyze", "analyse"),
    ("organize", "organise"),
    ("realize", "realise"),
    ("catalog", "catalogue"),
    ("dialog", "dialogue"),
    ("canceled", "cancelled"),
    ("traveling", "travelling"),
];

const PT_BR_PT_PAIRS: &[(&str, &str)] = &[
    ("fato", "facto"),
    ("ótimo", "óptimo"),
    ("ação", "acção"),
    ("direto", "directo"),
    ("projeto", "projecto"),
    ("recepção", "receção"),
    ("adoção", "adopção"),
    ("equipe", "equipa"),
    ("trem", "comboio"),
    ("ônibus", "autocarro"),
];

pub(crate) fn check_paragraph(para_text: &str, lang: ProofLang) -> Vec<RawFinding> {
    let mut out = Vec::new();
    match lang {
        ProofLang::En => {
            scan_pairs(
                para_text,
                EN_US_GB_PAIRS,
                false,
                "variants.en_us",
                "en-US",
                &mut out,
            );
        }
        ProofLang::EnGb | ProofLang::EnAu | ProofLang::EnCa => {
            scan_pairs(
                para_text,
                EN_US_GB_PAIRS,
                true,
                "variants.en_gb",
                "en-GB",
                &mut out,
            );
        }
        ProofLang::Pt => {
            scan_pairs(
                para_text,
                PT_BR_PT_PAIRS,
                false,
                "variants.pt_br",
                "pt-BR",
                &mut out,
            );
        }
        ProofLang::Pack(locale) if locale.as_str() == "pt-pt" => {
            scan_pairs(
                para_text,
                PT_BR_PT_PAIRS,
                true,
                "variants.pt_pt",
                "pt-PT",
                &mut out,
            );
        }
        _ => {}
    }
    out
}

fn scan_pairs(
    para_text: &str,
    pairs: &[(&str, &str)],
    expect_first_as_wrong: bool,
    rule_id: &'static str,
    target_tag: &str,
    out: &mut Vec<RawFinding>,
) {
    for (start, end, word) in word_spans(para_text) {
        let lower = word.to_lowercase();
        let matched = pairs.iter().find_map(|&(preferred_a, preferred_b)| {
            if expect_first_as_wrong && lower == preferred_a {
                Some(preferred_b)
            } else if !expect_first_as_wrong && lower == preferred_b {
                Some(preferred_a)
            } else {
                None
            }
        });
        let Some(target_word) = matched else {
            continue;
        };
        let suggested = preserve_case(word, target_word);
        let is_pt = target_tag.starts_with("pt");
        let title = if is_pt {
            "Consistência regional".to_owned()
        } else {
            "Regional spelling consistency".to_owned()
        };
        let explanation = if is_pt {
            format!(
                "Grafia de outra variante regional para `{target_tag}`; considere `{suggested}`."
            )
        } else {
            format!("Mixed regional spelling for `{target_tag}`; consider `{suggested}`.")
        };
        out.push(RawFinding {
            rule_id,
            category: IssueCategory::SpellingAndGrammar,
            severity: Severity::Warning,
            confidence: Confidence::High,
            rel_start: start,
            rel_end: end,
            title,
            explanation,
            replacements: vec![Replacement::new(suggested)],
        });
    }
}

fn preserve_case(original: &str, target: &str) -> String {
    if original.chars().count() > 1 && original.chars().all(|c| !c.is_lowercase()) {
        return target.to_uppercase();
    }
    if original.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = target.chars();
        return match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        };
    }
    target.to_owned()
}
