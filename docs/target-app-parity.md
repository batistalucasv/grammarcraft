# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** M4 implemented (`craft-ai-core`/`craft-ai-openai`/`craft-ai-client` consent-gated hybrid AI, `inclusive_formality.rs`, `similarity.rs`, 12 citation styles + BibTeX, 35 `CommandSpec`s, and `AI Assist` / `Citations & Similarity` desktop tabs with 7 `egui_kittest` tests). · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M4 hybrid AI, local similarity & citations)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **35 / 47 planned (74.5%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~40%** | estimated from implemented L0/L1/L2 + CLI/MCP + native desktop UI + M4 AI/similarity/citations |
| Feature depth (weighted by typical writer usage, table below) | **~40%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~49%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~44%** | estimated |
| **Essentials proofreader** | **~65%** | estimated (native `egui` desktop editor + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing + clarity/readability/inclusive + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + offline AI/citations/similarity + CLI/MCP working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft`, `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` (`craft-libs` `8be80b0`); `craft-ai-core`, `craft-ai-openai`, `craft-ai-client` (`craft-ai-integrations` `a822ad5`) | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M4 AI, similarity & citations.** `cargo xtask ci` passes across 6 workspace crates (`grammarcraft-core`, `grammarcraft-rules`, `grammarcraft-engine`, `grammarcraft`, `grammarcraft-cli`, `xtask`) with 7 headless `egui_kittest` + AccessKit UI tests; local ONNX `rten`/`candle` NLP `ModelRunner`, `SecretStore`/`ConsentFile` persistence, `craft-design-lint`, `--control`, expanded grammar/clarity depth, and M5 ecosystem packaging remain open.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 45% | 45% | `crates/rules` integrates `craft-proof`, EN, PT-BR, and ES (`es*`) contextual paronyms/homophones (`contextual.rs`), and `en-US`/`en-GB` + `pt-BR`/`pt-PT` variant mix checks (`variants.rs`) with incremental caching; corpus benchmarks remain. |
| Grammar, punctuation & mechanics | 16 | 10% | 10% | `craft-proof` conservative base checks wired in `crates/rules`; expanded agreement, crasis/government (PT), punctuation/comma-splice rules, and full goal-sensitive filtering remain. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 15% | 15% | Initial deterministic EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, and common redundancy rules landed in `crates/rules/src/clarity.rs` (isolated by language); full rewrite tables & structural simplification remain. |
| Style, vocabulary & readability metrics | 10 | 25% | 25% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 55% | 20% | Span-anchored `ToneProfile` (`tone.rs`) plus `inclusive_formality.rs` (formality-mismatch contractions/colloquialisms under `Formality::Formal` and opt-in `WritingGoals::inclusive_language` in EN/PT-BR/ES) landed; treatment consistency & optional local ONNX classifier remain. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 45% | 30% | `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`, `inclusive_language`) modulates passive voice, formality, and inclusive rules across L0/L1/L2/UI; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 65% | 0% | `LocalOfflineBackend` (zero-network) + `CraftProviderBackend` (`craft-ai-core`, `craft-ai-openai`, `craft-ai-client`) with `ConsentLedger`/`Disclosure` gating remote calls, 9 `ai.*` commands, word-level `AiDiffHunk` preview, `AI Assist` UI tab, and single-step undo landed; local ONNX `ModelRunner` (`rten`/`candle`) and OS `SecretStore`/`ConsentFile` persistence remain. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 80% | 75% | `apps/grammarcraft` ships `egui` desktop UI (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`), redundant underlines, bidirectional 6-tab Review Pane (`Alerts`, `Goals & Score`, `Tone Profile`, `Style & Snippets`, `AI Assist`, `Citations & Similarity`), keyboard nav, and 7 headless `egui_kittest` + AccessKit tests; `craft-design-lint` remains. |
| Local similarity check & citation formatting | 3 | 85% | 0% | Deterministic N-gram/Jaccard `check_local_similarity` (`similarity.check` confined by `AutomationRoots`), BibTeX parser, `citations.validate`/`citations.format` across all 12 standard bibliography styles, and `Citations & Similarity` UI tab landed; optional opt-in web `SimilarityProvider` remains future. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 60% (beyond target) | 45% | 35 `CommandSpec`s, `grammarcraft-cli check <file>`, and stdio JSON-RPC `mcp` requiring `--automation-read-root` and `--automation-write-root` (`AutomationRoots`) landed; SARIF & desktop `craft-automation` `--control` remain. |

Weighted feature depth: **39.95% (~40%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~49%** | All dimensions below including local ONNX NLP `ModelRunner`, `craft-i18n`, packaging, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~44%** | Native UI + spelling/contextual + inclusive/formality + AI diff + citations/similarity + style/snippets working; expanded grammar/clarity depth & `craft-design-lint` pending |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~65%** | Native `egui` desktop app + `craft-proof` + EN/PT-BR/ES + clarity/readability + offline AI/citations/similarity + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 40% | this file | Weighted sum of the 11 user-facing areas above (39.95%) |
| Linguistic accuracy & low false-positive rate | 20% | 28% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style/inclusive tests passing; open GEC/proofing corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 75% | [`architecture.md`](architecture.md) | 6-tab Review Pane (`AI Assist`, `Citations & Similarity`), redundant underlines, keyboard nav, AccessKit tree verified in 7 `egui_kittest` tests; `craft-design-lint` pending |
| Privacy, licensing & local-first AI | 10% | 75% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | Zero-network `LocalOfflineBackend` default, `ConsentLedger`/`Disclosure` blocking unconsented remote calls, `deny.toml` SPDX allowlist (`cargo-deny` & `SecretStore`/`ConsentFile` persistence pending) |
| Stability & never-crash guarantees | 10% | 65% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, bounded AI/citation/similarity/regex/`.docx` limits, `hostile_parameters_never_panic` (35 commands), 7 `egui_kittest` tests, `craft-atomic`, `AutomationRoots` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 50% | [`roadmap.md`](roadmap.md) | `craft`, `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel`, and `craft-ai-{core,openai,client}` integrated; `craft-i18n` & upstreaming in M5 |
