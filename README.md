# GrammarCraft

A clean-room, pure-Rust writing assistant, grammar checker, hybrid AI & live-translation editor, and native `egui` desktop app built on the Craft template: a panic-free domain crate (`grammarcraft-core`, `wasm32`-ready), a deterministic incremental proofing, contextual (EN/PT-BR/ES), variant, clarity, inclusive/formality, upstream-ready punctuation/typography, style-guide, tone, and local similarity crate (`grammarcraft-rules`, `wasm32`-ready), a command-driven session engine (`grammarcraft-engine`) with `.docx`/`.grammarcraft.json`/`.txt`/`.md` persistence, `craft-ai-native` storage, SHA-256-verified `ModelRunner` support, and consent-gated `translate.live`, a native desktop UI (`apps/grammarcraft`) with floating `--quick-review` mode and `Live Translate` selector (`Off | EN | PT-BR | ES`), a thin CLI with `quick-review`, root-confined `mcp`, and WebExtension `native-messaging` (`apps/grammarcraft-cli`), and automated CI/release/OS-context-menu packaging (`cargo xtask ci`, `packaging/`, `.github/workflows/{ci,house-rules,release}.yml`).

## Quick start

```bash
cargo xtask ci
cargo run -p grammarcraft
cargo run -p grammarcraft -- --quick-review --text "Their going to the office."
cargo run -p grammarcraft -- --quick-review --file path/to/notes.md
cargo run -p grammarcraft -- --control 0 --control-token-file /path/to/token --automation-read-root ./docs --automation-write-root ./docs
cargo run -p grammarcraft-cli -- --version
cargo run -p grammarcraft-cli -- commands
cargo run -p grammarcraft-cli -- run doc.set_text '{"text":"Olá, bom dia."}'
cargo run -p grammarcraft-cli -- run translate.live '{"target_lang":"en","apply":true}'
cargo run -p grammarcraft-cli -- check path/to/document.docx
cargo run -p grammarcraft-cli -- quick-review --text "Their going to the office." --apply-all
cargo run -p grammarcraft-cli -- mcp --automation-read-root ./docs --automation-write-root ./docs
cargo run -p grammarcraft-cli -- native-messaging
```

Expected:
- `cargo run -p grammarcraft` launches the full native `egui` desktop app (light/dark themes, `craft::ui` controls, redundant category underlines, bidirectional 6-tab Review Pane including `AI Assist` and `Citations & Similarity`, `Live Translate` toolbar selector translating completed sentences on `.`, `!`, `?`, `\n` with immediate `Undo`, native file dialogs, keyboard navigation, and AccessKit support); `--quick-review` opens a compact `460x540` always-on-top window (`--text`, `--file`, `--from-clipboard`, `Apply All`, `Apply All & Copy`, `Paste & Review`, `Copy Text`, `Live Translate`, and `Escape` without focus traps), and `--control` starts an authenticated loopback control server (`craft/automation`).
- `grammarcraft-cli -V` / `--version` prints `grammarcraft-cli <version>`; `commands` lists all **39** registered `CommandSpec` entries (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*` incl. `review.apply_all`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`, `translate.*`).
- `grammarcraft-cli quick-review` accepts `--text <TEXT>`, `--file <PATH>`, or `--stdin` (bounded by `MAX_DOCUMENT_BYTES`), optional `--lang`/`--audience`/`--formality` goals, and outputs compact JSON (`language`, `stats`, `issues`, `score`, `tone`) or corrected text with `--apply-all`.
- `native-messaging` runs a 4-byte little-endian `u32` length-prefixed JSON stdio host (`MAX_NATIVE_MESSAGE_BYTES = 1 MiB`) restricted to a safe command allowlist (`doc.*`, `goals.set`, `review.*`, `stats.measure`, `translate.*`) via `Session::run`.
- `mcp` refuses to start unless both `--automation-read-root <dir>` and `--automation-write-root <dir>` are supplied.

## Layout

| Path | What lives there | Rule it enforces |
|---|---|---|
| `crates/core` | `Document`, `TextSpan`, `WritingGoals` (`inclusive_language`), `LiveTranslationConfig`, `TranslationResult`, `Issue`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`CUSTOM.DIC`), `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, `SimilarityReport`, 12 `CitationStyle`s + BibTeX, `AiProposal` + `compute_word_diff`, `CoreError` | L0: no I/O, no `serde_json`, no UI, `wasm32`-clean, bounded input, no panics |
| `crates/rules` | `AnalysisEngine`, paragraph cache, `craft-proof` adapter, EN/PT-BR/ES contextual & inclusive/formality rules, `upstream_ready` (`check_text`), regional variant mix checks, linear-time `regex` `StyleGuide` evaluator, clarity/style/readability/tone rules, `check_local_similarity` | L1: pure deterministic analysis (`core`, `craft-proof`, `regex`), `wasm32`-clean |
| `crates/engine` | `Session`, `CommandSpec` (39 commands incl. `review.apply_all`, `translate.*`), `Registry`, `CmdError`, bounded undo/redo, `io` (`craft-atomic`, `craft-docx`, `craft-wordmodel`), `file_access` (`AutomationRoots`), `ai` (`EngineRegistry`/`ModelRunner` + `LocalOfflineBackend` + `CraftProviderBackend` + `craft-ai-native`), `translate` (`LocalOfflineTranslator` + cloud fallback under `ConsentLedger`) | L2: every action is a command; parameters, paths, weights, and AI consent/secrets are validated once |
| `apps/grammarcraft` | Native `egui`/`eframe` desktop app (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`, `automation`), full editor + floating `--quick-review` mode, `Live Translate` (`Off \| EN \| PT-BR \| ES`), `craft::ui` controls, AccessKit, `--control` loopback server, 11 headless UI/control tests | Front end: all mutations routed through `Session::run` |
| `apps/grammarcraft-cli` | `commands`, `run`, `check`, `quick-review`, `stats`, `-V`/`--version`, JSON-RPC stdio `mcp`, WebExtension `native-messaging` host | Front end: stays thin; behaviour lives in `grammarcraft-engine` |
| `xtask` | `ci`, `layers`, `assets` | Quality rules are executable gates |
| `locales/` | `en/grammarcraft.tsv` English string catalog (`context<TAB>English<TAB>English`) | Compatible with `storytold/craft-i18n` |
| `assets/` & `packaging/` | 12 app-icon assets (`assets/app-icon/`), `packaging/env.sh`, `check-icons.sh`, `linux/` (tarball, `.deb`/`.rpm`, `.AppImage`, `.flatpak`, Quick Review `.desktop` & `install-quick-review.sh`), `windows/` (WiX v5 `.msi`, portable/symbols `.zip`, `sign.ps1`, `register-context-menu.ps1`) | All 12 icon assets attributed in `ATTRIBUTION.md` and checked by `cargo xtask assets` + `packaging/check-icons.sh` |
| `clean-room/` | `README.md`, `assumptions.md`, `plan.md`, `source-log.md` | Clean-room provenance and derivation records |
| `docs/` | architecture, naming, parity, gaps, roadmap | Repository state and engineering plan |
| `deny.toml` | SPDX licence allowlist matching `craft-contrib/standards/licences.md` | Blocks GPL/LGPL/AGPL and unallowed licences |
| `AGENTS.md` | Shared `craft-agents` rules + repository rules | Humans and agents follow the same rules |

## Core rules

1. **Never crash.** `#![deny(clippy::unwrap_used, …)]` in every production crate root, `unsafe_code = "forbid"`, errors returned with `?`, and all external input bounded before allocating.
2. **Everything is a command.** Every user-visible action is a `CommandSpec` dispatched through `Session::run`.
3. **Layers point downward.** `cargo xtask layers` fails on upward, lateral, or unregistered workspace dependencies (`grammarcraft` / `grammarcraft-cli` → `engine` → `rules` → `core`).

## Licence

MIT OR Apache-2.0, at your option. See [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE), [`NOTICE`](NOTICE), [`ATTRIBUTION.md`](ATTRIBUTION.md), and [`deny.toml`](deny.toml).
