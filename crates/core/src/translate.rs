use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, TextSpan};

/// Maximum length accepted for a live-translation language tag, in bytes.
pub const MAX_LANG_TAG_BYTES: usize = 16;

/// Per-document configuration for real-time translation on sentence completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveTranslationConfig {
    pub enabled: bool,
    #[serde(default)]
    pub source_lang: Option<String>,
    #[serde(default = "default_target_lang")]
    pub target_lang: String,
}

fn default_target_lang() -> String {
    "en".to_owned()
}

impl Default for LiveTranslationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source_lang: None,
            target_lang: default_target_lang(),
        }
    }
}

/// Validates and normalizes a language tag against [`MAX_LANG_TAG_BYTES`] before allocating.
pub fn validate_lang_tag(raw: &str, field: &'static str) -> CoreResult<String> {
    if raw.len() > MAX_LANG_TAG_BYTES.saturating_mul(4) {
        return Err(CoreError::InvalidLangTag {
            field,
            value: String::new(),
            max: MAX_LANG_TAG_BYTES,
        });
    }
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_LANG_TAG_BYTES
        || trimmed.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        let preview: String = trimmed.chars().take(MAX_LANG_TAG_BYTES).collect();
        return Err(CoreError::InvalidLangTag {
            field,
            value: preview,
            max: MAX_LANG_TAG_BYTES,
        });
    }
    Ok(trimmed.to_owned())
}

/// Result produced by a live or on-demand translation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationResult {
    pub span: Option<TextSpan>,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub backend: String,
    pub applied: bool,
}
