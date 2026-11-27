# grammarcraft

A small Rust workspace that already has the shape a larger application needs: a domain crate that
cannot panic, an engine where every user-visible action is a registered command, a thin CLI, and a
quality gate (`cargo xtask ci`) that checks formatting, lints, tests, layering and asset
attribution.

It is a **template**, not a product. The note domain is deliberately tiny so the structure is
visible; replace the noun and keep the seams.

## Quick start

```bash
cargo xtask ci                      # the full gate: fmt, clippy, tests, layers, assets
cargo run -p grammarcraft-cli -- commands  # every command, as JSON
cargo run -p grammarcraft-cli -- run note.add '{"text":"first note"}'
cargo run -p grammarcraft-cli -- run note.list '{}'
```

Expected: `commands` lists `note.add`, `note.remove`, `note.list` and `note.clear`; `note.add`
prints `{"id": 1}`; `note.list` prints `{"notes": []}` — **each invocation is a new process with a
new in-memory session**, so the note added by the previous call is gone. Persistence is gap #1 in
[`docs/gaps.md`](docs/gaps.md), not a bug in the example.

## Layout

| Path | What lives there | Rule it enforces |
|---|---|---|
| `crates/core` | `Document`, `Note`, `NoteId`, `CoreError` | L0: no I/O, no UI, bounded input, no panics |
| `crates/engine` | `Session`, `CommandSpec`, `Registry`, `CmdError` | L1: every action is a command; parameters are validated once |
| `apps/grammarcraft-cli` | argument parsing, exit codes | front ends stay thin; behaviour lives in the registry |
| `xtask` | `ci`, `layers`, `assets` | the rules are executed, not just documented |
| `docs/` | architecture, gaps, roadmap | the plan is in the repository, not in a chat |
| `AGENTS.md` | the shared house rules + this project's rules | humans and agents read the same file |

## The three rules worth keeping

1. **Never crash.** `#![deny(clippy::unwrap_used, …)]` in every production crate root, errors
   returned with `?`, and every value from outside treated as hostile.
2. **Everything is a command.** A `CommandSpec` with an id, parameters and tests; front ends
   dispatch by id and cannot drift from each other.
3. **Layers point downward.** `cargo xtask layers` fails on an upward dependency or on a crate
   that was never classified.

## Before the first real commit

The placeholder name `grammarcraft` appears in manifests, crate names and docs. The `craft-template` guide
this scaffold came from has the exact rename steps and the checklist to run afterwards (chapter 10,
"Instanciar", in `docs/how-to/`); the steps also work on their own: `rg -l 'grammarcraft'`, rename, then
`cargo xtask ci`.

## Licence

MIT OR Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.