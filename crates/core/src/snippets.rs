use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, MAX_DOCUMENT_BYTES};

/// Maximum reusable snippets held in a [`SnippetCatalog`].
pub const MAX_SNIPPETS: usize = 2_000;

/// Maximum UTF-8 byte length of a single snippet body.
pub const MAX_SNIPPET_BYTES: usize = 16_384;

/// Maximum UTF-8 byte length of a snippet shortcut prefix or title.
pub const MAX_SNIPPET_PREFIX_BYTES: usize = 64;

/// A reusable text snippet with optional `{{placeholder}}` markers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Snippet {
    pub prefix: String,
    pub title: String,
    pub body: String,
}

/// Bounded catalog of reusable text snippets keyed by shortcut `prefix`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnippetCatalog {
    entries: BTreeMap<String, Snippet>,
}

impl SnippetCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, prefix: &str, title: &str, body: &str) -> CoreResult<bool> {
        let prefix = normalize_prefix(prefix)?;
        if title.len() > MAX_SNIPPET_PREFIX_BYTES * 2 {
            return Err(CoreError::InvalidSnippet(
                "snippet title exceeds maximum length".to_owned(),
            ));
        }
        let title = title.trim();
        if title.is_empty() {
            return Err(CoreError::InvalidSnippet(
                "snippet title must not be empty".to_owned(),
            ));
        }
        if body.len() > MAX_SNIPPET_BYTES {
            return Err(CoreError::SnippetTooLong {
                max: MAX_SNIPPET_BYTES,
            });
        }
        if body.trim().is_empty() {
            return Err(CoreError::InvalidSnippet(
                "snippet body must not be empty".to_owned(),
            ));
        }
        if self.entries.contains_key(&prefix) {
            return Ok(false);
        }
        if self.entries.len() >= MAX_SNIPPETS {
            return Err(CoreError::TooManySnippets { max: MAX_SNIPPETS });
        }
        let snippet = Snippet {
            prefix: prefix.clone(),
            title: title.to_owned(),
            body: body.to_owned(),
        };
        self.entries.insert(prefix, snippet);
        Ok(true)
    }

    pub fn remove(&mut self, prefix: &str) -> CoreResult<bool> {
        let prefix = normalize_prefix(prefix)?;
        Ok(self.entries.remove(&prefix).is_some())
    }

    pub fn get(&self, prefix: &str) -> Option<&Snippet> {
        normalize_prefix(prefix)
            .ok()
            .and_then(|key| self.entries.get(&key))
    }

    pub fn list(&self) -> Vec<&Snippet> {
        self.entries.values().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Expands `prefix` by substituting `{{placeholder}}` markers with `values`.
    pub fn expand(&self, prefix: &str, values: &BTreeMap<String, String>) -> CoreResult<String> {
        let key = normalize_prefix(prefix)?;
        let Some(snippet) = self.entries.get(&key) else {
            return Err(CoreError::UnknownSnippet(key));
        };
        expand_template(&snippet.body, values)
    }
}

fn normalize_prefix(raw: &str) -> CoreResult<String> {
    if raw.len() > MAX_SNIPPET_PREFIX_BYTES {
        return Err(CoreError::InvalidSnippet(
            "snippet prefix exceeds maximum length".to_owned(),
        ));
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(CoreError::InvalidSnippet(
            "snippet prefix must be a non-empty token without whitespace".to_owned(),
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn expand_template(body: &str, values: &BTreeMap<String, String>) -> CoreResult<String> {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(open) = rest.find("{{") {
        let Some(prefix) = rest.get(..open) else {
            break;
        };
        push_bounded(&mut out, prefix)?;
        let after_open = &rest[open + 2..];
        let Some(close) = after_open.find("}}") else {
            push_bounded(&mut out, &rest[open..])?;
            rest = "";
            break;
        };
        let placeholder_raw = &after_open[..close];
        let key = placeholder_raw.trim();
        if let Some(val) = values.get(key) {
            push_bounded(&mut out, val)?;
        } else {
            push_bounded(&mut out, &rest[open..open + 2 + close + 2])?;
        }
        rest = &after_open[close + 2..];
    }
    push_bounded(&mut out, rest)?;
    Ok(out)
}

fn push_bounded(out: &mut String, segment: &str) -> CoreResult<()> {
    let new_len = out
        .len()
        .checked_add(segment.len())
        .ok_or(CoreError::TextTooLong {
            max: MAX_DOCUMENT_BYTES,
        })?;
    if new_len > MAX_DOCUMENT_BYTES {
        return Err(CoreError::TextTooLong {
            max: MAX_DOCUMENT_BYTES,
        });
    }
    out.push_str(segment);
    Ok(())
}
