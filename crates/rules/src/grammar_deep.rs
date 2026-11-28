use craft_proof::ProofLang;
use grammarcraft_core::{Confidence, IssueCategory, Replacement, Severity};

use crate::RawFinding;
use crate::clarity::{is_portuguese, is_spanish, is_word_boundary, match_initial_case, word_spans};

const EN_CONJUNCTIVE_ADVERBS: &[(&str, &str)] = &[
    ("however", "However"),
    ("therefore", "Therefore"),
    ("moreover", "Moreover"),
    ("furthermore", "Furthermore"),
];

const EN_COUNTABLE_PLURALS: &[&str] = &[
    "apples",
    "books",
    "bugs",
    "changes",
    "clicks",
    "days",
    "errors",
    "files",
    "hours",
    "issues",
    "items",
    "lines",
    "minutes",
    "mistakes",
    "options",
    "pages",
    "people",
    "problems",
    "sentences",
    "steps",
    "tests",
    "users",
    "warnings",
    "words",
];

const PT_CRASE_RULES: &[(&str, &str, &str, &str)] = &[
    (
        "a medida que",
        "a medida que",
        "à medida que",
        "Locuções conjuntivas proporcionais femininas exigem crase (`à medida que`).",
    ),
    (
        "a proporção que",
        "a proporção que",
        "à proporção que",
        "Locuções proporcionais exigem crase (`à proporção que`).",
    ),
    (
        "à partir",
        "à partir",
        "a partir",
        "Não se usa crase antes de verbo no infinitivo (`a partir`).",
    ),
    (
        "dia à dia",
        "dia à dia",
        "dia a dia",
        "Expressões com palavras repetidas não levam crase (`dia a dia`).",
    ),
    (
        "frente à frente",
        "frente à frente",
        "frente a frente",
        "Expressões com palavras repetidas não levam crase (`frente a frente`).",
    ),
    (
        "lado à lado",
        "lado à lado",
        "lado a lado",
        "Expressões com palavras repetidas não levam crase (`lado a lado`).",
    ),
    (
        "cara à cara",
        "cara à cara",
        "cara a cara",
        "Expressões com palavras repetidas não levam crase (`cara a cara`).",
    ),
    (
        "gota à gota",
        "gota à gota",
        "gota a gota",
        "Expressões com palavras repetidas não levam crase (`gota a gota`).",
    ),
    (
        "a nível de",
        "a nível de",
        "em nível de",
        "Prefira `em nível de` em vez da locução `a nível de` na norma culta.",
    ),
];

const PT_TIME_UNITS: &[&str] = &["anos", "meses", "dias", "semanas", "horas", "minutos"];
const PT_COUNT_WORDS: &[&str] = &[
    "dois", "duas", "três", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez", "muitos",
    "muitas", "vários", "várias", "alguns", "algumas", "poucos", "poucas",
];

const ES_DEQUEISMO_VERBS: &[&str] = &["pienso", "creo", "opino", "considero", "digo"];

pub(crate) fn check_paragraph(para_text: &str, lang: ProofLang) -> Vec<RawFinding> {
    let lower = para_text.to_lowercase();
    if lower.len() != para_text.len() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if is_portuguese(lang) {
        check_pt_crase(para_text, &lower, &mut out);
        check_pt_existential(para_text, &lower, &mut out);
    } else if is_spanish(lang) {
        check_es_dequeismo(para_text, &lower, &mut out);
        check_es_existential(para_text, &lower, &mut out);
    } else if lang.is_english() {
        check_en_comma_splices(para_text, &lower, &mut out);
        check_en_agreement(para_text, &mut out);
        check_en_less_fewer(para_text, &mut out);
    }
    out
}

fn check_en_comma_splices(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    for &(adv, cap_adv) in EN_CONJUNCTIVE_ADVERBS {
        let needle = format!(", {adv}, ");
        let mut cursor = 0usize;
        while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(&needle)) {
            let start = cursor + rel;
            let end = start + needle.len();
            let before_ok = para_text
                .get(..start)
                .and_then(|s| s.chars().next_back())
                .is_some_and(char::is_alphanumeric);
            let after_ok = para_text
                .get(end..)
                .and_then(|s| s.chars().next())
                .is_some_and(char::is_alphabetic);
            if before_ok && after_ok {
                out.push(RawFinding {
                    rule_id: "grammar.comma_splice",
                    category: IssueCategory::SpellingAndGrammar,
                    severity: Severity::Warning,
                    confidence: Confidence::Medium,
                    rel_start: start,
                    rel_end: end,
                    title: "Possible comma splice".to_owned(),
                    explanation: format!(
                        "A comma before `{adv}` between independent clauses often creates a comma splice; use `; {adv}, ` or start a new sentence."
                    ),
                    replacements: vec![
                        Replacement::new(format!("; {adv}, ")),
                        Replacement::new(format!(". {cap_adv}, ")),
                    ],
                });
            }
            cursor = end;
        }
    }
}

fn check_en_agreement(para_text: &str, out: &mut Vec<RawFinding>) {
    let spans = word_spans(para_text);
    for idx in 0..spans.len() {
        let head = spans[idx].2.to_lowercase();
        let pattern_len = if head == "the"
            && spans
                .get(idx + 1)
                .is_some_and(|w| w.2.eq_ignore_ascii_case("list"))
            && spans
                .get(idx + 2)
                .is_some_and(|w| w.2.eq_ignore_ascii_case("of"))
        {
            Some((
                3usize,
                "Use singular verb `is` with singular subject `the list`.",
            ))
        } else if head == "each"
            && spans
                .get(idx + 1)
                .is_some_and(|w| w.2.eq_ignore_ascii_case("of"))
            && spans
                .get(idx + 2)
                .is_some_and(|w| w.2.eq_ignore_ascii_case("the"))
        {
            Some((
                3usize,
                "Use singular verb `is` after singular pronoun `each`.",
            ))
        } else if head == "neither"
            && spans
                .get(idx + 1)
                .is_some_and(|w| w.2.eq_ignore_ascii_case("of"))
        {
            Some((
                2usize,
                "In formal prose, `neither of ...` takes singular `is`.",
            ))
        } else {
            None
        };

        let Some((prefix_words, explanation)) = pattern_len else {
            continue;
        };
        let search_start = idx + prefix_words;
        let search_end = (search_start + 4).min(spans.len());
        for &(a_start, a_end, orig) in spans.get(search_start..search_end).unwrap_or(&[]) {
            let lower_cand = orig.to_lowercase();
            if matches!(lower_cand.as_str(), "and" | "who" | "that" | "which" | "is") {
                break;
            }
            if lower_cand == "are" {
                out.push(RawFinding {
                    rule_id: "grammar.subject_verb_agreement",
                    category: IssueCategory::SpellingAndGrammar,
                    severity: Severity::Warning,
                    confidence: Confidence::High,
                    rel_start: a_start,
                    rel_end: a_end,
                    title: "Subject-verb agreement".to_owned(),
                    explanation: explanation.to_owned(),
                    replacements: vec![Replacement::new(match_initial_case(orig, "is"))],
                });
                break;
            }
        }
    }
}

fn check_en_less_fewer(para_text: &str, out: &mut Vec<RawFinding>) {
    let spans = word_spans(para_text);
    for pair in spans.windows(2) {
        let (l_start, l_end, less_word) = pair[0];
        let (n_start, _, noun_word) = pair[1];
        if !less_word.eq_ignore_ascii_case("less") {
            continue;
        }
        let gap_ok = para_text
            .get(l_end..n_start)
            .is_some_and(|g| !g.is_empty() && g.chars().all(|c| c == ' '));
        if !gap_ok {
            continue;
        }
        let noun_lower = noun_word.to_lowercase();
        if EN_COUNTABLE_PLURALS.contains(&noun_lower.as_str()) {
            out.push(RawFinding {
                rule_id: "grammar.less_vs_fewer",
                category: IssueCategory::SpellingAndGrammar,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: l_start,
                rel_end: l_end,
                title: "Countable noun quantifier".to_owned(),
                explanation: format!(
                    "Use `fewer` instead of `less` before countable noun `{noun_word}`."
                ),
                replacements: vec![Replacement::new(match_initial_case(less_word, "fewer"))],
            });
        }
    }
}

fn check_pt_crase(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    for &(needle, target_sub, replacement, explanation) in PT_CRASE_RULES {
        let mut cursor = 0usize;
        while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(needle)) {
            let start = cursor + rel;
            let end = start + needle.len();
            if is_word_boundary(lower, start, end)
                && let Some(sub_rel) = needle.find(target_sub)
            {
                let t_start = start + sub_rel;
                let t_end = t_start + target_sub.len();
                if let Some(orig) = para_text.get(t_start..t_end) {
                    out.push(RawFinding {
                        rule_id: "grammar.pt_crase",
                        category: IssueCategory::SpellingAndGrammar,
                        severity: Severity::Warning,
                        confidence: Confidence::High,
                        rel_start: t_start,
                        rel_end: t_end,
                        title: "Uso da crase".to_owned(),
                        explanation: explanation.to_owned(),
                        replacements: vec![Replacement::new(match_initial_case(orig, replacement))],
                    });
                }
            }
            cursor = end;
        }
    }
}

fn check_pt_existential(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find("houveram")) {
        let start = cursor + rel;
        let end = start + "houveram".len();
        let tail_ok = lower
            .get(end..)
            .is_some_and(|rest| !rest.trim_start().starts_with("por bem"));
        if is_word_boundary(lower, start, end)
            && tail_ok
            && let Some(orig) = para_text.get(start..end)
        {
            out.push(RawFinding {
                rule_id: "grammar.pt_existential",
                category: IssueCategory::SpellingAndGrammar,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: start,
                rel_end: end,
                title: "Verbo impessoal `haver`".to_owned(),
                explanation: "No sentido de existir, o verbo `haver` é impessoal e fica no singular (`houve`).".to_owned(),
                replacements: vec![Replacement::new(match_initial_case(orig, "houve"))],
            });
        }
        cursor = end;
    }

    let tokens = raw_space_tokens(para_text);
    for win in tokens.windows(3) {
        let (f_start, f_end, verb) = win[0];
        let (_, _, mid) = win[1];
        let (_, _, unit) = win[2];
        if !verb.eq_ignore_ascii_case("fazem") {
            continue;
        }
        let mid_lower = mid.to_lowercase();
        let unit_lower = unit
            .trim_end_matches(['.', ',', ';', '!', '?'])
            .to_lowercase();
        let is_count =
            mid.chars().all(|c| c.is_ascii_digit()) || PT_COUNT_WORDS.contains(&mid_lower.as_str());
        if is_count && PT_TIME_UNITS.contains(&unit_lower.as_str()) {
            out.push(RawFinding {
                rule_id: "grammar.pt_existential",
                category: IssueCategory::SpellingAndGrammar,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: f_start,
                rel_end: f_end,
                title: "Verbo impessoal `fazer`".to_owned(),
                explanation: "Indicando tempo decorrido, o verbo `fazer` é impessoal e permanece no singular (`faz`).".to_owned(),
                replacements: vec![Replacement::new(match_initial_case(verb, "faz"))],
            });
        }
    }
}

fn check_es_dequeismo(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    for &verb in ES_DEQUEISMO_VERBS {
        let needle = format!("{verb} de que");
        let mut cursor = 0usize;
        while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(&needle)) {
            let start = cursor + rel;
            let end = start + needle.len();
            if is_word_boundary(lower, start, end) {
                let de_que_start = start + verb.len() + 1;
                if let Some(orig) = para_text.get(de_que_start..end) {
                    out.push(RawFinding {
                        rule_id: "grammar.es_dequeismo",
                        category: IssueCategory::SpellingAndGrammar,
                        severity: Severity::Warning,
                        confidence: Confidence::High,
                        rel_start: de_que_start,
                        rel_end: end,
                        title: "Posible dequeísmo".to_owned(),
                        explanation: format!(
                            "El verbo `{verb}` introduce subordinadas con `que`, sin la preposición `de` (`{verb} que`)."
                        ),
                        replacements: vec![Replacement::new(match_initial_case(orig, "que"))],
                    });
                }
            }
            cursor = end;
        }
    }
}

fn check_es_existential(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find("hubieron")) {
        let start = cursor + rel;
        let end = start + "hubieron".len();
        let next_word = lower
            .get(end..)
            .and_then(|rest| rest.split_whitespace().next())
            .map(|w| w.trim_end_matches(|c: char| !c.is_alphabetic()))
            .unwrap_or("");
        let looks_participle = next_word.ends_with("ado")
            || next_word.ends_with("ido")
            || matches!(
                next_word,
                "hecho" | "visto" | "escrito" | "dicho" | "puesto"
            );
        if is_word_boundary(lower, start, end)
            && !looks_participle
            && let Some(orig) = para_text.get(start..end)
        {
            out.push(RawFinding {
                rule_id: "grammar.es_existential",
                category: IssueCategory::SpellingAndGrammar,
                severity: Severity::Warning,
                confidence: Confidence::High,
                rel_start: start,
                rel_end: end,
                title: "Uso impersonal de `haber`".to_owned(),
                explanation:
                    "Con valor existencial, `haber` es impersonal y se usa en singular (`hubo`)."
                        .to_owned(),
                replacements: vec![Replacement::new(match_initial_case(orig, "hubo"))],
            });
        }
        cursor = end;
    }
}

fn raw_space_tokens(text: &str) -> Vec<(usize, usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if let Some(s) = start.take()
                && let Some(slice) = text.get(s..idx)
            {
                tokens.push((s, idx, slice));
            }
        } else if start.is_none() {
            start = Some(idx);
        }
    }
    if let Some(s) = start
        && let Some(slice) = text.get(s..)
    {
        tokens.push((s, text.len(), slice));
    }
    tokens
}
