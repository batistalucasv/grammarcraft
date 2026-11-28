use grammarcraft_core::{
    Audience, Context, Document, IgnoredIssueKey, IgnoredIssueSet, IssueCategory,
    PersonalDictionary, StyleGuide, StyleRule, StyleRuleKind, ToneSignalKind, WritingGoals,
};

use crate::{AnalysisEngine, validate_style_rule};

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
            &StyleGuide::new(),
        )
        .expect("analysis succeeds");

    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "proof.spelling" && i.span.slice(doc.text()).ok() == Some("Teh"))
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice")
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.redundancy"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("to"))
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
    let guide = StyleGuide::new();

    let default_report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert_eq!(engine.cache_misses(), 1);
    assert!(
        default_report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.passive_voice")
    );

    let _ = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert_eq!(engine.cache_hits(), 1);

    let academic_goals = WritingGoals {
        audience: Audience::Specialized,
        context: Context::Academic,
        ..WritingGoals::default()
    };
    let academic_report = engine
        .analyze(&doc, Some("en-US"), academic_goals, &dict, &ignored, &guide)
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
            &StyleGuide::new(),
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
            &StyleGuide::new(),
        )
        .expect("valid");
    assert!(!report.issues.iter().any(|i| i.rule_id == "proof.spelling"));
    assert!(
        !report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.redundancy")
    );
}

#[test]
fn contextual_english_rules_catch_errors_without_false_positives() {
    let mut engine = AnalysisEngine::new();
    let dict = PersonalDictionary::new();
    let ignored = IgnoredIssueSet::new();
    let guide = StyleGuide::new();

    let mut bad = Document::new();
    bad.set_text("Their is a way better then that, and your welcome.")
        .expect("valid");
    let report = engine
        .analyze(
            &bad,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    let contextual: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "contextual.confusable")
        .collect();
    assert_eq!(contextual.len(), 3, "got {contextual:?}");

    let mut good = Document::new();
    good.set_text("There is a way better than that, and you're welcome.")
        .expect("valid");
    let clean = engine
        .analyze(
            &good,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        !clean
            .issues
            .iter()
            .any(|i| i.rule_id.starts_with("contextual.")),
        "unexpected contextual issue in clean text: {:?}",
        clean.issues
    );
}

#[test]
fn contextual_portuguese_rules_catch_paronyms_without_false_positives() {
    let mut engine = AnalysisEngine::new();
    let dict = PersonalDictionary::new();
    let ignored = IgnoredIssueSet::new();
    let guide = StyleGuide::new();

    let mut bad = Document::new();
    bad.set_text("Ele chegou a anos com mal humor e perguntou porque?")
        .expect("valid");
    let report = engine
        .analyze(
            &bad,
            Some("pt-BR"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "contextual.confusable"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("há"))
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "contextual.confusable"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("mau"))
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "contextual.por_que"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("por quê"))
    );

    let mut good = Document::new();
    good.set_text("Ele chegou há anos com mau humor e explicou o porquê.")
        .expect("valid");
    let clean = engine
        .analyze(
            &good,
            Some("pt-BR"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        !clean
            .issues
            .iter()
            .any(|i| i.rule_id.starts_with("contextual.")),
        "unexpected contextual issue: {:?}",
        clean.issues
    );
}

#[test]
fn contextual_spanish_rules_catch_errors_without_false_positives() {
    let mut engine = AnalysisEngine::new();
    let dict = PersonalDictionary::new();
    let ignored = IgnoredIssueSet::new();
    let guide = StyleGuide::new();

    let mut bad = Document::new();
    bad.set_text(
        "Haber si vamos por hay; ya ha echo una ves eso, ojalá halla paz sino vienes. ¿Porque no?",
    )
    .expect("valid");
    let report = engine
        .analyze(
            &bad,
            Some("es-ES"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    let replacements: Vec<&str> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "contextual.confusable")
        .filter_map(|i| i.replacements.first().map(|r| r.text.as_str()))
        .collect();
    assert!(replacements.contains(&"A ver"), "got {replacements:?}");
    assert!(replacements.contains(&"ahí"), "got {replacements:?}");
    assert!(replacements.contains(&"hecho"), "got {replacements:?}");
    assert!(replacements.contains(&"vez"), "got {replacements:?}");
    assert!(replacements.contains(&"haya"), "got {replacements:?}");
    assert!(replacements.contains(&"si no"), "got {replacements:?}");
    assert!(replacements.contains(&"Por qué"), "got {replacements:?}");

    let mut good = Document::new();
    good.set_text(
        "A ver si vamos por ahí; ya ha hecho una vez eso, espero que haya paz si no vienes y va a haber fiesta; lo echo de menos. ¿Por qué no?",
    )
    .expect("valid");
    let clean = engine
        .analyze(
            &good,
            Some("es-ES"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        !clean
            .issues
            .iter()
            .any(|i| i.rule_id.starts_with("contextual.")),
        "unexpected Spanish contextual issue in clean text: {:?}",
        clean.issues
    );
}

#[test]
fn regional_variant_mix_is_flagged_in_english_and_portuguese() {
    let mut engine = AnalysisEngine::new();
    let dict = PersonalDictionary::new();
    let ignored = IgnoredIssueSet::new();
    let guide = StyleGuide::new();

    let mut en_doc = Document::new();
    en_doc
        .set_text("The color and the colour must match.")
        .expect("valid");
    let en_report = engine
        .analyze(
            &en_doc,
            Some("en-US"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        en_report
            .issues
            .iter()
            .any(|i| i.rule_id == "variants.en_us"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("color"))
    );

    let mut pt_doc = Document::new();
    pt_doc
        .set_text("O facto foi um ótimo projeto para a equipa.")
        .expect("valid");
    let pt_report = engine
        .analyze(
            &pt_doc,
            Some("pt-BR"),
            WritingGoals::default(),
            &dict,
            &ignored,
            &guide,
        )
        .expect("valid");
    assert!(
        pt_report
            .issues
            .iter()
            .any(|i| i.rule_id == "variants.pt_br"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("fato"))
    );
}

#[test]
fn style_guide_rules_evaluate_and_reject_invalid_regex() {
    let invalid = StyleRule::new(
        "bad-regex",
        StyleRuleKind::PatternRule {
            pattern: "[unclosed".into(),
            replacement: None,
            rationale: "broken".into(),
        },
    )
    .expect("syntactic fields valid");
    assert!(validate_style_rule(&invalid).is_err());

    let mut guide = StyleGuide::new();
    guide
        .add_rule(
            StyleRule::new(
                "prefer-use",
                StyleRuleKind::PreferredTerm {
                    discouraged: "utilize".into(),
                    preferred: "use".into(),
                    rationale: "Prefer plain wording.".into(),
                },
            )
            .expect("valid"),
        )
        .expect("added");
    guide
        .add_rule(
            StyleRule::new(
                "brand-case",
                StyleRuleKind::BrandCasing {
                    exact: "GrammarCraft".into(),
                    rationale: "Use official capitalization.".into(),
                },
            )
            .expect("valid"),
        )
        .expect("added");

    let mut doc = Document::new();
    doc.set_text("Teams utilize grammarcraft daily.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();
    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
            &guide,
        )
        .expect("valid");

    let style_issues: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.category == IssueCategory::StyleGuide)
        .collect();
    assert_eq!(style_issues.len(), 2, "got {style_issues:?}");
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
            &StyleGuide::new(),
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

#[test]
fn formality_and_inclusive_language_respect_goals_and_avoid_false_positives() {
    use grammarcraft_core::Formality;

    let mut doc = Document::new();
    doc.set_text("We don't allow the chairman to rely on humanity or a manual blacklist.")
        .expect("valid");
    let mut engine = AnalysisEngine::new();

    let default_report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
            &StyleGuide::new(),
        )
        .expect("valid");
    assert!(
        !default_report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.formality" || i.rule_id == "style.inclusive"),
        "expected neutral opt-out defaults to skip formal contractions and inclusive rules"
    );

    let formal_inclusive = WritingGoals {
        formality: Formality::Formal,
        inclusive_language: true,
        ..WritingGoals::default()
    };
    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            formal_inclusive,
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
            &StyleGuide::new(),
        )
        .expect("valid");
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "clarity.formality"
                && i.replacements.first().map(|r| r.text.as_str()) == Some("do not"))
    );
    let inclusive_hits: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "style.inclusive")
        .collect();
    assert_eq!(inclusive_hits.len(), 2, "got {inclusive_hits:?}");
}

#[test]
fn local_similarity_detects_overlapping_passages_deterministically() {
    use crate::{LocalCorpusDoc, check_local_similarity};

    let doc_text = "Clean-room verification prevents license contamination across open source crates. Completely original closing remark.";
    let corpus = [
        LocalCorpusDoc {
            source_name: "policy.md".to_owned(),
            text: "Clean-room verification prevents license contamination across open source crates in practice.".to_owned(),
        },
        LocalCorpusDoc {
            source_name: "unrelated.txt".to_owned(),
            text: "Astronomers measured distant galaxies using infrared spectroscopy.".to_owned(),
        },
    ];
    let report = check_local_similarity(doc_text, &corpus).expect("similarity check succeeds");
    assert_eq!(report.sources_scanned, 2);
    assert!(report.overall_percent > 0);
    assert_eq!(report.matches.len(), 1);
    assert_eq!(report.matches[0].source_name, "policy.md");
    assert!(report.matches[0].jaccard_percent >= 50);
    assert!(report.matches[0].span.slice(doc_text).is_ok());
}

#[test]
fn analysis_engine_integrates_upstream_ready_punctuation_and_spacing_rules() {
    let mut doc = Document::new();
    doc.set_text("Check ( this ),next.").expect("valid");
    let mut engine = AnalysisEngine::new();
    let report = engine
        .analyze(
            &doc,
            Some("en-US"),
            WritingGoals::default(),
            &PersonalDictionary::new(),
            &IgnoredIssueSet::new(),
            &StyleGuide::new(),
        )
        .expect("analysis succeeds");

    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "proof.bracket_spacing")
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.rule_id == "proof.missing_space")
    );
}
