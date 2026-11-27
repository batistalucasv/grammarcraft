use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, MAX_DOCUMENT_BYTES};

/// Maximum UTF-8 bytes accepted for a single AI diff or transform proposal.
pub const MAX_AI_TEXT_BYTES: usize = MAX_DOCUMENT_BYTES;

/// Maximum diff hunks emitted for a single preview.
pub const MAX_DIFF_HUNKS: usize = 10_000;

/// Supported AI writing assistance actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    Rewrite,
    AdjustTone,
    Summarize,
    Expand,
    Draft,
}

impl AiMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rewrite => "rewrite",
            Self::AdjustTone => "adjust_tone",
            Self::Summarize => "summarize",
            Self::Expand => "expand",
            Self::Draft => "draft",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Rewrite => "Rewrite",
            Self::AdjustTone => "Adjust Tone",
            Self::Summarize => "Summarize",
            Self::Expand => "Expand",
            Self::Draft => "Draft",
        }
    }
}

/// Execution locality mode for AI assistance.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiLocalityMode {
    #[default]
    LocalOffline,
    CloudConsentRequired,
}

impl AiLocalityMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalOffline => "local",
            Self::CloudConsentRequired => "cloud",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "local" | "local_offline" | "offline" => Ok(Self::LocalOffline),
            "cloud" | "remote" | "cloud_consent_required" => Ok(Self::CloudConsentRequired),
            other => Err(CoreError::InvalidGoal {
                field: "ai_locality",
                value: other.to_owned(),
            }),
        }
    }
}

/// Explicit consent state exposed before any remote transmission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiConsentStatus {
    NotRequiredLocal,
    RequiredPending {
        destination: String,
        capability: String,
        payload_bytes: usize,
    },
    Granted {
        destination: String,
    },
}

/// Operation kind for one word-level diff hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDiffOp {
    Equal,
    Insert,
    Delete,
}

/// One merged word-level diff hunk between original and proposed text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiDiffHunk {
    pub op: AiDiffOp,
    pub text: String,
}

/// Staged AI edit proposal awaiting user `Apply` or `Discard`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiProposal {
    pub id: u64,
    pub mode: AiMode,
    pub provider_label: String,
    pub original: String,
    pub proposed: String,
    pub hunks: Vec<AiDiffHunk>,
}

/// Computes a bounded word-by-word diff between `original` and `proposed`.
pub fn compute_word_diff(original: &str, proposed: &str) -> CoreResult<Vec<AiDiffHunk>> {
    if original.len() > MAX_AI_TEXT_BYTES || proposed.len() > MAX_AI_TEXT_BYTES {
        return Err(CoreError::AiTextTooLong {
            max: MAX_AI_TEXT_BYTES,
        });
    }
    let a = tokenize_words_and_spaces(original);
    let b = tokenize_words_and_spaces(proposed);
    if a.len().saturating_mul(b.len()) > 200_000 {
        return Ok(fallback_whole_diff(original, proposed));
    }

    let ops = lcs_diff_ops(&a, &b);
    merge_hunks(ops)
}

fn tokenize_words_and_spaces(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = 0usize;
    let mut in_space = None::<bool>;

    for (idx, ch) in input.char_indices() {
        let space = ch.is_whitespace();
        match in_space {
            Some(prev) if prev != space => {
                if let Some(slice) = input.get(start..idx) {
                    tokens.push(slice);
                }
                start = idx;
                in_space = Some(space);
            }
            None => in_space = Some(space),
            _ => {}
        }
    }
    if start < input.len()
        && let Some(tail) = input.get(start..)
    {
        tokens.push(tail);
    }
    tokens
}

fn lcs_diff_ops<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<(AiDiffOp, &'a str)> {
    let n = a.len();
    let m = b.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push((AiDiffOp::Equal, a[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            ops.push((AiDiffOp::Delete, a[i]));
            i += 1;
        } else {
            ops.push((AiDiffOp::Insert, b[j]));
            j += 1;
        }
    }
    while i < n {
        ops.push((AiDiffOp::Delete, a[i]));
        i += 1;
    }
    while j < m {
        ops.push((AiDiffOp::Insert, b[j]));
        j += 1;
    }
    ops
}

fn merge_hunks(ops: Vec<(AiDiffOp, &str)>) -> CoreResult<Vec<AiDiffHunk>> {
    let mut hunks: Vec<AiDiffHunk> = Vec::new();
    for (op, token) in ops {
        if let Some(last) = hunks.last_mut()
            && last.op == op
        {
            last.text.push_str(token);
            continue;
        }
        if hunks.len() >= MAX_DIFF_HUNKS {
            return Err(CoreError::TooManyDiffHunks {
                max: MAX_DIFF_HUNKS,
            });
        }
        hunks.push(AiDiffHunk {
            op,
            text: token.to_owned(),
        });
    }
    Ok(hunks)
}

fn fallback_whole_diff(original: &str, proposed: &str) -> Vec<AiDiffHunk> {
    if original == proposed {
        return if original.is_empty() {
            Vec::new()
        } else {
            vec![AiDiffHunk {
                op: AiDiffOp::Equal,
                text: original.to_owned(),
            }]
        };
    }
    let mut out = Vec::new();
    if !original.is_empty() {
        out.push(AiDiffHunk {
            op: AiDiffOp::Delete,
            text: original.to_owned(),
        });
    }
    if !proposed.is_empty() {
        out.push(AiDiffHunk {
            op: AiDiffOp::Insert,
            text: proposed.to_owned(),
        });
    }
    out
}
