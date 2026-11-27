use std::io::Read;
use std::path::Path;

use grammarcraft_core::{Document, MAX_DOCUMENT_BYTES, PersonalDictionary, WritingGoals};
use serde::{Deserialize, Serialize};

use crate::{CmdError, file_access};

/// Current version of the `.grammarcraft.json` document format.
pub const FORMAT_VERSION: u32 = 1;

/// Maximum raw file bytes read from disk when opening a document.
const MAX_OPEN_FILE_BYTES: u64 = (MAX_DOCUMENT_BYTES as u64) * 2;

#[derive(Debug, Serialize, Deserialize)]
struct DocumentFileV1 {
    version: u32,
    text: String,
    language: String,
    goals: WritingGoals,
    personal_dictionary: Vec<String>,
}

pub(crate) struct LoadedDocument {
    pub doc: Document,
    pub language: String,
    pub goals: WritingGoals,
    pub personal_dict: PersonalDictionary,
    pub format: &'static str,
}

pub(crate) fn save_document(
    path: &str,
    doc: &Document,
    language: &str,
    goals: WritingGoals,
    personal_dict: &PersonalDictionary,
) -> Result<&'static str, CmdError> {
    if path.trim().is_empty() {
        return Err(CmdError::Params("`path` must not be empty".to_owned()));
    }
    file_access::check_write(path).map_err(CmdError::Failed)?;

    let (format, payload) = if is_json_format(path) {
        let record = DocumentFileV1 {
            version: FORMAT_VERSION,
            text: doc.text().to_owned(),
            language: language.to_owned(),
            goals,
            personal_dictionary: personal_dict.words(),
        };
        let bytes = serde_json::to_vec_pretty(&record)
            .map_err(|e| CmdError::Failed(format!("failed to serialize document: {e}")))?;
        ("grammarcraft.json", bytes)
    } else {
        ("plain_text", doc.text().as_bytes().to_vec())
    };

    craft_atomic::atomic_write(Path::new(path), &payload)
        .map_err(|e| CmdError::Failed(format!("failed to write `{path}`: {e}")))?;
    Ok(format)
}

pub(crate) fn open_document(path: &str) -> Result<LoadedDocument, CmdError> {
    if path.trim().is_empty() {
        return Err(CmdError::Params("`path` must not be empty".to_owned()));
    }
    file_access::check_read(path).map_err(CmdError::Failed)?;

    let raw = read_bounded(path)?;
    if is_json_format(path) {
        load_json_document(path, &raw)
    } else {
        load_plain_document(path, &raw)
    }
}

fn read_bounded(path: &str) -> Result<Vec<u8>, CmdError> {
    let file = std::fs::File::open(path)
        .map_err(|e| CmdError::Failed(format!("failed to open `{path}`: {e}")))?;
    let mut raw = Vec::new();
    file.take(MAX_OPEN_FILE_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|e| CmdError::Failed(format!("failed to read `{path}`: {e}")))?;
    if (raw.len() as u64) > MAX_OPEN_FILE_BYTES {
        return Err(CmdError::Params(format!(
            "file `{path}` exceeds maximum size of {MAX_OPEN_FILE_BYTES} bytes"
        )));
    }
    Ok(raw)
}

fn load_json_document(path: &str, raw: &[u8]) -> Result<LoadedDocument, CmdError> {
    let parsed: DocumentFileV1 = serde_json::from_slice(raw)
        .map_err(|e| CmdError::Params(format!("invalid document JSON in `{path}`: {e}")))?;
    if parsed.version != FORMAT_VERSION {
        return Err(CmdError::Params(format!(
            "unsupported document format version {} in `{path}`",
            parsed.version
        )));
    }
    let mut doc = Document::new();
    if !parsed.text.is_empty() {
        doc.set_text(&parsed.text)?;
    }
    let mut personal_dict = PersonalDictionary::new();
    for term in &parsed.personal_dictionary {
        personal_dict.add(term)?;
    }
    let language = if parsed.language.trim().is_empty() {
        "en-US".to_owned()
    } else {
        parsed.language
    };
    Ok(LoadedDocument {
        doc,
        language,
        goals: parsed.goals,
        personal_dict,
        format: "grammarcraft.json",
    })
}

fn load_plain_document(path: &str, raw: &[u8]) -> Result<LoadedDocument, CmdError> {
    let text = std::str::from_utf8(raw)
        .map_err(|e| CmdError::Params(format!("`{path}` is not valid UTF-8: {e}")))?;
    let mut doc = Document::new();
    if !text.trim().is_empty() {
        doc.set_text(text)?;
    } else if text.len() > MAX_DOCUMENT_BYTES {
        return Err(CmdError::Params(format!(
            "the document text is longer than {MAX_DOCUMENT_BYTES} bytes"
        )));
    }
    Ok(LoadedDocument {
        doc,
        language: "en-US".to_owned(),
        goals: WritingGoals::default(),
        personal_dict: PersonalDictionary::new(),
        format: "plain_text",
    })
}

fn is_json_format(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".grammarcraft.json") || lower.ends_with(".json")
}
