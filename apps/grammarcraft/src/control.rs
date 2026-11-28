use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use craft::automation::args::{ControlOptions, EnvNames, FlagNames, USAGE_EXIT_CODE, control_port};
use craft::automation::control::{ControlRequest, Reply};
use craft::automation::server::{Listening, serve};
use craft::ui::runtime::windows::WindowHost;
use serde_json::{Value, json};

use crate::state::{GrammarCraftApp, SidebarTab};

#[derive(Debug, Clone, Default)]
pub struct LaunchArgs {
    pub control: ControlOptions,
    pub read_root: Option<PathBuf>,
    pub write_root: Option<PathBuf>,
    pub files: Vec<String>,
    pub quick_review: bool,
    pub overlay: bool,
    pub initial_text: Option<String>,
    pub from_clipboard: bool,
    pub help: bool,
    pub version: bool,
}

pub fn parse_launch_args(raw: &[String]) -> Result<LaunchArgs, (i32, String)> {
    let mut parsed = LaunchArgs::default();
    let mut flags = ControlOptions::default();
    let mut i = 0usize;
    while let Some(arg) = raw.get(i) {
        match arg.as_str() {
            "--help" | "-h" => parsed.help = true,
            "--version" | "-V" => parsed.version = true,
            "--quick-review" => parsed.quick_review = true,
            "--overlay" => parsed.overlay = true,
            "--from-clipboard" => parsed.from_clipboard = true,
            "--text" => {
                let val = next_arg(raw, &mut i, "--text")?;
                parsed.initial_text = Some(val.to_owned());
            }
            other if other.starts_with("--text=") => {
                parsed.initial_text = Some(other.trim_start_matches("--text=").to_owned());
            }
            "--file" => {
                let val = next_arg(raw, &mut i, "--file")?;
                parsed.files.push(val.to_owned());
            }
            other if other.starts_with("--file=") => {
                parsed
                    .files
                    .push(other.trim_start_matches("--file=").to_owned());
            }
            "--control" => {
                let val = next_arg(raw, &mut i, "--control")?;
                let port = control_port(val).map_err(|_| {
                    (
                        USAGE_EXIT_CODE,
                        format!("error: invalid value '{val}' for '--control <PORT>'"),
                    )
                })?;
                flags.port = Some(port);
                flags.sources.port = Some(craft::automation::args::Source::Flag);
            }
            "--control-token" => {
                let val = next_arg(raw, &mut i, "--control-token")?;
                flags.token = Some(val.to_owned());
                flags.sources.token = Some(craft::automation::args::Source::Flag);
            }
            "--control-token-file" => {
                let val = next_arg(raw, &mut i, "--control-token-file")?;
                flags.token_file = Some(PathBuf::from(val));
                flags.sources.token_file = Some(craft::automation::args::Source::Flag);
            }
            "--automation-read-root" => {
                let val = next_arg(raw, &mut i, "--automation-read-root")?;
                parsed.read_root = Some(PathBuf::from(val));
            }
            "--automation-write-root" => {
                let val = next_arg(raw, &mut i, "--automation-write-root")?;
                parsed.write_root = Some(PathBuf::from(val));
            }
            other if other.starts_with('-') => {
                return Err((USAGE_EXIT_CODE, format!("error: unknown flag `{other}`")));
            }
            file => parsed.files.push(file.to_owned()),
        }
        i += 1;
    }

    let env_names = EnvNames::for_app("GRAMMARCRAFT");
    let flag_names = FlagNames::default();
    let resolved = ControlOptions::resolve(flags, &env_names)
        .map_err(|e| (USAGE_EXIT_CODE, format!("error: {e}")))?;
    resolved
        .check(&flag_names, &env_names)
        .map_err(|e| (USAGE_EXIT_CODE, format!("error: {e}")))?;
    parsed.control = resolved;
    Ok(parsed)
}

fn next_arg<'a>(raw: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, (i32, String)> {
    *i += 1;
    raw.get(*i)
        .map(String::as_str)
        .ok_or_else(|| (USAGE_EXIT_CODE, format!("error: `{flag}` requires a value")))
}

pub struct ControlServer {
    pub addr: std::net::SocketAddr,
    listening: Listening,
}

impl ControlServer {
    pub fn start(
        options: &ControlOptions,
        port: u16,
        host: WindowHost,
    ) -> Result<(Receiver<ControlRequest>, Self), String> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|e| format!("cannot bind 127.0.0.1:{port}: {e}"))?;
        let token = options
            .server_token()
            .map_err(|e| format!("cannot configure control token: {e}"))?;
        let notice = options.token_notice(&token);
        let (rx, listening) = serve(listener, token, host)
            .map_err(|e| format!("cannot start control server: {e}"))?;
        let addr = listening.addr;
        eprintln!("control server listening on {addr}, {notice}");
        Ok((rx, Self { addr, listening }))
    }

    pub fn shut_down(self) {
        drop(self.listening);
    }
}

pub fn handle_request(
    app: &mut GrammarCraftApp,
    _ctx: &egui::Context,
    req: &ControlRequest,
) -> Reply {
    match req.method.as_str() {
        "engine.commands" => Reply::from_result(Ok(grammarcraft_engine::registry().describe())),
        "engine.execute" => handle_execute(app, &req.params),
        "document.inspect" => Reply::from_result(Ok(inspect_document(app))),
        "ui.inspect" => Reply::from_result(Ok(inspect_ui(app))),
        "ui.accessTree" => Reply::access_tree(),
        other => Reply::from_result(Err(format!("unknown control method `{other}`"))),
    }
}

fn handle_execute(app: &mut GrammarCraftApp, params: &Value) -> Reply {
    let Some(command) = params.get("command").and_then(Value::as_str) else {
        return Reply::from_result(Err("`command` (string) is required".to_owned()));
    };
    let cmd_params = params
        .get("params")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    let result = app.dispatch(command, cmd_params).map_err(|e| e.to_string());
    if result.is_ok() {
        app.sync_editor_from_session();
    }
    Reply::from_result(result)
}

fn inspect_document(app: &GrammarCraftApp) -> Value {
    let stats = app.session.doc().stats();
    let score = app.writing_score();
    json!({
        "text": app.session.doc().text(),
        "language": app.session.language_tag(),
        "stats": {
            "bytes": stats.bytes,
            "chars": stats.chars,
            "words": stats.words,
            "sentences": stats.sentences,
            "paragraphs": stats.paragraphs,
        },
        "score": score.score,
        "issue_count": app.issues().len(),
        "has_pending_ai_proposal": app.session.has_pending_ai_proposal(),
    })
}

fn inspect_ui(app: &GrammarCraftApp) -> Value {
    let tab = match app.active_tab {
        SidebarTab::Alerts => "alerts",
        SidebarTab::GoalsAndScore => "goals_and_score",
        SidebarTab::Tone => "tone",
        SidebarTab::StyleAndSnippets => "style_and_snippets",
        SidebarTab::AiAssist => "ai_assist",
        SidebarTab::CitationsAndSimilarity => "citations_and_similarity",
    };
    json!({
        "theme": app.theme_mode.label().to_ascii_lowercase(),
        "quick_review": app.quick_review,
        "overlay": app.overlay_mode,
        "active_tab": tab,
        "score": app.writing_score().score,
        "issue_count": app.issues().len(),
        "can_undo": app.session.can_undo(),
        "can_redo": app.session.can_redo(),
    })
}
