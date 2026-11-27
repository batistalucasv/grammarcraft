# GrammarCraft

A clean-room, pure-Rust writing assistant, grammar checker, hybrid AI editor, and native `egui` desktop app built on the Craft template: a panic-free domain crate (`grammarcraft-core`), a deterministic incremental proofing, contextual (EN/PT-BR/ES), variant, clarity, inclusive/formality, style-guide, tone, and local similarity crate (`grammarcraft-rules`), a command-driven session engine (`grammarcraft-engine`) with `.docx`/`.grammarcraft.json`/`.txt`/`.md` persistence and `craft-ai-native` consent/history/secret storage, a native desktop UI (`apps/grammarcraft`) with authenticated loopback `--control`, a thin CLI with root-confined MCP (`apps/grammarcraft-cli`), and an automated quality gate (`cargo xtask ci`).

## Quick start

```bash
cargo xtask ci
cargo run -p grammarcraft
cargo run -p grammarcraft -- --control 0 --control-token-file /path/to/token --automation-read-root ./docs --automation-write-root ./docs
cargo run -p grammarcraft-cli -- commands
cargo run -p grammarcraft-cli -- run doc.set_text '{"text":"Their going to the office to review the colour draft."}'
cargo run -p grammarcraft-cli -- run goals.set '{"audience":"specialized","formality":"formal","inclusive_language":true}'
cargo run -p grammarcraft-cli -- check path/to/document.docx
cargo run -p grammarcraft-cli -- mcp --automation-read-root ./docs --automation-write-root ./docs
```

Expected:
- `cargo run -p grammarcraft` launches the native `egui` desktop app powered by the Craft design system (light/dark themes, redundant category underlines, bidirectional 6-tab Review Pane including `AI Assist` and `Citations & Similarity`, native file dialogs, keyboard navigation, and AccessKit support); passing `--control` starts an authenticated loopback control server (`craft/automation`) confined when `--automation-read-root`/`--automation-write-root` are supplied.
- `commands` lists all **35** registered `CommandSpec` entries (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`).
- `run` executes a single command in a fresh in-memory `Session`; use `doc.save` and `doc.open` (`.grammarcraft.json`, `.docx`, `.txt`, `.md`) or `check <file>` / `mcp` to persist state across operations.
- `mcp` refuses to start unless both `--automation-read-root <dir>` and `--automation-write-root <dir>` are supplied.

## Layout

| Path | What lives there | Rule it enforces |
|---|---|---|
| `crates/core` | `Document`, `TextSpan`, `WritingGoals` (`inclusive_language`), `Issue`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`CUSTOM.DIC`), `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, `SimilarityReport`, 12 `CitationStyle`s + BibTeX, `AiProposal` + `compute_word_diff`, `CoreError` | L0: no I/O, no `serde_json`, no UI, bounded input, no panics |
| `crates/rules` | `AnalysisEngine`, paragraph cache, `craft-proof` adapter, EN/PT-BR/ES contextual & inclusive/formality rules, regional variant mix checks, linear-time `regex` `StyleGuide` evaluator, clarity/style/readability/tone rules, `check_local_similarity` | L1: pure deterministic analysis (depends on `core`, `craft-proof`, `regex`) |
| `crates/engine` | `Session`, `CommandSpec` (35 commands), `Registry`, `CmdError`, bounded undo/redo, `io` (`craft-atomic`, `craft-docx`, `craft-wordmodel`), `file_access` (`AutomationRoots`), `ai` (`LocalOfflineBackend` + `CraftProviderBackend` with `craft-ai-native` `ConsentFile`, `HistoryFile`, `SecretStore`) | L2: every action is a command; parameters, paths, and AI consent/secrets are validated once |
| `apps/grammarcraft` | Native `egui`/`eframe` desktop app (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`, `automation`), editor underlines, bidirectional 6-tab Review Pane, AccessKit, `--control` loopback server, headless `egui_kittest` & control tests | Front end: all mutations routed through `Session::run` |
| `apps/grammarcraft-cli` | `commands`, `run`, `check`, `stats`, JSON-RPC stdio `mcp` | Front end: stays thin; behaviour lives in `grammarcraft-engine` |
| `xtask` | `ci`, `layers`, `assets` | Quality rules are executable gates |
| `locales/` | `en/grammarcraft.tsv` English string catalog (`context<TAB>English<TAB>English`) | Compatible with `storytold/craft-i18n` |
| `assets/` & `packaging/` | `assets/app-icon/grammarcraft.svg` + Linux `.desktop` & AppStream `.metainfo.xml.in` | Attributed in `ATTRIBUTION.md` and checked by `cargo xtask assets` |
| `clean-room/` | `README.md`, `assumptions.md`, `plan.md`, `source-log.md` | Clean-room provenance and derivation records |
| `docs/` | architecture, naming, parity, gaps, roadmap | Repository state and engineering plan |
| `deny.toml` | SPDX licence allowlist matching `craft-contrib/standards/licences.md` | Blocks GPL/LGPL/AGPL/CDDL and unallowed licences |
| `AGENTS.md` | Shared `craft-agents` rules + repository rules | Humans and agents follow the same rules |

## Core rules

1. **Never crash.** `#![deny(clippy::unwrap_used, …)]` in every production crate root, `unsafe_code = "forbid"`, errors returned with `?`, and all external input bounded before allocating.
2. **Everything is a command.** Every user-visible action is a `CommandSpec` dispatched through `Session::run`.
3. **Layers point downward.** `cargo xtask layers` fails on upward, lateral, or unregistered workspace dependencies (`grammarcraft` / `grammarcraft-cli` → `engine` → `rules` → `core`).

## Licence

MIT OR Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE), [`NOTICE`](NOTICE), [`ATTRIBUTION.md`](ATTRIBUTION.md), and [`deny.toml`](deny.toml).
