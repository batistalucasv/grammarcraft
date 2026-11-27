use grammarcraft_core::CoreError;
use serde_json::Value;
use thiserror::Error;

/// Why a command did not run or did not finish.
///
/// Four variants are enough for a front end to react differently: retry or fix
/// the call (`Params`), tell the user the action is unavailable (`Disabled`),
/// or report a failure that came from the domain (`Failed`).
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
            // A broken invariant caused by what the caller passed is a
            // parameter error, so the caller knows to change the request.
            CoreError::EmptyText
            | CoreError::TextTooLong { .. }
            | CoreError::TooManyNotes { .. }
            | CoreError::UnknownNote(_) => CmdError::Params(error.to_string()),
            CoreError::IdsExhausted => CmdError::Failed(error.to_string()),
        }
    }
}

pub type CmdResult = Result<Value, CmdError>;
