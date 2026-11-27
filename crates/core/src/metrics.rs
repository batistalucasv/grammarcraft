use serde::{Deserialize, Serialize};

use crate::{Issue, IssueCategory, TextSpan};

/// Qualitative readability band derived from open formulas.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadabilityLevel {
    VeryEasy,
    Easy,
    #[default]
    Standard,
    Difficult,
    Academic,
}

/// Open readability metrics and time estimates for a document.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReadabilityMetrics {
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
    pub gulpease_index: f64,
    pub reading_minutes: f64,
    pub speaking_minutes: f64,
    pub level: ReadabilityLevel,
}

/// Open-issue count per [`IssueCategory`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryScoreBreakdown {
    pub spelling_and_grammar: u32,
    pub clarity_and_conciseness: u32,
    pub style_and_vocabulary: u32,
    pub tone_and_appropriateness: u32,
    pub style_guide: u32,
}

impl CategoryScoreBreakdown {
    pub fn from_issues(issues: &[Issue]) -> Self {
        let mut counts = Self::default();
        for issue in issues {
            match issue.category {
                IssueCategory::SpellingAndGrammar => {
                    counts.spelling_and_grammar = counts.spelling_and_grammar.saturating_add(1);
                }
                IssueCategory::ClarityAndConciseness => {
                    counts.clarity_and_conciseness =
                        counts.clarity_and_conciseness.saturating_add(1);
                }
                IssueCategory::StyleAndVocabulary => {
                    counts.style_and_vocabulary = counts.style_and_vocabulary.saturating_add(1);
                }
                IssueCategory::ToneAndAppropriateness => {
                    counts.tone_and_appropriateness =
                        counts.tone_and_appropriateness.saturating_add(1);
                }
                IssueCategory::StyleGuide => {
                    counts.style_guide = counts.style_guide.saturating_add(1);
                }
            }
        }
        counts
    }
}

/// Transparent, open-formula overall writing score (`0..=100`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WritingScore {
    pub score: u8,
    pub penalty_points: f64,
    pub breakdown: CategoryScoreBreakdown,
}

impl WritingScore {
    /// Computes an open 0..=100 score from open issues normalized by word count.
    ///
    /// Empty documents or documents without issues score `100`.
    pub fn compute(issues: &[Issue], words: usize) -> Self {
        let breakdown = CategoryScoreBreakdown::from_issues(issues);
        if words == 0 || issues.is_empty() {
            return Self {
                score: 100,
                penalty_points: 0.0,
                breakdown,
            };
        }
        let penalty_points: f64 = issues.iter().map(|i| i.severity.score_weight()).sum();
        let word_scale = (words.max(20) as f64) / 20.0;
        let normalized_penalty = (penalty_points / word_scale) * 4.0;
        let raw_score = (100.0 - normalized_penalty).clamp(0.0, 100.0);
        Self {
            score: raw_score.round() as u8,
            penalty_points,
            breakdown,
        }
    }
}

impl Default for WritingScore {
    fn default() -> Self {
        Self {
            score: 100,
            penalty_points: 0.0,
            breakdown: CategoryScoreBreakdown::default(),
        }
    }
}

/// Tone categories detected by multi-signal analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToneSignalKind {
    Assertive,
    Formal,
    Cordial,
    Hesitant,
    Urgent,
    Analytical,
    Empathetic,
    Critical,
}

impl ToneSignalKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assertive => "assertive",
            Self::Formal => "formal",
            Self::Cordial => "cordial",
            Self::Hesitant => "hesitant",
            Self::Urgent => "urgent",
            Self::Analytical => "analytical",
            Self::Empathetic => "empathetic",
            Self::Critical => "critical",
        }
    }
}

/// A single detected tone with relative strength (`0..=100`) and evidence spans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToneSignal {
    pub kind: ToneSignalKind,
    pub strength: u8,
    pub evidence_spans: Vec<TextSpan>,
}

/// Top dominant tones detected in a document (up to 3).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToneProfile {
    pub dominant: Vec<ToneSignal>,
}
