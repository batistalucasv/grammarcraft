//! Drives the built binary, not the library: argument parsing, exit codes and
//! stdout are part of the interface and only the binary can prove them.

use std::process::{Command, Output};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grammarcraft-cli"))
        .args(args)
        .output()
        .expect("the test binary can be spawned")
}

#[test]
fn commands_lists_the_registry_as_json() {
    let output = cli(&["commands"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    let ids: Vec<&str> = parsed
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
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
}

#[test]
fn a_command_runs_and_prints_its_result() {
    let output = cli(&[
        "run",
        "doc.set_text",
        r#"{"text":"Hello, world! Second line."}"#,
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(
        parsed.get("words").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        parsed.get("sentences").and_then(serde_json::Value::as_u64),
        Some(2)
    );
}

#[test]
fn a_failing_command_exits_one() {
    let bad_params = cli(&["run", "doc.set_text", r#"{"text":""}"#]);
    assert_eq!(bad_params.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&bad_params.stderr).contains("bad parameters"));

    let disabled = cli(&["run", "doc.clear"]);
    assert_eq!(disabled.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&disabled.stderr).contains("not available"));
}

#[test]
fn an_unknown_command_exits_one() {
    let output = cli(&["run", "doc.teleport"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command"));
}

#[test]
fn a_wrong_command_line_exits_two() {
    assert_eq!(cli(&["frobnicate"]).status.code(), Some(2));
    assert_eq!(cli(&["run"]).status.code(), Some(2));
    assert_eq!(
        cli(&["run", "doc.set_text", "{not json"]).status.code(),
        Some(2)
    );
}

#[test]
fn stats_on_a_fresh_session_is_all_zeros() {
    let output = cli(&["stats"]);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(
        parsed.get("words").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        parsed.get("bytes").and_then(serde_json::Value::as_u64),
        Some(0)
    );
}
