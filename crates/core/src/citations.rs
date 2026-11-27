use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

/// Maximum citation entries parsed or formatted in one call.
pub const MAX_CITATIONS: usize = 1_000;

/// Maximum raw BibTeX byte size accepted by the clean-room parser.
pub const MAX_BIBTEX_BYTES: usize = 512 * 1024;

/// Maximum byte length of a single citation field.
pub const MAX_CITATION_FIELD_BYTES: usize = 1_024;

/// Maximum authors stored per citation entry.
pub const MAX_AUTHORS: usize = 64;

/// Twelve supported bibliographic citation styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationStyle {
    Apa,
    Mla,
    Chicago,
    Ieee,
    Harvard,
    Iso690AuthorDate,
    Iso690Numeric,
    Turabian,
    GbT7714,
    GostName,
    GostTitle,
    Sist02,
}

impl CitationStyle {
    pub const ALL: [Self; 12] = [
        Self::Apa,
        Self::Mla,
        Self::Chicago,
        Self::Ieee,
        Self::Harvard,
        Self::Iso690AuthorDate,
        Self::Iso690Numeric,
        Self::Turabian,
        Self::GbT7714,
        Self::GostName,
        Self::GostTitle,
        Self::Sist02,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apa => "apa",
            Self::Mla => "mla",
            Self::Chicago => "chicago",
            Self::Ieee => "ieee",
            Self::Harvard => "harvard",
            Self::Iso690AuthorDate => "iso690_author_date",
            Self::Iso690Numeric => "iso690_numeric",
            Self::Turabian => "turabian",
            Self::GbT7714 => "gbt7714",
            Self::GostName => "gost_name",
            Self::GostTitle => "gost_title",
            Self::Sist02 => "sist02",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Apa => "APA 7",
            Self::Mla => "MLA 9",
            Self::Chicago => "Chicago 17",
            Self::Ieee => "IEEE",
            Self::Harvard => "Harvard",
            Self::Iso690AuthorDate => "ISO 690 (Author-Date)",
            Self::Iso690Numeric => "ISO 690 (Numeric)",
            Self::Turabian => "Turabian",
            Self::GbT7714 => "GB/T 7714",
            Self::GostName => "GOST R 7.0.5 (Name)",
            Self::GostTitle => "GOST R 7.0.5 (Title)",
            Self::Sist02 => "SIST02",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        let key = raw.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        match key.as_str() {
            "apa" => Ok(Self::Apa),
            "mla" => Ok(Self::Mla),
            "chicago" => Ok(Self::Chicago),
            "ieee" => Ok(Self::Ieee),
            "harvard" => Ok(Self::Harvard),
            "iso690_author_date" | "iso_690_author_date" => Ok(Self::Iso690AuthorDate),
            "iso690_numeric" | "iso_690_numeric" => Ok(Self::Iso690Numeric),
            "turabian" => Ok(Self::Turabian),
            "gbt7714" | "gb_t7714" | "gb_t_7714" => Ok(Self::GbT7714),
            "gost_name" => Ok(Self::GostName),
            "gost_title" => Ok(Self::GostTitle),
            "sist02" | "sist_02" => Ok(Self::Sist02),
            _ => Err(CoreError::InvalidCitation(format!(
                "unsupported citation style `{raw}`"
            ))),
        }
    }
}

/// Structured bibliographic record validated by L0 bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationEntry {
    pub key: String,
    pub entry_type: String,
    pub authors: Vec<String>,
    pub title: String,
    pub year: String,
    pub venue: String,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
}

/// Diagnostic emitted when validating a [`CitationEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationValidationIssue {
    pub key: String,
    pub field: &'static str,
    pub message: String,
}

impl CitationEntry {
    pub fn validate(&self) -> CoreResult<()> {
        check_field("key", &self.key, true)?;
        check_field("entry_type", &self.entry_type, true)?;
        check_field("title", &self.title, true)?;
        check_field("year", &self.year, false)?;
        check_field("venue", &self.venue, false)?;
        if self.authors.len() > MAX_AUTHORS {
            return Err(CoreError::InvalidCitation(format!(
                "citation `{}` exceeds {MAX_AUTHORS} authors",
                self.key
            )));
        }
        for author in &self.authors {
            check_field("author", author, true)?;
        }
        Ok(())
    }
}

fn check_field(field: &'static str, value: &str, required: bool) -> CoreResult<()> {
    if value.len() > MAX_CITATION_FIELD_BYTES {
        return Err(CoreError::InvalidCitation(format!(
            "field `{field}` exceeds {MAX_CITATION_FIELD_BYTES} bytes"
        )));
    }
    if required && value.trim().is_empty() {
        return Err(CoreError::InvalidCitation(format!(
            "field `{field}` must not be empty"
        )));
    }
    Ok(())
}

/// Validates completeness of parsed citations without rejecting warnings.
pub fn validate_citations(entries: &[CitationEntry]) -> Vec<CitationValidationIssue> {
    let mut diagnostics = Vec::new();
    for entry in entries {
        if entry.authors.is_empty() {
            diagnostics.push(CitationValidationIssue {
                key: entry.key.clone(),
                field: "author",
                message: "missing author; formatter will fall back to Anonymous".to_owned(),
            });
        }
        if entry.year.trim().is_empty() {
            diagnostics.push(CitationValidationIssue {
                key: entry.key.clone(),
                field: "year",
                message: "missing publication year; formatter will emit n.d.".to_owned(),
            });
        }
        if entry.venue.trim().is_empty() && entry.publisher.as_deref().unwrap_or("").is_empty() {
            diagnostics.push(CitationValidationIssue {
                key: entry.key.clone(),
                field: "venue",
                message: "missing journal, booktitle, or publisher".to_owned(),
            });
        }
    }
    diagnostics
}

/// Clean-room bounded BibTeX parser supporting `@type{key, field = {value}, ...}` entries.
pub fn parse_bibtex_entries(input: &str) -> CoreResult<Vec<CitationEntry>> {
    if input.len() > MAX_BIBTEX_BYTES {
        return Err(CoreError::InvalidBibtex(format!(
            "BibTeX input exceeds {MAX_BIBTEX_BYTES} bytes"
        )));
    }
    let mut entries = Vec::new();
    let bytes = input.as_bytes();
    let mut pos = 0usize;

    while let Some(at_rel) = input.get(pos..).and_then(|s| s.find('@')) {
        let at = pos + at_rel;
        let Some((entry, next_pos)) = parse_single_entry(input, bytes, at)? else {
            pos = at + 1;
            continue;
        };
        if entries.len() >= MAX_CITATIONS {
            return Err(CoreError::TooManyCitations { max: MAX_CITATIONS });
        }
        entries.push(entry);
        pos = next_pos;
    }

    if entries.is_empty() && !input.trim().is_empty() {
        return Err(CoreError::InvalidBibtex(
            "no valid `@type{key, ...}` BibTeX entries found".to_owned(),
        ));
    }
    Ok(entries)
}

fn parse_single_entry(
    input: &str,
    bytes: &[u8],
    at: usize,
) -> CoreResult<Option<(CitationEntry, usize)>> {
    let Some(open_rel) = input.get(at + 1..).and_then(|s| s.find('{')) else {
        return Ok(None);
    };
    let brace_start = at + 1 + open_rel;
    let entry_type = input
        .get(at + 1..brace_start)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if entry_type.is_empty() || entry_type == "comment" || entry_type == "preamble" {
        return Ok(None);
    }

    let body_end = find_matching_brace(bytes, brace_start)?;
    let body = input.get(brace_start + 1..body_end).unwrap_or("");
    let Some((key, fields_str)) = body.split_once(',') else {
        return Err(CoreError::InvalidBibtex(
            "missing citation key after `{`".to_owned(),
        ));
    };
    let key = key.trim().to_owned();
    if key.is_empty() {
        return Err(CoreError::InvalidBibtex("empty citation key".to_owned()));
    }

    let entry = build_entry(entry_type, key, fields_str)?;
    Ok(Some((entry, body_end + 1)))
}

fn find_matching_brace(bytes: &[u8], start: usize) -> CoreResult<usize> {
    let mut depth = 0usize;
    for (offset, &b) in bytes.get(start..).unwrap_or(&[]).iter().enumerate() {
        match b {
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(start + offset);
                }
            }
            _ => {}
        }
    }
    Err(CoreError::InvalidBibtex(
        "unclosed `{` in BibTeX entry".to_owned(),
    ))
}

fn build_entry(entry_type: String, key: String, fields_str: &str) -> CoreResult<CitationEntry> {
    let mut entry = CitationEntry {
        key,
        entry_type,
        authors: Vec::new(),
        title: String::new(),
        year: String::new(),
        venue: String::new(),
        volume: None,
        issue: None,
        pages: None,
        publisher: None,
        doi: None,
        url: None,
    };

    for (name, val) in parse_fields(fields_str) {
        assign_field(&mut entry, &name, val);
    }
    entry.validate()?;
    Ok(entry)
}

fn assign_field(entry: &mut CitationEntry, name: &str, val: String) {
    let cleaned = val.split_whitespace().collect::<Vec<_>>().join(" ");
    match name {
        "author" | "editor" if entry.authors.is_empty() => {
            entry.authors = cleaned
                .split(" and ")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .take(MAX_AUTHORS)
                .map(ToOwned::to_owned)
                .collect();
        }
        "title" => entry.title = cleaned,
        "year" | "date" => entry.year = cleaned,
        "journal" | "booktitle" | "venue" => entry.venue = cleaned,
        "volume" => entry.volume = opt_non_empty(cleaned),
        "number" | "issue" => entry.issue = opt_non_empty(cleaned),
        "pages" => entry.pages = opt_non_empty(cleaned),
        "publisher" => entry.publisher = opt_non_empty(cleaned),
        "doi" => entry.doi = opt_non_empty(cleaned),
        "url" => entry.url = opt_non_empty(cleaned),
        _ => {}
    }
}

fn opt_non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn parse_fields(raw: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut rest = raw.trim();
    while !rest.is_empty() {
        let Some((name_part, after_eq)) = rest.split_once('=') else {
            break;
        };
        let name = name_part
            .trim()
            .trim_start_matches(',')
            .trim()
            .to_ascii_lowercase();
        let after_eq = after_eq.trim_start();
        let (value, tail) = take_field_value(after_eq);
        if !name.is_empty() {
            fields.push((name, value));
        }
        rest = tail.trim_start_matches(',').trim();
    }
    fields
}

fn take_field_value(raw: &str) -> (String, &str) {
    if let Some(inner) = raw.strip_prefix('{') {
        let mut depth = 1usize;
        for (idx, ch) in inner.char_indices() {
            match ch {
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let val = inner.get(..idx).unwrap_or("").replace(['{', '}'], "");
                        let tail = inner.get(idx + 1..).unwrap_or("");
                        return (val, tail);
                    }
                }
                _ => {}
            }
        }
        return (inner.replace(['{', '}'], ""), "");
    }
    if let Some(inner) = raw.strip_prefix('"') {
        if let Some(end) = inner.find('"') {
            let val = inner.get(..end).unwrap_or("").to_owned();
            let tail = inner.get(end + 1..).unwrap_or("");
            return (val, tail);
        }
        return (inner.to_owned(), "");
    }
    if let Some((val, tail)) = raw.split_once(',') {
        (val.trim().to_owned(), tail)
    } else {
        (raw.trim().to_owned(), "")
    }
}

/// Formats a single [`CitationEntry`] according to any of the 12 supported [`CitationStyle`]s.
pub fn format_citation(entry: &CitationEntry, style: CitationStyle, index: usize) -> String {
    let authors = if entry.authors.is_empty() {
        "Anonymous".to_owned()
    } else {
        entry.authors.join(", ")
    };
    let year = if entry.year.trim().is_empty() {
        "n.d."
    } else {
        entry.year.trim()
    };
    let venue = non_empty_venue(entry);
    let pages = entry
        .pages
        .as_deref()
        .map(|p| format!(", pp. {p}"))
        .unwrap_or_default();
    let doi = entry
        .doi
        .as_deref()
        .map(|d| format!(" https://doi.org/{d}"))
        .unwrap_or_default();
    let num = index.saturating_add(1);

    match style {
        CitationStyle::Apa => {
            format!("{authors} ({year}). {}. {venue}{pages}.{doi}", entry.title)
        }
        CitationStyle::Mla => format!("{authors}. \"{}\". {venue}, {year}{pages}.", entry.title),
        CitationStyle::Chicago => format!(
            "{authors}. \"{}.\" {venue} ({year}){pages}.{doi}",
            entry.title
        ),
        CitationStyle::Ieee => format!(
            "[{num}] {authors}, \"{},\" {venue}, {year}{pages}.",
            entry.title
        ),
        CitationStyle::Harvard => format!("{authors} ({year}) '{}', {venue}{pages}.", entry.title),
        CitationStyle::Iso690AuthorDate => {
            format!("{authors}, {year}. {}. {venue}{pages}.", entry.title)
        }
        CitationStyle::Iso690Numeric => {
            format!(
                "[{num}] {authors}. {}. {venue}, {year}{pages}.",
                entry.title
            )
        }
        CitationStyle::Turabian => {
            format!("{authors}. \"{}.\" {venue} ({year}){pages}.", entry.title)
        }
        CitationStyle::GbT7714 => format!(
            "[{num}] {authors}. {}[J]. {venue}, {year}{pages}.",
            entry.title
        ),
        CitationStyle::GostName => {
            format!(
                "{num}. {authors} {} // {venue}. — {year}{pages}.",
                entry.title
            )
        }
        CitationStyle::GostTitle => {
            format!(
                "{num}. {} / {authors} // {venue}. — {year}{pages}.",
                entry.title
            )
        }
        CitationStyle::Sist02 => {
            format!(
                "[{num}] {authors}. {}. {venue}. {year}{pages}.",
                entry.title
            )
        }
    }
}

fn non_empty_venue(entry: &CitationEntry) -> &str {
    if !entry.venue.trim().is_empty() {
        entry.venue.trim()
    } else if let Some(pub_name) = entry.publisher.as_deref()
        && !pub_name.trim().is_empty()
    {
        pub_name.trim()
    } else {
        "Unpublished"
    }
}

/// Formats a list of citation entries into ordered bibliography lines.
pub fn format_bibliography(entries: &[CitationEntry], style: CitationStyle) -> Vec<String> {
    entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| format_citation(entry, style, idx))
        .collect()
}
