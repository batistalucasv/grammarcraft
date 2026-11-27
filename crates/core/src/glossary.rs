use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

/// Maximum words stored in a session's personal dictionary.
pub const MAX_CUSTOM_WORDS: usize = 100_000;

/// Maximum UTF-8 byte length for a single dictionary term.
pub const MAX_TERM_BYTES: usize = 256;

/// Maximum byte size accepted when parsing a freestanding `CUSTOM.DIC` file.
pub const MAX_CUSTOM_FILE_BYTES: usize = 1_000_000;

/// Maximum dismissed/ignored issue entries stored per document session.
pub const MAX_IGNORED_ISSUES: usize = 10_000;

/// Document-scoped personal dictionary isolated from global process state.
///
/// Supports both case-insensitive terms and exact-case terms (e.g. acronyms or
/// brand spellings) plus pure-byte `CUSTOM.DIC` (UTF-8 / UTF-16LE BOM) import.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalDictionary {
    words: BTreeSet<String>,
    #[serde(default)]
    exact_words: BTreeSet<String>,
}

impl PersonalDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and adds `term` as a case-insensitive entry.
    pub fn add(&mut self, term: &str) -> CoreResult<bool> {
        self.add_with_case(term, false)
    }

    /// Validates and adds `term` with explicit case-sensitivity.
    pub fn add_with_case(&mut self, term: &str, case_sensitive: bool) -> CoreResult<bool> {
        let trimmed = validate_term(term)?;
        let key = if case_sensitive {
            trimmed.to_owned()
        } else {
            trimmed.to_lowercase()
        };
        let target_has = if case_sensitive {
            self.exact_words.contains(&key)
        } else {
            self.words.contains(&key)
        };
        if target_has {
            return Ok(false);
        }
        if self.len() >= MAX_CUSTOM_WORDS {
            return Err(CoreError::TooManyCustomWords {
                max: MAX_CUSTOM_WORDS,
            });
        }
        if case_sensitive {
            Ok(self.exact_words.insert(key))
        } else {
            Ok(self.words.insert(key))
        }
    }

    /// Removes `term` from either case-insensitive or exact-case entries.
    pub fn remove(&mut self, term: &str) -> CoreResult<bool> {
        let trimmed = validate_term(term)?;
        let exact_removed = self.exact_words.remove(trimmed);
        let lower_removed = self.words.remove(&trimmed.to_lowercase());
        Ok(exact_removed || lower_removed)
    }

    /// Returns `true` if `term` matches an exact or case-insensitive entry.
    pub fn contains(&self, term: &str) -> bool {
        let Ok(trimmed) = validate_term(term) else {
            return false;
        };
        self.exact_words.contains(trimmed) || self.words.contains(&trimmed.to_lowercase())
    }

    /// Merges all entries from `other`, respecting [`MAX_CUSTOM_WORDS`].
    pub fn merge(&mut self, other: &Self) -> CoreResult<usize> {
        let mut added = 0usize;
        for term in &other.words {
            if self.add_with_case(term, false)? {
                added = added.saturating_add(1);
            }
        }
        for term in &other.exact_words {
            if self.add_with_case(term, true)? {
                added = added.saturating_add(1);
            }
        }
        Ok(added)
    }

    pub fn words(&self) -> Vec<String> {
        let mut combined: BTreeSet<String> = self.words.iter().cloned().collect();
        combined.extend(self.exact_words.iter().cloned());
        combined.into_iter().collect()
    }

    pub fn exact_words(&self) -> Vec<String> {
        self.exact_words.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.words.len().saturating_add(self.exact_words.len())
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty() && self.exact_words.is_empty()
    }

    /// Parses a `CUSTOM.DIC` payload in UTF-8 (optional BOM) or UTF-16LE with BOM (`0xFF 0xFE`).
    ///
    /// Size is checked before decoding so oversized inputs fail immediately.
    pub fn from_custom_dic_bytes(bytes: &[u8]) -> CoreResult<Self> {
        if bytes.len() > MAX_CUSTOM_FILE_BYTES {
            return Err(CoreError::InvalidCustomDictionary(format!(
                "file exceeds {MAX_CUSTOM_FILE_BYTES} bytes"
            )));
        }
        let decoded = decode_custom_dic(bytes)?;
        let mut dict = Self::new();
        for raw_line in decoded.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case_sensitive = line.chars().any(char::is_uppercase);
            dict.add_with_case(line, case_sensitive)?;
        }
        Ok(dict)
    }

    /// Serializes entries as UTF-8 lines.
    pub fn to_custom_dic_utf8(&self) -> String {
        let mut out = self.words().join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }

    /// Serializes entries as UTF-16LE with BOM (`0xFF 0xFE`), compatible with Word `CUSTOM.DIC`.
    pub fn to_custom_dic_utf16le(&self) -> Vec<u8> {
        let text = self.to_custom_dic_utf8().replace('\n', "\r\n");
        let mut out = Vec::with_capacity(2 + text.len() * 2);
        out.extend_from_slice(&[0xff, 0xfe]);
        for unit in text.encode_utf16() {
            out.extend_from_slice(&unit.to_le_bytes());
        }
        out
    }
}

fn decode_custom_dic(bytes: &[u8]) -> CoreResult<String> {
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        if !rest.len().is_multiple_of(2) {
            return Err(CoreError::InvalidCustomDictionary(
                "odd byte count in UTF-16LE stream".to_owned(),
            ));
        }
        let units = rest
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&[lo, hi]| u16::from_le_bytes([lo, hi]));
        return char::decode_utf16(units)
            .collect::<Result<String, _>>()
            .map_err(|_| {
                CoreError::InvalidCustomDictionary("invalid UTF-16LE sequence".to_owned())
            });
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return Err(CoreError::InvalidCustomDictionary(
            "unsupported UTF-16BE encoding; use UTF-8 or UTF-16LE with BOM".to_owned(),
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| {
        CoreError::InvalidCustomDictionary(
            "unsupported encoding; expected UTF-8 or UTF-16LE with BOM".to_owned(),
        )
    })?;
    Ok(text.strip_prefix('\u{feff}').unwrap_or(text).to_owned())
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

fn validate_term(raw: &str) -> CoreResult<&str> {
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
    Ok(trimmed)
}
