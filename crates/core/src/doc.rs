use crate::{CoreError, CoreResult};

/// Most notes one document may hold.
///
/// A bound is not decoration: without it, a caller looping over untrusted
/// input can exhaust memory. Every collection in a domain type gets one.
pub const MAX_NOTES: usize = 10_000;

/// Longest note text accepted, in bytes.
pub const MAX_NOTE_BYTES: usize = 64 * 1024;

/// A note's identity, stable for the life of the note.
///
/// Deliberately opaque: ids are not indices, so removing one note never
/// renumbers the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteId(u64);

impl NoteId {
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Builds an id from a number that arrived from outside (JSON, a file, a
    /// socket). It does **not** check that the note exists: every operation
    /// that takes an id still looks it up and fails with
    /// [`CoreError::UnknownNote`].
    pub const fn from_raw(value: u64) -> Self {
        NoteId(value)
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One note. The text is trimmed but otherwise kept as given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: NoteId,
    pub text: String,
}

/// The document: an ordered list of notes.
///
/// `next_id` only grows, so an id is never reused after a removal. Reuse would
/// make a stale id silently point at a different note.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Document {
    notes: Vec<Note>,
    next_id: u64,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a note and returns its id.
    ///
    /// The length is checked *before* trimming and before allocating, because
    /// the caller's string may be arbitrarily large.
    pub fn add_note(&mut self, text: &str) -> CoreResult<NoteId> {
        if text.len() > MAX_NOTE_BYTES {
            return Err(CoreError::TextTooLong {
                max: MAX_NOTE_BYTES,
            });
        }
        let text = text.trim();
        if text.is_empty() {
            return Err(CoreError::EmptyText);
        }
        if self.notes.len() >= MAX_NOTES {
            return Err(CoreError::TooManyNotes { max: MAX_NOTES });
        }
        let next = self.next_id.checked_add(1).ok_or(CoreError::IdsExhausted)?;
        self.next_id = next;
        let id = NoteId(next);
        self.notes.push(Note {
            id,
            text: text.to_owned(),
        });
        Ok(id)
    }

    /// Removes a note. The id must exist; removing twice is an error, not a
    /// silent success, so a caller notices a stale id.
    pub fn remove_note(&mut self, id: NoteId) -> CoreResult<()> {
        let position = self
            .notes
            .iter()
            .position(|n| n.id == id)
            .ok_or(CoreError::UnknownNote(id))?;
        self.notes.remove(position);
        Ok(())
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn get(&self, id: NoteId) -> Option<&Note> {
        self.notes.iter().find(|n| n.id == id)
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// Drops every note. Ids already handed out stay invalid: they are not
    /// reused after a reset.
    pub fn clear(&mut self) -> usize {
        let removed = self.notes.len();
        self.notes.clear();
        removed
    }
}
