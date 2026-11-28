# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-28 · **Last updated:** 2026-11-28 · **Change:** M6-6 CLI `--format text|json|sarif` (SARIF 2.1.0), `craft-text` UAX #14/#29 segmentation (`segmentation.rs`), `craft::ui::pickers::Select` migration, and `packaging/macos/` landed. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-28, M6-6 SARIF, `craft-text` Segmentation, `craft::ui::pickers::Select` & `packaging/macos/`)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **42 / 47 planned (89.4%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*` incl. `review.apply_all`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`, `translate.*`, `overlay.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~51%** | estimated from implemented L0/L1/L2 + CLI/MCP/`quick-review`/`native-messaging`/`hotkey-overlay`/SARIF + full/`--quick-review`/`--overlay` UI + `Live Translate` + `rten` ONNX runner |
| Feature depth (weighted by typical writer usage, table below) | **~51%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~64%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~62%** | estimated |
| **Essentials proofreader** | **~84%** | estimated (native `egui` desktop + `--quick-review --file` + `--overlay` + `hotkey-overlay` + `Live Translate` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing/deep grammar/punctuation + clarity/nominalization/rhythm/readability/inclusive + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + persistent AI/`RtenTextModelRunner`/citations/similarity + CLI/SARIF/MCP/`native-messaging`/`--control` + OS context menus working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft` (`automation` incl.), `craft-proof`, `craft-text` (`segment`), `craft-atomic`, `craft-docx`, `craft-wordmodel` (`craft-libs` `8be80b0`); `craft-ai-core`, `craft-ai-openai`, `craft-ai-client` (`craft-ai-engine` traits), `craft-ai-native` (`craft-ai-integrations` `a822ad5`) | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M6 integration, SARIF, `craft-text` segmentation & `packaging/macos/`.** `cargo xtask ci` passes across 6 workspace crates (`12/12` attributed icon assets, 13 UI/control tests), `wasm32-unknown-unknown` compiles cleanly for `grammarcraft-core` + `grammarcraft-rules`, and `release.yml` `dry-run=true` passes on `main` (Draft Release `v0.1.0`); on-demand real ONNX weights, `craft-i18n` sync, `craft-ai-agents` registration, `craft-design-lint` 100% (`TextEdit::multiline`), `storytold/craft-libs#3975` → `wordcraft`, RFC-gated browser extension, and Rule 4 searches remain open.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, live translation, similarity/citation checks, desktop/CLI/MCP/overlay/browser surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 45% | 45% | `crates/rules` integrates `craft-proof`, EN, PT-BR, and ES (`es*`) contextual paronyms/homophones (`contextual.rs`), and `en-US`/`en-GB` + `pt-BR`/`pt-PT` variant mix checks (`variants.rs`) with incremental caching; corpus benchmarks remain. |
| Grammar, punctuation & mechanics | 16 | 35% | 18% | `craft-proof` conservative base checks, `crates/rules/src/upstream_ready.rs` (`check_text`), and `crates/rules/src/grammar_deep.rs` (EN comma splice, `subject_verb_agreement`, `less` vs `fewer`; PT-BR conservative crasis & `houveram`/`fazem`; ES `dequeísmo` & `hubieron` with anti-false-positive guards) landed; open GEC corpus benchmarks remain. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 30% | 15% | `crates/rules/src/clarity.rs` and `scoring.rs` ship EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, redundancies, EN/PT-BR nominalizations (`clarity.nominalization`), and `clarity.monotonous_rhythm` (`has_uniform_sentence_rhythm`); broader vague-verb/adjective tables remain. |
| Style, vocabulary & readability metrics | 10 | 25% | 25% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 55% | 55% | Span-anchored `ToneProfile` (`tone.rs`) plus `inclusive_formality.rs` (formality-mismatch contractions/colloquialisms under `Formality::Formal` and opt-in `WritingGoals::inclusive_language` in EN/PT-BR/ES) landed; treatment consistency & optional local ONNX classifier remain. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 45% | 45% | `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`, `inclusive_language`) modulates passive voice, formality, and inclusive rules across L0/L1/L2/UI; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite, AI assistance & live translation (hybrid local/cloud with diff) | 8 | 96% | 94% | `EngineRegistry`/`ModelRunner` + `RtenTextModelRunner` (`rten` `0.28.0` loading `.onnx` after streaming `sha2::Sha256` verification), `LocalOfflineBackend` + `CraftProviderBackend` (`craft-ai-native`), 9 `ai.*` commands, and `translate.rs` (`LocalOfflineTranslator` `pt`/`en`/`es` + `translate_with_cloud` under `ConsentLedger`, `translate.*`, atomic undo) landed; on-demand open NLP/translation weights remain. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 96% | 94% | `apps/grammarcraft` ships full `egui` desktop UI + floating `--quick-review` mode (`--file`, `Apply All & Copy`, `Escape`) + anchored `--overlay` mode (`AlwaysOnTop`, `ViewportCommand::OuterPosition`, `Capture Selection`, `Replace in App`), `Live Translate` (`Off \| EN \| PT-BR \| ES`) on toolbar, Quick Review, and Overlay triggering on sentence completion with immediate `Undo`, `craft::ui` controls (`Select`, `Divider`, `Checkbox`, `TextField`), redundant underlines, 6-tab Review Pane, keyboard nav, and 13 headless UI/control tests; migrating remaining `TextEdit::multiline` remains. |
| Local similarity check & citation formatting | 3 | 85% | 85% | Deterministic N-gram/Jaccard `check_local_similarity` (`similarity.check` confined by `AutomationRoots`), BibTeX parser, `citations.validate`/`citations.format` across all 12 standard bibliography styles, and `Citations & Similarity` UI tab landed; optional opt-in web `SimilarityProvider` remains future. |
| System-wide OS overlay & context menus (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 85% | 85% | Pure-Rust `os_bridge.rs` (`OsTextBridge`, multi-OS zero-`unsafe` `PlatformOsBridge`, `MockOsTextBridge`), `hotkey.rs` (`HotkeyBinding`, `GlobalHotkeyListener`, `PlatformHotkeyListener`, `MockHotkeyListener`, `run_hotkey_overlay_once`), `grammarcraft-cli hotkey-overlay`, `overlay.*` (incl. `auto_translate` + `apply_all`), anchored `--overlay` floating UI, floating `--quick-review --file`, and Linux/Windows context-menu hooks landed; optional direct in-process crate adapters without subprocesses remain future. |
| Agent control, CLI, MCP & Native Messaging (`grammarcraft-cli`) | 0 | 99% (beyond target) | 98% | 42 `CommandSpec`s (incl. `review.apply_all`, `translate.*`, and `overlay.*`), `check <file>` & `quick-review` with `--format text\|json\|sarif` (SARIF 2.1.0), `hotkey-overlay`, stdio JSON-RPC `mcp`, `native-messaging` WebExtension host (incl. `translate.*` allowlist), and desktop loopback `--control` landed. |

Weighted feature depth: **51.35% (~51%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone + integrated writing & review suite) | **~64%** | All dimensions below including on-demand open ONNX weights, `storytold/craft-libs#3975` → `wordcraft`, RFC-gated browser extension, and optional direct in-process crate overlay adapters |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~62%** | Native `craft::ui` editor + `--quick-review --file` + `--overlay` + `hotkey-overlay` + `Apply All & Copy` + `Live Translate` + spelling/deep grammar/punctuation + clarity/nominalization/rhythm + inclusive/formality + `RtenTextModelRunner`/persistent AI diff + citations/similarity + style/snippets working |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~84%** | Native `egui` desktop + `--quick-review` + `--overlay` + `hotkey-overlay` + `Live Translate` + OS context menus + `craft-proof` + `craft-text` + EN/PT-BR/ES deep grammar + `upstream_ready` punctuation + clarity/readability + persistent AI/`rten`/citations/similarity + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/SARIF/MCP/`native-messaging`/`--control` working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 51% | this file | Weighted sum of the 11 user-facing areas above (51.35%) |
| Linguistic accuracy & low false-positive rate | 20% | 38% | [`clean-room/verification.md`](../clean-room/verification.md) | `craft-proof` + `craft-text` UAX #14/#29 + EN/PT-BR/ES contextual/deep-grammar/variant/clarity/rhythm/tone/style/inclusive + `upstream_ready` tests passing with documented anti-false-positive guards; open GEC corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 94% | [`architecture.md`](architecture.md) | `craft::ui` (`Select`, `Divider`, `Checkbox`, `TextField`, `Button`), `gamma_multiply`, 6-tab Review Pane + floating `--quick-review` + anchored `--overlay` + `Live Translate`, keyboard nav, AccessKit tree verified in 13 UI/control tests |
| Privacy, licensing & local-first AI | 10% | 96% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | `LocalOfflineTranslator` + `LocalOfflineBackend` zero-network default, `translate_with_cloud` gated by `ConsentLedger`, streaming `sha2::Sha256` weight check, `ConsentFile`/`HistoryFile`/`SecretStore`, `deny.toml` allowlist, 12 attributed icon assets |
| Stability & never-crash guarantees | 10% | 85% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `MAX_LANG_TAG_BYTES = 16`, `MAX_DOCUMENT_BYTES` bound on OS capture/replace, anti-loop `Live Translate`, v1 `Document` retro-deserialization, `quick-review --stdin` bounded, `review.apply_all` atomic undo, `native-messaging` `1 MiB` bound, `hostile_parameters_never_panic` (42 commands), green `release.yml` `dry-run` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `wasm32`, `craft-i18n`) | 5% | 90% | [`roadmap.md`](roadmap.md) | Green `release.yml` `dry-run=true` (`46092799920`), `packaging/macos/`, `wasm32-unknown-unknown` `core`/`rules` (with `craft-text`), SARIF 2.1.0 CLI, `native-messaging` host (incl. `translate.*`), Linux/Windows OS context-menu hooks, `upstream_ready.rs`, `locales/en/grammarcraft.tsv`; `craft-libs#3975` → `wordcraft` & RFC-gated browser extension ahead |
