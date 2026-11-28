use grammarcraft_core::{TextSpan, TranslationResult};

use crate::{CmdError, Session};

#[derive(Clone, Copy)]
struct PhraseEntry {
    pt: &'static str,
    en: &'static str,
    es: &'static str,
}

const PHRASEBOOK: &[PhraseEntry] = &[
    PhraseEntry {
        pt: "olá, bom dia",
        en: "hello, good morning",
        es: "hola, buenos días",
    },
    PhraseEntry {
        pt: "olá, boa tarde",
        en: "hello, good afternoon",
        es: "hola, buenas tardes",
    },
    PhraseEntry {
        pt: "olá, boa noite",
        en: "hello, good evening",
        es: "hola, buenas noches",
    },
    PhraseEntry {
        pt: "bom dia",
        en: "good morning",
        es: "buenos días",
    },
    PhraseEntry {
        pt: "boa tarde",
        en: "good afternoon",
        es: "buenas tardes",
    },
    PhraseEntry {
        pt: "boa noite",
        en: "good evening",
        es: "buenas noches",
    },
    PhraseEntry {
        pt: "muito obrigado",
        en: "thank you very much",
        es: "muchas gracias",
    },
    PhraseEntry {
        pt: "muito obrigada",
        en: "thank you very much",
        es: "muchas gracias",
    },
    PhraseEntry {
        pt: "obrigado",
        en: "thank you",
        es: "gracias",
    },
    PhraseEntry {
        pt: "obrigada",
        en: "thank you",
        es: "gracias",
    },
    PhraseEntry {
        pt: "por favor",
        en: "please",
        es: "por favor",
    },
    PhraseEntry {
        pt: "bom trabalho",
        en: "good job",
        es: "buen trabajo",
    },
    PhraseEntry {
        pt: "tudo bem",
        en: "how are you",
        es: "cómo estás",
    },
    PhraseEntry {
        pt: "de nada",
        en: "you're welcome",
        es: "de nada",
    },
    PhraseEntry {
        pt: "com licença",
        en: "excuse me",
        es: "con permiso",
    },
    PhraseEntry {
        pt: "até logo",
        en: "see you soon",
        es: "hasta luego",
    },
    PhraseEntry {
        pt: "olá",
        en: "hello",
        es: "hola",
    },
    PhraseEntry {
        pt: "tchau",
        en: "goodbye",
        es: "adiós",
    },
];

/// Deterministic zero-network phrase translator supporting `pt`, `en`, and `es`.
#[derive(Debug, Default)]
pub struct LocalOfflineTranslator;

impl LocalOfflineTranslator {
    pub fn translate(
        text: &str,
        source_lang: Option<&str>,
        target_lang: &str,
    ) -> Option<(&'static str, String)> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        let target_base = normalize_primary(target_lang)?;

        let (lead_open, body_after_open) = if let Some(rest) = trimmed.strip_prefix('¿') {
            (Some('¿'), rest)
        } else if let Some(rest) = trimmed.strip_prefix('¡') {
            (Some('¡'), rest)
        } else {
            (None, trimmed)
        };
        let mut end = body_after_open.len();
        while let Some(ch) = body_after_open
            .get(..end)
            .and_then(|s| s.chars().next_back())
        {
            if matches!(ch, '.' | '!' | '?') {
                end = end.saturating_sub(ch.len_utf8());
            } else {
                break;
            }
        }
        let core = body_after_open.get(..end).unwrap_or("").trim();
        let trailing_punct = body_after_open.get(end..).unwrap_or("");
        if core.is_empty() {
            return None;
        }

        let core_lower = core.to_lowercase();
        let forced_src = source_lang.and_then(normalize_primary);

        if let Some(src) = forced_src
            && src == target_base
        {
            return Some((src, trimmed.to_owned()));
        }

        let (detected_src, entry) = match forced_src {
            Some("pt") => (
                "pt",
                PHRASEBOOK.iter().find(|entry| entry.pt == core_lower)?,
            ),
            Some("en") => (
                "en",
                PHRASEBOOK.iter().find(|entry| entry.en == core_lower)?,
            ),
            Some("es") => (
                "es",
                PHRASEBOOK.iter().find(|entry| entry.es == core_lower)?,
            ),
            Some(_) => return None,
            None => detect_entry(&core_lower, target_base)?,
        };

        let target_core = match target_base {
            "pt" => entry.pt,
            "en" => entry.en,
            "es" => entry.es,
            _ => return None,
        };
        let cased = match_initial_case(core, target_core);
        let mut formatted = String::with_capacity(cased.len() + trailing_punct.len() + 2);
        if target_base == "es" {
            if trailing_punct.contains('?') {
                formatted.push('¿');
            } else if trailing_punct.contains('!') {
                formatted.push('¡');
            }
        } else {
            let _ = lead_open;
        }
        formatted.push_str(&cased);
        formatted.push_str(trailing_punct);
        Some((detected_src, formatted))
    }
}

fn detect_entry(
    core_lower: &str,
    target_base: &str,
) -> Option<(&'static str, &'static PhraseEntry)> {
    if target_base != "pt"
        && let Some(entry) = PHRASEBOOK.iter().find(|entry| entry.pt == core_lower)
    {
        return Some(("pt", entry));
    }
    if target_base != "en"
        && let Some(entry) = PHRASEBOOK.iter().find(|entry| entry.en == core_lower)
    {
        return Some(("en", entry));
    }
    if target_base != "es"
        && let Some(entry) = PHRASEBOOK.iter().find(|entry| entry.es == core_lower)
    {
        return Some(("es", entry));
    }
    None
}

fn normalize_primary(tag: &str) -> Option<&'static str> {
    let primary = tag
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match primary.as_str() {
        "pt" => Some("pt"),
        "en" => Some("en"),
        "es" => Some("es"),
        _ => None,
    }
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

/// Returns the byte span of the last non-empty sentence or line segment in `doc_text`.
pub(crate) fn last_segment_span(doc_text: &str) -> Option<(usize, usize)> {
    let trimmed = doc_text.trim_end();
    let end = trimmed.len();
    if end == 0 {
        return None;
    }

    let body_prefix = match trimmed.chars().next_back() {
        Some(ch @ ('.' | '!' | '?')) => end.saturating_sub(ch.len_utf8()),
        _ => end,
    };

    let raw_start = doc_text
        .get(..body_prefix)
        .and_then(|slice| {
            slice
                .char_indices()
                .rev()
                .find(|&(_, ch)| matches!(ch, '\n' | '.' | '!' | '?'))
                .map(|(idx, ch)| idx + ch.len_utf8())
        })
        .unwrap_or(0);

    let leading_ws = doc_text
        .get(raw_start..end)
        .map(|slice| slice.len() - slice.trim_start().len())
        .unwrap_or(0);
    let start = raw_start + leading_ws;
    (start < end).then_some((start, end))
}

pub(crate) fn translate_live(
    session: &mut Session,
    explicit_text: Option<&str>,
    apply: bool,
) -> Result<TranslationResult, CmdError> {
    translate_live_with(session, explicit_text, apply, false)
}

pub(crate) fn translate_live_with(
    session: &mut Session,
    explicit_text: Option<&str>,
    apply: bool,
    allow_when_disabled: bool,
) -> Result<TranslationResult, CmdError> {
    let config = session.doc.live_translation.clone();
    if !config.enabled && !allow_when_disabled && explicit_text.is_none() {
        return Err(CmdError::Disabled(
            "live translation is disabled; enable it via `translate.configure`".to_owned(),
        ));
    }

    let (source_text, target_span) = if let Some(text) = explicit_text {
        if text.trim().is_empty() {
            return Err(CmdError::Params("`text` must not be empty".to_owned()));
        }
        let doc_text = session.doc.text();
        let matched_span = last_segment_span(doc_text)
            .filter(|&(start, end)| doc_text.get(start..end) == Some(text.trim()))
            .and_then(|(start, end)| session.doc.span(start, end).ok());
        (text.trim().to_owned(), matched_span)
    } else {
        let doc_text = session.doc.text();
        let Some((start, end)) = last_segment_span(doc_text) else {
            return Err(CmdError::Disabled("the document is empty".to_owned()));
        };
        let span = session.doc.span(start, end)?;
        let slice = session.doc.slice(span)?.to_owned();
        (slice, Some(span))
    };

    let (detected_src, translated_text, backend) = match LocalOfflineTranslator::translate(
        &source_text,
        config.source_lang.as_deref(),
        &config.target_lang,
    ) {
        Some((src, out)) => (src.to_owned(), out, "local-offline".to_owned()),
        None => {
            let (out, label) = session.ai.translate_with_cloud(
                config.source_lang.as_deref(),
                &config.target_lang,
                &source_text,
            )?;
            let src = config
                .source_lang
                .clone()
                .unwrap_or_else(|| session.language_tag.clone());
            (src, out, label)
        }
    };

    let mut applied = false;
    let mut result_span = target_span;
    if apply {
        if let Some(span) = target_span {
            let mut candidate = session.doc.clone();
            candidate.replace_span(span, &translated_text)?;
            session.push_undo();
            session.doc = candidate;
            session.invalidate_report();
            let new_end = span.start().saturating_add(translated_text.len());
            result_span = session.doc.span(span.start(), new_end).ok();
            applied = true;
        } else if session.doc.is_empty() {
            let mut candidate = session.doc.clone();
            candidate.set_text(&translated_text)?;
            session.push_undo();
            session.doc = candidate;
            session.invalidate_report();
            result_span = session.doc.span(0, translated_text.len()).ok();
            applied = true;
        } else {
            let suffix_sep = if session.doc.text().ends_with([' ', '\n']) {
                ""
            } else {
                " "
            };
            let insertion = format!("{suffix_sep}{translated_text}");
            let end = session.doc.len_bytes();
            let tail = TextSpan::new(session.doc.text(), end, end)?;
            let mut candidate = session.doc.clone();
            candidate.replace_span(tail, &insertion)?;
            session.push_undo();
            session.doc = candidate;
            session.invalidate_report();
            let start = end.saturating_add(suffix_sep.len());
            result_span = session
                .doc
                .span(start, start.saturating_add(translated_text.len()))
                .ok();
            applied = true;
        }
    }

    Ok(TranslationResult {
        span: result_span,
        source_text,
        translated_text,
        source_lang: detected_src,
        target_lang: config.target_lang,
        backend,
        applied,
    })
}
