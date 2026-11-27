use grammarcraft_core::{Audience, Context, Formality, MAX_DOCUMENT_BYTES, Purpose};
use serde_json::{Value, json};

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
        "goals.set",
        "stats.measure",
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
    assert_eq!(
        fetched
            .get("stats")
            .and_then(|s| s.get("words"))
            .and_then(Value::as_u64),
        Some(4)
    );

    let measured = run(&mut session, "stats.measure", json!({})).expect("pure stats");
    assert_eq!(measured, stats);
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

    // Invalid `purpose` must fail without applying the new `audience`.
    let error = run(
        &mut session,
        "goals.set",
        json!({"audience": "general", "purpose": "not_a_purpose"}),
    )
    .expect_err("invalid purpose must be rejected");
    assert!(matches!(error, CmdError::Params(_)), "{error:?}");
    assert_eq!(session.goals().audience, Audience::Specialized);

    let mistyped =
        run(&mut session, "goals.set", json!({"formality": 42})).expect_err("non-string goal");
    assert!(
        matches!(&mistyped, CmdError::Params(msg) if msg.contains("formality")),
        "{mistyped:?}"
    );
}

#[test]
fn an_unknown_command_is_an_error() {
    let mut session = Session::new();
    let error = run(&mut session, "doc.teleport", json!({})).expect_err("the id does not exist");
    assert!(matches!(error, CmdError::UnknownCommand(id) if id == "doc.teleport"));
}

#[test]
fn missing_and_mistyped_parameters_are_rejected() {
    let mut session = Session::new();
    for params in [
        json!({}),
        json!({"text": ""}),
        json!({"text": "   "}),
        json!({"text": 1}),
        json!({"text": null}),
        json!({"text": []}),
        json!(null),
        json!([]),
    ] {
        let error = run(&mut session, "doc.set_text", params.clone()).expect_err("bad parameters");
        assert!(matches!(error, CmdError::Params(_)), "{params}: {error:?}");
    }
    assert!(
        session.doc().is_empty(),
        "a rejected call must not change the document"
    );
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
        json!({"audience": 123}),
        json!({"audience": "x".repeat(512)}),
        json!({"id": u64::MAX}),
        json!({"id": 1e300}),
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
fn a_duplicate_id_does_not_replace_the_first_command() {
    use crate::CommandSpec;
    let first = CommandSpec::new("x", "First", "Here", |_, _| Ok(json!({"first": true})));
    let second = CommandSpec::new("x", "Second", "Here", |_, _| Ok(json!({"second": true})));
    let registry = crate::Registry::new(vec![first, second]);
    assert_eq!(registry.all().len(), 2);
    assert_eq!(registry.get("x").map(|spec| spec.label), Some("First"));
}
