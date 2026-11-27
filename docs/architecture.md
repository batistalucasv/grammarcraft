# GrammarCraft architecture

> **Last reviewed:** 2026-11-27 · **Change:** replace `Note` template example with initial `Document`, `TextSpan`, `WritingGoals`, `TextStats`, and command registry.

How the workspace is built today. The rules for working in it are in [`AGENTS.md`](../AGENTS.md); this file describes the shape, the layer boundaries, and why they exist.

## Layers

`Cargo.toml` declares `crates/*`, `apps/*`, and `xtask` as members. `cargo xtask layers` reads `cargo metadata` and fails if a crate depends on a layer at or above its own (including `[dev-dependencies]`), or if a workspace crate was never added to `TABLE` in `xtask/src/layers.rs`.

| Layer | Crate | Job |
|---|---|---|
| L0 | [`core`](../crates/core/src) | `Document`, `TextSpan`, `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`), `TextStats`, `CoreError`, `MAX_DOCUMENT_BYTES`, `MAX_GOAL_VALUE_BYTES`; bounds, UTF-8 boundary checks, no I/O, no JSON, no UI |
| L1 | [`engine`](../crates/engine/src) | `Session`, `CommandSpec`, `Registry`, `CmdError`; parameter validation (`p::*`) and command dispatch (`doc.set_text`, `doc.get`, `doc.clear`, `goals.set`, `stats.measure`) |
| app | [`grammarcraft-cli`](../apps/grammarcraft-cli/src) | argument parsing (`commands`, `run`, `stats`), session lifetime, exit codes (`0`, `1`, `2`), JSON output |
| tooling | [`xtask`](../xtask/src) | `ci`, `layers`, `assets` |

Dependency direction: `engine → core`, `grammarcraft-cli → engine`. Nothing points upward or sideways. L0 has one external dependency (`thiserror`); L1 adds `serde_json` because parameters and results cross the command boundary as JSON.

A front end is allowed to depend on `grammarcraft-engine` and nothing else in the workspace. If a future UI needs a type from L0, it gets it through the engine's API or explicit re-export — never by bypassing `Session::run`.

## Data flow

1. **A call arrives** as a command id plus JSON parameters from the CLI today (and from a UI or MCP server in later milestones).
2. **`Session::run`** looks the id up in `Registry`, asks `spec.enabled(self)` whether it may run (returning `CmdError::Disabled` when unavailable), and invokes `spec.run(self, params)`.
3. **The command** validates parameters using `p::*` (`p::require_object`, `p::str`, `p::req_str`, `p::opt_str`), returning `CmdError::Params` naming any missing or mistyped key.
4. **The domain** enforces its invariants before allocating (`text.len() > MAX_DOCUMENT_BYTES` before scanning, non-blank check, UTF-8 `is_char_boundary` checks in `TextSpan`, bounded goal token normalization) and returns `CoreError`.
5. **`From<CoreError> for CmdError`** maps caller-fixable invariant violations to `CmdError::Params`.
6. **The front end** prints the JSON result or error and exits with `0`, `1`, or `2`.

Every step returns `Result`; production crates forbid `unsafe` (`unsafe_code = "forbid"`) and deny `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, and `unreachable!`.

## Testing and gates

- Domain invariant tests live beside L0 (`crates/core/src/tests.rs`): `MAX_DOCUMENT_BYTES` / `MAX_DOCUMENT_BYTES + 1`, empty/blank rejection, multi-byte UTF-8 `TextSpan` char-boundary checks, `TextStats` measurements, and `WritingGoals` parsing/bounds.
- Engine contract tests live in `crates/engine/src/tests.rs`, including `hostile_parameters_never_panic` running *every* registered command with malformed JSON, oversized strings, non-object payloads, and boundary numbers.
- `apps/grammarcraft-cli/tests/cli.rs` drives the built `grammarcraft-cli` binary to verify stdout JSON, stderr error categories, and exit codes `0`, `1`, and `2`.
- `cargo xtask ci` runs `fmt --check`, `clippy -D warnings`, workspace tests (`nextest` / `cargo test`), `layers`, and `assets`.
- GitHub Actions runs shared workflows from `storytold/craft-common-actions` (`gate.yml@main` and `house-rules.yml@main`).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-13 | major | Initial template baseline: layers, data flow, gates |
| 2026-11-27 | major | Replace `Note` example with GrammarCraft `Document`, `TextSpan`, `WritingGoals`, `TextStats`, and `doc.*`/`goals.set`/`stats.measure` commands |
