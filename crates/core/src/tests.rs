use crate::{CoreError, Document, MAX_NOTE_BYTES, MAX_NOTES, NoteId};

#[test]
fn a_note_is_added_with_a_fresh_id() {
    let mut doc = Document::new();
    let first = doc.add_note("buy milk").expect("a short note is valid");
    let second = doc.add_note("book a room").expect("a short note is valid");
    assert_ne!(first, second);
    assert_eq!(doc.len(), 2);
    assert_eq!(doc.get(first).map(|n| n.text.as_str()), Some("buy milk"));
}

#[test]
fn text_is_trimmed_before_storage() {
    let mut doc = Document::new();
    let id = doc
        .add_note("  padded  ")
        .expect("text with padding is valid");
    assert_eq!(doc.get(id).map(|n| n.text.as_str()), Some("padded"));
}

#[test]
fn empty_and_blank_text_is_rejected() {
    let mut doc = Document::new();
    for text in ["", " ", "\n\t ", "\u{00a0}"] {
        assert_eq!(doc.add_note(text), Err(CoreError::EmptyText), "{text:?}");
    }
    assert!(doc.is_empty());
}

#[test]
fn oversized_text_is_rejected_before_it_is_trimmed() {
    let mut doc = Document::new();
    let text = "a".repeat(MAX_NOTE_BYTES + 1);
    assert_eq!(
        doc.add_note(&text),
        Err(CoreError::TextTooLong {
            max: MAX_NOTE_BYTES
        })
    );
    assert!(doc.is_empty());
}

#[test]
fn the_note_count_is_bounded() {
    let mut doc = Document::new();
    for _ in 0..MAX_NOTES {
        doc.add_note("x").expect("within the bound");
    }
    assert_eq!(
        doc.add_note("one more"),
        Err(CoreError::TooManyNotes { max: MAX_NOTES })
    );
    assert_eq!(doc.len(), MAX_NOTES);
}

#[test]
fn removing_a_note_keeps_the_other_ids_stable() {
    let mut doc = Document::new();
    let first = doc.add_note("first").expect("valid");
    let second = doc.add_note("second").expect("valid");
    doc.remove_note(first).expect("the note exists");
    assert_eq!(doc.len(), 1);
    assert_eq!(doc.get(second).map(|n| n.text.as_str()), Some("second"));
    assert!(doc.get(first).is_none());
}

#[test]
fn removing_twice_reports_the_stale_id() {
    let mut doc = Document::new();
    let id = doc.add_note("only").expect("valid");
    doc.remove_note(id).expect("the note exists");
    assert_eq!(doc.remove_note(id), Err(CoreError::UnknownNote(id)));
}

#[test]
fn ids_are_not_reused_after_clear() {
    let mut doc = Document::new();
    let old = doc.add_note("before").expect("valid");
    assert_eq!(doc.clear(), 1);
    let new = doc.add_note("after").expect("valid");
    assert_ne!(old, new, "a stale id must not point at a new note");
}

#[test]
fn the_id_is_printable() {
    let mut doc = Document::new();
    let id = doc.add_note("x").expect("valid");
    assert_eq!(NoteId::get(id).to_string(), id.to_string());
}
