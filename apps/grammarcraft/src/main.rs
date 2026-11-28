#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::process::ExitCode;

use grammarcraft_engine::file_access::{self, AutomationRoots};

const USAGE: &str = "\
grammarcraft — native writing assistant and grammar checker

USAGE:
  grammarcraft [OPTIONS] [FILE]

OPTIONS:
  --quick-review                     launch compact floating Quick Review popover (460x540, always-on-top)
  --text <TEXT>                      pre-fill the editor text and run an immediate check
  --from-clipboard                   load initial text from the clipboard for immediate review
  --control <PORT|auto>              bind a loopback JSON-lines control socket (default: off)
  --control-token <TOKEN>            authentication token for the control socket
  --control-token-file <PATH>        file containing the authentication token
  --automation-read-root <DIR>       confine document reads to DIR
  --automation-write-root <DIR>      confine document writes to DIR
  -h, --help                         print help
  -V, --version                      print version
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let launch = match grammarcraft::control::parse_launch_args(&args) {
        Ok(parsed) => parsed,
        Err((code, msg)) => {
            eprintln!("{msg}");
            return ExitCode::from(u8::try_from(code).unwrap_or(2));
        }
    };
    if launch.help {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    if launch.version {
        println!("grammarcraft {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if launch.read_root.is_some() || launch.write_root.is_some() {
        match AutomationRoots::new(launch.read_root.as_deref(), launch.write_root.as_deref()) {
            Ok(Some(roots)) => {
                let _ = file_access::confine_process(roots);
            }
            Ok(None) => {}
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::from(2);
            }
        }
    }
    if let Err(err) = grammarcraft::run_desktop_with(launch) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
