# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** M3 native `egui` desktop app (`apps/grammarcraft`) implemented with `craft` UI/theme/dialogs, redundant category underlines, bidirectional Review Pane, AccessKit tree, and 5 headless `egui_kittest` tests. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M3 native desktop app + M1/M2 deterministic engine)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **23 / 47 planned (48.9%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~28%** | estimated from implemented L0/L1/L2 + CLI/MCP + native desktop UI |
| Feature depth (weighted by typical writer usage, table below) | **~28%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~37%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~32%** | estimated |
| **Essentials proofreader** | **~60%** | estimated (native `egui` desktop editor + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing + clarity/readability + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + undo/redo + CLI/MCP working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft`, `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` wired (`craft-libs` rev `8be80b0`); `craft-text`, `craft-ai-integrations`, `craft-mcp-server` available | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M3 desktop UI.** `cargo xtask ci` passes across 6 workspace crates (`grammarcraft-core`, `grammarcraft-rules`, `grammarcraft-engine`, `grammarcraft`, `grammarcraft-cli`, `xtask`) with 5 headless `egui_kittest` + AccessKit UI tests; expanded grammar/clarity depth, `craft-design-lint`, `craft-automation` `--control`, and hybrid AI (M4) remain open.

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
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 20% | 20% | Deterministic multi-signal `ToneProfile` anchored to exact `TextSpan`s landed in `crates/rules/src/tone.rs`; formality-mismatch checks, opt-in inclusive language, and optional ONNX classifier remain in M4. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 30% | 30% | Four-axis `WritingGoals` model in `grammarcraft-core` + `goals.set` command modulate passive-voice diagnostics; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 0% | 0% | `craft-ai-client`, `craft-ai-openai` (`text.complete`), `craft-ai-native`, and `craft-ai-egui` exist upstream; prompt modes, diff preview, atomic undo, and NLP runner in `craft-ai-engine-rten` remain in M4. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 75% | 0% | `apps/grammarcraft` ships `egui` desktop UI over `craft` (`ui`, `ui-app`, `theme-store`, `dialogs`), light/dark themes, redundant category underlines (`[!]`/`[~]`/`[i]` + distinct strokes), suggestion cards, bidirectional 4-tab Review Pane, `Tab`/`Shift+Tab`/`F6`/`Escape` navigation, and 5 headless `egui_kittest` + AccessKit tests; `craft-design-lint` remains. |
| Local similarity check & citation formatting | 3 | 0% | 0% | Local N-gram/MinHash comparison over authorized directories plus validation/formatting for the 12 standard bibliography styles remain in M4. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 45% (beyond target) | 45% | 23 `CommandSpec`s, `grammarcraft-cli check <file>`, and stdio JSON-RPC `mcp` requiring `--automation-read-root` and `--automation-write-root` (`AutomationRoots`) landed; SARIF & desktop `craft-automation` `--control` remain. |

Weighted feature depth: **28.15% (~28%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~37%** | All dimensions below including hybrid AI, local similarity, i18n, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~32%** | Native UI + spelling/contextual + style/goals/snippets working; expanded grammar/clarity depth & `craft-design-lint` pending |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~60%** | Native `egui` desktop app + `craft-proof` + EN/PT-BR/ES contextuals/variants + clarity/readability + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 28% | this file | Weighted sum of the 11 user-facing areas above (28.15%) |
| Linguistic accuracy & low false-positive rate | 20% | 22% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style tests passing; open GEC/proofing corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 70% | [`architecture.md`](architecture.md) | `apps/grammarcraft` redundant underlines, keyboard navigation (`Tab`/`Shift+Tab`/`F6`/`Escape`), `craft::theme_store` light/dark, and AccessKit tree verified in `egui_kittest`; `craft-design-lint` pending |
| Privacy, licensing & local-first AI | 10% | 25% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | 100% offline local pipeline, `NOTICE` / `ATTRIBUTION.md`, and `deny.toml` SPDX allowlist configured (`cargo-deny` check pending) |
| Stability & never-crash guarantees | 10% | 60% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `RegexBuilder` size/DFA limits, `MAX_DOCX_FILE_BYTES`, `MAX_CUSTOM_FILE_BYTES`, `hostile_parameters_never_panic` (23 commands), 5 headless `egui_kittest` tests, `craft-atomic`, `AutomationRoots` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 35% | [`roadmap.md`](roadmap.md) | `craft` (`ui`, `ui-app`, `theme-store`, `dialogs`), `craft-proof`, `craft-atomic`, `craft-docx`, and `craft-wordmodel` integrated; `grammarcraft-cli mcp` active; `craft-i18n` & upstreaming in M5 |
