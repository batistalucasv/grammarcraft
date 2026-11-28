use std::process::ExitCode;
use std::sync::Arc;

use grammarcraft_engine::{
    GlobalHotkeyListener, HotkeyBinding, MockHotkeyListener, MockOsTextBridge, OsSelection,
    OsTextBridge, PlatformHotkeyListener, Session, run_hotkey_overlay_once,
};

const HOTKEY_USAGE: &str = "\
grammarcraft-cli hotkey-overlay — wait for global hotkey and trigger --overlay

USAGE:
  grammarcraft-cli hotkey-overlay [--hotkey <BINDING>] [--once] [--mock-trigger] [--mock-text <TEXT>]

OPTIONS:
  --hotkey <BINDING>   shortcut to bind (default: Ctrl+Shift+G)
  --once               wait for a single trigger event then exit
  --mock-trigger       simulate an immediate trigger (for headless CI/testing)
  --mock-text <TEXT>   supply synthetic external selection text alongside --mock-trigger
  -h, --help           print this help
";

#[derive(Debug)]
struct HotkeyOverlayArgs {
    hotkey: String,
    once: bool,
    mock_trigger: bool,
    mock_text: Option<String>,
    help: bool,
}

impl Default for HotkeyOverlayArgs {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+G".to_owned(),
            once: false,
            mock_trigger: false,
            mock_text: None,
            help: false,
        }
    }
}

pub(crate) fn run(args: &[String]) -> Result<ExitCode, String> {
    let parsed = parse_args(args)?;
    if parsed.help {
        print!("{HOTKEY_USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    let binding = match HotkeyBinding::parse(&parsed.hotkey) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("error: {err}");
            return Ok(ExitCode::from(2));
        }
    };

    let mut session = Session::new();
    if let Some(mock_text) = parsed.mock_text {
        let bridge = Arc::new(MockOsTextBridge::new(Some(OsSelection {
            text: mock_text,
            anchor: None,
            app_name: Some("mock-app".to_owned()),
        })));
        session.set_os_bridge(bridge as Arc<dyn OsTextBridge>);
    }

    let listener: Box<dyn GlobalHotkeyListener> = if parsed.mock_trigger {
        Box::new(MockHotkeyListener::triggered_once())
    } else {
        Box::new(PlatformHotkeyListener)
    };

    loop {
        match run_hotkey_overlay_once(listener.as_ref(), &binding, &mut session) {
            Ok(report) => {
                let line = serde_json::to_string(&report)
                    .map_err(|e| format!("failed to serialize hotkey output: {e}"))?;
                println!("{line}");
                if parsed.once || !report["triggered"].as_bool().unwrap_or(false) {
                    return Ok(ExitCode::SUCCESS);
                }
            }
            Err(err) => {
                eprintln!("error: {err}");
                return Ok(ExitCode::FAILURE);
            }
        }
    }
}

fn parse_args(args: &[String]) -> Result<HotkeyOverlayArgs, String> {
    let mut parsed = HotkeyOverlayArgs::default();
    let mut i = 0usize;
    while let Some(arg) = args.get(i) {
        match arg.as_str() {
            "-h" | "--help" => {
                parsed.help = true;
                return Ok(parsed);
            }
            "--once" => parsed.once = true,
            "--mock-trigger" => parsed.mock_trigger = true,
            "--hotkey" => {
                parsed.hotkey = next_val(args, &mut i, "--hotkey")?.to_owned();
            }
            other if other.starts_with("--hotkey=") => {
                parsed.hotkey = other.trim_start_matches("--hotkey=").to_owned();
            }
            "--mock-text" => {
                parsed.mock_text = Some(next_val(args, &mut i, "--mock-text")?.to_owned());
            }
            other if other.starts_with("--mock-text=") => {
                parsed.mock_text = Some(other.trim_start_matches("--mock-text=").to_owned());
            }
            other => {
                return Err(format!(
                    "unknown hotkey-overlay option `{other}`\n\n{HOTKEY_USAGE}"
                ));
            }
        }
        i += 1;
    }
    Ok(parsed)
}

fn next_val<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    *i += 1;
    args.get(*i)
        .map(String::as_str)
        .ok_or_else(|| format!("`{flag}` requires a value\n\n{HOTKEY_USAGE}"))
}
