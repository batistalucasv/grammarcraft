use crate::{
    Audience, Context, CoreError, Document, Formality, MAX_DOCUMENT_BYTES, MAX_GOAL_VALUE_BYTES,
    Purpose, TextSpan, TextStats, WritingGoals,
};

#[test]
fn setting_text_stores_exact_content_and_measures_stats() {
    let mut doc = Document::new();
    let stats = doc
        .set_text("  First sentence. Second sentence!  ")
        .expect("non-blank text is accepted");
    assert_eq!(doc.text(), "  First sentence. Second sentence!  ");
    assert_eq!(stats.words, 4);
    assert_eq!(stats.sentences, 2);
    assert_eq!(stats.paragraphs, 1);
    assert!(!doc.is_empty());
}

#[test]
fn empty_and_blank_text_is_rejected() {
    let mut doc = Document::new();
    for text in ["", " ", "\n\t ", "\u{00a0}"] {
        assert_eq!(doc.set_text(text), Err(CoreError::EmptyText), "{text:?}");
    }
    assert!(doc.is_empty());
}

#[test]
fn max_document_bytes_is_accepted_and_max_plus_one_is_rejected() {
    let mut doc = Document::new();
    let exact = "a".repeat(MAX_DOCUMENT_BYTES);
    let stats = doc.set_text(&exact).expect("MAX_DOCUMENT_BYTES fits");
    assert_eq!(stats.bytes, MAX_DOCUMENT_BYTES);

    let too_long = "b".repeat(MAX_DOCUMENT_BYTES + 1);
    assert_eq!(
        doc.set_text(&too_long),
        Err(CoreError::TextTooLong {
            max: MAX_DOCUMENT_BYTES
        })
    );
    assert_eq!(
        doc.len_bytes(),
        MAX_DOCUMENT_BYTES,
        "rejected input must not overwrite the existing document"
    );
}

#[test]
fn multibyte_utf8_spans_enforce_char_boundaries() {
    let mut doc = Document::new();
    doc.set_text("Olá, café!").expect("valid UTF-8 text");
    // 'á' starts at byte 2 and takes 2 bytes (bytes 2..4); 'é' is bytes 8..10.
    let valid = doc.span(0, 4).expect("valid char boundaries");
    assert_eq!(doc.slice(valid).expect("valid slice"), "Olá");
    assert_eq!(valid.len(), 4);
    assert!(!valid.is_empty());

    assert_eq!(
        doc.span(0, 3),
        Err(CoreError::NonCharBoundary { offset: 3 })
    );
    assert_eq!(
        doc.span(3, 4),
        Err(CoreError::NonCharBoundary { offset: 3 })
    );
    assert_eq!(
        doc.span(5, 2),
        Err(CoreError::InvalidSpan {
            start: 5,
            end: 2,
            len: doc.len_bytes(),
        })
    );
    assert_eq!(
        TextSpan::new(doc.text(), 0, doc.len_bytes() + 1),
        Err(CoreError::InvalidSpan {
            start: 0,
            end: doc.len_bytes() + 1,
            len: doc.len_bytes(),
        })
    );
}

#[test]
fn stats_measure_handles_empty_decimals_and_multiple_paragraphs() {
    assert_eq!(TextStats::measure(""), TextStats::default());

    let text = "Versão 3.14 funciona bem!\n\nSegunda linha com ação.\n";
    let stats = TextStats::measure(text);
    assert_eq!(stats.bytes, text.len());
    assert_eq!(stats.chars, text.chars().count());
    assert_eq!(stats.words, 8);
    assert_eq!(stats.sentences, 2);
    assert_eq!(stats.paragraphs, 2);
}

#[test]
fn writing_goals_parse_valid_and_reject_invalid_values() {
    let goals = WritingGoals {
        audience: Audience::parse(" Technical-Executive ").expect("valid audience"),
        formality: Formality::parse("FORMAL").expect("valid formality"),
        context: Context::parse("academic").expect("valid context"),
        purpose: Purpose::parse("persuade").expect("valid purpose"),
    };
    assert_eq!(goals.audience, Audience::TechnicalExecutive);
    assert_eq!(goals.formality, Formality::Formal);
    assert_eq!(goals.context, Context::Academic);
    assert_eq!(goals.purpose, Purpose::Persuade);

    assert!(matches!(
        Audience::parse("unknown"),
        Err(CoreError::InvalidGoal {
            field: "audience",
            ..
        })
    ));
    let oversized = "x".repeat(MAX_GOAL_VALUE_BYTES + 1);
    assert!(matches!(
        Purpose::parse(&oversized),
        Err(CoreError::InvalidGoal {
            field: "purpose",
            ..
        })
    ));
}

#[test]
fn clear_empties_document_and_returns_removed_bytes() {
    let mut doc = Document::new();
    doc.set_text("hello").expect("valid text");
    assert_eq!(doc.clear(), 5);
    assert!(doc.is_empty());
    assert_eq!(doc.clear(), 0);
}
