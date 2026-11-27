use std::io::Read;
use std::path::Path;

use grammarcraft_core::{
    Document, MAX_CUSTOM_FILE_BYTES, MAX_DOCUMENT_BYTES, PersonalDictionary, SnippetCatalog,
    StyleGuide, WritingGoals,
};
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
    #[serde(default)]
    exact_dictionary: Vec<String>,
    #[serde(default)]
    style_guide: StyleGuide,
    #[serde(default)]
    snippets: SnippetCatalog,
}

pub(crate) struct LoadedDocument {
    pub doc: Document,
    pub language: String,
    pub goals: WritingGoals,
    pub personal_dict: PersonalDictionary,
    pub style_guide: StyleGuide,
    pub snippets: SnippetCatalog,
    pub format: &'static str,
}

pub(crate) fn save_document(
    path: &str,
    doc: &Document,
    language: &str,
    goals: WritingGoals,
    personal_dict: &PersonalDictionary,
    style_guide: &StyleGuide,
    snippets: &SnippetCatalog,
) -> Result<&'static str, CmdError> {
    if path.trim().is_empty() {
        return Err(CmdError::Params("`path` must not be empty".to_owned()));
    }
    file_access::check_write(path).map_err(CmdError::Failed)?;

    let (format, payload) = if is_json_format(path) {
        let exact_dictionary = personal_dict.exact_words();
        let lower_only: Vec<String> = personal_dict
            .words()
            .into_iter()
            .filter(|w| !exact_dictionary.contains(w))
            .collect();
        let record = DocumentFileV1 {
            version: FORMAT_VERSION,
            text: doc.text().to_owned(),
            language: language.to_owned(),
            goals,
            personal_dictionary: lower_only,
            exact_dictionary,
            style_guide: style_guide.clone(),
            snippets: snippets.clone(),
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

    let raw = read_bounded(path, MAX_OPEN_FILE_BYTES)?;
    if is_json_format(path) {
        load_json_document(path, &raw)
    } else {
        load_plain_document(path, &raw)
    }
}

pub(crate) fn import_custom_dictionary(path: &str) -> Result<PersonalDictionary, CmdError> {
    if path.trim().is_empty() {
        return Err(CmdError::Params("`path` must not be empty".to_owned()));
    }
    file_access::check_read(path).map_err(CmdError::Failed)?;
    let raw = read_bounded(path, MAX_CUSTOM_FILE_BYTES as u64)?;
    Ok(PersonalDictionary::from_custom_dic_bytes(&raw)?)
}

pub(crate) fn export_custom_dictionary(
    path: &str,
    dict: &PersonalDictionary,
    utf16le: bool,
) -> Result<usize, CmdError> {
    if path.trim().is_empty() {
        return Err(CmdError::Params("`path` must not be empty".to_owned()));
    }
    file_access::check_write(path).map_err(CmdError::Failed)?;
    let bytes = if utf16le {
        dict.to_custom_dic_utf16le()
    } else {
        dict.to_custom_dic_utf8().into_bytes()
    };
    let len = bytes.len();
    craft_atomic::atomic_write(Path::new(path), &bytes)
        .map_err(|e| CmdError::Failed(format!("failed to write `{path}`: {e}")))?;
    Ok(len)
}

fn read_bounded(path: &str, max_bytes: u64) -> Result<Vec<u8>, CmdError> {
    let file = std::fs::File::open(path)
        .map_err(|e| CmdError::Failed(format!("failed to open `{path}`: {e}")))?;
    let mut raw = Vec::new();
    file.take(max_bytes + 1)
        .read_to_end(&mut raw)
        .map_err(|e| CmdError::Failed(format!("failed to read `{path}`: {e}")))?;
    if (raw.len() as u64) > max_bytes {
        return Err(CmdError::Params(format!(
            "file `{path}` exceeds maximum size of {max_bytes} bytes"
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
    for term in &parsed.exact_dictionary {
        personal_dict.add_with_case(term, true)?;
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
        style_guide: parsed.style_guide,
        snippets: parsed.snippets,
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
        style_guide: StyleGuide::new(),
        snippets: SnippetCatalog::new(),
        format: "plain_text",
    })
}

fn is_json_format(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".grammarcraft.json") || lower.ends_with(".json")
}
