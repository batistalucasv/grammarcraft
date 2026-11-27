# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** M1 foundation implemented (`grammarcraft-rules` L1 pipeline with `craft-proof`, clarity/readability/`WritingScore`/`ToneProfile`, bounded undo/redo, `craft-atomic` persistence, `AutomationRoots`, 14 `CommandSpec`s, and CLI `check`/`mcp`). · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M1 foundation)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **14 / 47 planned (29.8%)** | measured from `crates/engine/src/cmd.rs` (`doc.set_text`, `doc.get`, `doc.clear`, `doc.save`, `doc.open`, `history.undo`, `history.redo`, `goals.set`, `stats.measure`, `review.check`, `review.apply`, `review.dismiss`, `dict.add`, `dict.remove`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~14%** | estimated from implemented L0/L1/L2 + CLI/MCP foundations |
| Feature depth (weighted by typical writer usage, table below) | **~14%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~15%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~14%** | estimated |
| **Essentials proofreader** | **~25%** | estimated (`craft-proof` + clarity/readability + undo/redo + `.grammarcraft.json`/`.txt`/`.md` + CLI/MCP working; `.docx` & desktop UI pending) |
| Existing shared foundation in the ArtCraft ecosystem | `craft-proof` & `craft-atomic` wired (`craft-libs` rev `8be80b0`); `craft-text`, `craft-docx`, `craft-ai-integrations`, `craft-ui-*`, `craft-mcp-server` available | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M1 foundation.** `cargo xtask ci` passes across 5 workspace crates (`grammarcraft-core`, `grammarcraft-rules`, `grammarcraft-engine`, `grammarcraft-cli`, `xtask`) with deterministic proofing, bounded undo/redo, atomic `.grammarcraft.json`/`.txt`/`.md` I/O, and root-confined stdio MCP; `.docx`, contextual rules, `StyleRules`/`CUSTOM.DIC`/`Snippets`, desktop UI, and hybrid AI remain in M1–M4.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned for M2 under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 15% | 0% | `crates/rules` wraps `craft_proof::check_text_with` with per-paragraph cache (`dictionary_generation()`) and per-doc `PersonalDictionary` / `IgnoredIssueSet` filtering; contextual paronym/homophone rules and regional mix alerts remain in M2. |
| Grammar, punctuation & mechanics | 16 | 10% | 0% | `craft-proof` conservative base checks wired in `crates/rules`; agreement, crasis/government (PT), punctuation/comma-splice rules, and full goal-sensitive filtering remain in M2. |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 15% | 0% | Initial deterministic EN/PT-BR passive voice (modulated by `WritingGoals`), long-sentence alerts, and common redundancy rules landed in `crates/rules/src/clarity.rs`; full rewrite tables & structural simplification remain in M2. |
| Style, vocabulary & readability metrics | 10 | 25% | 0% | Sliding-window lexical repetition, Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes, and open 0–100 `WritingScore` landed in `crates/rules/src/scoring.rs`; vague-verb precision and syntactic rhythm remain in M2. |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 20% | 0% | Deterministic multi-signal `ToneProfile` anchored to exact `TextSpan`s landed in `crates/rules/src/tone.rs`; formality-mismatch checks, opt-in inclusive language, and optional ONNX classifier remain in M4. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 30% | 0% | Four-axis `WritingGoals` model in `grammarcraft-core` + `goals.set` command already modulate passive-voice diagnostics; broader category modulation remains in M2. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 15% | 0% | Document-scoped `PersonalDictionary` (`dict.add`/`dict.remove`) and `IgnoredIssueSet` (`review.dismiss`) landed in L0/L1/L2; `CUSTOM.DIC`, declarative `StyleRules`, and `{{field}}` `SnippetCatalog` remain in M2. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 0% | 0% | `craft-ai-client`, `craft-ai-openai` (`text.complete`), `craft-ai-native`, and `craft-ai-egui` exist upstream; prompt modes, diff preview, atomic undo, and NLP runner in `craft-ai-engine-rten` remain in M4. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 0% | 0% | `craft-ui-shell`, `craft-ui-patterns`, `craft-ui-widgets`, `craft-text`, and `wordcraft`'s `SpellingForm` pattern exist upstream; desktop UI, category underlines, and bidirectional Review Pane remain in M3. |
| Local similarity check & citation formatting | 3 | 0% | 0% | Local N-gram/MinHash comparison over authorized directories plus validation/formatting for the 12 standard bibliography styles remain in M4. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 35% (beyond target) | 0% | 14 `CommandSpec`s, `grammarcraft-cli check <file>`, and stdio JSON-RPC `mcp` requiring `--automation-read-root` and `--automation-write-root` (`AutomationRoots`) landed in M1; SARIF & desktop `--control` remain. |

Weighted feature depth: **13.5% (~14%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~15%** | All dimensions below including hybrid AI, local similarity, i18n, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~14%** | Spelling + contextual + grammar + clarity + style + goals + style rules + native UI (M0–M3) |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~25%** | `craft-proof` + clarity/readability + undo/redo + `.grammarcraft.json`/`.txt`/`.md` + CLI/MCP done; `.docx` & desktop UI remaining |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 14% | this file | Weighted sum of the 11 user-facing areas above (13.5%) |
| Linguistic accuracy & low false-positive rate | 20% | 10% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + deterministic clarity/tone tests passing; open GEC/proofing corpus benchmarks planned in M2 & M4 |
| UI/UX fidelity & accessibility | 15% | 0% | [`architecture.md`](architecture.md) | Redundant category underlines, keyboard-first flow, AccessKit tree, `craft-design-lint` (M3) |
| Privacy, licensing & local-first AI | 10% | 15% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | 100% offline local pipeline & `NOTICE` updated; `deny.toml` + AI consent ledger planned (M0, M4) |
| Stability & never-crash guarantees | 10% | 45% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `hostile_parameters_never_panic`, `craft-atomic` save, and `AutomationRoots` active in M1 |
| Performance (incremental typing latency) | 5% | 30% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache active in `crates/rules` |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 20% | [`roadmap.md`](roadmap.md) | `craft-proof` & `craft-atomic` integrated, `grammarcraft-cli mcp` active; `craft-i18n` & upstreaming in M5 |
