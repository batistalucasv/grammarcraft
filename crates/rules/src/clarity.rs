use craft_proof::ProofLang;
use grammarcraft_core::{
    Audience, Confidence, Context, IssueCategory, Replacement, Severity, WritingGoals,
};

use crate::RawFinding;

const REPETITION_WINDOW_WORDS: usize = 12;
const LONG_SENTENCE_DEFAULT_WORDS: usize = 30;
const LONG_SENTENCE_TECHNICAL_WORDS: usize = 40;

const EN_REDUNDANCIES: &[(&str, &str)] = &[
    ("due to the fact that", "because"),
    ("at this point in time", "now"),
    ("for the purpose of", "for"),
    ("each and every", "each"),
    ("in order to", "to"),
    ("very unique", "unique"),
];

const PT_REDUNDANCIES: &[(&str, &str)] = &[
    ("devido ao fato de que", "porque"),
    ("com o intuito de", "para"),
    ("entrar para dentro", "entrar"),
    ("sair para fora", "sair"),
    ("elo de ligação", "elo"),
    ("há anos atrás", "há anos"),
];

const EN_AUXILIARIES: &[&str] = &["is", "are", "was", "were", "be", "been", "being"];
const EN_IRREGULAR_PARTICIPLES: &[&str] = &[
    "written", "built", "made", "sent", "reviewed", "checked", "given", "taken", "shown", "found",
    "done", "seen", "known", "broken", "chosen", "spoken",
];
const EN_NOT_PARTICIPLES: &[&str] = &[
    "bed", "red", "need", "speed", "seed", "feed", "shed", "sled", "bleed", "breed", "hundred",
    "tired", "excited",
];

const PT_AUXILIARIES: &[&str] = &[
    "é", "são", "foi", "foram", "era", "eram", "será", "serão", "sendo", "sido",
];
const PT_IRREGULAR_PARTICIPLES: &[&str] = &[
    "feito", "feita", "feitos", "feitas", "escrito", "escrita", "dito", "dita", "visto", "vista",
    "aberto", "aberta", "posto", "posta",
];
const PT_NOT_PARTICIPLES: &[&str] = &[
    "estado",
    "lado",
    "resultado",
    "conteúdo",
    "sentido",
    "período",
    "rápido",
    "sólido",
    "válido",
    "líquido",
    "vida",
    "medida",
    "saída",
    "entrada",
    "partido",
    "marido",
];

const STOP_WORDS: &[&str] = &[
    "that", "this", "with", "from", "they", "them", "have", "were", "been", "will", "would",
    "could", "should", "there", "their", "about", "which", "when", "where", "what", "into", "para",
    "como", "mais", "muito", "pela", "pelo", "pelos", "pelas", "sobre", "entre", "esta", "este",
    "isso", "isto", "esse", "essa", "aqui", "onde", "quando", "porque", "também",
];

pub(crate) fn check_paragraph(
    para_text: &str,
    lang: ProofLang,
    goals: WritingGoals,
) -> Vec<RawFinding> {
    let is_pt = is_portuguese(lang);
    let mut out = Vec::new();
    check_redundancies(para_text, is_pt, &mut out);
    check_passive_voice(para_text, is_pt, goals, &mut out);
    check_long_sentences(para_text, is_pt, goals, &mut out);
    check_repetition(para_text, is_pt, &mut out);
    out
}

pub(crate) fn is_portuguese(lang: ProofLang) -> bool {
    matches!(lang, ProofLang::Pt)
        || lang
            .pack_locale()
            .is_some_and(|locale| locale.primary() == "pt")
}

fn check_redundancies(para_text: &str, is_pt: bool, out: &mut Vec<RawFinding>) {
    let lower = para_text.to_lowercase();
    if lower.len() != para_text.len() {
        // Skip byte-offset alignment when case folding changes byte lengths.
        return;
    }
    let table = if is_pt {
        PT_REDUNDANCIES
    } else {
        EN_REDUNDANCIES
    };
    for &(phrase, replacement) in table {
        let mut cursor = 0usize;
        while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(phrase)) {
            let start = cursor + rel;
            let end = start + phrase.len();
            if is_word_boundary(&lower, start, end)
                && let Some(original) = para_text.get(start..end)
            {
                let suggested = match_initial_case(original, replacement);
                let explanation = if is_pt {
                    format!("Expressão prolixa ou redundante; prefira `{suggested}`.")
                } else {
                    format!("Wordy or redundant phrasing; consider `{suggested}`.")
                };
                out.push(RawFinding {
                    rule_id: "clarity.redundancy",
                    category: IssueCategory::ClarityAndConciseness,
                    severity: Severity::Suggestion,
                    confidence: Confidence::High,
                    rel_start: start,
                    rel_end: end,
                    title: if is_pt {
                        "Redundância".to_owned()
                    } else {
                        "Conciseness".to_owned()
                    },
                    explanation,
                    replacements: vec![Replacement::new(suggested)],
                });
            }
            cursor = end;
        }
    }
}

fn check_passive_voice(
    para_text: &str,
    is_pt: bool,
    goals: WritingGoals,
    out: &mut Vec<RawFinding>,
) {
    if goals.context == Context::Academic && goals.audience == Audience::Specialized {
        return;
    }
    let spans = word_spans(para_text);
    for pair in spans.windows(2) {
        let (a_start, a_end, aux) = pair[0];
        let (_, p_end, part) = pair[1];
        if !gap_is_plain_spaces(para_text, a_end, pair[1].0) {
            continue;
        }
        let aux_lower = aux.to_lowercase();
        let part_lower = part.to_lowercase();
        let matched = if is_pt {
            PT_AUXILIARIES.contains(&aux_lower.as_str()) && is_pt_participle(&part_lower)
        } else {
            EN_AUXILIARIES.contains(&aux_lower.as_str()) && is_en_participle(&part_lower)
        };
        if matched {
            out.push(RawFinding {
                rule_id: "clarity.passive_voice",
                category: IssueCategory::ClarityAndConciseness,
                severity: Severity::Suggestion,
                confidence: Confidence::Medium,
                rel_start: a_start,
                rel_end: p_end,
                title: if is_pt {
                    "Voz passiva".to_owned()
                } else {
                    "Passive voice".to_owned()
                },
                explanation: if is_pt {
                    "Construção na voz passiva; considere reescrever na voz ativa para maior clareza."
                        .to_owned()
                } else {
                    "Passive voice construction; consider rewriting in active voice for directness."
                        .to_owned()
                },
                replacements: Vec::new(),
            });
        }
    }
}

fn check_long_sentences(
    para_text: &str,
    is_pt: bool,
    goals: WritingGoals,
    out: &mut Vec<RawFinding>,
) {
    let limit = match goals.context {
        Context::Academic | Context::Technical => LONG_SENTENCE_TECHNICAL_WORDS,
        _ => LONG_SENTENCE_DEFAULT_WORDS,
    };
    for (start, end, sentence) in sentence_spans(para_text) {
        let word_count = word_spans(sentence).len();
        if word_count > limit {
            out.push(RawFinding {
                rule_id: "clarity.long_sentence",
                category: IssueCategory::ClarityAndConciseness,
                severity: Severity::Suggestion,
                confidence: Confidence::Medium,
                rel_start: start,
                rel_end: end,
                title: if is_pt {
                    "Frase longa".to_owned()
                } else {
                    "Long sentence".to_owned()
                },
                explanation: if is_pt {
                    format!(
                        "Esta frase tem {word_count} palavras (limite sugerido: {limit}); considere dividi-la."
                    )
                } else {
                    format!(
                        "This sentence has {word_count} words (suggested limit: {limit}); consider splitting it."
                    )
                },
                replacements: Vec::new(),
            });
        }
    }
}

fn check_repetition(para_text: &str, is_pt: bool, out: &mut Vec<RawFinding>) {
    let spans = word_spans(para_text);
    let mut recent: Vec<(usize, String)> = Vec::new();
    for (idx, (start, end, word)) in spans.into_iter().enumerate() {
        if word.chars().count() < 4 {
            continue;
        }
        let lower = word.to_lowercase();
        if STOP_WORDS.contains(&lower.as_str()) {
            continue;
        }
        let repeated = recent.iter().any(|(prev_idx, prev_word)| {
            idx - *prev_idx <= REPETITION_WINDOW_WORDS && prev_word == &lower
        });
        if repeated {
            out.push(RawFinding {
                rule_id: "style.repetition",
                category: IssueCategory::StyleAndVocabulary,
                severity: Severity::Suggestion,
                confidence: Confidence::Low,
                rel_start: start,
                rel_end: end,
                title: if is_pt {
                    "Repetição vocabular".to_owned()
                } else {
                    "Repeated word".to_owned()
                },
                explanation: if is_pt {
                    format!("A palavra `{word}` aparece repetida em curto intervalo; avalie usar um sinônimo.")
                } else {
                    format!("`{word}` is repeated nearby; consider varying your word choice.")
                },
                replacements: Vec::new(),
            });
        }
        recent.push((idx, lower));
        if recent.len() > REPETITION_WINDOW_WORDS {
            recent.remove(0);
        }
    }
}

fn is_en_participle(lower: &str) -> bool {
    if EN_IRREGULAR_PARTICIPLES.contains(&lower) {
        return true;
    }
    lower.ends_with("ed") && lower.len() >= 4 && !EN_NOT_PARTICIPLES.contains(&lower)
}

fn is_pt_participle(lower: &str) -> bool {
    if PT_IRREGULAR_PARTICIPLES.contains(&lower) {
        return true;
    }
    let suffix_match = ["ado", "ada", "ados", "adas", "ido", "ida", "idos", "idas"]
        .iter()
        .any(|suf| lower.ends_with(suf));
    suffix_match && lower.chars().count() >= 5 && !PT_NOT_PARTICIPLES.contains(&lower)
}

pub(crate) fn word_spans(text: &str) -> Vec<(usize, usize, &str)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    for (idx, ch) in text.char_indices() {
        let is_word = ch.is_alphanumeric() || ch == '-' || ch == '\'';
        match (start, is_word) {
            (None, true) => start = Some(idx),
            (Some(s), false) => {
                push_trimmed_word(text, s, idx, &mut spans);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        push_trimmed_word(text, s, text.len(), &mut spans);
    }
    spans
}

fn push_trimmed_word<'a>(
    text: &'a str,
    raw_start: usize,
    raw_end: usize,
    out: &mut Vec<(usize, usize, &'a str)>,
) {
    let Some(slice) = text.get(raw_start..raw_end) else {
        return;
    };
    let trimmed_start = slice.trim_start_matches(['-', '\'']);
    let lead = slice.len() - trimmed_start.len();
    let trimmed = trimmed_start.trim_end_matches(['-', '\'']);
    if trimmed.is_empty() || !trimmed.chars().any(char::is_alphabetic) {
        return;
    }
    let start = raw_start + lead;
    let end = start + trimmed.len();
    out.push((start, end, trimmed));
}

fn sentence_spans(text: &str) -> Vec<(usize, usize, &str)> {
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if !matches!(ch, '.' | '!' | '?' | '…') {
            continue;
        }
        let end = idx + ch.len_utf8();
        let followed_by_space = chars.peek().is_none_or(|(_, next)| next.is_whitespace());
        if followed_by_space {
            if let Some(raw) = text.get(start..end) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    let lead = raw.len() - raw.trim_start().len();
                    let s = start + lead;
                    sentences.push((s, s + trimmed.len(), trimmed));
                }
            }
            start = end;
        }
    }
    if start < text.len()
        && let Some(raw) = text.get(start..)
    {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let lead = raw.len() - raw.trim_start().len();
            let s = start + lead;
            sentences.push((s, s + trimmed.len(), trimmed));
        }
    }
    sentences
}

fn gap_is_plain_spaces(text: &str, left_end: usize, right_start: usize) -> bool {
    text.get(left_end..right_start)
        .is_some_and(|gap| !gap.is_empty() && gap.chars().all(|c| c == ' '))
}

fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
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
    let first_upper = original.chars().next().is_some_and(char::is_uppercase);
    if !first_upper {
        return replacement.to_owned();
    }
    let mut chars = replacement.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
