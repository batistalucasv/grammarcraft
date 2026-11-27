use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::IssueId;

/// Everything that can go wrong while enforcing a domain invariant.
///
/// Variants name the rule that was broken and carry the bound or field that
/// failed, so callers can distinguish invalid arguments from internal errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    #[error("the document text is empty")]
    EmptyText,
    #[error("the document text is longer than {max} bytes")]
    TextTooLong { max: usize },
    #[error("invalid span {start}..{end} for document of {len} bytes")]
    InvalidSpan {
        start: usize,
        end: usize,
        len: usize,
    },
    #[error("byte offset {offset} is not on a UTF-8 character boundary")]
    NonCharBoundary { offset: usize },
    #[error("invalid {field} goal `{value}`")]
    InvalidGoal { field: &'static str, value: String },
    #[error("too many issues (limit {max})")]
    TooManyIssues { max: usize },
    #[error("too many suggestions for one issue (limit {max})")]
    TooManySuggestions { max: usize },
    #[error("the dictionary term is empty")]
    EmptyTerm,
    #[error("the dictionary term is longer than {max} bytes")]
    TermTooLong { max: usize },
    #[error("invalid dictionary term `{0}`")]
    InvalidTerm(String),
    #[error("personal dictionary holds at most {max} words")]
    TooManyCustomWords { max: usize },
    #[error("invalid custom dictionary file: {0}")]
    InvalidCustomDictionary(String),
    #[error("ignored issue list holds at most {max} entries")]
    TooManyIgnoredIssues { max: usize },
    #[error("style guide holds at most {max} rules")]
    TooManyStyleRules { max: usize },
    #[error("style rule field is longer than {max} bytes")]
    RulePatternTooLong { max: usize },
    #[error("invalid style rule: {0}")]
    InvalidStyleRule(String),
    #[error("snippet catalog holds at most {max} snippets")]
    TooManySnippets { max: usize },
    #[error("snippet body is longer than {max} bytes")]
    SnippetTooLong { max: usize },
    #[error("invalid snippet: {0}")]
    InvalidSnippet(String),
    #[error("unknown snippet `{0}`")]
    UnknownSnippet(String),
    #[error("too many similarity matches (limit {max})")]
    TooManySimilarityMatches { max: usize },
    #[error("citation catalog holds at most {max} entries")]
    TooManyCitations { max: usize },
    #[error("invalid citation: {0}")]
    InvalidCitation(String),
    #[error("invalid BibTeX input: {0}")]
    InvalidBibtex(String),
    #[error("AI input text exceeds {max} bytes")]
    AiTextTooLong { max: usize },
    #[error("too many diff hunks (limit {max})")]
    TooManyDiffHunks { max: usize },
    #[error("there is no issue with id {0}")]
    UnknownIssue(IssueId),
    #[error("issue ids are exhausted")]
    IdsExhausted,
}

pub type CoreResult<T> = Result<T, CoreError>;

/// Convenience helper for serializing error-free domain snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundsSnapshot {
    pub max_document_bytes: usize,
    pub max_issues: usize,
    pub max_suggestions_per_issue: usize,
    pub max_custom_words: usize,
    pub max_term_bytes: usize,
    pub max_undo_steps: usize,
}
