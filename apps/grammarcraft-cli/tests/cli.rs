//! Drives the built binary, not the library: argument parsing, exit codes,
//! `check` diagnostics and `mcp` root confinement.

use std::io::Write;
use std::process::{Command, Output, Stdio};

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
        "ai.status",
        "ai.consent",
        "ai.rewrite",
        "ai.adjustTone",
        "ai.summarize",
        "ai.expand",
        "ai.draft",
        "ai.applyResult",
        "ai.discardResult",
        "similarity.check",
        "citations.validate",
        "citations.format",
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
fn check_subcommand_analyzes_a_file_and_prints_json_report() {
    let base = std::env::temp_dir().join(format!("gc-cli-check-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let file = base.join("sample.txt");
    std::fs::write(&file, "Teh system was reviewed in order to ship.").expect("seed");

    let output = cli(&["check", &file.to_string_lossy()]);
    let _ = std::fs::remove_dir_all(&base);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert!(
        parsed
            .get("issues")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|a| !a.is_empty())
    );
    assert!(parsed.get("readability").is_some());
    assert!(parsed.get("score").is_some());
    assert!(parsed.get("tone").is_some());
}

#[test]
fn mcp_refuses_to_start_without_both_automation_roots() {
    let base = std::env::temp_dir().join(format!("gc-cli-mcp-missing-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let dir = base.to_string_lossy().into_owned();

    let missing_both = cli(&["mcp"]);
    assert!(!missing_both.status.success());
    assert!(String::from_utf8_lossy(&missing_both.stderr).contains("--automation-read-root"));

    let missing_write = cli(&["mcp", "--automation-read-root", &dir]);
    assert!(!missing_write.status.success());
    assert!(String::from_utf8_lossy(&missing_write.stderr).contains("--automation-write-root"));

    let missing_read = cli(&["mcp", "--automation-write-root", &dir]);
    assert!(!missing_read.status.success());
    assert!(String::from_utf8_lossy(&missing_read.stderr).contains("--automation-read-root"));

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn mcp_serves_initialize_tools_list_and_confines_file_access() {
    let base = std::env::temp_dir().join(format!("gc-cli-mcp-stdio-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let read_dir = base.join("read");
    let write_dir = base.join("write");
    let outside_dir = base.join("outside");
    std::fs::create_dir_all(&read_dir).expect("read dir");
    std::fs::create_dir_all(&write_dir).expect("write dir");
    std::fs::create_dir_all(&outside_dir).expect("outside dir");
    std::fs::write(outside_dir.join("secret.txt"), "secret").expect("outside file");

    let mut child = Command::new(env!("CARGO_BIN_EXE_grammarcraft-cli"))
        .args([
            "mcp",
            "--automation-read-root",
            &read_dir.to_string_lossy(),
            "--automation-write-root",
            &write_dir.to_string_lossy(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mcp starts with valid roots");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        let outside_path = outside_dir.join("secret.txt");
        let requests = format!(
            "{}\n{}\n{}\n",
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "engine_execute",
                    "arguments": {
                        "command": "doc.open",
                        "params": { "path": outside_path }
                    }
                }
            })
        );
        stdin.write_all(requests.as_bytes()).expect("write stdin");
    }

    let output = child.wait_with_output().expect("mcp exits cleanly on EOF");
    let _ = std::fs::remove_dir_all(&base);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid json line"))
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["id"], 1);
    assert!(lines[0]["result"]["capabilities"]["tools"].is_object());
    assert_eq!(lines[1]["id"], 2);
    assert!(lines[1]["result"]["tools"].is_array());
    assert_eq!(lines[2]["id"], 3);
    assert_eq!(lines[2]["result"]["isError"], true);
    assert!(
        lines[2]["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("outside the read root"))
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
    assert_eq!(cli(&["check"]).status.code(), Some(2));
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
