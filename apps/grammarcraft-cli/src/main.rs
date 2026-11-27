//! `grammarcraft-cli`: the engine's commands from a terminal.
//!
//! The CLI is deliberately thin. It owns argument parsing, session lifetime
//! and exit codes; everything a user can *do* is a command id that already
//! exists in `grammarcraft-engine`. A second front end (a window, an agent socket)
//! uses the same ids, so behaviour cannot drift between them.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::process::ExitCode;

use grammarcraft_engine::{Session, registry};
use serde_json::{Value, json};

const USAGE: &str = "\
grammarcraft-cli — grammarcraft from the command line

USAGE:
  grammarcraft-cli commands             list every command as JSON
  grammarcraft-cli run <id> [json]      run a command with its parameters (default {})
  grammarcraft-cli stats                measure text stats in a fresh session
  grammarcraft-cli -h | --help          this text

EXIT CODES:
  0  success
  1  the command failed (unknown id, bad parameters, disabled)
  2  the command line itself was wrong
";

fn main() -> ExitCode {
    match dispatch(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

fn dispatch(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(first) = args.first().map(String::as_str) else {
        print!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    };
    match first {
        "commands" => {
            let list = registry().describe();
            println!("{}", pretty(&list)?);
            Ok(ExitCode::SUCCESS)
        }
        "run" => run_command(
            args.get(1).map(String::as_str),
            args.get(2).map(String::as_str),
        ),
        "stats" => run_command(Some("stats.measure"), None),
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand `{other}`\n\n{USAGE}")),
    }
}

fn run_command(id: Option<&str>, raw_params: Option<&str>) -> Result<ExitCode, String> {
    let id = id.ok_or_else(|| format!("`run` needs a command id\n\n{USAGE}"))?;
    let params: Value = match raw_params {
        Some(raw) => serde_json::from_str(raw)
            .map_err(|error| format!("parameters are not valid JSON: {error}"))?,
        None => json!({}),
    };
    let mut session = Session::new();
    // A command that only reads still goes through `run`, so the CLI has one
    // path and the registry stays the source of truth.
    match session.run(id, &params) {
        Ok(value) => {
            println!("{}", pretty(&value)?);
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            eprintln!("error: {error}");
            Ok(ExitCode::FAILURE)
        }
    }
}

fn pretty(value: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| format!("could not print JSON: {error}"))
}
