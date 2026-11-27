//! Workspace tooling: `cargo xtask <command>`.
//!
//! Pure Rust (std + serde_json). External tools (`cargo`, `git`) are invoked
//! through `std::process::Command`, never through a shell string, so paths with
//! spaces survive.
//!
//! Why a crate instead of a shell script: the checks below are part of the
//! quality gate, so they need tests, and `.cargo/config.toml` makes them one
//! command (`cargo xtask`) that CI and contributors share.

mod assets;
mod layers;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  ci       fmt --check, clippy -D warnings, tests, layers, assets (stops at the first failure)
  fmt      cargo fmt --all --check
  clippy   cargo clippy --workspace --all-targets -- -D warnings
  test     workspace tests (cargo nextest when installed, else cargo test)
  layers   enforce the crate layering (docs/architecture.md § Layers)
  assets   check that every asset file is attributed in ATTRIBUTION.md
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("ci") => cmd_ci(),
        Some("fmt") => cmd_fmt(),
        Some("clippy") => cmd_clippy(),
        Some("test") => cmd_test(),
        Some("layers") => cmd_layers(),
        Some("assets") => assets::run(&root()),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Workspace root: the parent of the `xtask` crate.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn cargo() -> Command {
    let mut command = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    command.current_dir(root());
    command
}

fn run(mut command: Command, what: &str) -> Result<(), String> {
    eprintln!("$ {what}");
    let status = command
        .status()
        .map_err(|error| format!("{what}: could not start: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{what}: exited with {status}"))
    }
}

fn cmd_ci() -> Result<(), String> {
    cmd_fmt()?;
    cmd_clippy()?;
    cmd_test()?;
    cmd_layers()?;
    assets::run(&root())
}

fn cmd_fmt() -> Result<(), String> {
    let mut command = cargo();
    command.args(["fmt", "--all", "--check"]);
    run(command, "cargo fmt --all --check")
}

fn cmd_clippy() -> Result<(), String> {
    let mut command = cargo();
    command.args([
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ]);
    run(
        command,
        "cargo clippy --workspace --all-targets -- -D warnings",
    )
}

fn cmd_test() -> Result<(), String> {
    // nextest runs each test in its own process and reports better, but it is
    // not a hard requirement: a contributor without it still runs the suite.
    let has_nextest = cargo()
        .args(["nextest", "--version"])
        .output()
        .is_ok_and(|out| out.status.success());
    let mut command = cargo();
    if has_nextest {
        command.args(["nextest", "run", "--workspace"]);
        run(command, "cargo nextest run --workspace")
    } else {
        command.args(["test", "--workspace"]);
        run(command, "cargo test --workspace")
    }
}

fn cmd_layers() -> Result<(), String> {
    let metadata = metadata()?;
    let crates = layers::from_metadata(&metadata)?;
    println!("Dependency layering (docs/architecture.md § Layers)\n");
    println!("{:<16} {:<8} workspace dependencies", "crate", "layer");
    for krate in &crates {
        let deps: Vec<String> = krate
            .deps
            .iter()
            .filter(|dep| dep.workspace)
            .map(|dep| {
                format!(
                    "{}{}",
                    layers::short_name(&dep.name),
                    if dep.kind == layers::DepKind::Dev {
                        " (dev)"
                    } else {
                        ""
                    }
                )
            })
            .collect();
        println!(
            "{:<16} {:<8} {}",
            krate.name,
            layers::describe(layers::classify(&krate.name)),
            deps.join(", ")
        );
    }
    let violations = layers::check(&crates);
    if violations.is_empty() {
        println!("\nlayers: {} workspace crates, no violations", crates.len());
        Ok(())
    } else {
        Err(format!(
            "{} layering violation(s):\n  {}",
            violations.len(),
            violations
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n  ")
        ))
    }
}

fn metadata() -> Result<serde_json::Value, String> {
    let output = cargo()
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("cargo metadata: bad JSON: {error}"))
}
