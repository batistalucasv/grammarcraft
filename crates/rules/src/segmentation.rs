//! Safe UAX #14 line-break and UAX #29 extended-grapheme segmentation powered by `craft-text`.

use std::ops::Range;

use craft_text::segment::{Break, Options, emergency_breaks, line_breaks};

/// Computes UAX #14 line-break opportunities (with language-aware kinsoku/Korean options).
pub fn line_break_opportunities(text: &str, language_tag: Option<&str>) -> Vec<Break> {
    line_breaks(text, &Options::for_language(language_tag))
}

/// Computes UAX #29 extended grapheme cluster boundaries strictly inside `range`,
/// keeping combining marks, flags, and emoji ZWJ sequences unbroken.
pub fn grapheme_boundaries(text: &str, range: Range<usize>) -> Vec<usize> {
    emergency_breaks(text, range)
}

/// Snaps `offset` backward to the nearest valid UTF-8 UAX #29 extended grapheme cluster boundary.
pub fn snap_to_grapheme_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    if offset == 0 || text.is_empty() {
        return 0;
    }
    let boundaries = emergency_breaks(text, 0..text.len());
    let idx = boundaries.partition_point(|&b| b <= offset);
    if idx == 0 { 0 } else { boundaries[idx - 1] }
}

/// Splits `text` into `(byte_offset, paragraph_slice)` pairs along UAX #14 mandatory
/// line breaks (`\n`, `\r\n`, U+0085, U+2028, U+2029), skipping blank lines while
/// preserving exact UTF-8 byte offsets into `text`.
pub(crate) fn paragraph_slices(text: &str) -> Vec<(usize, &str)> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut slices = Vec::new();
    let mut start = 0usize;
    for brk in line_breaks(text, &Options::default()) {
        if !brk.mandatory || brk.offset <= start {
            continue;
        }
        if let Some(raw) = text.get(start..brk.offset) {
            let trimmed = raw.trim_end_matches(['\r', '\n', '\u{0085}', '\u{2028}', '\u{2029}']);
            if !trimmed.trim().is_empty() {
                slices.push((start, trimmed));
            }
        }
        start = brk.offset;
    }
    if start < text.len()
        && let Some(tail) = text.get(start..)
        && !tail.trim().is_empty()
    {
        slices.push((start, tail));
    }
    slices
}
