//! Domain types for grammarcraft.
//!
//! Layer 0. This crate knows nothing about files, the terminal, JSON or a UI:
//! it holds the data and the invariants that must stay true whatever calls it.
//! Keeping it dependency-light is what makes the invariants testable in
//! isolation and keeps the dependency direction one-way.
//!
//! The crate never panics. Every failure is a [`CoreError`] returned through
//! [`CoreResult`], and input is treated as hostile (lengths are bounded before
//! anything is allocated).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod doc;
mod error;
mod glossary;
mod goals;
mod issue;
mod metrics;
mod snippets;
mod stats;
mod style_guide;

pub use doc::{Document, MAX_DOCUMENT_BYTES, MAX_UNDO_STEPS, TextSpan};
pub use error::{BoundsSnapshot, CoreError, CoreResult};
pub use glossary::{
    IgnoredIssueKey, IgnoredIssueSet, MAX_CUSTOM_FILE_BYTES, MAX_CUSTOM_WORDS, MAX_IGNORED_ISSUES,
    MAX_TERM_BYTES, PersonalDictionary,
};
pub use goals::{Audience, Context, Formality, MAX_GOAL_VALUE_BYTES, Purpose, WritingGoals};
pub use issue::{
    Confidence, Issue, IssueCategory, IssueId, IssueList, MAX_ISSUES, MAX_SUGGESTIONS_PER_ISSUE,
    Replacement, Severity,
};
pub use metrics::{
    CategoryScoreBreakdown, ReadabilityLevel, ReadabilityMetrics, ToneProfile, ToneSignal,
    ToneSignalKind, WritingScore,
};
pub use snippets::{
    MAX_SNIPPET_BYTES, MAX_SNIPPET_PREFIX_BYTES, MAX_SNIPPETS, Snippet, SnippetCatalog,
};
pub use stats::TextStats;
pub use style_guide::{
    MAX_RULE_PATTERN_BYTES, MAX_STYLE_RULES, StyleGuide, StyleRule, StyleRuleKind,
};

#[cfg(test)]
mod tests;
