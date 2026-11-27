use serde::{Deserialize, Serialize};

use crate::TextSpan;

/// Maximum similarity matches retained for a single local corpus check.
pub const MAX_SIMILARITY_MATCHES: usize = 512;

/// Maximum byte length of a local source document label.
pub const MAX_SOURCE_NAME_BYTES: usize = 256;

/// One local passage matching a source document in the authorized corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub source_name: String,
    pub span: TextSpan,
    pub matched_excerpt: String,
    pub jaccard_percent: u8,
    pub shared_shingles: usize,
}

/// Summary report of deterministic local shingle/Jaccard similarity checks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimilarityReport {
    pub overall_percent: u8,
    pub sources_scanned: usize,
    pub matches: Vec<SimilarityMatch>,
}
