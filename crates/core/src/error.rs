use thiserror::Error;

use crate::NoteId;

/// Everything that can go wrong while enforcing a document invariant.
///
/// Variants name the rule that was broken and carry the bound that was
/// exceeded, so a caller can report a useful message without repeating the
/// limit in its own text.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    #[error("the note text is empty")]
    EmptyText,
    #[error("the note text is longer than {max} bytes")]
    TextTooLong { max: usize },
    #[error("a document holds at most {max} notes")]
    TooManyNotes { max: usize },
    #[error("there is no note with id {0}")]
    UnknownNote(NoteId),
    #[error("note ids are exhausted")]
    IdsExhausted,
}

pub type CoreResult<T> = Result<T, CoreError>;
