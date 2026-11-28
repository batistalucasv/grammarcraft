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

#[test]
fn quick_review_supports_text_file_stdin_goals_and_apply_all() {
    let text_out = cli(&[
        "quick-review",
        "--text",
        "Teh system is gonna ship in order to help.",
        "--lang",
        "en-US",
        "--audience",
        "specialized",
        "--formality",
        "formal",
    ]);
    assert!(
        text_out.status.success(),
        "{}",
        String::from_utf8_lossy(&text_out.stderr)
    );
    let compact = String::from_utf8_lossy(&text_out.stdout);
    assert_eq!(compact.trim().lines().count(), 1, "expected compact JSON");
    let parsed: serde_json::Value =
        serde_json::from_str(compact.trim()).expect("compact JSON parses");
    assert_eq!(parsed["language"], "en-US");
    assert!(parsed["issues"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(parsed["score"]["score"].is_number());
    assert!(parsed["tone"].is_object());

    let apply_out = cli(&[
        "quick-review",
        "--text",
        "Teh cat recieve teh note.",
        "--apply-all",
    ]);
    assert!(
        apply_out.status.success(),
        "{}",
        String::from_utf8_lossy(&apply_out.stderr)
    );
    let corrected = String::from_utf8_lossy(&apply_out.stdout);
    assert!(corrected.contains("receive"), "{corrected}");
    assert!(!corrected.contains("recieve"), "{corrected}");
    assert!(!corrected.contains("Teh"), "{corrected}");

    let base = std::env::temp_dir().join(format!("gc-cli-qr-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let file = base.join("note.md");
    std::fs::write(&file, "Ficou cada vez mas claro.").expect("write md");
    let file_out = cli(&[
        "quick-review",
        "--file",
        file.to_str().expect("utf8 path"),
        "--lang",
        "pt-BR",
        "--apply-all",
    ]);
    let _ = std::fs::remove_dir_all(&base);
    assert!(
        file_out.status.success(),
        "{}",
        String::from_utf8_lossy(&file_out.stderr)
    );
    let file_corrected = String::from_utf8_lossy(&file_out.stdout);
    assert_eq!(file_corrected.trim(), "Ficou cada vez mais claro.");

    let mut child = Command::new(env!("CARGO_BIN_EXE_grammarcraft-cli"))
        .args(["quick-review", "--stdin", "--lang", "en-US"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn quick-review --stdin");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"Their is an issue.")
        .expect("write stdin");
    let stdin_out = child.wait_with_output().expect("wait stdin");
    assert!(
        stdin_out.status.success(),
        "{}",
        String::from_utf8_lossy(&stdin_out.stderr)
    );
    let stdin_json: serde_json::Value =
        serde_json::from_slice(&stdin_out.stdout).expect("stdin JSON");
    assert!(
        stdin_json["issues"]
            .as_array()
            .is_some_and(|a| a.iter().any(|i| i["excerpt"] == "Their"))
    );

    assert_eq!(cli(&["quick-review"]).status.code(), Some(2));
    assert_eq!(
        cli(&["quick-review", "--text", "a", "--stdin"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        cli(&["quick-review", "--text", "a", "--bogus"])
            .status
            .code(),
        Some(2)
    );

    let mut oversized = Command::new(env!("CARGO_BIN_EXE_grammarcraft-cli"))
        .args(["quick-review", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn oversized quick-review --stdin");
    let too_big = vec![b'a'; 1024 * 1024 + 1];
    oversized
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(&too_big)
        .expect("write oversized stdin");
    let oversized_out = oversized.wait_with_output().expect("wait oversized");
    assert_eq!(oversized_out.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&oversized_out.stderr).contains("exceeds maximum size"),
        "{}",
        String::from_utf8_lossy(&oversized_out.stderr)
    );
}

#[test]
fn hotkey_overlay_validates_binding_and_triggers_overlay_with_mock_once() {
    let bad = cli(&["hotkey-overlay", "--hotkey", "NoModifier", "--once"]);
    assert_eq!(bad.status.code(), Some(2));

    let triggered = cli(&[
        "hotkey-overlay",
        "--hotkey",
        "ctrl+shift+g",
        "--once",
        "--mock-trigger",
        "--mock-text",
        "Teh draft.",
    ]);
    assert!(
        triggered.status.success(),
        "{}",
        String::from_utf8_lossy(&triggered.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&triggered.stdout).expect("hotkey JSON output");
    assert_eq!(parsed["triggered"], true);
    assert_eq!(parsed["hotkey"], "Ctrl+Shift+G");
    assert_eq!(parsed["action"], "overlay");
    assert_eq!(parsed["overlay"]["text"], "Teh draft.");
}

#[test]
fn check_and_quick_review_support_sarif_and_text_formats() {
    let base = std::env::temp_dir().join(format!("gc-cli-sarif-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let file = base.join("sample.txt");
    std::fs::write(&file, "First line stays clear.\nSecond line has Teh error.")
        .expect("seed file");

    let sarif_out = cli(&["check", &file.to_string_lossy(), "--format", "sarif"]);
    assert!(
        sarif_out.status.success(),
        "{}",
        String::from_utf8_lossy(&sarif_out.stderr)
    );
    let sarif: serde_json::Value =
        serde_json::from_slice(&sarif_out.stdout).expect("valid SARIF JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(
        sarif["$schema"]
            .as_str()
            .is_some_and(|s| s.contains("sarif"))
    );
    assert_eq!(sarif["runs"][0]["tool"]["driver"]["name"], "GrammarCraft");
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("results array");
    assert!(!results.is_empty());
    let first_region = &results[0]["locations"][0]["physicalLocation"]["region"];
    assert_eq!(first_region["startLine"], 2);
    assert!(first_region["startColumn"].as_u64().is_some_and(|c| c >= 1));
    assert!(first_region["endLine"].as_u64().is_some_and(|l| l >= 2));
    assert!(first_region["endColumn"].as_u64().is_some_and(|c| c >= 1));
    assert!(first_region["byteOffset"].as_u64().is_some());
    assert!(first_region["byteLength"].as_u64().is_some_and(|n| n >= 1));

    let text_out = cli(&["check", &file.to_string_lossy(), "--format=text"]);
    let _ = std::fs::remove_dir_all(&base);
    assert!(
        text_out.status.success(),
        "{}",
        String::from_utf8_lossy(&text_out.stderr)
    );
    let text_str = String::from_utf8_lossy(&text_out.stdout);
    assert!(text_str.contains("GrammarCraft report"), "{text_str}");
    assert!(text_str.contains(":2:"), "{text_str}");

    let qr_sarif = cli(&[
        "quick-review",
        "--text",
        "We need less steps.",
        "--format",
        "sarif",
    ]);
    assert!(qr_sarif.status.success());
    let qr_parsed: serde_json::Value =
        serde_json::from_slice(&qr_sarif.stdout).expect("valid quick-review SARIF");
    assert_eq!(qr_parsed["version"], "2.1.0");
    assert!(
        qr_parsed["runs"][0]["results"]
            .as_array()
            .is_some_and(|r| r.iter().any(|res| res["ruleId"] == "grammar.less_vs_fewer"))
    );

    let qr_text = cli(&[
        "quick-review",
        "--text",
        "We need less steps.",
        "--format=text",
    ]);
    assert!(qr_text.status.success());
    assert!(String::from_utf8_lossy(&qr_text.stdout).contains("grammar.less_vs_fewer"));

    let bad_fmt = cli(&["quick-review", "--text", "Ok.", "--format", "xml"]);
    assert_eq!(bad_fmt.status.code(), Some(2));
}
