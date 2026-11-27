use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

/// Maximum words stored in a session's personal dictionary.
pub const MAX_CUSTOM_WORDS: usize = 100_000;

/// Maximum UTF-8 byte length for a single dictionary term.
pub const MAX_TERM_BYTES: usize = 256;

/// Maximum dismissed/ignored issue entries stored per document session.
pub const MAX_IGNORED_ISSUES: usize = 10_000;

/// Document-scoped personal dictionary isolated from global process state.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalDictionary {
    words: BTreeSet<String>,
}

impl PersonalDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and adds `term` to the dictionary.
    ///
    /// Byte length is checked before trimming or allocating.
    pub fn add(&mut self, term: &str) -> CoreResult<bool> {
        let normalized = normalize_term(term)?;
        if self.words.contains(&normalized) {
            return Ok(false);
        }
        if self.words.len() >= MAX_CUSTOM_WORDS {
            return Err(CoreError::TooManyCustomWords {
                max: MAX_CUSTOM_WORDS,
            });
        }
        Ok(self.words.insert(normalized))
    }

    /// Removes `term` from the dictionary, returning whether it was present.
    pub fn remove(&mut self, term: &str) -> CoreResult<bool> {
        let normalized = normalize_term(term)?;
        Ok(self.words.remove(&normalized))
    }

    /// Returns `true` if `term` matches an entry in the dictionary.
    pub fn contains(&self, term: &str) -> bool {
        normalize_term(term).is_ok_and(|key| self.words.contains(&key))
    }

    pub fn words(&self) -> Vec<String> {
        self.words.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// Stable fingerprint used to keep an ignored issue suppressed across re-analysis.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IgnoredIssueKey {
    pub rule_id: String,
    pub start: usize,
    pub snippet: String,
}

impl IgnoredIssueKey {
    pub fn new(
        rule_id: impl Into<String>,
        start: usize,
        snippet: impl Into<String>,
    ) -> CoreResult<Self> {
        let rule_id = rule_id.into();
        let snippet = snippet.into();
        if snippet.len() > MAX_TERM_BYTES * 4 {
            return Err(CoreError::TermTooLong {
                max: MAX_TERM_BYTES * 4,
            });
        }
        Ok(Self {
            rule_id,
            start,
            snippet,
        })
    }
}

/// Bounded set of dismissed/ignored issues within a document session.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoredIssueSet {
    entries: BTreeSet<IgnoredIssueKey>,
}

impl IgnoredIssueSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ignore(&mut self, key: IgnoredIssueKey) -> CoreResult<bool> {
        if self.entries.contains(&key) {
            return Ok(false);
        }
        if self.entries.len() >= MAX_IGNORED_ISSUES {
            return Err(CoreError::TooManyIgnoredIssues {
                max: MAX_IGNORED_ISSUES,
            });
        }
        Ok(self.entries.insert(key))
    }

    pub fn is_ignored(&self, key: &IgnoredIssueKey) -> bool {
        self.entries.contains(key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn normalize_term(raw: &str) -> CoreResult<String> {
    if raw.len() > MAX_TERM_BYTES {
        return Err(CoreError::TermTooLong {
            max: MAX_TERM_BYTES,
        });
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyTerm);
    }
    if trimmed
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == char::REPLACEMENT_CHARACTER)
    {
        return Err(CoreError::InvalidTerm(trimmed.to_owned()));
    }
    Ok(trimmed.to_lowercase())
}
