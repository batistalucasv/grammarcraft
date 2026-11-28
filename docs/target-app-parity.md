# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** `.github/workflows/release.yml` (PR #9), `EngineRegistry`/`ModelRunner` with `sha2` SHA-256 weight verification, decoupled `crates/rules/src/upstream_ready.rs` rules, and `craft::ui` (`Divider`, `Checkbox`, `TextField`, `gamma_multiply`) migration landed. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, local `ModelRunner` verification, `upstream_ready` rules & UI polish)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **35 / 47 planned (74.5%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~44%** | estimated from implemented L0/L1/L2 + CLI/MCP + native desktop UI + `--control` + M4/M5 foundations |
| Feature depth (weighted by typical writer usage, table below) | **~44%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~56%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~50%** | estimated |
| **Essentials proofreader** | **~72%** | estimated (native `egui` desktop editor + `craft::ui` controls + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing/punctuation + clarity/readability/inclusive + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + persistent AI/local `ModelRunner`/citations/similarity + CLI/MCP/`--control` working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft` (`automation` incl.), `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` (`craft-libs` `8be80b0`); `craft-ai-core`, `craft-ai-openai`, `craft-ai-client` (`craft-ai-engine` traits), `craft-ai-native` (`craft-ai-integrations` `a822ad5`) | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M5 polish & release baseline.** `cargo xtask ci` passes across 6 workspace crates with 8 headless `egui_kittest` / `--control` tests, SHA-256-verified `ModelRunner` tests, `upstream_ready` rules, and `.github/workflows/release.yml`; concrete `rten` ONNX NLP weights, `release.yml` `dry-run`, macOS/Windows packaging, external upstream PRs, and remaining `ComboBox`/`TextEdit::multiline` `craft-design-lint` cleanup remain open.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 45% | 45% | `crates/rules` integrates `craft-proof`, EN, PT-BR, and ES (`es*`) contextual paronyms/homophones (`contextual.rs`), and `en-US`/`en-GB` + `pt-BR`/`pt-PT` variant mix checks (`variants.rs`) with incremental caching; corpus benchmarks remain. |
| Grammar, punctuation & mechanics | 16 | 18% | 10% | `craft-proof` conservative base checks plus `crates/rules/src/upstream_ready.rs` (`check_text` delimiter/quote balance, bracket spacing, and comma/semicolon spacing with URL/number/contraction guards in EN/PT-BR/ES) landed; expanded agreement/crasis/comma-splice rules remain. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 15% | 15% | Initial deterministic EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, and common redundancy rules landed in `crates/rules/src/clarity.rs` (isolated by language); full rewrite tables & structural simplification remain. |
| Style, vocabulary & readability metrics | 10 | 25% | 25% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 55% | 55% | Span-anchored `ToneProfile` (`tone.rs`) plus `inclusive_formality.rs` (formality-mismatch contractions/colloquialisms under `Formality::Formal` and opt-in `WritingGoals::inclusive_language` in EN/PT-BR/ES) landed; treatment consistency & optional local ONNX classifier remain. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 45% | 45% | `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`, `inclusive_language`) modulates passive voice, formality, and inclusive rules across L0/L1/L2/UI; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 90% | 80% | `EngineRegistry`/`ModelRunner` (`craft-ai-engine` via `craft-ai-client`) with streaming `sha2::Sha256` weight verification and `local_engines` in `ai.status` ahead of `LocalOfflineBackend` and `CraftProviderBackend`, `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`), 9 `ai.*` commands, word-level diff, and `AI Assist` UI tab landed; concrete `rten` ONNX NLP model weights remain. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 85% | 80% | `apps/grammarcraft` ships `egui` desktop UI (`craft::ui` `Button`, `Badge`, `Divider`, `Checkbox`, `TextField` via `Bind::bound`, `theme_store` + `gamma_multiply`, `dialogs`, `automation`), redundant underlines, 6-tab Review Pane, keyboard nav, and 8 headless UI/control tests; migrating remaining `ComboBox`/`TextEdit::multiline` for 100% `craft-design-lint` remains. |
| Local similarity check & citation formatting | 3 | 85% | 85% | Deterministic N-gram/Jaccard `check_local_similarity` (`similarity.check` confined by `AutomationRoots`), BibTeX parser, `citations.validate`/`citations.format` across all 12 standard bibliography styles, and `Citations & Similarity` UI tab landed; optional opt-in web `SimilarityProvider` remains future. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 85% (beyond target) | 85% | 35 `CommandSpec`s, `grammarcraft-cli check <file>`, stdio JSON-RPC `mcp` requiring `--automation-read-root`/`--automation-write-root`, and desktop loopback `--control` landed; SARIF CLI format remains. |

Weighted feature depth: **43.58% (~44%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~56%** | All dimensions below including concrete `rten` ONNX NLP weights, `release.yml` `dry-run`, `craft-i18n` sync, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~50%** | Native `craft::ui` editor + spelling/contextual/punctuation + inclusive/formality + local `ModelRunner`/persistent AI diff + citations/similarity + style/snippets working; expanded grammar/clarity depth & remaining `craft-design-lint` cleanup pending |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~72%** | Native `egui` desktop app + `craft-proof` + EN/PT-BR/ES + `upstream_ready` punctuation + clarity/readability + persistent AI/citations/similarity + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP/`--control` working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 44% | this file | Weighted sum of the 11 user-facing areas above (43.58%) |
| Linguistic accuracy & low false-positive rate | 20% | 32% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style/inclusive + `upstream_ready` punctuation tests passing; open GEC corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 80% | [`architecture.md`](architecture.md) | `craft::ui` (`Divider`, `Checkbox`, `TextField`, `Button`), `gamma_multiply`, 6-tab Review Pane, keyboard nav, AccessKit tree verified in 8 UI/control tests (`ComboBox`/`TextEdit::multiline` cleanup pending) |
| Privacy, licensing & local-first AI | 10% | 95% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | Streaming `sha2::Sha256` weight check before `ModelRunner::load`, local runner precedence ahead of cloud, `ConsentFile`/`HistoryFile`/`SecretStore`, `deny.toml` allowlist (`cargo-deny` pending) |
| Stability & never-crash guarantees | 10% | 75% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, tampered SHA-256 refusal test, fail-closed persistence, token-rejected `--control` tests, `hostile_parameters_never_panic` (35 commands), `craft-atomic`, `AutomationRoots` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 75% | [`roadmap.md`](roadmap.md) | `.github/workflows/release.yml`, `upstream_ready.rs` decoupled for `craft-proof`, `EngineRegistry`/`ModelRunner`, `craft-ai-native`, `locales/en/grammarcraft.tsv`, `packaging/linux/*` wired; upstream PRs & `dry-run` remain |
