use grammarcraft_core::{
    Audience, Context, Formality, MAX_DOCUMENT_BYTES, MAX_UNDO_STEPS, Purpose,
};
use serde_json::{Value, json};

use crate::file_access::{self, AutomationRoots};
use crate::{CmdError, Session, registry};

fn run(session: &mut Session, id: &str, params: Value) -> Result<Value, CmdError> {
    session.run(id, &params)
}

#[test]
fn every_registered_id_is_unique_and_documented() {
    let mut seen = std::collections::HashSet::new();
    for spec in registry().all() {
        assert!(seen.insert(spec.id), "duplicate command id `{}`", spec.id);
        assert!(!spec.label.is_empty(), "`{}` has no label", spec.id);
        assert!(!spec.location.is_empty(), "`{}` has no location", spec.id);
        assert!(
            spec.params.trim_start().starts_with('{'),
            "`{}` params are not a JSON object",
            spec.id
        );
    }
}

#[test]
fn the_registry_reports_every_command() {
    let described = registry().describe();
    let ids: Vec<&str> = described
        .as_array()
        .expect("describe returns an array")
        .iter()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .collect();
    for expected in [
        "doc.set_text",
        "doc.get",
        "doc.clear",
        "doc.save",
        "doc.open",
        "history.undo",
        "history.redo",
        "goals.set",
        "stats.measure",
        "review.check",
        "review.apply",
        "review.dismiss",
        "dict.add",
        "dict.remove",
        "dict.import",
        "dict.export",
        "style.add_rule",
        "style.remove_rule",
        "style.list",
        "snippet.add",
        "snippet.remove",
        "snippet.expand",
        "snippet.list",
    ] {
        assert!(ids.contains(&expected), "missing `{expected}` in {ids:?}");
    }
    assert_eq!(ids.len(), registry().all().len());
}

#[test]
fn set_text_get_and_measure_round_trip() {
    let mut session = Session::new();
    let stats = run(
        &mut session,
        "doc.set_text",
        json!({"text": "First sentence. Second sentence!"}),
    )
    .expect("valid text");
    assert_eq!(stats.get("words").and_then(Value::as_u64), Some(4));
    assert_eq!(stats.get("sentences").and_then(Value::as_u64), Some(2));
    assert_eq!(stats.get("paragraphs").and_then(Value::as_u64), Some(1));

    let fetched = run(&mut session, "doc.get", json!({})).expect("pure read");
    assert_eq!(
        fetched.get("text").and_then(Value::as_str),
        Some("First sentence. Second sentence!")
    );
    let measured = run(&mut session, "stats.measure", json!({})).expect("pure stats");
    assert_eq!(measured, stats);
}

#[test]
fn undo_and_redo_restore_exact_state_and_bound_history() {
    let mut session = Session::new();
    assert!(matches!(
        run(&mut session, "history.undo", json!({})),
        Err(CmdError::Disabled(_))
    ));

    run(&mut session, "doc.set_text", json!({"text": "first"})).expect("set 1");
    run(&mut session, "doc.set_text", json!({"text": "second"})).expect("set 2");
    assert_eq!(session.doc().text(), "second");

    run(&mut session, "history.undo", json!({})).expect("undo to first");
    assert_eq!(session.doc().text(), "first");
    run(&mut session, "history.redo", json!({})).expect("redo to second");
    assert_eq!(session.doc().text(), "second");

    for i in 0..(MAX_UNDO_STEPS + 20) {
        run(
            &mut session,
            "doc.set_text",
            json!({"text": format!("v{i}")}),
        )
        .expect("bounded set");
    }
    assert_eq!(session.undo_len(), MAX_UNDO_STEPS);
}

#[test]
fn review_check_apply_dismiss_and_dict_round_trip() {
    let mut session = Session::new();
    run(
        &mut session,
        "doc.set_text",
        json!({"text": "Xylophonia in order to finish."}),
    )
    .expect("set");

    let initial = run(&mut session, "review.check", json!({})).expect("check");
    let issues = initial["issues"].as_array().expect("issues array");
    let redundancy_id = issues
        .iter()
        .find(|i| i["rule_id"] == "clarity.redundancy")
        .and_then(|i| i["id"].as_u64())
        .expect("redundancy issue id");

    run(
        &mut session,
        "review.apply",
        json!({"issue_id": redundancy_id}),
    )
    .expect("apply suggestion");
    assert_eq!(session.doc().text(), "Xylophonia to finish.");

    // Add unknown word to document dictionary and verify it disappears.
    let after_apply = run(&mut session, "review.check", json!({})).expect("recheck");
    assert!(
        after_apply["issues"]
            .as_array()
            .expect("issues")
            .iter()
            .any(|i| i["rule_id"] == "proof.spelling")
    );
    run(&mut session, "dict.add", json!({"word": "Xylophonia"})).expect("add word");
    let after_dict = run(&mut session, "review.check", json!({})).expect("recheck after dict");
    assert!(
        !after_dict["issues"]
            .as_array()
            .expect("issues")
            .iter()
            .any(|i| i["rule_id"] == "proof.spelling")
    );

    // Removing word brings spelling issue back, then dismiss suppresses it.
    run(&mut session, "dict.remove", json!({"word": "Xylophonia"})).expect("remove word");
    let after_remove = run(&mut session, "review.check", json!({})).expect("recheck");
    let spell_id = after_remove["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .find(|i| i["rule_id"] == "proof.spelling")
        .and_then(|i| i["id"].as_u64())
        .expect("spelling id");
    run(
        &mut session,
        "review.dismiss",
        json!({"issue_id": spell_id}),
    )
    .expect("dismiss");
    let after_dismiss = run(&mut session, "review.check", json!({})).expect("recheck");
    assert!(
        after_dismiss["issues"]
            .as_array()
            .expect("issues")
            .is_empty()
    );
}

#[test]
fn save_and_open_atomic_round_trip_and_confines_automation_roots() {
    let base = std::env::temp_dir().join(format!("gc-engine-io-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let read_dir = base.join("read");
    let write_dir = base.join("write");
    let outside_dir = base.join("outside");
    std::fs::create_dir_all(&read_dir).expect("read dir");
    std::fs::create_dir_all(&write_dir).expect("write dir");
    std::fs::create_dir_all(&outside_dir).expect("outside dir");
    std::fs::write(read_dir.join("note.txt"), "Sample in read root.").expect("seed file");
    std::fs::write(outside_dir.join("secret.txt"), "Secret").expect("outside file");

    let roots = AutomationRoots::new(Some(&read_dir), Some(&write_dir))
        .expect("valid roots")
        .expect("some roots");

    file_access::confine(Some(&roots), || {
        let mut session = Session::new();
        // Reading inside read_dir succeeds.
        let read_path = read_dir.join("note.txt").to_string_lossy().into_owned();
        run(&mut session, "doc.open", json!({"path": read_path})).expect("open inside read root");
        assert_eq!(session.doc().text(), "Sample in read root.");

        // Reading outside read_dir fails.
        let outside_read = outside_dir
            .join("secret.txt")
            .to_string_lossy()
            .into_owned();
        let err = run(&mut session, "doc.open", json!({"path": outside_read}))
            .expect_err("read outside root must fail");
        assert!(matches!(err, CmdError::Failed(m) if m.contains("outside the read root")));

        // Traversal out of write_dir fails.
        let traversal = format!(
            "{}/../outside/escape.grammarcraft.json",
            write_dir.display()
        );
        let err = run(&mut session, "doc.save", json!({"path": traversal}))
            .expect_err("traversal outside write root must fail");
        assert!(matches!(err, CmdError::Failed(m) if m.contains("outside the write root")));

        // Writing inside write_dir succeeds atomically.
        run(
            &mut session,
            "goals.set",
            json!({"formality": "formal", "context": "academic"}),
        )
        .expect("goals");
        run(&mut session, "dict.add", json!({"word": "GrammarCraft"})).expect("dict");
        let save_path = write_dir
            .join("saved.grammarcraft.json")
            .to_string_lossy()
            .into_owned();
        run(&mut session, "doc.save", json!({"path": save_path})).expect("save inside write root");
    });

    // Open saved JSON unconfined and verify round-trip fidelity.
    let mut restored = Session::new();
    let save_path = write_dir
        .join("saved.grammarcraft.json")
        .to_string_lossy()
        .into_owned();
    run(&mut restored, "doc.open", json!({"path": save_path})).expect("open saved json");
    assert_eq!(restored.doc().text(), "Sample in read root.");
    assert_eq!(restored.goals().formality, Formality::Formal);
    assert_eq!(restored.goals().context, Context::Academic);
    assert!(restored.personal_dict().contains("grammarcraft"));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn doc_clear_is_disabled_when_empty_and_clears_when_populated() {
    let mut session = Session::new();
    let error = run(&mut session, "doc.clear", json!({})).expect_err("document starts empty");
    assert!(
        matches!(&error, CmdError::Disabled(reason) if reason.contains("empty")),
        "{error:?}"
    );

    run(&mut session, "doc.set_text", json!({"text": "hello"})).expect("valid set");
    let cleared = run(&mut session, "doc.clear", json!({})).expect("document is populated");
    assert_eq!(
        cleared.get("cleared_bytes").and_then(Value::as_u64),
        Some(5)
    );
    assert!(session.doc().is_empty());
}

#[test]
fn goals_set_updates_atomically_and_rejects_invalid_values() {
    let mut session = Session::new();
    let updated = run(
        &mut session,
        "goals.set",
        json!({
            "audience": "specialized",
            "formality": "formal",
            "context": "academic",
            "purpose": "explain"
        }),
    )
    .expect("valid goals");
    assert_eq!(
        updated.get("audience").and_then(Value::as_str),
        Some("specialized")
    );
    assert_eq!(session.goals().audience, Audience::Specialized);
    assert_eq!(session.goals().formality, Formality::Formal);
    assert_eq!(session.goals().context, Context::Academic);
    assert_eq!(session.goals().purpose, Purpose::Explain);

    let error = run(
        &mut session,
        "goals.set",
        json!({"audience": "general", "purpose": "not_a_purpose"}),
    )
    .expect_err("invalid purpose must be rejected");
    assert!(matches!(error, CmdError::Params(_)), "{error:?}");
    assert_eq!(session.goals().audience, Audience::Specialized);
}

#[test]
fn hostile_parameters_never_panic() {
    let junk = [
        json!(null),
        json!(true),
        json!(0),
        json!(-1),
        json!(1.5),
        json!(""),
        json!("x"),
        json!([]),
        json!([1, 2]),
        json!({}),
        json!({"text": 1}),
        json!({"text": ""}),
        json!({"text": "\u{0}\u{1}"}),
        json!({"text": "a".repeat(200_000)}),
        json!({"text": "a".repeat(MAX_DOCUMENT_BYTES + 1)}),
        json!({"path": ""}),
        json!({"path": "/nonexistent/nowhere/foo.txt"}),
        json!({"word": 123}),
        json!({"word": "a".repeat(512)}),
        json!({"issue_id": u64::MAX}),
        json!({"issue_id": 1e300}),
        json!({"audience": 123}),
        json!({"id": "r1", "kind": "pattern", "pattern": "[unclosed", "rationale": "x"}),
        json!({"prefix": "/greet", "values": 123}),
        json!({"unknown": {"nested": [true, null]}}),
    ];
    for spec in registry().all() {
        for params in &junk {
            let mut session = Session::new();
            let _ = session.run(spec.id, params);
        }
    }
}

#[test]
fn style_guide_and_snippet_commands_round_trip_with_undo() {
    let mut session = Session::new();
    run(
        &mut session,
        "doc.set_text",
        json!({"text": "We utilize grammarcraft today."}),
    )
    .expect("set text");

    run(
        &mut session,
        "style.add_rule",
        json!({
            "id": "prefer-use",
            "kind": "preferred_term",
            "discouraged": "utilize",
            "preferred": "use",
            "rationale": "Keep wording direct."
        }),
    )
    .expect("add preferred_term");
    run(
        &mut session,
        "style.add_rule",
        json!({
            "id": "brand-case",
            "kind": "brand_casing",
            "exact": "GrammarCraft",
            "rationale": "Use official casing."
        }),
    )
    .expect("add brand_casing");
    let listed = run(&mut session, "style.list", json!({})).expect("list");
    assert_eq!(listed["count"], 2);

    let report = run(&mut session, "review.check", json!({})).expect("check");
    let style_issues: Vec<_> = report["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .filter(|i| i["category"] == "style_guide")
        .collect();
    assert_eq!(style_issues.len(), 2);

    // Invalid pattern rule fails with Params and leaves guide intact.
    let err = run(
        &mut session,
        "style.add_rule",
        json!({"id": "bad", "kind": "pattern", "pattern": "(unclosed", "rationale": "x"}),
    )
    .expect_err("invalid regex");
    assert!(matches!(err, CmdError::Params(_)));

    run(
        &mut session,
        "style.remove_rule",
        json!({"id": "brand-case"}),
    )
    .expect("remove");
    assert_eq!(session.style_guide().len(), 1);
    run(&mut session, "history.undo", json!({})).expect("undo removal");
    assert_eq!(session.style_guide().len(), 2);

    run(
        &mut session,
        "snippet.add",
        json!({"prefix": "/sig", "title": "Signature", "body": "Regards, {{name}}"}),
    )
    .expect("add snippet");
    let expanded = run(
        &mut session,
        "snippet.expand",
        json!({"prefix": "/sig", "values": {"name": "Ana"}}),
    )
    .expect("expand");
    assert_eq!(expanded["text"], "Regards, Ana");
    assert_eq!(
        run(&mut session, "snippet.list", json!({})).expect("list")["count"],
        1
    );
    run(&mut session, "snippet.remove", json!({"prefix": "/sig"})).expect("remove");
    assert!(session.snippets().is_empty());
}

#[test]
fn dict_import_and_export_respect_automation_roots_and_utf16le() {
    let base = std::env::temp_dir().join(format!("gc-engine-dic-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let read_dir = base.join("read");
    let write_dir = base.join("write");
    let outside_dir = base.join("outside");
    std::fs::create_dir_all(&read_dir).expect("read");
    std::fs::create_dir_all(&write_dir).expect("write");
    std::fs::create_dir_all(&outside_dir).expect("outside");

    let mut seed = grammarcraft_core::PersonalDictionary::new();
    seed.add_with_case("ArtCraft", true).expect("seed");
    let dic_in = read_dir.join("CUSTOM.DIC");
    std::fs::write(&dic_in, seed.to_custom_dic_utf16le()).expect("write seed");

    let roots = AutomationRoots::new(Some(&read_dir), Some(&write_dir))
        .expect("roots")
        .expect("some");
    file_access::confine(Some(&roots), || {
        let mut session = Session::new();
        run(
            &mut session,
            "dict.import",
            json!({"path": dic_in.to_string_lossy()}),
        )
        .expect("import inside read root");
        assert!(session.personal_dict().contains("ArtCraft"));
        assert!(!session.personal_dict().contains("artcraft"));

        let out_file = write_dir.join("OUT.DIC");
        let exported = run(
            &mut session,
            "dict.export",
            json!({"path": out_file.to_string_lossy(), "encoding": "utf16le"}),
        )
        .expect("export inside write root");
        assert_eq!(exported["encoding"], "utf16le");

        let outside_file = outside_dir.join("BAD.DIC");
        let err = run(
            &mut session,
            "dict.export",
            json!({"path": outside_file.to_string_lossy()}),
        )
        .expect_err("outside write root blocked");
        assert!(matches!(err, CmdError::Failed(m) if m.contains("outside the write root")));
    });

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn docx_save_and_open_round_trips_and_rejects_corrupted_bytes() {
    let base = std::env::temp_dir().join(format!("gc-engine-docx-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");

    let docx_path = base.join("sample.docx").to_string_lossy().into_owned();
    let text = "First paragraph.\nSecond paragraph with café.";

    let mut session = Session::new();
    run(&mut session, "doc.set_text", json!({"text": text})).expect("set text");
    let saved = run(&mut session, "doc.save", json!({"path": docx_path})).expect("save docx");
    assert_eq!(saved["format"], "docx");

    let mut restored = Session::new();
    let opened = run(&mut restored, "doc.open", json!({"path": docx_path})).expect("open docx");
    assert_eq!(opened["format"], "docx");
    assert_eq!(restored.doc().text(), text);

    let corrupt_path = base.join("corrupt.docx").to_string_lossy().into_owned();
    std::fs::write(&corrupt_path, b"PK\x03\x04not-a-valid-docx").expect("write corrupt");
    let err = run(&mut restored, "doc.open", json!({"path": corrupt_path}))
        .expect_err("corrupt docx returns Params error");
    assert!(matches!(err, CmdError::Params(_)), "{err:?}");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn ai_cloud_consent_blocks_without_grant_and_mock_provider_applies_with_undo() {
    use craft_ai_core::mock::{MockProvider, Outcome, Script};
    use craft_ai_core::{
        CapabilityId, CapabilitySpec, InputRole, ModelId, ModelInfo, Output, Payload,
    };
    use std::sync::Arc;

    let mut session = Session::new();
    run(
        &mut session,
        "doc.set_text",
        json!({"text": "Original draft sentence."}),
    )
    .expect("set text");

    // Switching to cloud without granting consent must block remote execution.
    run(
        &mut session,
        "ai.consent",
        json!({"granted": false, "locality": "cloud"}),
    )
    .expect("set cloud unconsented");
    let blocked = run(
        &mut session,
        "ai.rewrite",
        json!({"instruction": "tighten"}),
    )
    .expect_err("cloud call without consent must be blocked");
    assert!(matches!(blocked, CmdError::Disabled(m) if m.contains("consent")));

    // Wire a network-free MockProvider and grant consent.
    let model_id = ModelId::new("mock-writer").expect("valid model id");
    let spec = CapabilitySpec::new(CapabilityId::parse("text.complete").expect("cap"))
        .accepting(InputRole::System, 1)
        .accepting(InputRole::Prompt, 1);
    let provider = Arc::new(MockProvider::new(
        vec![ModelInfo::new(model_id, "Mock Writer").with(spec)],
        Script {
            running_polls: 0,
            text_chunks: Vec::new(),
            outcome: Outcome::Succeed(vec![Output {
                payload: Payload::text("Refined concise sentence."),
                label: None,
            }]),
        },
    ));
    session
        .set_cloud_provider("mock-writer", provider)
        .expect("wire mock provider");
    run(
        &mut session,
        "ai.consent",
        json!({"granted": true, "locality": "cloud"}),
    )
    .expect("grant consent");

    let staged =
        run(&mut session, "ai.rewrite", json!({})).expect("consented mock rewrite succeeds");
    assert_eq!(staged["proposal"]["proposed"], "Refined concise sentence.");
    assert_eq!(session.doc().text(), "Original draft sentence.");

    run(&mut session, "ai.applyResult", json!({})).expect("apply AI proposal");
    assert_eq!(session.doc().text(), "Refined concise sentence.");

    run(&mut session, "history.undo", json!({})).expect("undo AI proposal");
    assert_eq!(session.doc().text(), "Original draft sentence.");
}

#[test]
fn similarity_check_enforces_automation_roots_and_citations_format_all_styles() {
    let base = std::env::temp_dir().join(format!("gc-engine-m4-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let read_dir = base.join("corpus");
    let outside_dir = base.join("outside");
    std::fs::create_dir_all(&read_dir).expect("corpus dir");
    std::fs::create_dir_all(&outside_dir).expect("outside dir");
    std::fs::write(
        read_dir.join("ref.txt"),
        "Deterministic shingle overlap detection keeps user files local and auditable.",
    )
    .expect("seed corpus");

    let roots = AutomationRoots::new(Some(&read_dir), None)
        .expect("roots")
        .expect("some");
    file_access::confine(Some(&roots), || {
        let mut session = Session::new();
        run(
            &mut session,
            "doc.set_text",
            json!({"text": "Deterministic shingle overlap detection keeps user files local and auditable."}),
        )
        .expect("set text");

        let blocked = run(
            &mut session,
            "similarity.check",
            json!({"directory": outside_dir.to_string_lossy()}),
        )
        .expect_err("outside dir must be blocked");
        assert!(matches!(blocked, CmdError::Failed(m) if m.contains("outside the read root")));

        let sim = run(
            &mut session,
            "similarity.check",
            json!({"directory": read_dir.to_string_lossy()}),
        )
        .expect("authorized similarity check");
        assert!(sim["overall_percent"].as_u64().unwrap_or(0) >= 80);

        let bib = r#"@article{rust2026, author={Ada Lovelace}, title={Rust Craft}, journal={Systems}, year={2026}}"#;
        let val = run(&mut session, "citations.validate", json!({"bibtex": bib}))
            .expect("validate bibtex");
        assert_eq!(val["entry_count"], 1);
        for style in grammarcraft_core::CitationStyle::ALL {
            let formatted = run(
                &mut session,
                "citations.format",
                json!({"bibtex": bib, "style": style.as_str()}),
            )
            .expect("format style");
            assert!(
                formatted["formatted"][0]
                    .as_str()
                    .is_some_and(|s| s.contains("Rust Craft"))
            );
        }
    });

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn legacy_goals_json_without_inclusive_language_deserializes_cleanly() {
    let parsed: grammarcraft_core::WritingGoals = serde_json::from_str(
        r#"{"audience":"general","formality":"neutral","context":"general","purpose":"inform"}"#,
    )
    .expect("backward compatible goals json");
    assert!(!parsed.inclusive_language);
}

#[test]
fn ai_native_consent_history_and_memory_secrets_persist_in_isolated_dir() {
    use craft_ai_native::secrets::{MemoryStore, SecretStore};
    use grammarcraft_core::{AiConsentStatus, AiLocalityMode};
    use std::sync::Arc;

    let base = std::env::temp_dir().join(format!("gc-engine-native-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let consent_path = base.join("consent.json");
    let history_path = base.join("jobs.jsonl");
    let store: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());

    let mut session = Session::new();
    session.configure_ai_storage(
        Some(consent_path.clone()),
        Some(history_path.clone()),
        Arc::clone(&store),
    );
    session
        .store_ai_secret("openai-default", "sk-test-123")
        .expect("store in memory store");
    assert!(session.has_ai_secret("openai-default").expect("read back"));

    run(
        &mut session,
        "doc.set_text",
        json!({"text": "This change is due to the fact that we tested persistence."}),
    )
    .expect("set text");
    run(
        &mut session,
        "ai.consent",
        json!({"granted": true, "locality": "cloud"}),
    )
    .expect("persist granted consent");
    run(
        &mut session,
        "ai.consent",
        json!({"granted": true, "locality": "local"}),
    )
    .expect("switch to local for offline run");
    run(&mut session, "ai.rewrite", json!({})).expect("rewrite logs history");

    // Reopen consent + history from disk into a fresh Session without re-granting: consent remains granted.
    let mut reopened = Session::new();
    reopened.configure_ai_storage(
        Some(consent_path.clone()),
        Some(history_path.clone()),
        store,
    );
    reopened.set_ai_locality_mode(AiLocalityMode::CloudConsentRequired);
    assert!(matches!(
        reopened.ai_consent_status(),
        Ok(AiConsentStatus::Granted { .. })
    ));
    let entries = reopened.ai_history_entries().expect("read history");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].start.role_id, "rewrite");

    // Corrupt consent file fails closed (empty ledger -> RequiredPending).
    std::fs::write(&consent_path, b"{corrupt json").expect("corrupt file");
    let mut damaged = Session::new();
    damaged.configure_ai_storage(Some(consent_path), None, Arc::new(MemoryStore::default()));
    damaged.set_ai_locality_mode(AiLocalityMode::CloudConsentRequired);
    assert!(matches!(
        damaged.ai_consent_status(),
        Ok(AiConsentStatus::RequiredPending { .. })
    ));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn local_model_runner_verifies_sha256_and_takes_precedence_over_fallback_and_cloud() {
    use craft_ai_client::{LoadedModel, ModelRunner, RunContext};
    use craft_ai_core::{
        AiError, CapabilityId, CapabilitySpec, InputRole, ModelId, ModelInfo, Output, Payload,
        Request,
    };
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestLoadedModel {
        runs: Arc<AtomicUsize>,
    }

    impl LoadedModel for TestLoadedModel {
        fn run(&mut self, request: &Request, _ctx: &RunContext) -> Result<Vec<Output>, AiError> {
            self.runs.fetch_add(1, Ordering::AcqRel);
            let prompt = request
                .inputs
                .iter()
                .find(|input| input.role == InputRole::Prompt)
                .and_then(|input| match &input.payload {
                    Payload::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            let rewritten = if prompt.contains("draft") {
                "Refined by local ModelRunner.".to_owned()
            } else {
                "Local runner output.".to_owned()
            };
            Ok(vec![Output {
                payload: Payload::text(rewritten),
                label: None,
            }])
        }

        fn memory_bytes(&self) -> Option<u64> {
            Some(64 * 1024 * 1024)
        }
    }

    struct TestRunner {
        loads: Arc<AtomicUsize>,
        runs: Arc<AtomicUsize>,
    }

    impl ModelRunner for TestRunner {
        fn engine(&self) -> &str {
            "candle"
        }

        fn model(&self) -> ModelInfo {
            let cap = CapabilityId::parse("text.complete").expect("valid cap");
            ModelInfo::new(
                ModelId::new("grammar-rewrite-mini").expect("valid model id"),
                "Grammar Rewrite Mini",
            )
            .with(
                CapabilitySpec::new(cap)
                    .accepting(InputRole::System, 1)
                    .accepting(InputRole::Prompt, 1)
                    .requiring(InputRole::Prompt),
            )
        }

        fn load(&self, _weights: &Path) -> Result<Box<dyn LoadedModel>, AiError> {
            self.loads.fetch_add(1, Ordering::AcqRel);
            Ok(Box::new(TestLoadedModel {
                runs: Arc::clone(&self.runs),
            }))
        }
    }

    let base = std::env::temp_dir().join(format!("gc-local-runner-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let weights_path = base.join("model.bin");
    let payload = b"clean-room-test-weights-v1";
    std::fs::write(&weights_path, payload).expect("write dummy weights");

    let mut hex_sha = String::with_capacity(64);
    for b in Sha256::digest(payload) {
        let _ = write!(hex_sha, "{b:02x}");
    }

    let loads = Arc::new(AtomicUsize::new(0));
    let runs = Arc::new(AtomicUsize::new(0));
    let runner = Arc::new(TestRunner {
        loads: Arc::clone(&loads),
        runs: Arc::clone(&runs),
    });

    let mut session = Session::new();
    session
        .register_local_runner(runner, weights_path.clone(), Some(hex_sha))
        .expect("register local runner");

    let status = run(&mut session, "ai.status", json!({})).expect("ai.status");
    assert_eq!(status["local_engines"], json!(["candle"]));

    run(
        &mut session,
        "doc.set_text",
        json!({"text": "Initial draft sentence."}),
    )
    .expect("set text");

    // Local ModelRunner must be chosen automatically ahead of deterministic offline fallback.
    let first = run(&mut session, "ai.rewrite", json!({})).expect("first local runner rewrite");
    assert_eq!(
        first["proposal"]["proposed"],
        "Refined by local ModelRunner."
    );
    assert_eq!(
        first["proposal"]["provider_label"],
        "local-engine:candle/grammar-rewrite-mini"
    );
    assert_eq!(loads.load(Ordering::Acquire), 1);
    assert_eq!(runs.load(Ordering::Acquire), 1);

    // Even when CloudConsentRequired is set without remote consent, registered local ModelRunner
    // handles inference on-device without falling through to remote AI.
    run(
        &mut session,
        "ai.consent",
        json!({"granted": false, "locality": "cloud"}),
    )
    .expect("set cloud locality");
    let second = run(&mut session, "ai.rewrite", json!({})).expect("second rewrite reuses cache");
    assert_eq!(
        second["proposal"]["proposed"],
        "Refined by local ModelRunner."
    );
    assert_eq!(loads.load(Ordering::Acquire), 1, "model stays cached");
    assert_eq!(runs.load(Ordering::Acquire), 2);

    let loaded_info = session
        .loaded_local_model_info()
        .expect("loaded model info available");
    assert_eq!(loaded_info.bytes, 64 * 1024 * 1024);
    assert!(!loaded_info.estimated);

    let unloaded = session.unload_local_model();
    assert_eq!(unloaded.freed.len(), 1);
    assert!(session.loaded_local_model_info().is_none());

    // Tampered weights must be rejected by SHA-256 check BEFORE runner.load() is invoked.
    std::fs::write(&weights_path, b"corrupted-weights").expect("corrupt weights");
    let err = run(&mut session, "ai.rewrite", json!({})).expect_err("sha mismatch must fail");
    assert!(err.to_string().contains("SHA-256 mismatch"));
    assert_eq!(
        loads.load(Ordering::Acquire),
        1,
        "load() not called on bad hash"
    );

    let _ = std::fs::remove_dir_all(&base);
}
