use grammarcraft_core::{
    Audience, Context, Document, IgnoredIssueKey, IgnoredIssueSet, IssueCategory,
    PersonalDictionary, ToneSignalKind, WritingGoals,
};

use crate::AnalysisEngine;

#[test]
fn detects_english_spelling_redundancy_and_passive_voice() {
    let mut doc = Document::new();
    doc.set_text("Teh report was reviewed in order to finish.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
        )
        .expect("analysis succeeds");

    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "proof.spelling" && i.span.slice(doc.text()).ok() == Some("Teh")),
        "expected spelling issue in {:?}",
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice"),
        "expected passive voice issue in {:?}",
        report.issues
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.redundancy"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("to")),
        "expected redundancy replacement in {:?}",
        report.issues
    );
    assert!(report.score.score < 100);
    assert!(report.readability.reading_minutes > 0.0);
}

#[test]
fn academic_specialized_goals_suppress_passive_voice_and_invalidate_cache() {
    let mut doc = Document::new();
    doc.set_text("The sample was reviewed carefully.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let dict = PersonalDictionary::new();
    let ignored = IgnoredIssueSet::new();

    let default_report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
        )
        .expect("valid");
    assert_eq!(engine.cache_misses(), 1);
    assert!(
        default_report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice")
    );

    // Identical call hits the incremental paragraph cache.
    let _ = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
        )
        .expect("valid");
    assert_eq!(engine.cache_hits(), 1);

    // Changing goals invalidates the paragraph cache key and suppresses passive voice.
    let academic_goals = WritingGoals {
        audience: Audience::Specialized,
        context: Context::Academic,
        ..WritingGoals::default()
    };
    let academic_report = engine
        .analyze(&doc, Some("en-US"), academic_goals, &dict, &ignored)
        .expect("valid");
    assert_eq!(engine.cache_misses(), 2);
    assert!(
        !academic_report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice")
    );
}

#[test]
fn portuguese_redundancy_passive_and_repetition_work() {
    let mut doc = Document::new();
    doc.set_text("O documento foi revisado com o intuito de melhorar texto com texto claro.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let report = engine
        .analyze(
            &doc,
            Some("pt-BR"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
        )
        .expect("valid");

    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.redundancy"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("para"))
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice")
    );
    assert!(report.issues.iter().any(
        |i| i.rule_id == "style.repetition" && i.category == IssueCategory::StyleAndVocabulary
    ));
}

#[test]
fn personal_dictionary_and_ignored_set_stay_document_scoped() {
    let mut doc = Document::new();
    doc.set_text("Xylophonia is in order to help.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let mut dict = PersonalDictionary::new();
    dict.add("Xylophonia").expect("valid word");

    let mut ignored = IgnoredIssueSet::new();
    let key = IgnoredIssueKey::new("clarity.redundancy", 14, "in order to").expect("valid key");
    ignored.ignore(key).expect("ignored");

    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
        )
        .expect("valid");
    assert!(
        !report.issues.iter().any(|i| i.rule_id == "proof.spelling"),
        "personal dictionary must suppress Xylophonia"
    );
    assert!(
        !report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.redundancy"),
        "ignored issue set must suppress dismissed redundancy"
    );
}

#[test]
fn tone_profile_anchors_evidence_spans() {
    let mut doc = Document::new();
    doc.set_text("Please review this urgent request immediately!")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
        )
        .expect("valid");

    assert!(!report.tone.dominant.is_empty());
    let top = &report.tone.dominant[0];
    assert_eq!(top.kind, ToneSignalKind::Urgent);
    assert!(!top.evidence_spans.is_empty());
    for span in &top.evidence_spans {
        assert!(span.slice(doc.text()).is_ok());
    }
}
