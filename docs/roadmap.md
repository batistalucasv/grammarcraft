# Roadmap

Status: **Linux/Flatpak/Windows packaging scripts, 12 hicolor/ICO/1024px icons, and `grammarcraft-cli -V/--version` implemented on `feat/packaging-release-dryrun` (PR #12 pending merge); `release.yml` `dry-run` verification on `main`, `packaging/macos/`, concrete `rten` ONNX NLP runner, external upstream PRs (`craft-proof`/`craft-i18n`/`craft-ai-agents`), full `craft-design-lint` clearance, Rule 4 searches, and post-M5 OS overlay remain open.**

A milestone is done only when every checkbox is verified in a fresh clone (and on GitHub Actions where noted), never when code is merely drafted.

## M0 — Repository instantiation & clean-room foundation

- [x] Copy `craft-template/template/`, rename `myapp` → `grammarcraft` before first commit (`42f4fb7`), verify `cargo xtask ci` locally, and publish `batistalucasv/grammarcraft` `main`
- [x] Initialize `clean-room/` (`README.md`, `assumptions.md`, `plan.md`, `source-log.md`), `docs/feature-names.md`, `docs/target-app-parity.md`, `docs/gaps.md`, and `docs/roadmap.md`
- [x] Set `rust-version = "1.95"` in root `Cargo.toml` and migrate `AGENTS.md` to the `craft-agents` block (`RFC-0072`)
- [x] Configure `deny.toml` enforcing `craft-contrib/standards/licences.md` allowlist (blocking GPL/LGPL/AGPL and NC datasets; `cargo-deny` binary not installed locally for offline execution)
- [x] Pin single `storytold/egui` (`932a73b28560acb1a695533307b2cd0d66a42081`) and `storytold/winit` (`a517b3846ee8457d84ee4975110bfda6797c44dc`) revisions matching `craft-libs` (`8be80b006888ae023d61eed265241c9b78b84981`) in root `Cargo.toml`
- [ ] Complete USPTO, EUIPO/TMview, and WIPO Rule 4 trademark checks in `docs/feature-names.md` before first public release
- [ ] Confirm `gate.yml@main` and `house-rules.yml@main` pass together on PR with branch protection enabled on `main`

## M1 — Core domain, persistence, undo/redo & confined CLI/MCP

- [x] Replace `Note` example with `grammarcraft-core` (L0): `Document`, UTF-8-safe `TextSpan`, `WritingGoals`, `TextStats`, `IssueId`, `IssueCategory`, `Severity`, `Confidence`, `Replacement`, `Issue`, `IssueList`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary`, `IgnoredIssueSet`, `CoreError`, and `MAX_*` bounds (zero I/O / zero `serde_json` / zero panics)
- [x] Implement `grammarcraft-rules` (L1) base pipeline wrapping `craft-proof` with per-paragraph incremental caching, per-document ignore/dictionary isolation, initial clarity/style rules, open readability metrics, `WritingScore`, and span-anchored `ToneProfile`
- [x] Implement `grammarcraft-engine` (L2): `Session`, bounded undo/redo stack (`MAX_UNDO_STEPS = 200`), versioned atomic save/open (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel`, `.txt`, `.md` via `craft-atomic`), and `AutomationRoots` file confinement (`file_access.rs`)
- [x] Register initial `CommandSpec` catalog (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`) with `hostile_parameters_never_panic` passing 100%
- [x] Ship `grammarcraft-cli` (`check`, `stats`, `run`, `commands`, `-V`/`--version`) + `grammarcraft-cli mcp` refusing startup without `--automation-read-root` and `--automation-write-root`
- [ ] Integrate `craft-text` UAX #14/#29 segmentation

## M2 — Deterministic writing & style engine

- [x] Add EN, PT-BR, and ES (`es*`) contextual spelling rules (homophones/paronyms) and regional variant consistency alerts (`en-US`/`en-GB`, `pt-BR`/`pt-PT`)
- [x] Implement `PersonalDictionary` (`case_sensitive` + `CUSTOM.DIC` UTF-16LE BOM and UTF-8), declarative `StyleGuide` (`PreferredTerm`, `BrandCasing`, linear-time `PatternRule` with `regex = "1"`), and `SnippetCatalog` (`{{placeholder}}` expansion) across L0/L1/L2 + 9 `style.*`/`snippet.*`/`dict.*` commands (23 total)
- [x] Implement initial clarity & style analyzers (EN/PT-BR passive voice modulated by `WritingGoals`, long sentences, common redundancies, sliding-window lexical repetition), open readability indices (Flesch, Flesch-Kincaid, Gulpease, reading/speaking times), and transparent 0–100 `WritingScore`
- [x] Add `Document`-independent `crates/rules/src/upstream_ready.rs` delimiter, straight-quote, bracket-spacing, and comma/semicolon-spacing rules with false-positive guards (EN/PT-BR/ES)
- [ ] Expand deterministic grammar rules (agreement, crasis/government in PT-BR, punctuation mechanics, comma splices) and vague-verb/syntactic-rhythm analyzers without copyleft code or rules
- [ ] Configure `clean-room/derivation/`, `freeze.md`, and `verification.md` corpus accuracy checks plus `SARIF` CLI output

## M3 — Native `egui` desktop app & accessibility

- [x] Build `apps/grammarcraft` using `craft` (`ui`, `ui-app`, `theme-store`, `dialogs`, `automation`; `craft::ui::shell::app::{AppSpec, run}`) with light/dark themes (`gamma_multiply`) and native file dialogs
- [x] Migrate toolbar, sidebar, and theme bridge controls to `craft::ui::display::Divider`, `craft::ui::choice::Checkbox`, and `craft::ui::text::TextField` (`Bind::bound`)
- [x] Render category underlines with redundant encoding (theme colour + distinct stroke pattern + `[!]`/`[~]`/`[i]` severity markers) and interactive suggestion cards
- [x] Build bidirectional Review Pane (`Alerts`, `Goals & Score`, `Tone Profile`, `Style & Snippets`, `AI Assist`, `Citations & Similarity`) routing every mutation strictly via `Session::run`
- [x] Verify keyboard navigation (`Tab`/`Shift+Tab`/`F6`/`Escape`), active AccessKit tree, and 8 headless `egui_kittest` / control tests
- [x] Enable authenticated `craft/automation` loopback `--control` channel (`ControlOptions` + `serve` + `ControlChannel` with `--automation-read-root`/`--automation-write-root` confinement and rejection/dispatch tests)
- [ ] Replace remaining `egui::ComboBox` / `TextEdit::multiline` instances where supported and pass `craft-design-lint` 100%

## M4 — Multi-signal tone, hybrid AI assistance & local similarity

- [x] Expand `ToneProfile` and `inclusive_formality.rs` with formality-mismatch checks (`Formality::Formal`) and opt-in inclusive language rules (`WritingGoals::inclusive_language` in EN/PT-BR/ES)
- [x] Integrate `craft-ai-core`, `craft-ai-openai` (`OpenAiProvider`), `craft-ai-client` (`AiClient`), and `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`) with `LocalOfflineBackend` zero-network default and fail-closed consent/history/secret persistence
- [x] Integrate `EngineRegistry`/`ModelRunner` (`craft-ai-engine` via `craft-ai-client`) with streaming `sha2::Sha256` weight verification before `load()`, automatic local precedence, and `local_engines` in `ai.status`
- [x] Ship `ai.status`, `ai.consent`, `ai.rewrite`, `ai.adjustTone`, `ai.summarize`, `ai.expand`, `ai.draft`, `ai.applyResult`, `ai.discardResult` (35 total commands) with word-level `AiDiffHunk` preview, `AI Assist` UI tab, and atomic single-step undo
- [x] Ship local N-gram/Jaccard `Similarity Check` (`similarity.check` confined by `AutomationRoots`) and 12-style citation validation/formatting (`BibTeX` parser + `citations.validate`/`citations.format` + `Citations & Similarity` UI tab)
- [ ] Wire a concrete `craft-ai-engine-rten` / `craft-ai-engine-candle` NLP/tone/GEC `ModelRunner` with permissive open weights downloaded on demand under SHA-256 verification

## M5 — Ecosystem integration, localization & release packaging

- [x] Author English string catalog (`locales/en/grammarcraft.tsv`), original app icon + 11 derived hicolor/ICO/1024px assets (`assets/app-icon/` attributed in `ATTRIBUTION.md`), Linux/Flatpak/Windows packaging scripts (`packaging/{env.sh,check-icons.sh,linux/,windows/}`), and `.github/workflows/release.yml` calling `storytold/craft-common-actions/.github/workflows/release.yml@main` (with `workflow_dispatch` `dry-run` input)
- [ ] Upstream `crates/rules/src/upstream_ready.rs` and reusable suggestion-card widgets to `craft-proof` / `craft-libs` (`wordcraft` integration PR)
- [ ] Register `grammarcraft` in `craft-ai-agents` (`KNOWN_APPS`) for `craft-ai-mcp` launcher discovery
- [ ] Coordinate translations in `storytold/craft-i18n` from `locales/en/grammarcraft.tsv`
- [ ] Add `packaging/macos/` assets and confirm green `dry-run` on `release.yml` on `main`

## Deliberately not planned (M0–M5)

- **Handwritten JS/TS browser extension or Electron/webview shell:** violates Rust-only Rule 11; any future browser extension requires a dedicated RFC in `storytold/craft-contrib` after M5.
- **Proprietary web-wide scraping for plagiarism:** violates privacy, licensing, and clean-room boundaries; external web similarity is only considered later via opt-in `SimilarityProvider` connectors with explicit user credentials and consent.
- **Calibrating scores against proprietary product outputs:** blocked by `standards/clean-room.md` (§ *Measurements*).

## Progress log

| Date | Milestone | What changed |
| --- | --- | --- |
| 2026-11-27 | M0 | Instantiated from `craft-template` (`42f4fb7`), pushed `main` to `batistalucasv/grammarcraft`, and initialized `clean-room/` records, `docs/feature-names.md`, `docs/target-app-parity.md`, `docs/gaps.md`, and `docs/roadmap.md` |
| 2026-11-27 | M0/M1 | PR #1 merged (`65136ca`): `Document`, `TextSpan`, `WritingGoals`, `TextStats`, `CoreError`, 5 commands, CLI tests, and `AGENTS.md` `craft-agents` migration (`RFC-0072`) |
| 2026-11-27 | M1 | PR #2 merged (`78766d4`): L0 review/metrics/glossary types, L1 `grammarcraft-rules` (`craft-proof`, incremental cache, clarity/readability/`WritingScore`/`ToneProfile`), L2 undo/redo, `craft-atomic` persistence, `AutomationRoots`, 14 `CommandSpec`s, and CLI `check` / `mcp` |
| 2026-11-27 | M2 | PR #3 merged (`d932c0e`): `deny.toml` SPDX allowlist, L0 `StyleGuide`/`SnippetCatalog`/`CUSTOM.DIC` UTF-16LE+UTF-8, L1 EN/PT-BR contextual confusables + regional variants + linear-time `regex` evaluator, and 9 `style.*`/`snippet.*`/`dict.*` commands (23 total) |
| 2026-11-27 | M1/M2 | PR #4 merged: `.docx` read/write via `craft-docx` + `craft-wordmodel` (`doc.open`, `doc.save`, `check`), Spanish (`es*`) contextual paronym rules, and clarity language isolation |
| 2026-11-27 | M3 | PR #5 merged (`ec52b11`, Linux UI packages in `ci.yml`); PR #6 merged (`8166126`): native `egui` desktop app (`apps/grammarcraft`), `storytold/egui` + `storytold/winit` pins, 5 `egui_kittest` + AccessKit tests |
| 2026-11-27 | M4 | PR #7 merged (`6df24cd`): `craft-ai-core`/`craft-ai-openai`/`craft-ai-client`, `inclusive_formality.rs`, `similarity.rs`, 12 citation styles + BibTeX, 12 new commands (35 total), and `AI Assist` / `Citations & Similarity` desktop tabs (7 UI tests) |
| 2026-11-27 | M5 | PR #8 merged (`a10f1f6`): `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`), loopback `--control`, `locales/en/grammarcraft.tsv`, `assets/app-icon/grammarcraft.svg`, `packaging/linux/*`; PR #9 merged (`a377174`): `.github/workflows/release.yml` |
| 2026-11-27 | M2/M3/M4 | PR #10 merged (`b002e5e`): `EngineRegistry`/`ModelRunner` with `sha2` SHA-256 verification and `local_engines` status, `crates/rules/src/upstream_ready.rs` (`check_text`), and `craft::ui` (`Divider`, `Checkbox`, `TextField`, `gamma_multiply`); PR #11 merged (`449c02d`): `workflow_dispatch` `dry-run` input in `release.yml` |
| 2026-11-27 | M5 | Added `-V`/`--version` to `grammarcraft-cli`, 11 derived hicolor/ICO/1024px icons in `assets/app-icon/` (`ATTRIBUTION.md` 12/12), `packaging/env.sh`, `check-icons.sh`, and Linux/Flatpak/Windows packaging scripts (`cargo xtask ci` green) |
