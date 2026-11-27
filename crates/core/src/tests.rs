use crate::{
    Audience, Confidence, Context, CoreError, Document, Formality, IgnoredIssueKey,
    IgnoredIssueSet, IssueCategory, IssueId, IssueList, MAX_CUSTOM_WORDS, MAX_DOCUMENT_BYTES,
    MAX_GOAL_VALUE_BYTES, MAX_IGNORED_ISSUES, MAX_ISSUES, MAX_SUGGESTIONS_PER_ISSUE,
    MAX_TERM_BYTES, PersonalDictionary, Purpose, Replacement, Severity, TextSpan, TextStats,
    WritingGoals, WritingScore,
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
fn replace_span_updates_text_and_rejects_oversized_replacement() {
    let mut doc = Document::new();
    doc.set_text("The cat slept.").expect("valid");
    let span = doc.span(4, 7).expect("cat span");
    doc.replace_span(span, "dog").expect("valid replacement");
    assert_eq!(doc.text(), "The dog slept.");

    let huge = "x".repeat(MAX_DOCUMENT_BYTES);
    let tiny_span = doc.span(0, 3).expect("tiny span");
    assert_eq!(
        doc.replace_span(tiny_span, &huge),
        Err(CoreError::TextTooLong {
            max: MAX_DOCUMENT_BYTES
        })
    );
    assert_eq!(doc.text(), "The dog slept.");
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

#[test]
fn issue_list_enforces_monotonic_ids_and_max_bounds() {
    let text = "hello world";
    let span = TextSpan::new(text, 0, 5).expect("valid span");
    let mut list = IssueList::new();

    let too_many_suggestions = (0..=MAX_SUGGESTIONS_PER_ISSUE)
        .map(|i| Replacement::new(format!("r{i}")))
        .collect();
    assert_eq!(
        list.push(
            "test.rule",
            IssueCategory::SpellingAndGrammar,
            Severity::Error,
            Confidence::High,
            span,
            "Title",
            "Explanation",
            too_many_suggestions,
        ),
        Err(CoreError::TooManySuggestions {
            max: MAX_SUGGESTIONS_PER_ISSUE
        })
    );

    let mut prev = IssueId::from_raw(0);
    for _ in 0..MAX_ISSUES {
        let id = list
            .push(
                "test.rule",
                IssueCategory::ClarityAndConciseness,
                Severity::Suggestion,
                Confidence::Medium,
                span,
                "Title",
                "Explanation",
                vec![Replacement::new("hi")],
            )
            .expect("within MAX_ISSUES");
        assert!(id > prev);
        prev = id;
    }
    assert_eq!(
        list.push(
            "test.rule",
            IssueCategory::ClarityAndConciseness,
            Severity::Suggestion,
            Confidence::Medium,
            span,
            "Title",
            "Explanation",
            vec![],
        ),
        Err(CoreError::TooManyIssues { max: MAX_ISSUES })
    );
}

#[test]
fn personal_dictionary_validates_terms_and_bounds() {
    let mut dict = PersonalDictionary::new();
    assert!(dict.add("  GrammarCraft ").expect("valid term"));
    assert!(!dict.add("grammarcraft").expect("duplicate is no-op"));
    assert!(dict.contains("GRAMMARCRAFT"));

    assert_eq!(dict.add(""), Err(CoreError::EmptyTerm));
    assert_eq!(
        dict.add(&"a".repeat(MAX_TERM_BYTES + 1)),
        Err(CoreError::TermTooLong {
            max: MAX_TERM_BYTES
        })
    );
    assert!(matches!(
        dict.add("two words"),
        Err(CoreError::InvalidTerm(_))
    ));
    assert!(dict.remove("grammarcraft").expect("valid removal"));
    assert!(!dict.contains("grammarcraft"));

    for i in 0..MAX_CUSTOM_WORDS {
        dict.add(&format!("w{i}")).expect("within MAX_CUSTOM_WORDS");
    }
    assert_eq!(
        dict.add("overflow"),
        Err(CoreError::TooManyCustomWords {
            max: MAX_CUSTOM_WORDS
        })
    );
}

#[test]
fn ignored_issue_set_is_bounded() {
    let mut set = IgnoredIssueSet::new();
    for i in 0..MAX_IGNORED_ISSUES {
        let key = IgnoredIssueKey::new("rule", i, "word").expect("valid key");
        assert!(set.ignore(key).expect("within MAX_IGNORED_ISSUES"));
    }
    let extra = IgnoredIssueKey::new("rule", MAX_IGNORED_ISSUES, "word").expect("valid key");
    assert_eq!(
        set.ignore(extra),
        Err(CoreError::TooManyIgnoredIssues {
            max: MAX_IGNORED_ISSUES
        })
    );
}

#[test]
fn writing_score_drops_with_open_issues_and_is_100_when_clean() {
    assert_eq!(WritingScore::compute(&[], 100).score, 100);
    let text = "One two three four five.";
    let span = TextSpan::new(text, 0, 3).expect("valid");
    let mut list = IssueList::new();
    list.push(
        "spell",
        IssueCategory::SpellingAndGrammar,
        Severity::Error,
        Confidence::High,
        span,
        "Spelling",
        "Check spelling",
        vec![],
    )
    .expect("fits");
    let score = WritingScore::compute(list.items(), 20);
    assert_eq!(score.score, 80);
    assert_eq!(score.breakdown.spelling_and_grammar, 1);
}
