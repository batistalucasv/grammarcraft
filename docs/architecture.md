# GrammarCraft architecture

> **Last reviewed:** 2026-11-27 · **Change:** add M4 hybrid consent-gated AI (`craft-ai-core`, `craft-ai-openai`, `craft-ai-client` from `storytold/craft-ai-integrations` rev `a822ad5b4db2c0dcd7a43bb62d6d75251fd3f8b5`), inclusive/formality rules (`inclusive_formality.rs`), local N-gram similarity (`similarity.rs`), 12 citation styles + BibTeX, 12 new commands (35 total), and `AI Assist` / `Citations & Similarity` desktop tabs (7 headless UI tests).

How the workspace is built today. The rules for working in it are in [`AGENTS.md`](../AGENTS.md); this file describes the shape, the layer boundaries, and why they exist.

## Layers

`Cargo.toml` declares `crates/*`, `apps/*`, and `xtask` as members. `cargo xtask layers` reads `cargo metadata` and fails if a crate depends on a layer at or above its own (including `[dev-dependencies]`), or if a workspace crate was never added to `TABLE` in `xtask/src/layers.rs`.

Because `xtask/src/layers.rs` forbids lateral dependencies (`target >= layer`), the deterministic linguistic pipeline (`grammarcraft-rules`) sits at **L1** below `grammarcraft-engine` (**L2**):

| Layer | Crate | Job |
|---|---|---|
| L0 | [`core`](../crates/core/src) | `Document`, UTF-8-safe `TextSpan`, `WritingGoals` (including `inclusive_language`), `TextStats`, `Issue*`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`CUSTOM.DIC`), `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, `SimilarityMatch`/`SimilarityReport`, 12 `CitationStyle`s + `CitationEntry` + `parse_bibtex_entries`/`validate_citations`/`format_bibliography`, `AiMode`/`AiConsentStatus`/`AiProposal` + `compute_word_diff`, `CoreError`, and `MAX_*` bounds (`MAX_DOCUMENT_BYTES`, `MAX_AI_TEXT_BYTES`, `MAX_DIFF_HUNKS`, `MAX_CITATIONS`, `MAX_BIBTEX_BYTES`, `MAX_SIMILARITY_MATCHES`, etc.); zero I/O, zero `serde_json`, zero UI |
| L1 | [`rules`](../crates/rules/src) | `AnalysisEngine` with per-paragraph cache (keyed by text hash, language tag, `WritingGoals` including `inclusive_language`, `craft_proof::dictionary_generation()`, and `StyleGuide` hash), `craft-proof` adapter, EN/PT-BR/ES contextual confusables and inclusive/formality checks (`inclusive_formality.rs`), regional variant checks (`en-US`/`en-GB`, `pt-BR`/`pt-PT`), `StyleGuide` linear-time `regex` evaluation, deterministic language-isolated clarity/style rules, open `ReadabilityMetrics` & `WritingScore`, span-anchored `ToneProfile`, and deterministic N-gram/Jaccard `check_local_similarity` (`similarity.rs`) |
| L2 | [`engine`](../crates/engine/src) | `Session` (`last_report()`), bounded undo/redo (`MAX_UNDO_STEPS = 200`), `file_access::AutomationRoots` confinement, `io` (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel` with `MAX_DOCX_FILE_BYTES = 16 MiB`, `.txt`/`.md` via `craft_atomic::atomic_write`, confined `CUSTOM.DIC` import/export), `ai.rs` (`LocalOfflineBackend` zero-network default + `CraftProviderBackend` wrapping `craft_ai_openai::OpenAiProvider` / `MockProvider` gated by `ConsentLedger` + `Disclosure`), and `Registry` of **35** `CommandSpec`s (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`) |
| app | [`grammarcraft`](../apps/grammarcraft/src) | Native `egui`/`eframe` desktop app (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`; `craft::ui::shell::app::{AppSpec, run}`): light/dark themes, top toolbar, editor with redundant category underlines and severity markers (`[!]`/`[~]`/`[i]`), bidirectional 6-tab Review Pane (`Alerts`, `Goals & Score` with `Inclusive language (opt-in)` toggle, `Tone Profile`, `Style & Snippets`, `AI Assist` with consent/diff/apply/discard, `Citations & Similarity` with 12 styles + BibTeX + local directory check), keyboard navigation (`Tab`/`Shift+Tab`/`F6`/`Escape`), and AccessKit integration; routes all mutations strictly via `Session::run` |
| app | [`grammarcraft-cli`](../apps/grammarcraft-cli/src) | Thin CLI (`commands`, `run`, `check` supporting `.txt`/`.md`/`.docx`/`.grammarcraft.json`, `stats`), JSON output, exit codes (`0`, `1`, `2`), and `mcp` JSON-RPC stdio server requiring both `--automation-read-root` and `--automation-write-root` |
| tooling | [`xtask`](../xtask/src) | `ci`, `layers`, `assets` |

Dependency direction:
- `grammarcraft-cli` → `grammarcraft-engine` (L2) → `grammarcraft-rules` (L1) → `grammarcraft-core` (L0).
- `apps/grammarcraft` (`Class::Exempt`) links `grammarcraft-engine`, `grammarcraft-rules` (for read-only `AnalysisReport`), and `grammarcraft-core` (for read-only domain types), while routing **every** state mutation exclusively through `Session::run(id, params)` and reading cached state via `Session` accessors.

Workspace pins in root `Cargo.toml`:
- `craft`, `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel`: `storytold/craft-libs` rev `8be80b006888ae023d61eed265241c9b78b84981`.
- `craft-ai-core`, `craft-ai-openai`, `craft-ai-client`: `storytold/craft-ai-integrations` rev `a822ad5b4db2c0dcd7a43bb62d6d75251fd3f8b5` (kept separate from `craft/ai-core` and `craft-ai-egui` per ecosystem sync boundaries).
- `[patch.crates-io]` for `egui` `0.37` family: `storytold/egui` rev `932a73b28560acb1a695533307b2cd0d66a42081`.
- `[patch.crates-io]` for `winit`: `storytold/winit` rev `a517b3846ee8457d84ee4975110bfda6797c44dc`.

## Data flow

1. **A call arrives** as a command id plus JSON parameters from the desktop UI (`apps/grammarcraft`), the CLI (`run`, `check`, `stats`), or JSON-RPC `tools/call` on `grammarcraft-cli mcp`.
2. **`Session::run`** looks the id up in `Registry`, calls `spec.enabled(self)` (returning `CmdError::Disabled` when unavailable), and invokes `spec.run(self, params)`.
3. **The command** validates parameters using `p::*` helpers, returning `CmdError::Params` naming any missing, invalid, or mistyped key.
4. **Confinement, domain, analysis & AI:**
   - Disk reads/writes (`doc.open`, `doc.save`, `dict.import`, `dict.export`, `similarity.check`) pass through `file_access` (`AutomationRoots`) and bounded reads, rejecting paths outside `--automation-read-root` / `--automation-write-root` when confined.
   - Mutating commands push a bounded `HistorySnapshot` onto `undo_stack` before mutating.
   - `ai.rewrite` / `ai.adjustTone` / `ai.summarize` / `ai.expand` / `ai.draft` default to `LocalOfflineBackend` (zero network); cloud calls via `CraftProviderBackend` require explicit `ai.consent` approval recorded in `ConsentLedger` + `Disclosure` and stage an `AiProposal` with word-level `AiDiffHunk`s without mutating `Document` until `ai.applyResult` runs (recorded as a single undo step).
   - `review.check` runs `AnalysisEngine::analyze` and caches the resulting `AnalysisReport` on `Session::last_report()`.
5. **`From<CoreError> for CmdError`** maps invariant violations to `CmdError::Params`.
6. **The front end** renders `AnalysisReport`, `AiProposal` diffs, or bibliography/similarity output (desktop UI), prints JSON with exit code `0`/`1`/`2` (CLI), or returns JSON-RPC 2.0 responses (MCP).

Every step returns `Result`; production crates forbid `unsafe` (`unsafe_code = "forbid"`) and deny `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, and `unreachable!`.

## Testing and gates

- Domain invariant tests in `crates/core/src/tests.rs` (`MAX` / `MAX + 1` bounds, UTF-8 `TextSpan`, `IssueId`, `WritingGoals`, `PersonalDictionary` + `CUSTOM.DIC`, `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, BibTeX parsing + 12 citation styles, word-diff computation).
- Linguistic rule and similarity tests in `crates/rules/src/tests.rs` (EN/PT-BR/ES proofing, contextual confusables, regional variants, clarity/passive/redundancy, `inclusive_formality`, `check_local_similarity`, `ReadabilityMetrics`, `ToneProfile`, `StyleGuide` regex safety, incremental cache hits/invalidation, document-scoped suppression).
- Engine contract tests in `crates/engine/src/tests.rs` (undo/redo across edits/dictionary/style/snippets/AI apply, `.grammarcraft.json`/`.txt`/`.docx` round-trips and corrupted `.docx` handling, `CUSTOM.DIC` and `similarity.check` under `AutomationRoots`, cloud AI consent blocking + `MockProvider` flow, and `hostile_parameters_never_panic` sweeping all **35** `registry().all()` commands).
- Desktop UI headless tests in `apps/grammarcraft/src/tests.rs` (7 `egui_kittest` tests: typing → `review.check` → suggestion card `Apply`, `Apply` + `Undo`, `goals.set` → `WritingScore`, active `AccessKit` tree, theme toggle/`Tone Profile`/`Add to Dictionary`, AI cloud consent + rewrite + `Apply` + `Undo`, and `Citations & Similarity` formatting + inclusive toggle).
- `apps/grammarcraft-cli/tests/cli.rs` drives the built `grammarcraft-cli` binary for `commands`, `run`, `check`, `style.*`, `snippet.*`, `dict.*`, exit codes `0`/`1`/`2`, and `mcp` rejection when either automation root flag is missing.
- `cargo xtask ci` runs `fmt --check`, `clippy -D warnings`, `cargo nextest run --workspace`, `layers`, and `assets`.
- `deny.toml` records the SPDX licence allowlist matching `storytold/craft-contrib` (`standards/licences.md`).
- GitHub Actions runs shared workflows from `storytold/craft-common-actions` (`gate.yml@main` and `house-rules.yml@main`, installing Linux UI system packages in `ci.yml`).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-13 | major | Initial template baseline: layers, data flow, gates |
| 2026-11-27 | major | Replace `Note` example with GrammarCraft `Document`, `TextSpan`, `WritingGoals`, `TextStats`, and `doc.*`/`goals.set`/`stats.measure` commands |
| 2026-11-27 | major | Add L1 `grammarcraft-rules` (`craft-proof`, clarity/style/readability/tone), L2 bounded undo/redo, `craft-atomic` persistence, `AutomationRoots`, 14 `CommandSpec`s, and CLI `check` / `mcp` |
| 2026-11-27 | major | Add `deny.toml`, M2 `StyleGuide`, `SnippetCatalog`, `CUSTOM.DIC`, EN/PT-BR contextual confusables, regional variant checks, `regex = "1"` evaluator, and 9 `style.*`/`snippet.*`/`dict.*` commands (23 total) |
| 2026-11-27 | minor | Add `.docx` import/export via `craft-docx` and `craft-wordmodel`, Spanish (`es*`) contextual paronym rules, and clarity language isolation |
| 2026-11-27 | major | Add native `egui` desktop app (`apps/grammarcraft`), `storytold/egui` + `storytold/winit` workspace pins, `Session::last_report`, and 5 headless `egui_kittest` + AccessKit tests |
| 2026-11-27 | major | Add M4 consent-gated hybrid AI (`craft-ai-core`, `craft-ai-openai`, `craft-ai-client`), `inclusive_formality.rs`, `similarity.rs`, 12 citation styles + BibTeX, 12 commands (35 total), and `AI Assist` / `Citations & Similarity` tabs (7 UI tests) |
