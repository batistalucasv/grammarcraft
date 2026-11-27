# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** initial M0/M1 core domain (`Document`, `TextSpan`, `WritingGoals`, `TextStats`, `CoreError`) and 5 engine commands (`doc.set_text`, `doc.get`, `doc.clear`, `goals.set`, `stats.measure`) replaced placeholder `note.*` commands; weighted proofing/UI areas remain at 0% baseline. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, initial M0/M1 domain & CLI commands)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **5 / 47 planned (10.6%)** | measured from `crates/engine/src/cmd.rs` (`doc.set_text`, `doc.get`, `doc.clear`, `goals.set`, `stats.measure`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **0%** | baseline |
| Feature depth (weighted by typical writer usage, table below) | **0%** | baseline |
| **Ready for daily writing & review** (full target) | **0%** | baseline (weighted sum over [By dimension](#by-dimension)) |
| **Mainstream writer / professional** | **0%** | baseline |
| **Essentials proofreader** | **0%** | baseline |
| Existing shared foundation in the ArtCraft ecosystem | `craft-proof` (11 grammar locales + SCOWL/Hunspell/packs), `craft-text`, `craft-ai-integrations` (`text.complete` + `SecretStore`/`ConsentFile`), `craft-ui-*` + `craft-mcp-server` | verified in sibling ArtCraft repositories |

**Stage: M0/M1 foundation.** `cargo xtask ci` passes with `Document`, UTF-8 `TextSpan`, `WritingGoals`, `TextStats`, `CoreError`, and the 5 initial `doc.*` / `goals.set` / `stats.measure` commands; proofing rules, persistence, undo/redo, MCP confinement, and UI remain pending in M1–M3.

## What we measured against

- **Publicly documented behaviour** of commercial writing assistants (spelling/grammar review, clarity rewrites, readability/engagement signals, tone analysis, writing goals, personal dictionary/style rules/snippets, generative rewrite, similarity/citation checks, desktop/CLI/MCP surfaces).
- **Clean-room boundary (`standards/clean-room.md`):** no installer, browser extension bundle, internal lexicon, prompt, or numerical score was inspected or measured. All quality scores (`WritingScore`) and readability metrics use open, documented formulas.
- **Not measured yet:** precision/recall benchmarks against open permissive GEC/proofing corpora (planned for M2 under `clean-room/verification.md`).

## Feature areas

Weight estimates the share of a typical writer/editor's usage across the product (sums to 100).

| Area | Weight | Parity | Was | Evidence and what is planned |
| --- | --- | --- | --- | --- |
| Spelling & regional variants (lexical + contextual confusables) | 18 | 0% | — | Upstream `craft-proof` provides SCOWL, `en-variants.txt`, Moby, Hunspell `.aff`/`.dic` reader, and `packs`; app needs contextual paronym/homophone rules, regional mix alerts, and per-doc scoping (M1–M2). |
| Grammar, punctuation & mechanics | 16 | 0% | — | `craft-proof` provides conservative base checks (`en`, `pt`, `es`, `fr`, `pl`, `it`, `ru`, `tr`, `hr`, `cs`, `id`); app needs agreement, crasis/government (PT), punctuation/comma-splice rules, and goal-sensitive filtering (M2). |
| Clarity & conciseness (passive voice, long sentences, redundancy) | 14 | 0% | — | Not yet in `craft-proof`; planned in `grammarcraft-rules` (M2) with deterministic word/phrase rewrites and structural simplification. |
| Style, vocabulary & readability metrics | 10 | 0% | — | Hyphenation seam exists in `craft-proof` / `craft-text`; needs vague-verb alternatives, sliding-window repetition detection, sentence rhythm, open readability indices, and transparent 0–100 `WritingScore` (M2). |
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 0% | — | Planned deterministic multi-signal profile with span evidence plus optional local ONNX classifier, formality checks, and opt-in inclusive language rules (M4). |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 0% | — | Four-axis `WritingGoals` model in `grammarcraft-core` driving rule activation and severity weights (M1–M2). |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 0% | — | `craft-proof::user_dict` and `wordcraft` `CUSTOM.DIC` pattern exist; needs declarative TOML/JSON `StyleRules` (`regex` linear-time) and `{{field}}` `SnippetCatalog` (M2). |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 0% | — | `craft-ai-client`, `craft-ai-openai` (`text.complete`), `craft-ai-native`, and `craft-ai-egui` exist; needs prompt modes, diff preview, atomic undo, and NLP runner in `craft-ai-engine-rten` (M4). |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 0% | — | `craft-ui-shell`, `craft-ui-patterns`, `craft-ui-widgets`, `craft-text`, and `wordcraft`'s `SpellingForm` pattern exist; needs category underlines with redundant encoding and bidirectional Review Pane (M3). |
| Local similarity check & citation formatting | 3 | 0% | — | Needs local N-gram/MinHash comparison over authorized directories plus validation/formatting for the 12 standard bibliography styles validated in `wordcraft` (M4). |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | — | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 0% (beyond target) | — | Commercial reference has no local stdio MCP with root confinement; GrammarCraft ships `CommandSpec` + `craft-mcp-server` with `--automation-read-root` / `--automation-write-root` from M1. |

Weighted feature depth: **0%** (baseline).

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **0%** | All dimensions below including hybrid AI, local similarity, i18n, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **0%** | Spelling + contextual + grammar + clarity + style + goals + style rules + native UI (M0–M3) |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **0%** | `craft-proof` integration + undo/redo + `.txt`/`.md`/`.docx` + CLI/MCP (M0–M1) |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 0% | this file | Weighted sum of the 11 user-facing areas above |
| Linguistic accuracy & low false-positive rate | 20% | 0% | `clean-room/verification.md` *(planned)* | Measured on open permissive GEC/proofing corpora in M2 & M4 |
| UI/UX fidelity & accessibility | 15% | 0% | [`architecture.md`](architecture.md) | Redundant category underlines, keyboard-first flow, AccessKit tree, `craft-design-lint` (M3) |
| Privacy, licensing & local-first AI | 10% | 0% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | `licences.md` allowlist enforced (`deny.toml`); explicit consent & `Local (Offline)` indicator (M0, M4) |
| Stability & never-crash guarantees | 10% | 0% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, `hostile_parameters_never_panic`, atomic save (M0–M1) |
| Performance (incremental typing latency) | 5% | 0% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental analysis keeping UI frame time responsive (M1, M3) |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 0% | [`roadmap.md`](roadmap.md) | Shared `craft-proof` improvements, `grammarcraft-cli mcp`, `craft-i18n` TSV catalogs (M1, M5) |
