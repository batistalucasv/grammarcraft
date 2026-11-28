# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** `release.yml` `dry-run=true` verified green (run `46092799920`), `review.apply_all` (36 commands), floating `--quick-review` mode, `grammarcraft-cli native-messaging`, and `wasm32-unknown-unknown` verification of `core` + `rules` landed. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M5 release baseline + Quick Review & Native Messaging)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **36 / 47 planned (76.6%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*` incl. `review.apply_all`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~44%** | estimated from implemented L0/L1/L2 + CLI/MCP/`native-messaging` + full/`--quick-review` UI + `--control` + M4/M5 foundations |
| Feature depth (weighted by typical writer usage, table below) | **~44%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~58%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~52%** | estimated |
| **Essentials proofreader** | **~75%** | estimated (native `egui` desktop + `--quick-review` mode + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing/punctuation + clarity/readability/inclusive + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + persistent AI/local `ModelRunner`/citations/similarity + CLI/MCP/`native-messaging`/`--control` working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft` (`automation` incl.), `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` (`craft-libs` `8be80b0`); `craft-ai-core`, `craft-ai-openai`, `craft-ai-client` (`craft-ai-engine` traits), `craft-ai-native` (`craft-ai-integrations` `a822ad5`) | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M5 complete / M6+ preparation.** `cargo xtask ci` passes across 6 workspace crates (`12/12` attributed icon assets, 9 UI/control tests), `wasm32-unknown-unknown` compiles cleanly for `grammarcraft-core` + `grammarcraft-rules`, and `release.yml` `dry-run=true` passes on `main` across Linux, Windows, and Flatpak; M6+ (OS context menu & UIA/`AXUIElement`/`AT-SPI2` overlay, full `WordCraft` integration, RFC-gated browser extension, and live input translation) plus `packaging/macos/`, concrete `rten` ONNX NLP weights, and Rule 4 searches remain open.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP/overlay/browser surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 45% | 45% | `crates/rules` integrates `craft-proof`, EN, PT-BR, and ES (`es*`) contextual paronyms/homophones (`contextual.rs`), and `en-US`/`en-GB` + `pt-BR`/`pt-PT` variant mix checks (`variants.rs`) with incremental caching; corpus benchmarks remain. |
| Grammar, punctuation & mechanics | 16 | 18% | 18% | `craft-proof` conservative base checks plus `crates/rules/src/upstream_ready.rs` (`check_text` delimiter/quote balance, bracket spacing, and comma/semicolon spacing with URL/number/contraction guards in EN/PT-BR/ES) landed; expanded agreement/crasis/comma-splice rules remain. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 15% | 15% | Initial deterministic EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, and common redundancy rules landed in `crates/rules/src/clarity.rs` (isolated by language); full rewrite tables & structural simplification remain. |
| Style, vocabulary & readability metrics | 10 | 25% | 25% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 55% | 55% | Span-anchored `ToneProfile` (`tone.rs`) plus `inclusive_formality.rs` (formality-mismatch contractions/colloquialisms under `Formality::Formal` and opt-in `WritingGoals::inclusive_language` in EN/PT-BR/ES) landed; treatment consistency & optional local ONNX classifier remain. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 45% | 45% | `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`, `inclusive_language`) modulates passive voice, formality, and inclusive rules across L0/L1/L2/UI; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 90% | 90% | `EngineRegistry`/`ModelRunner` (`craft-ai-engine` via `craft-ai-client`) with streaming `sha2::Sha256` weight verification and `local_engines` in `ai.status` ahead of `LocalOfflineBackend` and `CraftProviderBackend`, `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`), 9 `ai.*` commands, word-level diff, and `AI Assist` UI tab landed; concrete `rten` ONNX NLP weights & M6 live input translation remain. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 88% | 85% | `apps/grammarcraft` ships full `egui` desktop UI + floating `--quick-review` mode (`460x540`, `AlwaysOnTop`, `--text`, `--from-clipboard`, `Apply All`, `Paste & Review`, `Copy Text`), `craft::ui` controls, redundant underlines, 6-tab Review Pane, keyboard nav, and 9 headless UI/control tests; migrating remaining `ComboBox`/`TextEdit::multiline` remains. |
| Local similarity check & citation formatting | 3 | 85% | 85% | Deterministic N-gram/Jaccard `check_local_similarity` (`similarity.check` confined by `AutomationRoots`), BibTeX parser, `citations.validate`/`citations.format` across all 12 standard bibliography styles, and `Citations & Similarity` UI tab landed; optional opt-in web `SimilarityProvider` remains future. |
| System-wide OS overlay (context menu & UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 15% | 0% | Compact floating `--quick-review` (`460x540`, `AlwaysOnTop`) + `--from-clipboard` foundation landed; M6 adds global shortcut, OS context menu, and UIA/`AXUIElement`/`AT-SPI2` adapters. |
| Agent control, CLI, MCP & Native Messaging (`grammarcraft-cli`) | 0 | 92% (beyond target) | 85% | 36 `CommandSpec`s (incl. `review.apply_all`), `grammarcraft-cli check <file>`, stdio JSON-RPC `mcp` requiring `--automation-read-root`/`--automation-write-root`, `native-messaging` WebExtension stdio host (4-byte LE `u32`, `1 MiB` bound, allowlist), and desktop loopback `--control` landed; SARIF CLI format remains. |

Weighted feature depth: **44.09% (~44%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone + integrated writing & review suite) | **~58%** | All dimensions below including M6 OS overlay/context menu, full `WordCraft` integration, RFC-gated browser extension, live input translation, and concrete `rten` ONNX weights |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~52%** | Native `craft::ui` editor + `--quick-review` + spelling/contextual/punctuation + inclusive/formality + local `ModelRunner`/persistent AI diff + citations/similarity + style/snippets working; expanded grammar/clarity depth pending |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~75%** | Native `egui` desktop + `--quick-review` + `craft-proof` + EN/PT-BR/ES + `upstream_ready` punctuation + clarity/readability + persistent AI/citations/similarity + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP/`native-messaging`/`--control` working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 44% | this file | Weighted sum of the 11 user-facing areas above (44.09%) |
| Linguistic accuracy & low false-positive rate | 20% | 32% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style/inclusive + `upstream_ready` punctuation tests passing; open GEC corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 85% | [`architecture.md`](architecture.md) | `craft::ui` (`Divider`, `Checkbox`, `TextField`, `Button`), `gamma_multiply`, 6-tab Review Pane + floating `--quick-review` mode, keyboard nav, AccessKit tree verified in 9 UI/control tests |
| Privacy, licensing & local-first AI | 10% | 95% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | Streaming `sha2::Sha256` weight check before `ModelRunner::load`, local runner precedence ahead of cloud, `ConsentFile`/`HistoryFile`/`SecretStore`, `deny.toml` allowlist, 12 attributed icon assets (`cargo-deny` pending) |
| Stability & never-crash guarantees | 10% | 80% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `review.apply_all` atomic undo, `native-messaging` `try_reserve_exact` + `1 MiB` bound, tampered SHA-256 refusal, token-rejected `--control`, `hostile_parameters_never_panic` (36 commands), green Linux/Windows/Flatpak `release.yml` `dry-run` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `wasm32`, `craft-i18n`) | 5% | 85% | [`roadmap.md`](roadmap.md) | Green `release.yml` `dry-run=true` (`46092799920`), `wasm32-unknown-unknown` `core`/`rules` passing, `native-messaging` host, `upstream_ready.rs`, `locales/en/grammarcraft.tsv`, `packaging/{linux,windows}`; M6 `WordCraft` & RFC-gated browser extension ahead |
