use std::collections::HashSet;

use grammarcraft_core::{
    CoreResult, MAX_SIMILARITY_MATCHES, MAX_SOURCE_NAME_BYTES, SimilarityMatch, SimilarityReport,
    TextSpan,
};

const SHINGLE_WORDS: usize = 4;
const MIN_MATCH_PERCENT: u8 = 25;

/// One local reference document provided by layer 2 for deterministic corpus comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCorpusDoc {
    pub source_name: String,
    pub text: String,
}

/// Computes deterministic N-gram shingle / Jaccard similarity against a local corpus.
pub fn check_local_similarity(
    doc_text: &str,
    corpus: &[LocalCorpusDoc],
) -> CoreResult<SimilarityReport> {
    if doc_text.trim().is_empty() || corpus.is_empty() {
        return Ok(SimilarityReport {
            overall_percent: 0,
            sources_scanned: corpus.len(),
            matches: Vec::new(),
        });
    }

    let doc_words = normalized_word_spans(doc_text);
    let doc_shingles = shingle_set(&doc_words);
    if doc_shingles.is_empty() {
        return Ok(SimilarityReport {
            overall_percent: 0,
            sources_scanned: corpus.len(),
            matches: Vec::new(),
        });
    }

    let corpus_shingles: Vec<(&str, HashSet<String>)> = corpus
        .iter()
        .map(|src| {
            let words = normalized_word_spans(&src.text);
            (trim_source_name(&src.source_name), shingle_set(&words))
        })
        .collect();

    let mut matches = Vec::new();
    let mut best_overall = 0u8;

    for (_, src_set) in &corpus_shingles {
        let overall = jaccard_percent(&doc_shingles, src_set);
        best_overall = best_overall.max(overall);
    }

    for (sent_start, sent_end, sentence) in sentence_spans(doc_text) {
        let sent_words = normalized_word_spans(sentence);
        let sent_set = shingle_set(&sent_words);
        if sent_set.is_empty() {
            continue;
        }
        if let Some((source_name, pct, shared)) =
            best_source_for_sentence(&sent_set, &corpus_shingles)
            && pct >= MIN_MATCH_PERCENT
            && let Ok(span) = TextSpan::new(doc_text, sent_start, sent_end)
        {
            if matches.len() >= MAX_SIMILARITY_MATCHES {
                break;
            }
            matches.push(SimilarityMatch {
                source_name: source_name.to_owned(),
                span,
                matched_excerpt: sentence.to_owned(),
                jaccard_percent: pct,
                shared_shingles: shared,
            });
        }
    }

    Ok(SimilarityReport {
        overall_percent: best_overall,
        sources_scanned: corpus.len(),
        matches,
    })
}

fn best_source_for_sentence<'a>(
    sent_set: &HashSet<String>,
    corpus_shingles: &[(&'a str, HashSet<String>)],
) -> Option<(&'a str, u8, usize)> {
    let mut best: Option<(&'a str, u8, usize)> = None;
    for (name, src_set) in corpus_shingles {
        if src_set.is_empty() {
            continue;
        }
        let shared = sent_set.intersection(src_set).count();
        if shared == 0 {
            continue;
        }
        let union = sent_set.len() + src_set.len() - shared;
        let pct = ((shared.saturating_mul(100)) / union.max(1)).min(100) as u8;
        let overlap_pct = ((shared.saturating_mul(100)) / sent_set.len().max(1)).min(100) as u8;
        let effective = pct.max(overlap_pct);
        if best.is_none_or(|(_, prev_pct, _)| effective > prev_pct) {
            best = Some((*name, effective, shared));
        }
    }
    best
}

fn jaccard_percent(a: &HashSet<String>, b: &HashSet<String>) -> u8 {
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let shared = a.intersection(b).count();
    let union = a.len() + b.len() - shared;
    ((shared.saturating_mul(100)) / union.max(1)).min(100) as u8
}

fn shingle_set(words: &[String]) -> HashSet<String> {
    if words.len() < SHINGLE_WORDS {
        return HashSet::new();
    }
    words.windows(SHINGLE_WORDS).map(|w| w.join(" ")).collect()
}

fn normalized_word_spans(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn sentence_spans(text: &str) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (idx, ch) in text.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let end = idx + ch.len_utf8();
            if let Some(raw) = text.get(start..end) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    let lead = raw.len() - raw.trim_start().len();
                    let s = start + lead;
                    out.push((s, s + trimmed.len(), trimmed));
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
            out.push((s, s + trimmed.len(), trimmed));
        }
    }
    out
}

fn trim_source_name(name: &str) -> &str {
    let trimmed = name.trim();
    if trimmed.len() <= MAX_SOURCE_NAME_BYTES {
        return trimmed;
    }
    let mut end = MAX_SOURCE_NAME_BYTES;
    while end > 0 && !trimmed.is_char_boundary(end) {
        end -= 1;
    }
    trimmed.get(..end).unwrap_or("source")
}
