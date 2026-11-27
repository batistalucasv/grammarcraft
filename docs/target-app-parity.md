# GrammarCraft parity with commercial writing assistants

> **Last reviewed:** 2026-11-27 · **Last updated:** 2026-11-27 · **Change:** M5 `craft-ai-native` persistence (`ConsentFile`, `HistoryFile`, `SecretStore`), loopback `--control` server (`craft/automation` + `AutomationRoots`), `locales/en/grammarcraft.tsv`, `assets/app-icon/grammarcraft.svg`, and `packaging/linux/` implemented. · **Target:** Commercial desktop/web writing assistants (black-box public feature surface)

Authoritative readiness tracker for GrammarCraft. [`roadmap.md`](roadmap.md) tracks milestone gates; [`gaps.md`](gaps.md) sequences dependency-ordered work. Following `standards/clean-room.md` (`storytold/craft-contrib`), parity targets **visible user-facing capabilities** only—never proprietary numerical scores, internal rules, copy, or assets.

## Headline (2026-11-27, M5 native persistence, `--control` automation & packaging baseline)

| Number | Value | Kind |
| --- | --- | --- |
| Domain & proofing command catalog coverage | **35 / 47 planned (74.5%)** | measured from `crates/engine/src/cmd.rs` (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`) |
| Feature breadth (weighted across writing areas, languages, AI modes, surfaces) | **~41%** | estimated from implemented L0/L1/L2 + CLI/MCP + native desktop UI + `--control` + M4/M5 foundations |
| Feature depth (weighted by typical writer usage, table below) | **~41%** | estimated: weighted sum over [Feature areas](#feature-areas) |
| **Ready for daily writing & review** (full target) | **~52%** | estimated: weighted sum over [By dimension](#by-dimension) |
| **Mainstream writer / professional** | **~46%** | estimated |
| **Essentials proofreader** | **~68%** | estimated (native `egui` desktop editor + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + EN/PT-BR/ES proofing + clarity/readability/inclusive + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + persistent AI consent/history/secrets + citations/similarity + CLI/MCP/`--control` working) |
| Existing shared foundation in the ArtCraft ecosystem | `craft` (`automation` incl.), `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel` (`craft-libs` `8be80b0`); `craft-ai-core`, `craft-ai-openai`, `craft-ai-client`, `craft-ai-native` (`craft-ai-integrations` `a822ad5`) | verified in workspace `Cargo.toml` and sibling ArtCraft repositories |

**Stage: M5 integration baseline.** `cargo xtask ci` passes across 6 workspace crates with 7 headless `egui_kittest` + AccessKit tests, `craft-ai-native` persistence tests, and authenticated loopback `--control` tests; local ONNX `rten`/`candle` NLP `ModelRunner`, `release.yml` CI PR, macOS/Windows packaging, upstream PRs (`craft-proof`/`craft-i18n`/`craft-ai-agents`), `craft-design-lint`, and expanded grammar/clarity depth remain open.

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
| Tone analysis & appropriateness (multi-signal + formality + inclusive) | 8 | 55% | 55% | Span-anchored `ToneProfile` (`tone.rs`) plus `inclusive_formality.rs` (formality-mismatch contractions/colloquialisms under `Formality::Formal` and opt-in `WritingGoals::inclusive_language` in EN/PT-BR/ES) landed; treatment consistency & optional local ONNX classifier remain. |
| Document goals & rule filtering (Audience, Formality, Context, Purpose) | 6 | 45% | 45% | `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`, `inclusive_language`) modulates passive voice, formality, and inclusive rules across L0/L1/L2/UI; broader category modulation remains. |
| Personal dictionary, ignore rules, team style guide & snippets | 8 | 65% | 65% | `PersonalDictionary` (`case_sensitive`, `CUSTOM.DIC` UTF-16LE BOM + UTF-8 via `dict.import`/`dict.export`), `IgnoredIssueSet`, declarative `StyleGuide` (`style.*` + linear-time `regex` evaluator), and `SnippetCatalog` (`snippet.*` with `{{placeholder}}` expansion) landed across L0/L1/L2; TOML/JSON style-file I/O remains. |
| Generative rewrite & AI assistance (hybrid local/cloud with diff) | 8 | 80% | 65% | `LocalOfflineBackend` + `CraftProviderBackend` (`craft-ai-core`, `craft-ai-openai`, `craft-ai-client`) with `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`) fail-closed persistence, 9 `ai.*` commands, word-level `AiDiffHunk` preview, `AI Assist` UI tab, and single-step undo landed; local ONNX `ModelRunner` (`rten`/`candle`) remains. |
| Native desktop editor, review pane & accessibility (AccessKit) | 7 | 80% | 80% | `apps/grammarcraft` ships `egui` desktop UI (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`, `automation`), redundant underlines, bidirectional 6-tab Review Pane, keyboard nav, and 7 headless `egui_kittest` + AccessKit tests; `craft-design-lint` remains. |
| Local similarity check & citation formatting | 3 | 85% | 85% | Deterministic N-gram/Jaccard `check_local_similarity` (`similarity.check` confined by `AutomationRoots`), BibTeX parser, `citations.validate`/`citations.format` across all 12 standard bibliography styles, and `Citations & Similarity` UI tab landed; optional opt-in web `SimilarityProvider` remains future. |
| System-wide OS overlay (UIA / AXUIElement / AT-SPI2 in Rust) | 2 | 0% | 0% | Post-M5 isolated platform adapters (`craft-win32`, `craft-appkit`, D-Bus AT-SPI2) without JS/Electron/webview (post-M5). |
| Agent control, CLI & MCP (`grammarcraft-cli mcp`) | 0 | 85% (beyond target) | 60% | 35 `CommandSpec`s, `grammarcraft-cli check <file>`, stdio JSON-RPC `mcp` requiring `--automation-read-root`/`--automation-write-root`, and desktop loopback `--control` (`ControlOptions`, `serve`, `ControlChannel`, `engine.*`/`document.inspect`/`ui.inspect`/`ui.accessTree`) landed; SARIF CLI format remains. |

Weighted feature depth: **41.15% (~41%)**.

## Audience readiness profiles

| Audience | Ready | Scope & critical gates |
| --- | --- | --- |
| **Full target** (complete standalone writing & review suite) | **~52%** | All dimensions below including local ONNX NLP `ModelRunner`, `release.yml`, `craft-i18n` sync, and OS integration |
| **Mainstream writer** (articles, business/academic writing in native editor + CLI) | **~46%** | Native UI + spelling/contextual + inclusive/formality + persistent AI diff + citations/similarity + style/snippets working; expanded grammar/clarity depth & `craft-design-lint` pending |
| **Essentials proofreader** (fast offline spelling, basic grammar, metrics, CLI) | **~68%** | Native `egui` desktop app + `craft-proof` + EN/PT-BR/ES + clarity/readability + persistent AI/citations/similarity + `CUSTOM.DIC`/`StyleGuide`/`Snippets` + `.docx`/`.grammarcraft.json`/`.txt`/`.md` + undo/redo + CLI/MCP/`--control` working |

## By dimension

Weights estimate how strongly each dimension determines whether a user can adopt GrammarCraft as their primary writing assistant.

| Dimension | Weight | Parity | Doc | Evidence & target gate |
| --- | --- | --- | --- | --- |
| Features (depth, table above) | 35% | 41% | this file | Weighted sum of the 11 user-facing areas above (41.15%) |
| Linguistic accuracy & low false-positive rate | 20% | 28% | `clean-room/verification.md` *(planned)* | Base `craft-proof` + EN/PT-BR/ES contextual/variant/clarity/tone/style/inclusive tests passing; open GEC/proofing corpus benchmarks planned |
| UI/UX fidelity & accessibility | 15% | 75% | [`architecture.md`](architecture.md) | 6-tab Review Pane (`AI Assist`, `Citations & Similarity`), redundant underlines, keyboard nav, AccessKit tree verified in 7 `egui_kittest` tests; `craft-design-lint` pending |
| Privacy, licensing & local-first AI | 10% | 90% | [`ATTRIBUTION.md`](../ATTRIBUTION.md) | Zero-network `LocalOfflineBackend` default, `ConsentFile`/`HistoryFile`/`SecretStore` persistence, `deny.toml` allowlist, SVG icon attributed (`cargo-deny` pending) |
| Stability & never-crash guarantees | 10% | 70% | [`gaps.md`](gaps.md) | `unsafe_code = "forbid"`, panic-free lints, fail-closed corrupt-file persistence, token-rejected `--control` tests, `hostile_parameters_never_panic` (35 commands), `craft-atomic`, `AutomationRoots` |
| Performance (incremental typing latency) | 5% | 35% | [`architecture.md`](architecture.md) | Paragraph-hashed incremental `AnalysisEngine` cache keyed by text, language, `WritingGoals`, `dictionary_generation()`, and `StyleGuide` hash |
| Ecosystem integration (`wordcraft`, CLI/MCP, `craft-i18n`) | 5% | 65% | [`roadmap.md`](roadmap.md) | `craft` (incl. `automation`), `craft-libs`, `craft-ai-integrations` (incl. `craft-ai-native`), `locales/en/grammarcraft.tsv`, `packaging/linux/*` wired; upstream PRs & `release.yml` remain |
