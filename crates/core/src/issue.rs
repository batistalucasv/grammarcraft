use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, TextSpan};

/// Maximum issues retained for a single analysis pass.
pub const MAX_ISSUES: usize = 10_000;

/// Maximum replacement suggestions attached to a single issue.
pub const MAX_SUGGESTIONS_PER_ISSUE: usize = 16;

/// Opaque, monotonic identifier for a reported issue within a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IssueId(u64);

impl IssueId {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Top-level diagnostic category aligned with the clean-room taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    SpellingAndGrammar,
    ClarityAndConciseness,
    StyleAndVocabulary,
    ToneAndAppropriateness,
    StyleGuide,
}

impl IssueCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SpellingAndGrammar => "spelling_and_grammar",
            Self::ClarityAndConciseness => "clarity_and_conciseness",
            Self::StyleAndVocabulary => "style_and_vocabulary",
            Self::ToneAndAppropriateness => "tone_and_appropriateness",
            Self::StyleGuide => "style_guide",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "spelling_and_grammar" | "spelling" | "grammar" => Some(Self::SpellingAndGrammar),
            "clarity_and_conciseness" | "clarity" => Some(Self::ClarityAndConciseness),
            "style_and_vocabulary" | "style" => Some(Self::StyleAndVocabulary),
            "tone_and_appropriateness" | "tone" => Some(Self::ToneAndAppropriateness),
            "style_guide" | "guide" => Some(Self::StyleGuide),
            _ => None,
        }
    }
}

/// Actionable severity of a reported issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Suggestion,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Suggestion => "suggestion",
        }
    }

    /// Penalty weight used by the transparent open writing score formula.
    pub const fn score_weight(self) -> f64 {
        match self {
            Self::Error => 5.0,
            Self::Warning => 2.5,
            Self::Suggestion => 1.0,
        }
    }
}

/// Confidence level of the rule that emitted an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// One candidate replacement for an [`Issue`]'s [`TextSpan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replacement {
    pub text: String,
    pub label: Option<String>,
}

impl Replacement {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            label: None,
        }
    }

    pub fn with_label(text: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            label: Some(label.into()),
        }
    }
}

/// A single writing or proofing issue anchored to a validated [`TextSpan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub id: IssueId,
    pub rule_id: String,
    pub category: IssueCategory,
    pub severity: Severity,
    pub confidence: Confidence,
    pub span: TextSpan,
    pub title: String,
    pub explanation: String,
    pub replacements: Vec<Replacement>,
}

/// Bounded builder for assigning monotonic [`IssueId`]s without exceeding [`MAX_ISSUES`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IssueList {
    items: Vec<Issue>,
    next_id: u64,
}

impl IssueList {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        rule_id: impl Into<String>,
        category: IssueCategory,
        severity: Severity,
        confidence: Confidence,
        span: TextSpan,
        title: impl Into<String>,
        explanation: impl Into<String>,
        replacements: Vec<Replacement>,
    ) -> CoreResult<IssueId> {
        if self.items.len() >= MAX_ISSUES {
            return Err(CoreError::TooManyIssues { max: MAX_ISSUES });
        }
        if replacements.len() > MAX_SUGGESTIONS_PER_ISSUE {
            return Err(CoreError::TooManySuggestions {
                max: MAX_SUGGESTIONS_PER_ISSUE,
            });
        }
        let next = self.next_id.checked_add(1).ok_or(CoreError::IdsExhausted)?;
        self.next_id = next;
        let id = IssueId(next);
        self.items.push(Issue {
            id,
            rule_id: rule_id.into(),
            category,
            severity,
            confidence,
            span,
            title: title.into(),
            explanation: explanation.into(),
            replacements,
        });
        Ok(id)
    }

    pub fn items(&self) -> &[Issue] {
        &self.items
    }

    pub fn get(&self, id: IssueId) -> Option<&Issue> {
        self.items.iter().find(|issue| issue.id == id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn into_vec(self) -> Vec<Issue> {
        self.items
    }
}
