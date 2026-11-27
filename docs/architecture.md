# grammarcraft architecture

> **Last reviewed:** 2026-11-13 · **Change:** first version, written from this template's code.

How the workspace is built today. The rules for working in it are in [`AGENTS.md`](../AGENTS.md);
this file describes the shape and why it is that shape.

## Layers

`Cargo.toml` declares `crates/*`, `apps/*` and `xtask` as members. `cargo xtask layers` reads
`cargo metadata` and fails if a crate depends on a layer at or above its own, or if a workspace
crate was never added to the table in `xtask/src/layers.rs`.

| Layer | Crate | Job |
|---|---|---|
| L0 | [`core`](../crates/core/src) | `Document`, `Note`, `NoteId`, `CoreError`; invariants, bounds, no I/O |
| L1 | [`engine`](../crates/engine/src) | `Session`, `CommandSpec`, `Registry`, `CmdError`; parameter validation and command dispatch |
| app | [`grammarcraft-cli`](../apps/grammarcraft-cli/src) | argument parsing, session lifetime, exit codes, JSON output |
| tooling | [`xtask`](../xtask/src) | `ci`, `layers`, `assets` |

Dependency direction: `engine → core`, `grammarcraft-cli → engine`. Nothing points upward. L0 has one
external dependency (`thiserror`); L1 adds `serde_json` because parameters cross the command
boundary as JSON.

A front end is allowed to depend on the engine and nothing else in the workspace. If a UI ever
needs a type from L0, it gets it through the engine's API or is re-exported deliberately — not by
reaching around the layer.

## Data flow

1. **A call arrives** as a command id plus JSON parameters, from the CLI today and from a UI, a
   socket or an agent later.
2. **`Session::run`** looks the id up in the registry, asks the command's `enabled` function whether
   it may run, and calls `run`.
3. **The command** asks `p::` for each parameter (typed, range-checked, with a message naming the
   missing key) and calls a domain method.
4. **The domain** enforces its invariants — text is non-empty and bounded, the note count is
   bounded, an id must exist — and returns `CoreError`.
5. **`From<CoreError> for CmdError`** decides whether the caller can fix the request (`Params`) or
   whether the failure is internal (`Failed`).
6. **The front end** prints the JSON result or the error and picks an exit code.

Every step returns `Result`; no step panics on input.

## Testing and gates

- Unit tests live beside the code (`crates/*/src/tests.rs`) and test invariants, not
  implementations.
- `crates/engine/src/tests.rs::hostile_parameters_never_panic` runs *every* command with junk
  parameters: `null`, arrays, wrong types, negative and huge numbers, a 200 kB string. The assertion
  is the absence of a panic.
- `apps/grammarcraft-cli/tests/cli.rs` drives the built binary, so argument parsing and exit codes are
  covered too.
- `cargo xtask ci` runs `fmt --check`, `clippy -D warnings`, the tests, `layers` and `assets`.
- GitHub Actions calls the shared `gate.yml` from `storytold/craft-common-actions`; there are no
  repository-specific jobs.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-13 | major | First committed version: layers, data flow, gates |