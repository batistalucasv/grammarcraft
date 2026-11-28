# GrammarCraft architecture

> **Last reviewed:** 2026-11-28 · **Change:** add M6-3 real-time live translation (`LiveTranslationConfig`/`TranslationResult` in `core`, `translate.rs` with `LocalOfflineTranslator` + `translate_with_cloud` under `ConsentLedger`, `translate.configure`/`translate.live`/`translate.status` bringing total to 39 commands, `native-messaging` allowlist, and `Live Translate` UI selector in toolbar and `--quick-review`).

How the workspace is built today. The rules for working in it are in [`AGENTS.md`](../AGENTS.md); this file describes the shape, the layer boundaries, and why they exist.

## Layers

`Cargo.toml` declares `crates/*`, `apps/*`, and `xtask` as members. `cargo xtask layers` reads `cargo metadata` and fails if a crate depends on a layer at or above its own (including `[dev-dependencies]`), or if a workspace crate was never added to `TABLE` in `xtask/src/layers.rs`.

Because `xtask/src/layers.rs` forbids lateral dependencies (`target >= layer`), the deterministic linguistic pipeline (`grammarcraft-rules`) sits at **L1** below `grammarcraft-engine` (**L2**):

| Layer | Crate | Job |
|---|---|---|
| L0 | [`core`](../crates/core/src) | `Document` (incl. `#[serde(default)] live_translation: LiveTranslationConfig`), UTF-8-safe `TextSpan`, `WritingGoals` (`inclusive_language`), `LiveTranslationConfig`, `TranslationResult`, `validate_lang_tag` (`MAX_LANG_TAG_BYTES = 16`), `TextStats`, `Issue*` (`IssueCategory::parse`), `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary` (`CUSTOM.DIC`), `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, `SimilarityMatch`/`SimilarityReport`, 12 `CitationStyle`s + BibTeX, `AiMode`/`AiConsentStatus`/`AiProposal` + `compute_word_diff`, `CoreError` (incl. `InvalidLangTag`), and `MAX_*` bounds; zero I/O, zero `serde_json`, zero UI; verified clean for `wasm32-unknown-unknown` |
| L1 | [`rules`](../crates/rules/src) | `AnalysisEngine` with per-paragraph cache, `craft-proof` adapter, EN/PT-BR/ES contextual confusables and inclusive/formality checks (`inclusive_formality.rs`), `upstream_ready` (`check_text(&str, ProofLang) -> Vec<GenericFinding>` decoupled from `Document` for future `craft-proof` upstreaming), regional variant checks, `StyleGuide` linear-time `regex` evaluation, deterministic clarity/style rules, open `ReadabilityMetrics` & `WritingScore`, span-anchored `ToneProfile`, and deterministic N-gram/Jaccard `check_local_similarity`; verified clean for `wasm32-unknown-unknown` (`std::fs`/`net`/`process`/`env` absent) |
| L2 | [`engine`](../crates/engine/src) | `Session` (`last_report()`, `register_local_runner`, `local_engines`, `loaded_local_model_info`, `unload_local_model`), `MAX_DOCUMENT_BYTES` re-export, bounded undo/redo (`MAX_UNDO_STEPS = 200`), `file_access::AutomationRoots` confinement, `io` (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel`, `.txt`/`.md` via `craft_atomic::atomic_write`, confined `CUSTOM.DIC` import/export), `ai.rs` (`EngineRegistry`/`ModelRunner` + `LocalOfflineBackend` + `CraftProviderBackend` + `translate_with_cloud` gated by `ConsentLedger`/`Disclosure` and persisted via `craft-ai-native`), `translate.rs` (`LocalOfflineTranslator` `pt`/`en`/`es` preserving capitalization and trailing punctuation, `last_segment_span`, `translate_live` with atomic `push_undo()`), and `Registry` of **39** `CommandSpec`s (incl. `review.apply_all`, `translate.configure`, `translate.live`, `translate.status`) |
| app | [`grammarcraft`](../apps/grammarcraft/src) | Native `egui`/`eframe` desktop app (`craft` `ui`, `ui-app`, `theme-store`, `dialogs`, `automation`): GUI Windows subsystem, `--version`, full editor + floating `--quick-review` mode (`460x540`, `AlwaysOnTop`, `--text`, `--file <PATH>`, `--from-clipboard`, `Apply All`, `Apply All & Copy`, `Paste & Review`, `Copy Text`, `Escape`), `Live Translate` selector (`Off \| EN \| PT-BR \| ES`) on main toolbar and Quick Review triggering on sentence completion (`.`, `!`, `?`, `\n`) with anti-loop `last_live_translated` marker and immediate `Undo`, light/dark themes (`gamma_multiply`), `craft::ui` controls, redundant category underlines, bidirectional 6-tab Review Pane, keyboard navigation, AccessKit integration, and `control.rs` authenticated loopback `--control` server; routes all mutations strictly via `Session::run` |
| app | [`grammarcraft-cli`](../apps/grammarcraft-cli/src) | Thin CLI (`commands`, `run`, `check`, `quick-review`, `stats`, `-V`/`--version`), JSON output, exit codes (`0`, `1`, `2`), `mcp` JSON-RPC stdio server requiring `--automation-read-root` + `--automation-write-root`, and `native-messaging` stdio host (`native_messaging.rs`: 4-byte LE `u32` framing, `MAX_NATIVE_MESSAGE_BYTES = 1_048_576`, `try_reserve_exact`, safe command allowlist including `translate.*` via `Session::run`) |
| tooling | [`xtask`](../xtask/src) | `ci`, `layers`, `assets` |

Dependency direction:
- `grammarcraft-cli` → `grammarcraft-engine` (L2) → `grammarcraft-rules` (L1) → `grammarcraft-core` (L0).
- `apps/grammarcraft` (`Class::Exempt`) links `grammarcraft-engine`, `grammarcraft-rules` (for read-only `AnalysisReport`), and `grammarcraft-core` (for read-only domain types), while routing **every** state mutation exclusively through `Session::run(id, params)` and reading cached state via `Session` accessors.

Workspace pins in root `Cargo.toml`:
- `craft` (`ui`, `ui-app`, `theme-store`, `dialogs`, `automation`), `craft-proof`, `craft-atomic`, `craft-docx`, `craft-wordmodel`: `storytold/craft-libs` rev `8be80b006888ae023d61eed265241c9b78b84981`.
- `craft-ai-core`, `craft-ai-openai`, `craft-ai-client`, `craft-ai-native`: `storytold/craft-ai-integrations` rev `a822ad5b4db2c0dcd7a43bb62d6d75251fd3f8b5` (kept separate from `craft/ai-core` and `craft-ai-egui`).
- `sha2` `0.10` (`MIT OR Apache-2.0`, `default-features = false`).
- `[patch.crates-io]` for `egui` `0.37` family: `storytold/egui` rev `932a73b28560acb1a695533307b2cd0d66a42081`.
- `[patch.crates-io]` for `winit`: `storytold/winit` rev `a517b3846ee8457d84ee4975110bfda6797c44dc`.

## Data flow

1. **A call arrives** as a command id plus JSON parameters from the desktop UI (`apps/grammarcraft` full editor or `--quick-review` popover), the authenticated loopback `--control` socket (`ControlOptions` / `serve` / `ControlChannel`), the CLI (`run`, `check`, `quick-review`, `stats`), JSON-RPC `tools/call` on `grammarcraft-cli mcp`, or a 4-byte LE framed request on `grammarcraft-cli native-messaging` (restricted to `doc.get`, `doc.set_text`, `goals.set`, `review.apply`, `review.apply_all`, `review.check`, `review.dismiss`, `stats.measure`, `translate.configure`, `translate.live`, `translate.status`).
2. **`Session::run`** looks the id up in `Registry`, calls `spec.enabled(self)` (returning `CmdError::Disabled` when unavailable), and invokes `spec.run(self, params)`.
3. **The command** validates parameters using `p::*` helpers, returning `CmdError::Params` naming any missing, invalid, or mistyped key.
4. **Confinement, domain, analysis, AI & live translation:**
   - Disk reads/writes (`doc.open`, `doc.save`, `dict.import`, `dict.export`, `similarity.check`, `--quick-review --file`) pass through `file_access` (`AutomationRoots`) and bounded reads; `native-messaging` blocks filesystem commands altogether.
   - Mutating commands (including `review.apply_all` and `translate.live` with `apply: true`) push a bounded `HistorySnapshot` onto `undo_stack` before mutating.
   - `ai.*` commands check for a registered local `ModelRunner` in `EngineRegistry` first (verifying streaming `Sha256` digest before `load()`), falling back to `LocalOfflineBackend` or `CraftProviderBackend` only with explicit `ai.consent` approval backed by `craft-ai-native`.
   - `translate.live` translates the explicit segment/span or the last completed sentence (`last_segment_span`) via `LocalOfflineTranslator` (`pt`/`en`/`es`, zero-network default) or `AiSessionState::translate_with_cloud` when remote consent is granted.
   - `review.check` runs `AnalysisEngine::analyze` (including `upstream_ready::check_paragraph`) and caches the resulting `AnalysisReport` on `Session::last_report()`.
5. **`From<CoreError> for CmdError`** maps invariant violations to `CmdError::Params`.
6. **The front end** renders `AnalysisReport`, `AiProposal` diffs, live-translated sentences, or bibliography/similarity output (desktop UI), returns control replies (`ControlRequest`/`Reply`), prints JSON or corrected text with exit code `0`/`1`/`2` (CLI), or returns JSON-RPC / 4-byte LE framed responses (MCP / Native Messaging).

Every step returns `Result`; production crates forbid `unsafe` (`unsafe_code = "forbid"`) and deny `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, and `unreachable!`.

## Testing, packaging, and gates

- Domain invariant tests in `crates/core/src/tests.rs` (`MAX` / `MAX + 1` bounds, UTF-8 `TextSpan`, `IssueId`, `WritingGoals`, `LiveTranslationConfig`/`validate_lang_tag`/`MAX_LANG_TAG_BYTES`, `PersonalDictionary` + `CUSTOM.DIC`, `IgnoredIssueSet`, `StyleGuide`, `SnippetCatalog`, BibTeX parsing + 12 citation styles, word-diff computation).
- Linguistic rule and similarity tests in `crates/rules/src/tests.rs` (EN/PT-BR/ES proofing, contextual confusables, `upstream_ready` delimiter/quote/spacing checks and false-positive guards, regional variants, clarity/passive/redundancy, `inclusive_formality`, `check_local_similarity`, `ReadabilityMetrics`, `ToneProfile`, `StyleGuide` regex safety, incremental cache hits/invalidation, document-scoped suppression).
- Engine contract tests in `crates/engine/src/tests.rs` (undo/redo incl. `review_apply_all_fixes_non_overlapping_issues_atomically_with_undo`, `live_translation_offline_translates_and_applies_with_atomic_undo` incl. v1 `Document` retro-deserialization, `live_translation_remote_provider_requires_consent_and_uses_mock_provider`, `.grammarcraft.json`/`.txt`/`.docx` round-trips, `CUSTOM.DIC` and `similarity.check` under `AutomationRoots`, cloud AI consent blocking + `MockProvider`, `ai_native_consent_history_and_memory_secrets_persist_in_isolated_dir`, `local_model_runner_verifies_sha256_and_takes_precedence_over_fallback_and_cloud`, and `hostile_parameters_never_panic` sweeping all **39** `registry().all()` commands).
- Desktop UI & control tests in `apps/grammarcraft/src/tests.rs` (10 headless `egui_kittest` + AccessKit tests including `live_translate_translates_completed_sentence_and_supports_immediate_undo`, `quick_review_mode_parses_flags_renders_cards_and_applies_all_via_session_run`, and `quick_review_apply_all_and_copy_file_arg_and_escape_without_focus_trap`, plus `control_server_rejects_bad_token_and_dispatches_session_commands`, 11 UI/control tests total).
- `apps/grammarcraft-cli/tests/cli.rs` drives the built `grammarcraft-cli` binary for `commands`, `run`, `check`, `quick-review`, `style.*`, `snippet.*`, `dict.*`, exit codes `0`/`1`/`2`, `mcp` root enforcement, and `native-messaging` (4-byte LE framing, `> MAX_NATIVE_MESSAGE_BYTES` rejection, allowlist including `translate.*`, clean EOF).
- `cargo check -p grammarcraft-core -p grammarcraft-rules --target wasm32-unknown-unknown` verifies `wasm32` compilation of L0/L1.
- `cargo xtask ci` runs `fmt --check`, `clippy -D warnings`, `cargo nextest run --workspace`, `layers`, and `assets`.
- Packaging checks and OS context-menu hooks live in `packaging/`; `release.yml` (`dry-run=true`) passes across Linux x86_64/aarch64, Windows x64/arm64, and Flatpak x86_64/aarch64.
- `deny.toml` records the SPDX licence allowlist matching `storytold/craft-contrib` (`standards/licences.md`).
- GitHub Actions runs shared workflows from `storytold/craft-common-actions` (`gate.yml@main`, `house-rules.yml@main`, and `release.yml@main`).

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
| 2026-11-27 | major | Add M5 `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`), loopback `--control` (`craft/automation` + root confinement), `locales/en/grammarcraft.tsv`, `assets/app-icon/grammarcraft.svg`, and `packaging/linux/` |
| 2026-11-27 | major | Add `.github/workflows/release.yml`, `EngineRegistry`/`ModelRunner` with `sha2` SHA-256 weight verification, `upstream_ready.rs` (`check_text`), and `craft::ui` widget migration (`Divider`, `Checkbox`, `TextField`, `gamma_multiply`) |
| 2026-11-27 | major | Add `-V`/`--version` to `grammarcraft-cli`, 12 hicolor/ICO/1024px app icons in `assets/app-icon/`, `packaging/env.sh`, `check-icons.sh`, Linux/Flatpak/Windows packaging scripts, and green `release.yml` `dry-run=true` |
| 2026-11-27 | major | Add `review.apply_all` (36 commands), floating `--quick-review` desktop mode (`460x540`, `AlwaysOnTop`, `--text`, `--from-clipboard`), `grammarcraft-cli native-messaging` WebExtension host, and `wasm32-unknown-unknown` verification of `core`/`rules` |
| 2026-11-27 | major | Add `grammarcraft-cli quick-review` (`--text`, `--file`, `--stdin`, goals, `--apply-all`), desktop `--quick-review --file`, `Apply All & Copy`, `Escape`, and Linux/Windows OS context-menu hooks |
| 2026-11-28 | major | Add M6-3 real-time live translation (`LiveTranslationConfig`, `TranslationResult`, `LocalOfflineTranslator`, `translate_with_cloud` under `ConsentLedger`, `translate.configure`/`live`/`status` — 39 commands, `native-messaging` allowlist, and `Live Translate` UI selector) |
