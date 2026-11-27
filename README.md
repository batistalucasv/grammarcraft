# GrammarCraft

A clean-room, pure-Rust writing assistant and grammar-checking workspace built on the Craft template: a panic-free domain crate (`grammarcraft-core`), a command-driven session engine (`grammarcraft-engine`), a thin CLI (`grammarcraft-cli`), and an automated quality gate (`cargo xtask ci`).

## Quick start

```bash
cargo xtask ci
cargo run -p grammarcraft-cli -- commands
cargo run -p grammarcraft-cli -- run doc.set_text '{"text":"Hello, world! Second sentence."}'
cargo run -p grammarcraft-cli -- run goals.set '{"audience":"specialized","formality":"formal"}'
cargo run -p grammarcraft-cli -- run stats.measure '{}'
```

Expected: `commands` lists `doc.set_text`, `doc.get`, `doc.clear`, `goals.set`, and `stats.measure`; `doc.set_text` returns `{"bytes": 30, "chars": 30, "words": 4, "sentences": 2, "paragraphs": 1}`; `stats.measure` in a separate invocation returns all zeros because **each CLI invocation starts a fresh in-memory session**. Persistence and undo/redo are tracked in [`docs/gaps.md`](docs/gaps.md).

## Layout

| Path | What lives there | Rule it enforces |
|---|---|---|
| `crates/core` | `Document`, `TextSpan`, `WritingGoals`, `TextStats`, `CoreError` | L0: no I/O, no JSON, no UI, bounded input, no panics |
| `crates/engine` | `Session`, `CommandSpec`, `Registry`, `CmdError` | L1: every action is a command; parameters are validated once |
| `apps/grammarcraft-cli` | argument parsing, exit codes | front ends stay thin; behaviour lives in the registry |
| `xtask` | `ci`, `layers`, `assets` | quality rules are executable gates |
| `clean-room/` | `assumptions.md`, `plan.md`, `source-log.md` | clean-room provenance and derivation records |
| `docs/` | architecture, naming, parity, gaps, roadmap | repository state and engineering plan |
| `AGENTS.md` | shared house rules + repository rules | humans and agents follow the same rules |

## Core rules

1. **Never crash.** `#![deny(clippy::unwrap_used, …)]` in every production crate root, `unsafe_code = "forbid"`, errors returned with `?`, and all external input validated before allocating.
2. **Everything is a command.** Every user-visible action is a `CommandSpec` dispatched through `Session::run`.
3. **Layers point downward.** `cargo xtask layers` fails on upward, lateral, or unregistered workspace dependencies.

## Licence

MIT OR Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
