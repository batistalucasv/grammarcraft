#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod clarity;
mod contextual;
mod proof_adapter;
mod scoring;
mod style_guide_eval;
mod tone;
mod variants;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use craft_proof::ProofLang;
use grammarcraft_core::{
    Confidence, CoreResult, Document, IgnoredIssueKey, IgnoredIssueSet, Issue, IssueCategory,
    IssueList, PersonalDictionary, ReadabilityMetrics, Replacement, Severity, StyleGuide, TextSpan,
    TextStats, ToneProfile, WritingGoals, WritingScore,
};

pub use style_guide_eval::validate_style_rule;

/// Maximum paragraph cache entries held in memory by [`AnalysisEngine`].
const MAX_CACHE_ENTRIES: usize = 4_096;

/// Complete deterministic analysis output for a document.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisReport {
    pub language_tag: String,
    pub stats: TextStats,
    pub issues: Vec<Issue>,
    pub readability: ReadabilityMetrics,
    pub score: WritingScore,
    pub tone: ToneProfile,
}

/// Paragraph-relative finding before document-level filtering and [`IssueId`] assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawFinding {
    pub rule_id: &'static str,
    pub category: IssueCategory,
    pub severity: Severity,
    pub confidence: Confidence,
    pub rel_start: usize,
    pub rel_end: usize,
    pub title: String,
    pub explanation: String,
    pub replacements: Vec<Replacement>,
}

/// Incremental deterministic rule engine with per-paragraph caching.
#[derive(Debug, Default)]
pub struct AnalysisEngine {
    paragraph_cache: HashMap<u64, Vec<RawFinding>>,
    cache_hits: u64,
    cache_misses: u64,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn cache_hits(&self) -> u64 {
        self.cache_hits
    }

    pub const fn cache_misses(&self) -> u64 {
        self.cache_misses
    }

    pub fn clear_cache(&mut self) {
        self.paragraph_cache.clear();
    }

    /// Runs the deterministic pipeline over `doc`, reusing cached per-paragraph
    /// results when text, language, goals, style guide and dictionary generation match.
    pub fn analyze(
        &mut self,
        doc: &Document,
        language_tag: Option<&str>,
        goals: WritingGoals,
        personal_dict: &PersonalDictionary,
        ignored: &IgnoredIssueSet,
        style_guide: &StyleGuide,
    ) -> CoreResult<AnalysisReport> {
        let (proof_lang, canonical_tag) = proof_adapter::resolve_lang(language_tag);
        let text = doc.text();
        let stats = doc.stats();
        let readability = scoring::compute_readability(text, &stats, proof_lang);
        let tone = tone::analyze_tone(text);

        if text.is_empty() {
            return Ok(AnalysisReport {
                language_tag: canonical_tag,
                stats,
                issues: Vec::new(),
                readability,
                score: WritingScore::default(),
                tone,
            });
        }

        let dict_gen = craft_proof::dictionary_generation();
        let guide_hash = hash_style_guide(style_guide);
        let mut issue_list = IssueList::new();

        for (para_start, para_text) in paragraph_slices(text) {
            let key = paragraph_cache_key(para_text, &canonical_tag, goals, dict_gen, guide_hash);
            let findings = self.cached_or_compute(key, para_text, proof_lang, goals, style_guide);
            for finding in findings {
                let abs_start = para_start.saturating_add(finding.rel_start);
                let abs_end = para_start.saturating_add(finding.rel_end);
                let Ok(span) = TextSpan::new(text, abs_start, abs_end) else {
                    continue;
                };
                let snippet = span.slice(text).unwrap_or("");
                if is_suppressed(&finding, snippet, abs_start, personal_dict, ignored)? {
                    continue;
                }
                issue_list.push(
                    finding.rule_id,
                    finding.category,
                    finding.severity,
                    finding.confidence,
                    span,
                    finding.title.clone(),
                    finding.explanation.clone(),
                    finding.replacements.clone(),
                )?;
            }
        }

        let issues = issue_list.into_vec();
        let score = WritingScore::compute(&issues, stats.words);

        Ok(AnalysisReport {
            language_tag: canonical_tag,
            stats,
            issues,
            readability,
            score,
            tone,
        })
    }

    fn cached_or_compute(
        &mut self,
        key: u64,
        para_text: &str,
        proof_lang: ProofLang,
        goals: WritingGoals,
        style_guide: &StyleGuide,
    ) -> Vec<RawFinding> {
        if let Some(cached) = self.paragraph_cache.get(&key) {
            self.cache_hits = self.cache_hits.saturating_add(1);
            return cached.clone();
        }
        self.cache_misses = self.cache_misses.saturating_add(1);
        let computed = compute_paragraph_findings(para_text, proof_lang, goals, style_guide);
        if self.paragraph_cache.len() >= MAX_CACHE_ENTRIES {
            self.paragraph_cache.clear();
        }
        self.paragraph_cache.insert(key, computed.clone());
        computed
    }
}

fn compute_paragraph_findings(
    para_text: &str,
    proof_lang: ProofLang,
    goals: WritingGoals,
    style_guide: &StyleGuide,
) -> Vec<RawFinding> {
    let mut findings = proof_adapter::check_paragraph(para_text, proof_lang);
    findings.extend(contextual::check_paragraph(para_text, proof_lang));
    findings.extend(variants::check_paragraph(para_text, proof_lang));
    findings.extend(clarity::check_paragraph(para_text, proof_lang, goals));
    findings.extend(style_guide_eval::check_paragraph(para_text, style_guide));
    findings.sort_by_key(|f| (f.rel_start, f.rel_end, f.rule_id));
    findings
}

fn is_suppressed(
    finding: &RawFinding,
    snippet: &str,
    abs_start: usize,
    personal_dict: &PersonalDictionary,
    ignored: &IgnoredIssueSet,
) -> CoreResult<bool> {
    if finding.rule_id == "proof.spelling" && personal_dict.contains(snippet) {
        return Ok(true);
    }
    let key = IgnoredIssueKey::new(finding.rule_id, abs_start, snippet)?;
    Ok(ignored.is_ignored(&key))
}

fn paragraph_slices(text: &str) -> Vec<(usize, &str)> {
    let mut slices = Vec::new();
    let mut offset = 0usize;
    for raw_line in text.split_inclusive('\n') {
        let trimmed = raw_line.trim_end_matches(['\r', '\n']);
        if !trimmed.trim().is_empty() {
            slices.push((offset, trimmed));
        }
        offset = offset.saturating_add(raw_line.len());
    }
    if offset < text.len()
        && let Some(tail) = text.get(offset..)
        && !tail.trim().is_empty()
    {
        slices.push((offset, tail));
    }
    slices
}

fn hash_style_guide(style_guide: &StyleGuide) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    style_guide.hash(&mut hasher);
    hasher.finish()
}

fn paragraph_cache_key(
    para_text: &str,
    lang_tag: &str,
    goals: WritingGoals,
    dict_gen: u64,
    guide_hash: u64,
) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    fnv_mix(&mut hash, para_text.as_bytes());
    fnv_mix(&mut hash, &[0xff]);
    fnv_mix(&mut hash, lang_tag.as_bytes());
    fnv_mix(&mut hash, &[0xff]);
    fnv_mix(&mut hash, goals.audience.as_str().as_bytes());
    fnv_mix(&mut hash, goals.formality.as_str().as_bytes());
    fnv_mix(&mut hash, goals.context.as_str().as_bytes());
    fnv_mix(&mut hash, goals.purpose.as_str().as_bytes());
    fnv_mix(&mut hash, &dict_gen.to_le_bytes());
    fnv_mix(&mut hash, &guide_hash.to_le_bytes());
    hash
}

fn fnv_mix(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests;
