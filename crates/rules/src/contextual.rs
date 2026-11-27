use craft_proof::ProofLang;
use grammarcraft_core::{Confidence, IssueCategory, Replacement, Severity};

use crate::RawFinding;
use crate::clarity::is_portuguese;

struct ContextualPattern {
    needle: &'static str,
    target_sub: &'static str,
    replacement: &'static str,
    title: &'static str,
    explanation: &'static str,
}

const EN_PATTERNS: &[ContextualPattern] = &[
    ContextualPattern {
        needle: "their is",
        target_sub: "their",
        replacement: "there",
        title: "Contextual word choice",
        explanation: "Use `there` as the existential pronoun before `is`.",
    },
    ContextualPattern {
        needle: "their are",
        target_sub: "their",
        replacement: "there",
        title: "Contextual word choice",
        explanation: "Use `there` as the existential pronoun before `are`.",
    },
    ContextualPattern {
        needle: "their was",
        target_sub: "their",
        replacement: "there",
        title: "Contextual word choice",
        explanation: "Use `there` as the existential pronoun before `was`.",
    },
    ContextualPattern {
        needle: "their were",
        target_sub: "their",
        replacement: "there",
        title: "Contextual word choice",
        explanation: "Use `there` as the existential pronoun before `were`.",
    },
    ContextualPattern {
        needle: "over their",
        target_sub: "their",
        replacement: "there",
        title: "Contextual word choice",
        explanation: "Use `there` to indicate place (`over there`).",
    },
    ContextualPattern {
        needle: "there own",
        target_sub: "there",
        replacement: "their",
        title: "Contextual word choice",
        explanation: "Use possessive `their` before `own`.",
    },
    ContextualPattern {
        needle: "there house",
        target_sub: "there",
        replacement: "their",
        title: "Contextual word choice",
        explanation: "Use possessive `their` before `house`.",
    },
    ContextualPattern {
        needle: "there team",
        target_sub: "there",
        replacement: "their",
        title: "Contextual word choice",
        explanation: "Use possessive `their` before `team`.",
    },
    ContextualPattern {
        needle: "there going to",
        target_sub: "there",
        replacement: "they're",
        title: "Contextual word choice",
        explanation: "Use contraction `they're` (`they are`) before `going to`.",
    },
    ContextualPattern {
        needle: "your welcome",
        target_sub: "your",
        replacement: "you're",
        title: "Contextual word choice",
        explanation: "Use contraction `you're` (`you are`) in `you're welcome`.",
    },
    ContextualPattern {
        needle: "you're name",
        target_sub: "you're",
        replacement: "your",
        title: "Contextual word choice",
        explanation: "Use possessive `your` before `name`.",
    },
    ContextualPattern {
        needle: "you're team",
        target_sub: "you're",
        replacement: "your",
        title: "Contextual word choice",
        explanation: "Use possessive `your` before `team`.",
    },
    ContextualPattern {
        needle: "it's own",
        target_sub: "it's",
        replacement: "its",
        title: "Contextual word choice",
        explanation: "Use possessive `its` (without apostrophe) before `own`.",
    },
    ContextualPattern {
        needle: "its been",
        target_sub: "its",
        replacement: "it's",
        title: "Contextual word choice",
        explanation: "Use contraction `it's` (`it has`) before `been`.",
    },
    ContextualPattern {
        needle: "its a",
        target_sub: "its",
        replacement: "it's",
        title: "Contextual word choice",
        explanation: "Use contraction `it's` (`it is`) before article `a`.",
    },
    ContextualPattern {
        needle: "its an",
        target_sub: "its",
        replacement: "it's",
        title: "Contextual word choice",
        explanation: "Use contraction `it's` (`it is`) before article `an`.",
    },
    ContextualPattern {
        needle: "better then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` for comparisons after `better`.",
    },
    ContextualPattern {
        needle: "more then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` for comparisons after `more`.",
    },
    ContextualPattern {
        needle: "less then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` for comparisons after `less`.",
    },
    ContextualPattern {
        needle: "rather then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` after `rather`.",
    },
    ContextualPattern {
        needle: "other then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` after `other`.",
    },
    ContextualPattern {
        needle: "greater then",
        target_sub: "then",
        replacement: "than",
        title: "Comparison conjunction",
        explanation: "Use `than` for comparisons after `greater`.",
    },
    ContextualPattern {
        needle: "the affect",
        target_sub: "affect",
        replacement: "effect",
        title: "Contextual word choice",
        explanation: "Use noun `effect` after `the` in standard prose.",
    },
    ContextualPattern {
        needle: "an affect",
        target_sub: "affect",
        replacement: "effect",
        title: "Contextual word choice",
        explanation: "Use noun `effect` after `an`.",
    },
    ContextualPattern {
        needle: "side affect",
        target_sub: "affect",
        replacement: "effect",
        title: "Contextual word choice",
        explanation: "Use noun `effect` in `side effect`.",
    },
    ContextualPattern {
        needle: "cause and affect",
        target_sub: "affect",
        replacement: "effect",
        title: "Contextual word choice",
        explanation: "Use noun `effect` in `cause and effect`.",
    },
];

const PT_PATTERNS: &[ContextualPattern] = &[
    ContextualPattern {
        needle: "cada vez mas",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use o advérbio de intensidade `mais` em `cada vez mais`.",
    },
    ContextualPattern {
        needle: "muito mas",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` (quantidade/intensidade) após `muito`.",
    },
    ContextualPattern {
        needle: "bem mas",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` (intensidade) após `bem`.",
    },
    ContextualPattern {
        needle: "o mas importante",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` antes do adjetivo `importante`.",
    },
    ContextualPattern {
        needle: "mas rápido",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` para comparação ou intensidade antes de `rápido`.",
    },
    ContextualPattern {
        needle: "mas fácil",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` para comparação ou intensidade antes de `fácil`.",
    },
    ContextualPattern {
        needle: "mas tarde",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` na expressão `mais tarde`.",
    },
    ContextualPattern {
        needle: "mas cedo",
        target_sub: "mas",
        replacement: "mais",
        title: "Confusão contextual",
        explanation: "Use `mais` na expressão `mais cedo`.",
    },
    ContextualPattern {
        needle: "mal humor",
        target_sub: "mal",
        replacement: "mau",
        title: "Confusão mal/mau",
        explanation: "Use o adjetivo `mau` (oposto de `bom`) antes de `humor`.",
    },
    ContextualPattern {
        needle: "mal caráter",
        target_sub: "mal",
        replacement: "mau",
        title: "Confusão mal/mau",
        explanation: "Use o adjetivo `mau` (oposto de `bom`) antes de `caráter`.",
    },
    ContextualPattern {
        needle: "mau feito",
        target_sub: "mau",
        replacement: "mal",
        title: "Confusão mal/mau",
        explanation: "Use o advérbio `mal` (oposto de `bem`) antes do particípio `feito`.",
    },
    ContextualPattern {
        needle: "passou mau",
        target_sub: "mau",
        replacement: "mal",
        title: "Confusão mal/mau",
        explanation: "Use o advérbio `mal` (oposto de `bem`) com o verbo `passar`.",
    },
    ContextualPattern {
        needle: "a anos",
        target_sub: "a",
        replacement: "há",
        title: "Confusão a/há",
        explanation: "Use o verbo `há` para indicar tempo decorrido (`há anos`).",
    },
    ContextualPattern {
        needle: "a meses",
        target_sub: "a",
        replacement: "há",
        title: "Confusão a/há",
        explanation: "Use o verbo `há` para indicar tempo decorrido (`há meses`).",
    },
    ContextualPattern {
        needle: "a dias",
        target_sub: "a",
        replacement: "há",
        title: "Confusão a/há",
        explanation: "Use o verbo `há` para indicar tempo decorrido (`há dias`).",
    },
    ContextualPattern {
        needle: "a muito tempo",
        target_sub: "a",
        replacement: "há",
        title: "Confusão a/há",
        explanation: "Use `há` para indicar tempo transcorrido (`há muito tempo`).",
    },
    ContextualPattern {
        needle: "daqui há",
        target_sub: "há",
        replacement: "a",
        title: "Confusão a/há",
        explanation: "Para tempo futuro (`daqui a`), use a preposição `a`.",
    },
    ContextualPattern {
        needle: "há partir de",
        target_sub: "há",
        replacement: "a",
        title: "Confusão a/há",
        explanation: "Na locução `a partir de`, use a preposição `a` sem `h`.",
    },
    ContextualPattern {
        needle: "aonde fica",
        target_sub: "aonde",
        replacement: "onde",
        title: "Confusão onde/aonde",
        explanation: "Use `onde` com verbos estáticos como `ficar`.",
    },
    ContextualPattern {
        needle: "aonde está",
        target_sub: "aonde",
        replacement: "onde",
        title: "Confusão onde/aonde",
        explanation: "Use `onde` com verbos estáticos como `estar`.",
    },
    ContextualPattern {
        needle: "seção de cinema",
        target_sub: "seção",
        replacement: "sessão",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `sessão` para evento ou período de exibição.",
    },
    ContextualPattern {
        needle: "seção plenária",
        target_sub: "seção",
        replacement: "sessão",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `sessão` para reunião ou encontro.",
    },
    ContextualPattern {
        needle: "sessão de livros",
        target_sub: "sessão",
        replacement: "seção",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `seção` para divisão ou departamento.",
    },
    ContextualPattern {
        needle: "sessão do artigo",
        target_sub: "sessão",
        replacement: "seção",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `seção` para parte ou subdivisão de um texto.",
    },
    ContextualPattern {
        needle: "sessão de direitos",
        target_sub: "sessão",
        replacement: "cessão",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `cessão` no sentido de ceder ou transferir direitos.",
    },
    ContextualPattern {
        needle: "seção de direitos",
        target_sub: "seção",
        replacement: "cessão",
        title: "Parônimos seção/sessão/cessão",
        explanation: "Use `cessão` no sentido de ceder ou transferir direitos.",
    },
    ContextualPattern {
        needle: "o porque",
        target_sub: "porque",
        replacement: "porquê",
        title: "Uso de porquê",
        explanation: "Substantivado após artigo (`o porquê`), escreve-se junto e com acento.",
    },
    ContextualPattern {
        needle: "um porque",
        target_sub: "porque",
        replacement: "porquê",
        title: "Uso de porquê",
        explanation: "Substantivado após artigo (`um porquê`), escreve-se junto e com acento.",
    },
];

pub(crate) fn check_paragraph(para_text: &str, lang: ProofLang) -> Vec<RawFinding> {
    let lower = para_text.to_lowercase();
    if lower.len() != para_text.len() {
        return Vec::new();
    }
    let is_pt = is_portuguese(lang);
    let patterns = if is_pt { PT_PATTERNS } else { EN_PATTERNS };
    let mut out = Vec::new();
    for pattern in patterns {
        scan_pattern(para_text, &lower, pattern, &mut out);
    }
    if is_pt {
        check_pt_end_of_question(para_text, &lower, &mut out);
    }
    out
}

fn scan_pattern(
    para_text: &str,
    lower: &str,
    pattern: &ContextualPattern,
    out: &mut Vec<RawFinding>,
) {
    let mut cursor = 0usize;
    while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(pattern.needle)) {
        let start = cursor + rel;
        let end = start + pattern.needle.len();
        if has_word_boundaries(lower, start, end)
            && let Some(sub_rel) = pattern.needle.find(pattern.target_sub)
        {
            let target_start = start + sub_rel;
            let target_end = target_start + pattern.target_sub.len();
            if let Some(orig) = para_text.get(target_start..target_end) {
                let replacement = match_case(orig, pattern.replacement);
                out.push(RawFinding {
                    rule_id: "contextual.confusable",
                    category: IssueCategory::SpellingAndGrammar,
                    severity: Severity::Warning,
                    confidence: Confidence::High,
                    rel_start: target_start,
                    rel_end: target_end,
                    title: pattern.title.to_owned(),
                    explanation: pattern.explanation.to_owned(),
                    replacements: vec![Replacement::new(replacement)],
                });
            }
        }
        cursor = end;
    }
}

fn check_pt_end_of_question(para_text: &str, lower: &str, out: &mut Vec<RawFinding>) {
    for (needle, target_sub) in [("porque?", "porque"), ("por que?", "por que")] {
        let mut cursor = 0usize;
        while let Some(rel) = lower.get(cursor..).and_then(|s| s.find(needle)) {
            let start = cursor + rel;
            let target_end = start + target_sub.len();
            if left_boundary(lower, start)
                && let Some(orig) = para_text.get(start..target_end)
            {
                let replacement = match_case(orig, "por quê");
                out.push(RawFinding {
                    rule_id: "contextual.por_que",
                    category: IssueCategory::SpellingAndGrammar,
                    severity: Severity::Warning,
                    confidence: Confidence::High,
                    rel_start: start,
                    rel_end: target_end,
                    title: "Uso de por quê".to_owned(),
                    explanation: "Em fim de frase antes de ponto de interrogação, use `por quê`."
                        .to_owned(),
                    replacements: vec![Replacement::new(replacement)],
                });
            }
            cursor = start + needle.len();
        }
    }
}

fn left_boundary(text: &str, start: usize) -> bool {
    text.get(..start)
        .and_then(|s| s.chars().next_back())
        .is_none_or(|c| !c.is_alphanumeric())
}

fn has_word_boundaries(text: &str, start: usize, end: usize) -> bool {
    let left_ok = left_boundary(text, start);
    let right_ok = text
        .get(end..)
        .and_then(|s| s.chars().next())
        .is_none_or(|c| !c.is_alphanumeric());
    left_ok && right_ok
}

fn match_case(original: &str, replacement: &str) -> String {
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
