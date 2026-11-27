# GrammarCraft

A clean-room, pure-Rust writing assistant and grammar-checking workspace built on the Craft template: a panic-free domain crate (`grammarcraft-core`), a deterministic incremental proofing, contextual (EN/PT-BR/ES), variant, clarity, style-guide, and tone crate (`grammarcraft-rules`), a command-driven session engine (`grammarcraft-engine`) with `.docx`/`.grammarcraft.json`/`.txt`/`.md` persistence, a thin CLI with root-confined MCP (`grammarcraft-cli`), and an automated quality gate (`cargo xtask ci`).

## Quick start

```bash
cargo xtask ci
cargo run -p grammarcraft-cli -- commands
cargo run -p grammarcraft-cli -- run doc.set_text '{"text":"Their going to the office to review the colour draft."}'
cargo run -p grammarcraft-cli -- run goals.set '{"audience":"specialized","formality":"formal"}'
cargo run -p grammarcraft-cli -- check path/to/document.docx
cargo run -p grammarcraft-cli -- mcp --automation-read-root ./docs --automation-write-root ./docs
```

Expected:
- `commands` lists all **23** registered `CommandSpec` entries (`doc.set_text`, `doc.get`, `doc.clear`, `doc.save`, `doc.open`, `history.undo`, `history.redo`, `goals.set`, `stats.measure`, `review.check`, `review.apply`, `review.dismiss`, `dict.add`, `dict.remove`, `dict.import`, `dict.export`, `style.add_rule`, `style.remove_rule`, `style.list`, `snippet.add`, `snippet.remove`, `snippet.expand`, `snippet.list`).
- `run` executes a single command in a fresh in-memory `Session`; use `doc.save` and `doc.open` (`.grammarcraft.json`, `.docx`, `.txt`, `.md`) or `check <file>` / `mcp` to persist state across operations.
- `mcp` refuses to start unless both `--automation-read-root <dir>` and `--automation-write-root <dir>` are supplied.

## Layout

| Path | What lives there | Rule it enforces |
|---|---|---|
| `crates/core` | `Document`, `TextSpan`, `WritingGoals`, `Issue`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`CUSTOM.DIC`), `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, `CoreError` | L0: no I/O, no `serde_json`, no UI, bounded input, no panics |
| `crates/rules` | `AnalysisEngine`, paragraph cache, `craft-proof` adapter, EN/PT-BR/ES contextual confusables, regional variant mix checks, linear-time `regex` `StyleGuide` evaluator, clarity/style/readability/tone rules | L1: pure deterministic analysis (depends on `core`, `craft-proof`, `regex`) |
| `crates/engine` | `Session`, `CommandSpec`, `Registry`, `CmdError`, bounded undo/redo, `io` (`craft-atomic`, `craft-docx`, `craft-wordmodel`), `file_access` (`AutomationRoots`) | L2: every action is a command; parameters and paths are validated once |
| `apps/grammarcraft-cli` | `commands`, `run`, `check`, `stats`, JSON-RPC stdio `mcp` | Front ends stay thin; behaviour lives in `grammarcraft-engine` |
| `xtask` | `ci`, `layers`, `assets` | Quality rules are executable gates |
| `clean-room/` | `README.md`, `assumptions.md`, `plan.md`, `source-log.md` | Clean-room provenance and derivation records |
| `docs/` | architecture, naming, parity, gaps, roadmap | Repository state and engineering plan |
| `deny.toml` | SPDX licence allowlist matching `craft-contrib/standards/licences.md` | Blocks GPL/LGPL/AGPL/CDDL and unallowed licences |
| `AGENTS.md` | Shared `craft-agents` rules + repository rules | Humans and agents follow the same rules |

## Core rules

1. **Never crash.** `#![deny(clippy::unwrap_used, …)]` in every production crate root, `unsafe_code = "forbid"`, errors returned with `?`, and all external input bounded before allocating.
2. **Everything is a command.** Every user-visible action is a `CommandSpec` dispatched through `Session::run`.
3. **Layers point downward.** `cargo xtask layers` fails on upward, lateral, or unregistered workspace dependencies (`cli -> engine -> rules -> core`).

## Licence

MIT OR Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE), [`NOTICE`](NOTICE), [`ATTRIBUTION.md`](ATTRIBUTION.md), and [`deny.toml`](deny.toml).
