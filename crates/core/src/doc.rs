use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, LiveTranslationConfig, TextStats};

/// Longest document text accepted, in bytes (1 MiB).
///
/// Checked before trimming or allocating so oversized untrusted input is
/// rejected in constant time.
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// Maximum undo/redo history entries retained per editing session.
pub const MAX_UNDO_STEPS: usize = 200;

/// A validated byte range within a UTF-8 document.
///
/// Constructing a span checks both bounds and UTF-8 character boundaries, so
/// downstream slicing never panics by landing inside a multi-byte code point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextSpan {
    start: usize,
    end: usize,
}

impl TextSpan {
    /// Builds a validated span over `text`.
    pub fn new(text: &str, start: usize, end: usize) -> CoreResult<Self> {
        if start > end || end > text.len() {
            return Err(CoreError::InvalidSpan {
                start,
                end,
                len: text.len(),
            });
        }
        if !text.is_char_boundary(start) {
            return Err(CoreError::NonCharBoundary { offset: start });
        }
        if !text.is_char_boundary(end) {
            return Err(CoreError::NonCharBoundary { offset: end });
        }
        Ok(Self { start, end })
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Slices `text` at this span, re-validating bounds and UTF-8 boundaries.
    pub fn slice(self, text: &str) -> CoreResult<&str> {
        let verified = Self::new(text, self.start, self.end)?;
        text.get(verified.start..verified.end)
            .ok_or(CoreError::InvalidSpan {
                start: self.start,
                end: self.end,
                len: text.len(),
            })
    }
}

/// The active document text held in memory.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    text: String,
    #[serde(default)]
    pub live_translation: LiveTranslationConfig,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the document text and returns its measured statistics.
    ///
    /// Byte length is checked *before* scanning or allocating, and exact byte
    /// offsets are preserved once the text is confirmed non-blank.
    pub fn set_text(&mut self, text: &str) -> CoreResult<TextStats> {
        if text.len() > MAX_DOCUMENT_BYTES {
            return Err(CoreError::TextTooLong {
                max: MAX_DOCUMENT_BYTES,
            });
        }
        if text.trim().is_empty() {
            return Err(CoreError::EmptyText);
        }
        self.text = text.to_owned();
        Ok(self.stats())
    }

    /// Replaces `span` with `replacement`, checking the resulting byte size
    /// before allocating a new `String`.
    pub fn replace_span(&mut self, span: TextSpan, replacement: &str) -> CoreResult<TextStats> {
        let verified = TextSpan::new(&self.text, span.start(), span.end())?;
        let new_len = self
            .text
            .len()
            .saturating_sub(verified.len())
            .checked_add(replacement.len())
            .ok_or(CoreError::TextTooLong {
                max: MAX_DOCUMENT_BYTES,
            })?;
        if new_len > MAX_DOCUMENT_BYTES {
            return Err(CoreError::TextTooLong {
                max: MAX_DOCUMENT_BYTES,
            });
        }
        let mut updated = String::with_capacity(new_len);
        if let Some(prefix) = self.text.get(..verified.start()) {
            updated.push_str(prefix);
        }
        updated.push_str(replacement);
        if let Some(suffix) = self.text.get(verified.end()..) {
            updated.push_str(suffix);
        }
        self.text = updated;
        Ok(self.stats())
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len_bytes(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn stats(&self) -> TextStats {
        TextStats::measure(&self.text)
    }

    /// Builds a validated [`TextSpan`] against the current document text.
    pub fn span(&self, start: usize, end: usize) -> CoreResult<TextSpan> {
        TextSpan::new(&self.text, start, end)
    }

    /// Slices the current document using a validated [`TextSpan`].
    pub fn slice(&self, span: TextSpan) -> CoreResult<&str> {
        span.slice(&self.text)
    }

    /// Clears the document and returns how many bytes were removed.
    pub fn clear(&mut self) -> usize {
        let removed = self.text.len();
        self.text.clear();
        removed
    }
}
