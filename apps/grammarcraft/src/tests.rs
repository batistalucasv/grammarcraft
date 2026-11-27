#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

use crate::state::GrammarCraftApp;
use crate::theme_bridge::{headless_install_spec, ui_fonts};

fn harness_with_text(text: &str) -> Harness<'static, GrammarCraftApp> {
    let app = GrammarCraftApp::with_initial_text(text);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 760.0))
        .with_context_setup(|ctx| {
            craft::ui::shell::embed::install(ctx, headless_install_spec(), &ui_fonts())
                .expect("craft runtime installs on test context");
        })
        .build_eframe(move |_| app);
    harness.run_steps(4);
    harness
}

#[test]
fn editing_text_triggers_review_check_and_renders_suggestion_cards() {
    let mut h = harness_with_text("");

    let editor = h.get_by_role_and_label(Role::MultilineTextInput, "Document Editor");
    editor.click();
    h.step();
    h.event(egui::Event::Text("He recieve the package.".to_owned()));
    h.run_steps(4);

    assert_eq!(h.state().session.doc().text(), "He recieve the package.");
    assert!(
        h.state()
            .issues()
            .iter()
            .any(|i| i.replacements.iter().any(|r| r.text == "receive")),
        "expected spelling issue with replacement `receive`"
    );

    let apply_btn = h.query_by_label_contains("Apply: receive");
    assert!(
        apply_btn.is_some(),
        "expected suggestion card with `Apply: receive` button to render"
    );
}

#[test]
fn applying_suggestion_and_undo_route_through_session_run() {
    let mut h = harness_with_text("He recieve the package.");

    let apply_node = h
        .query_by_label_contains("Apply: receive")
        .expect("Apply: receive button present");
    apply_node.click();
    h.run_steps(4);

    assert_eq!(h.state().session.doc().text(), "He receive the package.");
    assert!(h.state().session.can_undo());

    let undo_node = h.get_by_label("Undo");
    undo_node.click();
    h.run_steps(4);

    assert_eq!(h.state().session.doc().text(), "He recieve the package.");
}

#[test]
fn changing_writing_goals_updates_writing_score() {
    let mut h = harness_with_text("This is gonna be cool and awesome stuff.");
    let initial_issues = h.state().issues().len();
    let initial_score = h.state().writing_score().score;

    // Switching formality to `formal` triggers additional tone/appropriateness alerts
    // through Session::run("goals.set") and lowers the WritingScore.
    h.state_mut()
        .dispatch(
            "goals.set",
            json!({ "formality": "formal", "context": "academic" }),
        )
        .expect("goals.set succeeds");
    h.state_mut().refresh_check();
    h.run_steps(3);

    let formal_issues = h.state().issues().len();
    let formal_score = h.state().writing_score().score;
    assert!(
        formal_issues >= initial_issues,
        "expected formal academic goals to maintain or increase alerts ({formal_issues} vs {initial_issues})"
    );
    assert!(
        formal_score <= initial_score,
        "expected formal academic goals to reflect penalty in WritingScore ({formal_score} vs {initial_score})"
    );

    let goals_tab = h.get_by_label("Goals & Score");
    goals_tab.click();
    h.run_steps(3);
    assert!(
        h.query_by_label_contains(&format!("Overall: {formal_score} / 100"))
            .is_some(),
        "expected Goals & Score tab to display updated WritingScore"
    );
}

#[test]
fn accesskit_tree_exposes_named_interactive_controls() {
    let h = harness_with_text("He recieve the package.");

    assert!(
        h.query_by_role_and_label(Role::MultilineTextInput, "Document Editor")
            .is_some(),
        "Document Editor must be exposed in AccessKit with non-empty name"
    );
    for expected_label in [
        "Open",
        "Save",
        "Undo",
        "Redo",
        "Dismiss Once",
        "Dismiss Document",
    ] {
        assert!(
            h.query_by_label(expected_label).is_some(),
            "expected accessible control `{expected_label}` in AccessKit tree"
        );
    }

    let named_buttons = h
        .query_all_by_role(Role::Button)
        .filter(|n| {
            n.accesskit_node()
                .label()
                .is_some_and(|l| !l.trim().is_empty())
        })
        .count();
    assert!(
        named_buttons >= 6,
        "expected named interactive buttons in AccessKit tree, found {named_buttons}"
    );
}

#[test]
fn theme_toggle_and_tone_and_dismiss_actions_work_headlessly() {
    let mut h = harness_with_text("He recieve the package. Thank you for your patience.");

    // Toggle Light -> Dark theme
    h.get_by_label("Theme: Light").click();
    h.run_steps(2);
    assert!(h.query_by_label("Theme: Dark").is_some());

    // Switch to Tone tab and verify dominant tone renders
    h.get_by_label("Tone Profile").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("Detected Tone").is_some());

    // Switch back to Alerts and exercise Add to Dictionary via Session::run
    h.query_by_label_contains("Alerts (")
        .expect("Alerts tab present")
        .click();
    h.run_steps(2);
    h.get_by_label("Add to Dictionary").click();
    h.run_steps(3);
    assert!(h.state().session.personal_dict().contains("recieve"));
}

#[test]
fn ai_cloud_consent_rewrite_apply_and_undo_flow() {
    let mut h = harness_with_text("This update is due to the fact that we shipped.");

    h.get_by_label("AI Assist").click();
    h.run_steps(2);

    // Cloud mode without consent must refuse execution cleanly.
    h.get_by_label("Use Cloud Mode").click();
    h.run_steps(2);
    h.get_by_label("AI Rewrite").click();
    h.run_steps(2);
    assert!(h.state().session.pending_ai_proposal().is_none());

    // Grant consent on Cloud, then switch to Local (Offline) to stage deterministic diff.
    h.get_by_label("Grant Cloud Consent").click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Consent granted").is_some(),
        "expected granted cloud consent status"
    );

    h.get_by_label("Use Local (Offline)").click();
    h.run_steps(2);
    h.get_by_label("AI Rewrite").click();
    h.run_steps(3);

    assert!(h.query_by_label("Apply AI Proposal").is_some());
    h.get_by_label("Apply AI Proposal").click();
    h.run_steps(3);
    assert_eq!(
        h.state().session.doc().text(),
        "This update is because we shipped."
    );

    h.get_by_label("Undo").click();
    h.run_steps(3);
    assert_eq!(
        h.state().session.doc().text(),
        "This update is due to the fact that we shipped."
    );
}

#[test]
fn citations_tab_formats_bibliography_and_inclusive_toggle_updates_alerts() {
    let mut h = harness_with_text("We inform the chairman.");

    h.get_by_label("Citations & Similarity").click();
    h.run_steps(2);
    h.get_by_label("Format Bibliography").click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Clean-Room Writing Systems")
            .is_some(),
        "expected formatted bibliography output"
    );

    h.get_by_label("Goals & Score").click();
    h.run_steps(2);
    h.get_by_label("Inclusive language (opt-in)").click();
    h.run_steps(3);
    assert!(h.state().session.goals().inclusive_language);
    assert!(
        h.state()
            .issues()
            .iter()
            .any(|i| i.rule_id == "style.inclusive"),
        "expected opt-in inclusive rule alert for `chairman`"
    );
}

#[test]
fn control_server_rejects_bad_token_and_dispatches_session_commands() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use craft::automation::client::{ConnectError, ControlClient};
    use craft::automation::control::ControlChannel;
    use craft::ui::runtime::windows::WindowHost;

    use crate::control::{ControlServer, parse_launch_args};

    let valid_token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let wrong_token = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

    let args: Vec<String> = vec![
        "--control".into(),
        "0".into(),
        "--control-token".into(),
        valid_token.into(),
        "--automation-read-root".into(),
        ".".into(),
        "--automation-write-root".into(),
        ".".into(),
    ];
    let launch = parse_launch_args(&args).expect("valid control args");
    assert_eq!(launch.control.port, Some(0));
    assert!(launch.read_root.is_some());
    assert!(launch.write_root.is_some());

    let bad = parse_launch_args(&["--control".into(), "not-a-port".into()]);
    assert!(bad.is_err());

    let mut bound_addr = None;
    let control_opts = launch.control.clone();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 760.0))
        .with_context_setup(|ctx| {
            craft::ui::shell::embed::install(ctx, headless_install_spec(), &ui_fonts())
                .expect("craft runtime installs on test context");
        })
        .build_eframe(|cc| {
            let host = WindowHost::of(&cc.egui_ctx);
            let (rx, server) = ControlServer::start(&control_opts, 0, host)
                .expect("loopback control server binds port 0");
            bound_addr = Some(server.addr);
            GrammarCraftApp::new().with_control(ControlChannel::new(rx), Some(server))
        });

    let addr = bound_addr.expect("server bound address");
    harness.run_steps(2);

    let refused = ControlClient::connect(addr, wrong_token);
    assert!(
        matches!(refused, Err(ConnectError::Refused(_))),
        "expected wrong token to be refused"
    );

    let done = Arc::new(AtomicBool::new(false));
    let done_flag = Arc::clone(&done);
    let worker = std::thread::spawn(move || {
        let mut client = ControlClient::connect(addr, valid_token).expect("valid token connects");
        let cmds = client
            .call("engine.commands", json!({}))
            .expect("engine.commands succeeds");
        assert!(cmds.as_array().is_some_and(|arr| arr.len() >= 35));

        let exec = client
            .call(
                "engine.execute",
                json!({
                    "command": "doc.set_text",
                    "params": { "text": "She recieve the report." }
                }),
            )
            .expect("engine.execute doc.set_text succeeds");
        assert!(exec.is_object());

        let doc_info = client
            .call("document.inspect", json!({}))
            .expect("document.inspect succeeds");
        assert_eq!(doc_info["text"], "She recieve the report.");
        assert!(doc_info["issue_count"].as_u64().unwrap_or(0) >= 1);

        let ui_info = client
            .call("ui.inspect", json!({}))
            .expect("ui.inspect succeeds");
        assert_eq!(ui_info["active_tab"], "alerts");

        done_flag.store(true, Ordering::Release);
    });

    for _ in 0..200 {
        harness.step();
        if done.load(Ordering::Acquire) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }

    worker.join().expect("control client thread finishes");
    assert!(done.load(Ordering::Acquire));
    assert_eq!(
        harness.state().session.doc().text(),
        "She recieve the report."
    );
    assert_eq!(harness.state().editor_text, "She recieve the report.");
}
