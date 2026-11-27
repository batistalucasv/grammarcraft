# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** `.docx` import/export (`craft-docx` + `craft-wordmodel`), Spanish (`es*`) contextual paronym rules, and clarity language isolation added across L1/L2/CLI. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M1 `.docx` + M2 EN/PT-BR/ES deterministic engine)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **23 / 47 planned (48.9%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~23%** | estimated from implemented L0/L1/L2 + CLI/MCP foundations |
| Feature depth (weighted by typical writer usage, table below) | **~23%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~23%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~23%** | estimated |
| **Essentials proofreader** | **~40%** | estimated (`craft-proof` + EN/PT-BR/ES contextuals & variants + clarity/readability + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP working; desktop UI pending) |
| Existing shared foundation in the ArtCraft ecosystem | `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` wired (`craft-libs` rev `8be80b0`); `craft-text`, `craft-ai-integrations`, `craft-ui-*`, `craft-mcp-server` available | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M2 deterministic core (+ `.docx` I/O).** `cargo xtask ci` passes across 5 workspace crates (`grammarcraft-core`, `grammarcraft-rules`, `grammarcraft-engine`, `grammarcraft-cli`, `xtask`) with `deny.toml` SPDX allowlist configured; expanded grammar/clarity depth, desktop UI (M3), and hybrid AI (M4) remain open.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 45% | 40% | `crates/rules` integrates `craft-proof`, EN, PT-BR, and ES (`es*`) contextual paronyms/homophones (`contextual.rs`), and `en-US`/`en-GB` + `pt-BR`/`pt-PT` variant mix checks (`variants.rs`) with incremental caching; corpus benchmarks remain. |
| Grammar, punctuation & mechanics | 16 | 10% | 10% | `craft-proof` conservative base checks wired in `crates/rules`; expanded agreement, crasis/government (PT), punctuation/comma-splice rules, and full goal-sensitive filtering remain. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 15% | 15% | Initial deterministic EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, and common redundancy rules landed in `crates/rules/src/clarity.rs` (isolated by language); full rewrite tables & structural simplification remain. |
| Style, vocabulary & readability metrics | 10 | 25% | 25% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 20% | 20% | Deterministic multi-signal `ToneProfile` anchored to exact `TextSpan`s landed in `crates/rules/src/tone.rs`; formality-mismatch checks, opt-in inclusive language, and optional ONNX classifier remain in M4. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 30% | 30% | Four-axis `WritingGoals` model in `grammarcraft-core` + `goals.set` command modulate passive-voice diagnostics; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O & UI remain. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 0% | 0% | `craft-ai-client`, `craft-ai-openai` (`text.complete`), `craft-ai-native`, and `craft-ai-egui` exist upstream; prompt modes, diff preview, atomic undo, and NLP runner in `craft-ai-engine-rten` remain in M4. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 0% | 0% | `craft-ui-shell`, `craft-ui-patterns`, `craft-ui-widgets`, `craft-text`, and `wordcraft`'s `SpellingForm` pattern exist upstream; desktop UI, category underlines, and bidirectional Review Pane remain in M3. |
| Local similarity check & citation formatting | 3 | 0% | 0% | Local N-gram/MinHash comparison over authorized directories plus validation/formatting for the 12 standard bibliography styles remain in M4. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 45% (beyond target) | 45% | 23 `CommandSpec`s, `grammarcraft-cli check <file>` (`.txt`/`.md`/`.docx`/`.grammarcraft.json`), and stdio JSON-RPC `mcp` requiring `--automation-read-root` and `--automation-write-root` (`AutomationRoots`) landed; SARIF & desktop `--control` remain. |

Weighted feature depth: **22.9% (~23%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~23%** | All dimensions below including hybrid AI, local similarity, i18n, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~23%** | Spelling + contextual + grammar + clarity + style + goals + style rules + native UI (M0–M3) |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~40%** | `craft-proof` + EN/PT-BR/ES contextuals/variants + clarity/readability + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP done; desktop UI remaining |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 23% | this file | Weighted sum of the 11 user-facing areas above (22.9%) |
| Linguistic accuracy & low false-positive rate | 20% | 22% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style tests passing; open GEC/proofing corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 0% | [`architecture.md`](architecture.md) | Redundant category underlines, keyboard-first flow, AccessKit tree, `craft-design-lint` (M3) |
| Privacy, licensing & local-first AI | 10% | 25% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | 100% offline local pipeline, `NOTICE` / `ATTRIBUTION.md`, and `deny.toml` SPDX allowlist configured (`cargo-deny` check pending) |
| Stability & never-crash guarantees | 10% | 55% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `RegexBuilder` size/DFA limits, `MAX_DOCX_FILE_BYTES`, `MAX_CUSTOM_FILE_BYTES`, `hostile_parameters_never_panic` (23 commands), `craft-atomic`, `AutomationRoots` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 25% | [`roadmap.md`](roadmap.md) | `craft-proof`, `craft-atomic`, `craft-docx`, and `craft-wordmodel` integrated; `grammarcraft-cli mcp` active; `craft-i18n` & upstreaming in M5 |
