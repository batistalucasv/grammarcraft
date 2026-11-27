use thiserror::Error;

/// Everything that can go wrong while enforcing a domain invariant.
///
/// Variants name the rule that was broken and carry the bound or field that
/// failed, so callers can distinguish invalid arguments from internal errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    #[error("the document text is empty")]
    EmptyText,
    #[error("the document text is longer than {max} bytes")]
    TextTooLong { max: usize },
    #[error("invalid span {start}..{end} for document of {len} bytes")]
    InvalidSpan {
        start: usize,
        end: usize,
        len: usize,
    },
    #[error("byte offset {offset} is not on a UTF-8 character boundary")]
    NonCharBoundary { offset: usize },
    #[error("invalid {field} goal `{value}`")]
    InvalidGoal { field: &'static str, value: String },
}

pub type CoreResult<T> = Result<T, CoreError>;
