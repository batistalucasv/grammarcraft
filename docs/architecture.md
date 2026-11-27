# GrammarCraft architecture

> **Last reviewed:** 2026-11-27 · **Change:** add `.docx` import/export via `craft-docx` and `craft-wordmodel` (`storytold/craft-libs` rev `8be80b006888ae023d61eed265241c9b78b84981`) plus clean-room Spanish (`es*`) contextual paronym rules and language isolation in `crates/rules`.

How the workspace is built today. The rules for working in it are in [`AGENTS.md`](../AGENTS.md); this file describes the shape, the layer boundaries, and why they exist.

## Layers

`Cargo.toml` declares `crates/*`, `apps/*`, and `xtask` as members. `cargo xtask layers` reads `cargo metadata` and fails if a crate depends on a layer at or above its own (including `[dev-dependencies]`), or if a workspace crate was never added to `TABLE` in `xtask/src/layers.rs`.

Because `xtask/src/layers.rs` forbids lateral dependencies (`target >= layer`), the deterministic linguistic pipeline (`grammarcraft-rules`) sits at **L1** below `grammarcraft-engine` (**L2**):

| Layer | Crate | Job |
|---|---|---|
| L0 | [`core`](../crates/core/src) | `Document`, UTF-8-safe `TextSpan`, `WritingGoals` (`Audience`, `Formality`, `Context`, `Purpose`), `TextStats`, `IssueId`, `IssueCategory`, `Severity`, `Confidence`, `Replacement`, `Issue`, `IssueList`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`case_sensitive` + `CUSTOM.DIC` UTF-8/UTF-16LE BOM parser/serializer), `IgnoredIssueSet`, `StyleGuide` (`PreferredTerm`, `BrandCasing`, `PatternRule`), `SnippetCatalog` (`{{placeholder}}` expansion), `CoreError`, and `MAX_*` bounds (`MAX_DOCUMENT_BYTES`, `MAX_GOAL_VALUE_BYTES`, `MAX_ISSUES`, `MAX_SUGGESTIONS_PER_ISSUE`, `MAX_CUSTOM_WORDS`, `MAX_TERM_BYTES`, `MAX_CUSTOM_FILE_BYTES`, `MAX_IGNORED_ISSUES`, `MAX_STYLE_RULES`, `MAX_RULE_PATTERN_BYTES`, `MAX_SNIPPETS`, `MAX_SNIPPET_BYTES`, `MAX_SNIPPET_PREFIX_BYTES`, `MAX_UNDO_STEPS`); zero I/O, zero `serde_json`, zero UI |
| L1 | [`rules`](../crates/rules/src) | `AnalysisEngine` with per-paragraph cache (keyed by text hash, language tag, `WritingGoals`, `craft_proof::dictionary_generation()`, and `StyleGuide` hash), `craft-proof` adapter with document-scoped `PersonalDictionary` / `IgnoredIssueSet` filtering, EN/PT-BR/ES contextual confusables (`their`/`there`/`they're`, `mas`/`mais`, `haber`/`a ver`, `hay`/`ahí`/`ay`, `echo`/`hecho`, `ves`/`vez`, `haya`/`halla`, `sino`/`si no`, `porque`/`por qué`, etc.), regional variant mix checks (`en-US`/`en-GB`, `pt-BR`/`pt-PT`), `StyleGuide` linear-time `regex` evaluation (`validate_style_rule` with bounded DFA/size limits), deterministic language-isolated clarity/style rules, open `ReadabilityMetrics` (Flesch, Flesch-Kincaid, Gulpease, reading/speaking minutes), open 0–100 `WritingScore`, and span-anchored multi-signal `ToneProfile` |
| L2 | [`engine`](../crates/engine/src) | `Session`, bounded undo/redo (`MAX_UNDO_STEPS = 200`, covering `Document`, `WritingGoals`, `PersonalDictionary`, `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`), `file_access::AutomationRoots` confinement, `io` (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel` with `MAX_DOCX_FILE_BYTES = 16 MiB`, `.txt`/`.md` via `craft_atomic::atomic_write`, plus confined `CUSTOM.DIC` import/export), parameter validation (`p::*`), and `Registry` of **23** `CommandSpec`s (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`, `style.*`, `snippet.*`) |
| app | [`grammarcraft-cli`](../apps/grammarcraft-cli/src) | Thin CLI (`commands`, `run`, `check` supporting `.txt`/`.md`/`.docx`/`.grammarcraft.json`, `stats`), JSON output, exit codes (`0`, `1`, `2`), and `mcp` JSON-RPC stdio server requiring both `--automation-read-root` and `--automation-write-root` |
| tooling | [`xtask`](../xtask/src) | `ci`, `layers`, `assets` |

Dependency direction: `grammarcraft-cli` → `grammarcraft-engine` (L2) → `grammarcraft-rules` (L1) → `grammarcraft-core` (L0). Nothing points upward or sideways. L0 keeps `thiserror` and `serde` (derive only; no `serde_json`); L1 adds `craft-proof` (`storytold/craft-libs` rev `8be80b006888ae023d61eed265241c9b78b84981`) and `regex = "1"`; L2 adds `craft-atomic`, `craft-docx`, `craft-wordmodel` (all from `storytold/craft-libs` at the same pinned revision), and `serde_json`.

A front end depends on `grammarcraft-engine` only and routes every mutation through `Session::run`.

## Data flow

1. **A call arrives** as a command id plus JSON parameters from the CLI (`run`, `check`, `stats`) or from JSON-RPC `tools/call` on `grammarcraft-cli mcp`.
2. **`Session::run`** looks the id up in `Registry`, calls `spec.enabled(self)` (returning `CmdError::Disabled` when unavailable), and invokes `spec.run(self, params)`.
3. **The command** validates parameters using `p::*` helpers (and `validate_style_rule` before mutating `StyleGuide` with `PatternRule`), returning `CmdError::Params` naming any missing, invalid, or mistyped key.
4. **Confinement, domain & analysis:**
   - Disk reads/writes (`doc.open`, `doc.save`, `dict.import`, `dict.export`) pass through `file_access` (`AutomationRoots`), bounded reads (`MAX_OPEN_FILE_BYTES`, `MAX_DOCX_FILE_BYTES`, `MAX_CUSTOM_FILE_BYTES`), and `craft_atomic::atomic_write`, rejecting paths outside `--automation-read-root` / `--automation-write-root` when confined. `.docx` payloads use `craft_docx::read` / `craft_docx::write` (setting `craft_docx::set_application_name("GrammarCraft")`) and `craft_wordmodel::Document`.
   - Mutating commands (`doc.set_text`, `doc.clear`, `doc.open`, `goals.set`, `review.apply`, `review.dismiss`, `dict.add`, `dict.remove`, `dict.import`, `style.add_rule`, `style.remove_rule`, `snippet.add`, `snippet.remove`) push a bounded `HistorySnapshot` onto `undo_stack` before mutating.
   - `grammarcraft-core` enforces `MAX_*` limits and UTF-8 `is_char_boundary` checks before allocating, returning `CoreError`.
   - `review.check` runs `AnalysisEngine::analyze`, reusing cached per-paragraph `RawFinding`s and filtering by the session's `PersonalDictionary` and `IgnoredIssueSet`.
5. **`From<CoreError> for CmdError`** maps invariant violations to `CmdError::Params`.
6. **The front end** prints the JSON result or error with exit code `0`, `1`, or `2` (or returns a JSON-RPC 2.0 response over stdio in `mcp`).

Every step returns `Result`; production crates forbid `unsafe` (`unsafe_code = "forbid"`) and deny `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, and `unreachable!`.

## Testing and gates

- Domain invariant tests live in `crates/core/src/tests.rs` (`MAX` and `MAX + 1` bounds, UTF-8 `TextSpan`, `IssueId` monotonicity, `WritingGoals`, `PersonalDictionary` case-sensitivity and `CUSTOM.DIC` BOM/UTF-8 round-trips, `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog` `{{placeholder}}` expansion).
- Linguistic rule and cache tests live in `crates/rules/src/tests.rs` (EN/PT-BR/ES proofing, contextual confusables, regional variant mixes, clarity/passive/redundancy with language isolation, `ReadabilityMetrics`, `ToneProfile`, `StyleGuide` regex safety/matching, incremental cache hits/invalidation, document-scoped suppression).
- Engine contract tests live in `crates/engine/src/tests.rs` (undo/redo across dictionary/style/snippets, `.grammarcraft.json`/`.txt`/`.docx` round-trips and corrupted `.docx` error handling without panics, `CUSTOM.DIC` import/export under `AutomationRoots`, traversal rejection, and `hostile_parameters_never_panic` sweeping all 23 `registry().all()` commands).
- `apps/grammarcraft-cli/tests/cli.rs` drives the built `grammarcraft-cli` binary for `commands`, `run`, `check`, `style.*`, `snippet.*`, `dict.*`, exit codes `0`/`1`/`2`, and `mcp` rejection when either automation root flag is missing.
- `cargo xtask ci` runs `fmt --check`, `clippy -D warnings`, `cargo nextest run --workspace`, `layers`, and `assets`.
- `deny.toml` records the SPDX licence allowlist matching `storytold/craft-contrib` (`standards/licences.md`).
- GitHub Actions runs shared workflows from `storytold/craft-common-actions` (`gate.yml@main` and `house-rules.yml@main`).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-11-13 | major | Initial template baseline: layers, data flow, gates |
| 2026-11-27 | major | Replace `Note` example with GrammarCraft `Document`, `TextSpan`, `WritingGoals`, `TextStats`, and `doc.*`/`goals.set`/`stats.measure` commands |
| 2026-11-27 | major | Add L1 `grammarcraft-rules` (`craft-proof`, clarity/style/readability/tone), L2 bounded undo/redo, `craft-atomic` persistence, `AutomationRoots`, 14 `CommandSpec`s, and CLI `check` / `mcp` |
| 2026-11-27 | major | Add `deny.toml`, M2 `StyleGuide`, `SnippetCatalog`, `CUSTOM.DIC`, EN/PT-BR contextual confusables, regional variant checks, `regex = "1"` evaluator, and 9 `style.*`/`snippet.*`/`dict.*` commands (23 total) |
| 2026-11-27 | minor | Add `.docx` import/export via `craft-docx` and `craft-wordmodel`, Spanish (`es*`) contextual paronym rules, and clarity language isolation |
