use grammarcraft_core::CoreError;
use serde_json::Value;
use thiserror::Error;

/// Why a command did not run or did not finish.
///
/// Four variants are enough for a front end to react differently: retry or fix
/// the call (`Params`), tell the user the action is unavailable (`Disabled`),
/// or report an internal failure (`Failed`).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CmdError {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
    #[error("bad parameters: {0}")]
    Params(String),
    #[error("not available: {0}")]
    Disabled(String),
    #[error("{0}")]
    Failed(String),
}

impl From<CoreError> for CmdError {
    fn from(error: CoreError) -> Self {
        match error {
            // Domain invariant rejections triggered by caller arguments map to
            // `Params` so callers know to adjust the request.
            CoreError::EmptyText
            | CoreError::TextTooLong { .. }
            | CoreError::InvalidSpan { .. }
            | CoreError::NonCharBoundary { .. }
            | CoreError::InvalidGoal { .. } => CmdError::Params(error.to_string()),
        }
    }
}

pub type CmdResult = Result<Value, CmdError>;
