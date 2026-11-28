//! Pure, document-independent text rules designed for eventual upstreaming to
//! `craft-proof`.
//!
//! All checks operate directly on `&str` and [`ProofLang`] with conservative
//! guards so technical prose, numbers, URLs, contractions, measurements, and
//! list markers do not produce false positives.

use craft_proof::ProofLang;
use grammarcraft_core::{Confidence, IssueCategory, Replacement, Severity};

use crate::RawFinding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericFinding {
    pub rule_id: &'static str,
    pub start: usize,
    pub end: usize,
    pub title: String,
    pub explanation: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LangGroup {
    En,
    Pt,
    Es,
}

fn lang_group(lang: ProofLang) -> LangGroup {
    match lang {
        ProofLang::Pt => LangGroup::Pt,
        ProofLang::Es | ProofLang::EsMx | ProofLang::Es419 | ProofLang::EsUy => LangGroup::Es,
        _ => LangGroup::En,
    }
}

/// Evaluates conservative delimiter, quote, and typography rules on raw text.
pub fn check_text(text: &str, lang: ProofLang) -> Vec<GenericFinding> {
    if text.is_empty() {
        return Vec::new();
    }
    let group = lang_group(lang);
    let mut findings = Vec::new();
    check_balanced_delimiters(text, group, &mut findings);
    check_balanced_straight_double_quotes(text, group, &mut findings);
    check_bracket_spacing(text, group, &mut findings);
    check_missing_space_after_comma_semicolon(text, group, &mut findings);
    findings.sort_by_key(|f| (f.start, f.end, f.rule_id));
    findings
}

pub(crate) fn check_paragraph(para_text: &str, lang: ProofLang) -> Vec<RawFinding> {
    check_text(para_text, lang)
        .into_iter()
        .map(|f| RawFinding {
            rule_id: f.rule_id,
            category: IssueCategory::SpellingAndGrammar,
            severity: Severity::Warning,
            confidence: Confidence::High,
            rel_start: f.start,
            rel_end: f.end,
            title: f.title,
            explanation: f.explanation,
            replacements: f.suggestion.into_iter().map(Replacement::new).collect(),
        })
        .collect()
}

fn check_balanced_delimiters(text: &str, group: LangGroup, out: &mut Vec<GenericFinding>) {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut in_backticks = false;

    for (idx, ch) in text.char_indices() {
        if ch == '`' {
            in_backticks = !in_backticks;
            continue;
        }
        if in_backticks || inside_url(text, idx) {
            continue;
        }
        if matches!(ch, '(' | '[' | '“' | '«') {
            stack.push((ch, idx));
            continue;
        }
        let expected_open = match ch {
            ')' => Some('('),
            ']' => Some('['),
            '”' => Some('“'),
            '»' => Some('«'),
            _ => None,
        };
        let Some(open) = expected_open else {
            continue;
        };
        if ch == ')' && is_list_marker_close(text, idx) {
            continue;
        }
        if let Some(pos) = stack.iter().rposition(|(c, _)| *c == open) {
            for (unclosed, u_idx) in stack.drain((pos + 1)..) {
                out.push(unclosed_finding(unclosed, u_idx, group));
            }
            stack.pop();
        } else {
            out.push(unmatched_close_finding(ch, idx, group));
        }
    }

    for (unclosed, idx) in stack {
        out.push(unclosed_finding(unclosed, idx, group));
    }
}

fn is_list_marker_close(text: &str, close_idx: usize) -> bool {
    let Some(before) = text.get(..close_idx) else {
        return false;
    };
    let line_start = before.rfind('\n').map_or(0, |n| n + 1);
    let prefix = before.get(line_start..).unwrap_or("").trim_start();
    let len = prefix.len();
    (1..=3).contains(&len)
        && (prefix.chars().all(|c| c.is_ascii_digit())
            || len == 1 && prefix.chars().all(|c| c.is_ascii_alphabetic()))
}

fn unclosed_finding(ch: char, idx: usize, group: LangGroup) -> GenericFinding {
    let (title, explanation) = match group {
        LangGroup::Pt => (
            "Delimitador sem fechamento",
            format!("`{ch}` foi aberto sem delimitador de fechamento correspondente."),
        ),
        LangGroup::Es => (
            "Delimitador sin cierre",
            format!("Se abrió `{ch}` sin su delimitador de cierre correspondiente."),
        ),
        LangGroup::En => (
            "Unclosed delimiter",
            format!("Opening `{ch}` does not have a matching closing delimiter."),
        ),
    };
    GenericFinding {
        rule_id: "proof.unbalanced_delimiter",
        start: idx,
        end: idx.saturating_add(ch.len_utf8()),
        title: title.to_owned(),
        explanation,
        suggestion: None,
    }
}

fn unmatched_close_finding(ch: char, idx: usize, group: LangGroup) -> GenericFinding {
    let (title, explanation) = match group {
        LangGroup::Pt => (
            "Delimitador sem abertura",
            format!("`{ch}` aparece sem delimitador de abertura correspondente."),
        ),
        LangGroup::Es => (
            "Delimitador sin apertura",
            format!("`{ch}` aparece sin un delimitador de apertura correspondiente."),
        ),
        LangGroup::En => (
            "Unmatched closing delimiter",
            format!("Closing `{ch}` does not have a matching opening delimiter."),
        ),
    };
    GenericFinding {
        rule_id: "proof.unbalanced_delimiter",
        start: idx,
        end: idx.saturating_add(ch.len_utf8()),
        title: title.to_owned(),
        explanation,
        suggestion: None,
    }
}

fn check_balanced_straight_double_quotes(
    text: &str,
    group: LangGroup,
    out: &mut Vec<GenericFinding>,
) {
    let mut in_backticks = false;
    let mut positions = Vec::new();
    for (idx, ch) in text.char_indices() {
        if ch == '`' {
            in_backticks = !in_backticks;
            continue;
        }
        if in_backticks || ch != '"' || inside_url(text, idx) {
            continue;
        }
        let prev = text.get(..idx).and_then(|s| s.chars().next_back());
        let next = text.get(idx + 1..).and_then(|s| s.chars().next());
        let is_inch_mark =
            prev.is_some_and(|c| c.is_ascii_digit()) && !next.is_some_and(|c| c.is_alphabetic());
        if !is_inch_mark {
            positions.push(idx);
        }
    }

    if positions.len() % 2 == 1
        && let Some(&last_idx) = positions.last()
    {
        let (title, explanation) = match group {
            LangGroup::Pt => (
                "Aspas desbalanceadas",
                "Há uma aspa dupla sem par correspondente neste parágrafo.".to_owned(),
            ),
            LangGroup::Es => (
                "Comillas desbalanceadas",
                "Falta una comilla doble de cierre o apertura en este párrafo.".to_owned(),
            ),
            LangGroup::En => (
                "Unbalanced quotation mark",
                "There is an unmatched straight double quotation mark in this paragraph."
                    .to_owned(),
            ),
        };
        out.push(GenericFinding {
            rule_id: "proof.unbalanced_quote",
            start: last_idx,
            end: last_idx.saturating_add(1),
            title: title.to_owned(),
            explanation,
            suggestion: None,
        });
    }
}

fn check_bracket_spacing(text: &str, group: LangGroup, out: &mut Vec<GenericFinding>) {
    let mut in_backticks = false;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for i in 0..chars.len() {
        let (idx, ch) = chars[i];
        if ch == '`' {
            in_backticks = !in_backticks;
            continue;
        }
        if in_backticks || inside_url(text, idx) {
            continue;
        }
        if matches!(ch, '(' | '[')
            && let Some(&(sp_idx, ' ')) = chars.get(i + 1)
            && let Some(&(_, after_sp)) = chars.get(i + 2)
            && !after_sp.is_whitespace()
        {
            let (title, explanation) = match group {
                LangGroup::Pt => (
                    "Espaço após delimitador de abertura",
                    format!("Remova o espaço extra logo após `{ch}`."),
                ),
                LangGroup::Es => (
                    "Espacio tras delimitador de apertura",
                    format!("Elimine el espacio extra después de `{ch}`."),
                ),
                LangGroup::En => (
                    "Space after opening bracket",
                    format!("Remove the extra space immediately after `{ch}`."),
                ),
            };
            out.push(GenericFinding {
                rule_id: "proof.bracket_spacing",
                start: idx,
                end: sp_idx.saturating_add(1),
                title: title.to_owned(),
                explanation,
                suggestion: Some(ch.to_string()),
            });
        }

        if matches!(ch, ')' | ']')
            && i >= 2
            && let (Some(&(sp_idx, ' ')), Some(&(_, before_sp))) =
                (chars.get(i - 1), chars.get(i - 2))
            && !before_sp.is_whitespace()
            && !matches!(before_sp, '(' | '[')
        {
            let (title, explanation) = match group {
                LangGroup::Pt => (
                    "Espaço antes de delimitador de fechamento",
                    format!("Remova o espaço extra antes de `{ch}`."),
                ),
                LangGroup::Es => (
                    "Espacio antes del delimitador de cierre",
                    format!("Elimine el espacio extra antes de `{ch}`."),
                ),
                LangGroup::En => (
                    "Space before closing bracket",
                    format!("Remove the extra space immediately before `{ch}`."),
                ),
            };
            out.push(GenericFinding {
                rule_id: "proof.bracket_spacing",
                start: sp_idx,
                end: idx.saturating_add(1),
                title: title.to_owned(),
                explanation,
                suggestion: Some(ch.to_string()),
            });
        }
    }
}

fn check_missing_space_after_comma_semicolon(
    text: &str,
    group: LangGroup,
    out: &mut Vec<GenericFinding>,
) {
    let mut in_backticks = false;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for i in 1..chars.len().saturating_sub(1) {
        let (idx, ch) = chars[i];
        if ch == '`' {
            in_backticks = !in_backticks;
            continue;
        }
        if in_backticks || !matches!(ch, ',' | ';') {
            continue;
        }
        let prev = chars[i - 1].1;
        let next = chars[i + 1].1;
        if (prev.is_alphabetic() || matches!(prev, ')' | ']' | '”' | '»'))
            && next.is_alphabetic()
            && !inside_url(text, idx)
        {
            let (title, explanation) = match group {
                LangGroup::Pt => (
                    "Espaço ausente após pontuação",
                    format!("Insira um espaço após `{ch}`."),
                ),
                LangGroup::Es => (
                    "Falta espacio tras puntuación",
                    format!("Agregue un espacio después de `{ch}`."),
                ),
                LangGroup::En => (
                    "Missing space after punctuation",
                    format!("Add a space after `{ch}`."),
                ),
            };
            out.push(GenericFinding {
                rule_id: "proof.missing_space",
                start: idx,
                end: idx.saturating_add(1),
                title: title.to_owned(),
                explanation,
                suggestion: Some(format!("{ch} ")),
            });
        }
    }
}

fn inside_url(text: &str, idx: usize) -> bool {
    let Some(before) = text.get(..idx) else {
        return false;
    };
    let token_start = before
        .rfind(|c: char| c.is_whitespace())
        .map_or(0, |p| p + 1);
    let after = text.get(idx..).unwrap_or("");
    let token_end = after
        .find(|c: char| c.is_whitespace())
        .map_or(text.len(), |p| idx + p);
    let token = text.get(token_start..token_end).unwrap_or("");
    token.contains("://") || token.starts_with("www.")
}

#[cfg(test)]
mod tests {
    use craft_proof::ProofLang;

    use super::check_text;

    #[test]
    fn detects_unbalanced_delimiters_and_quotes_across_en_pt_es() {
        let en = check_text("Review the (unfinished note.", ProofLang::En);
        assert!(en.iter().any(|f| f.rule_id == "proof.unbalanced_delimiter"));

        let pt = check_text("Ele disse “olá sem fechar.", ProofLang::Pt);
        assert!(pt.iter().any(|f| f.rule_id == "proof.unbalanced_delimiter"));
        assert!(pt.iter().any(|f| f.title.contains("fechamento")));

        let es = check_text("Cita «incompleta en español.", ProofLang::Es);
        assert!(es.iter().any(|f| f.rule_id == "proof.unbalanced_delimiter"));
        assert!(es.iter().any(|f| f.title.contains("cierre")));

        let q = check_text(r#"She wrote "open quote only."#, ProofLang::En);
        assert!(q.iter().any(|f| f.rule_id == "proof.unbalanced_quote"));
    }

    #[test]
    fn detects_bracket_spacing_and_missing_space_after_comma_with_suggestions() {
        let spaced = check_text("Check ( this item ).", ProofLang::En);
        assert_eq!(
            spaced
                .iter()
                .filter(|f| f.rule_id == "proof.bracket_spacing")
                .count(),
            2
        );

        let comma = check_text("Olá,mundo;teste.", ProofLang::Pt);
        let hits: Vec<_> = comma
            .iter()
            .filter(|f| f.rule_id == "proof.missing_space")
            .collect();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].suggestion.as_deref(), Some(", "));
        assert_eq!(hits[1].suggestion.as_deref(), Some("; "));
    }

    #[test]
    fn suppresses_false_positives_on_contractions_inches_lists_numbers_urls_and_code() {
        let safe_samples = [
            (
                "Don't forget O'Brien's 27\" monitor (verified [ok]).",
                ProofLang::En,
            ),
            (
                "1) First item\na) Second item\nThe number is 1,000 and 3.14.",
                ProofLang::En,
            ),
            (
                "See https://example.com/a,b?x=(1 and `(unclosed` inline code.",
                ProofLang::En,
            ),
            (
                "Números como 10,5 ou 2,340 estão corretos «com aspas».",
                ProofLang::Pt,
            ),
            (
                "El valor 1,500 cumple «con estilo» (sin errores).",
                ProofLang::Es,
            ),
        ];
        for (sample, lang) in safe_samples {
            let findings = check_text(sample, lang);
            assert!(
                findings.is_empty(),
                "expected no false positives for `{sample}`, got {findings:?}"
            );
        }
    }
}
