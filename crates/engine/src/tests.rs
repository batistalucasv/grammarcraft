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
    assert!(ids.contains(&"note.add"));
    assert!(ids.contains(&"note.remove"));
    assert_eq!(ids.len(), registry().all().len());
}

#[test]
fn adding_then_listing_round_trips() {
    let mut session = Session::new();
    let added = run(&mut session, "note.add", json!({"text": "first"})).expect("a valid call");
    let id = added
        .get("id")
        .and_then(Value::as_u64)
        .expect("the id is a number");

    let listed = run(&mut session, "note.list", json!({})).expect("a read-only call");
    let notes = listed
        .get("notes")
        .and_then(Value::as_array)
        .expect("notes is an array");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].get("id").and_then(Value::as_u64), Some(id));
    assert_eq!(notes[0].get("text").and_then(Value::as_str), Some("first"));
}

#[test]
fn removing_a_missing_note_is_a_parameter_error() {
    let mut session = Session::new();
    let error =
        run(&mut session, "note.remove", json!({"id": 999})).expect_err("there is no such note");
    assert!(
        matches!(&error, CmdError::Params(message) if message.contains("999")),
        "{error:?}"
    );
}

#[test]
fn an_unknown_command_is_an_error() {
    let mut session = Session::new();
    let error = run(&mut session, "note.teleport", json!({})).expect_err("the id does not exist");
    assert!(matches!(error, CmdError::UnknownCommand(id) if id == "note.teleport"));
}

#[test]
fn a_disabled_command_reports_why() {
    let mut session = Session::new();
    let error = run(&mut session, "note.clear", json!({})).expect_err("there is nothing to clear");
    assert!(
        matches!(&error, CmdError::Disabled(reason) if reason.contains("no notes")),
        "{error:?}"
    );

    run(&mut session, "note.add", json!({"text": "x"})).expect("a valid call");
    let cleared = run(&mut session, "note.clear", json!({})).expect("there is a note to clear");
    assert_eq!(cleared.get("removed").and_then(Value::as_u64), Some(1));
}

#[test]
fn missing_and_mistyped_parameters_are_rejected() {
    let mut session = Session::new();
    for params in [
        json!({}),
        json!({"text": 1}),
        json!({"text": null}),
        json!({"text": []}),
        json!(null),
        json!([]),
    ] {
        let error = run(&mut session, "note.add", params.clone()).expect_err("bad parameters");
        assert!(matches!(error, CmdError::Params(_)), "{params}: {error:?}");
    }
    assert!(
        session.doc().is_empty(),
        "a rejected call must not change the document"
    );
}

#[test]
fn ids_accept_integral_floats_but_not_fractions_or_negatives() {
    let mut session = Session::new();
    let added = run(&mut session, "note.add", json!({"text": "x"})).expect("a valid call");
    let id = added
        .get("id")
        .and_then(Value::as_u64)
        .expect("the id is a number");
    let id_float = id as f64;

    run(&mut session, "note.remove", json!({"id": id_float}))
        .expect("an integral float is a valid id");
    assert!(session.doc().is_empty());

    for bad in [
        json!({"id": 1.5}),
        json!({"id": -1}),
        json!({"id": "1"}),
        json!({"id": null}),
    ] {
        let error = run(&mut session, "note.remove", bad.clone()).expect_err("bad id");
        assert!(matches!(error, CmdError::Params(_)), "{bad}: {error:?}");
    }
}

#[test]
fn hostile_parameters_never_panic() {
    // Every command with junk parameters. The assertion is the absence of a
    // panic: the values below reach `str::trim`, arithmetic and a lookup.
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
        json!({"id": u64::MAX}),
        json!({"id": 1e300}),
        json!({"unknown": {"nested": [true, null]}}),
    ];
    for spec in registry().all() {
        for params in &junk {
            let mut session = Session::new();
            // The result is irrelevant here; not panicking is the contract.
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
