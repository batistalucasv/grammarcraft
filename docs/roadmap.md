# Roadmap

Status: **M0–M5 complete, M6-3 real-time live translation merged on `main`, and M6-4 pure-Rust OS overlay bridge (`os_bridge.rs`: `OsTextBridge`, `PlatformOsBridge` multi-OS zero-`unsafe`, `MockOsTextBridge`; `overlay.status`/`overlay.capture`/`overlay.apply` — 42 commands total; `--overlay` anchored UI with `AlwaysOnTop`, `ViewportCommand::OuterPosition`, `Capture Selection`, `Replace in App`, and `translate.live`) implemented on `feat/m6-4-os-overlay`; RFC-gated browser extension, `storytold/craft-libs#3975` → `wordcraft`, optional future direct native crate adapters without subprocesses + global hotkey daemon, dedicated neural ONNX translation/NLP weights, `packaging/macos/`, and Rule 4 searches remain open.**

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

- [x] Replace `Note` example with `grammarcraft-core` (L0): `Document`, UTF-8-safe `TextSpan`, `WritingGoals`, `LiveTranslationConfig`, `TranslationResult`, `TextStats`, `IssueId`, `IssueCategory`, `Severity`, `Confidence`, `Replacement`, `Issue`, `IssueList`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary`, `IgnoredIssueSet`, `CoreError`, and `MAX_*` bounds (zero I/O / zero `serde_json` / zero panics)
- [x] Implement `grammarcraft-rules` (L1) base pipeline wrapping `craft-proof` with per-paragraph incremental caching, per-document ignore/dictionary isolation, initial clarity/style rules, open readability metrics, `WritingScore`, and span-anchored `ToneProfile`
- [x] Implement `grammarcraft-engine` (L2): `Session`, bounded undo/redo stack (`MAX_UNDO_STEPS = 200`), versioned atomic save/open (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel`, `.txt`, `.md` via `craft-atomic`), and `AutomationRoots` file confinement (`file_access.rs`)
- [x] Register `CommandSpec` catalog (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*` incl. `review.apply_all`, `dict.*`, `style.*`, `snippet.*`, `ai.*`, `similarity.check`, `citations.*`, `translate.*`, `overlay.*` — 42 commands) with `hostile_parameters_never_panic` passing 100%
- [x] Ship `grammarcraft-cli` (`check`, `quick-review`, `stats`, `run`, `commands`, `-V`/`--version`, `native-messaging`) + `grammarcraft-cli mcp` refusing startup without `--automation-read-root` and `--automation-write-root`
- [ ] Integrate `craft-text` UAX #14/#29 segmentation

## M2 — Deterministic writing & style engine

- [x] Add EN, PT-BR, and ES (`es*`) contextual spelling rules (homophones/paronyms) and regional variant consistency alerts (`en-US`/`en-GB`, `pt-BR`/`pt-PT`)
- [x] Implement `PersonalDictionary` (`case_sensitive` + `CUSTOM.DIC` UTF-16LE BOM and UTF-8), declarative `StyleGuide` (`PreferredTerm`, `BrandCasing`, linear-time `PatternRule` with `regex = "1"`), and `SnippetCatalog` (`{{placeholder}}` expansion) across L0/L1/L2 + `style.*`/`snippet.*`/`dict.*` commands
- [x] Implement initial clarity & style analyzers (EN/PT-BR passive voice modulated by `WritingGoals`, long sentences, common redundancies, sliding-window lexical repetition), open readability indices (Flesch, Flesch-Kincaid, Gulpease, reading/speaking times), and transparent 0–100 `WritingScore`
- [x] Add `Document`-independent `crates/rules/src/upstream_ready.rs` delimiter, straight-quote, bracket-spacing, and comma/semicolon-spacing rules with false-positive guards (EN/PT-BR/ES) and verify `grammarcraft-core` + `grammarcraft-rules` compile cleanly for `wasm32-unknown-unknown`
- [ ] Expand deterministic grammar rules (agreement, crasis/government in PT-BR, punctuation mechanics, comma splices) and vague-verb/syntactic-rhythm analyzers without copyleft code or rules
- [ ] Configure `clean-room/derivation/`, `freeze.md`, and `verification.md` corpus accuracy checks plus `SARIF` CLI output

## M3 — Native `egui` desktop app & accessibility

- [x] Build `apps/grammarcraft` using `craft` (`ui`, `ui-app`, `theme-store`, `dialogs`, `automation`; `craft::ui::shell::app::{AppSpec, run}`) with light/dark themes (`gamma_multiply`), native file dialogs, floating `--quick-review` mode (`460x540`, `AlwaysOnTop`, `--text`, `--file`, `--from-clipboard`, `Apply All`, `Apply All & Copy`, `Paste & Review`, `Copy Text`, `Escape`), anchored `--overlay` mode (`Capture Selection`, `Replace in App`), and `Live Translate` (`Off | EN | PT-BR | ES`)
- [x] Migrate toolbar, sidebar, and theme bridge controls to `craft::ui::display::Divider`, `craft::ui::choice::Checkbox`, and `craft::ui::text::TextField` (`Bind::bound`)
- [x] Render category underlines with redundant encoding (theme colour + distinct stroke pattern + `[!]`/`[~]`/`[i]` severity markers) and interactive suggestion cards
- [x] Build bidirectional Review Pane (`Alerts`, `Goals & Score`, `Tone Profile`, `Style & Snippets`, `AI Assist`, `Citations & Similarity`) routing every mutation strictly via `Session::run`
- [x] Verify keyboard navigation (`Tab`/`Shift+Tab`/`F6`/`Escape`), active AccessKit tree, and 12 headless `egui_kittest` / control tests
- [x] Enable authenticated `craft/automation` loopback `--control` channel (`ControlOptions` + `serve` + `ControlChannel` with `--automation-read-root`/`--automation-write-root` confinement and rejection/dispatch tests)
- [ ] Replace remaining `egui::ComboBox` / `TextEdit::multiline` instances where supported and pass `craft-design-lint` 100%

## M4 — Multi-signal tone, hybrid AI assistance & local similarity

- [x] Expand `ToneProfile` and `inclusive_formality.rs` with formality-mismatch checks (`Formality::Formal`) and opt-in inclusive language rules (`WritingGoals::inclusive_language` in EN/PT-BR/ES)
- [x] Integrate `craft-ai-core`, `craft-ai-openai` (`OpenAiProvider`), `craft-ai-client` (`AiClient`), and `craft-ai-native` (`ConsentFile`, `HistoryFile`, `SecretStore`) with `LocalOfflineBackend` zero-network default and fail-closed consent/history/secret persistence
- [x] Integrate `EngineRegistry`/`ModelRunner` (`craft-ai-engine` via `craft-ai-client`) with streaming `sha2::Sha256` weight verification before `load()`, automatic local precedence, and `local_engines` in `ai.status`
- [x] Ship `ai.status`, `ai.consent`, `ai.rewrite`, `ai.adjustTone`, `ai.summarize`, `ai.expand`, `ai.draft`, `ai.applyResult`, `ai.discardResult` with word-level `AiDiffHunk` preview, `AI Assist` UI tab, and atomic single-step undo
- [x] Ship local N-gram/Jaccard `Similarity Check` (`similarity.check` confined by `AutomationRoots`) and 12-style citation validation/formatting (`BibTeX` parser + `citations.validate`/`citations.format` + `Citations & Similarity` UI tab)
- [ ] Wire a concrete `craft-ai-engine-rten` / `craft-ai-engine-candle` NLP/tone/GEC/neural-translation `ModelRunner` with permissive open weights downloaded on demand under SHA-256 verification

## M5 — Ecosystem integration, localization & release packaging

- [x] Author English string catalog (`locales/en/grammarcraft.tsv`), original app icon + 11 derived hicolor/ICO/1024px assets (`assets/app-icon/` attributed in `ATTRIBUTION.md`), Linux/Flatpak/Windows packaging scripts (`packaging/{env.sh,check-icons.sh,linux/,windows/}`), and `.github/workflows/release.yml`
- [x] Confirm green `release.yml` `dry-run=true` on `main` across Linux x86_64/aarch64, Windows x64/arm64, and Flatpak x86_64/aarch64 (run `46092799920`)
- [x] Ship WebExtension stdio `grammarcraft-cli native-messaging` host (4-byte LE `u32` framing, `MAX_NATIVE_MESSAGE_BYTES = 1 MiB`, safe allowlist incl. `translate.*`) as the pure-Rust backend for browser integration
- [ ] Add `packaging/macos/` assets, coordinate translations in `storytold/craft-i18n`, and register `grammarcraft` in `craft-ai-agents` (`KNOWN_APPS`)

## M6+ — OS context menu & overlay, full WordCraft integration, browser extension & live translation

- [x] **OS Context-Menu Hooks & Quick Review CLI:** `grammarcraft-cli quick-review` (`--text`, `--file`, `--stdin`, `--lang`, `--audience`, `--formality`, `--apply-all`), desktop `--quick-review --file <PATH>`, `Apply All & Copy`, `Escape`, Linux XDG/Dolphin/Nautilus hooks (`packaging/linux/ai.storyteller.grammarcraft.quick-review.desktop`, `install-quick-review.sh`), and per-user Windows Explorer context-menu registration (`packaging/windows/register-context-menu.ps1`)
- [x] **Live Input Translation Core & Desktop UI (`translate.live`):** `LiveTranslationConfig` / `TranslationResult` in `core`, `LocalOfflineTranslator` (`pt`/`en`/`es`) + `translate_with_cloud` under `ConsentLedger` with atomic undo (`translate.configure`, `translate.live`, `translate.status`), `native-messaging` allowlist, and `Live Translate` (`Off | EN | PT-BR | ES`) selector in main toolbar and `--quick-review` triggering on sentence completion with immediate `Undo`
- [x] **Direct OS Overlay Base in Pure Rust (`os_bridge.rs`, `overlay.*`, `--overlay`):** `OsTextBridge`, `PlatformOsBridge` (Linux `AT-SPI2`/clipboard, Windows `UIA`/clipboard, macOS `AX`/clipboard, zero `unsafe`), `MockOsTextBridge`, `overlay.status`/`overlay.capture`/`overlay.apply` (42 commands total), and `--overlay` anchored floating UI (`AlwaysOnTop`, `ViewportCommand::OuterPosition`, `Capture Selection`, `Replace in App`) combined with `translate.live`
- [ ] **Global Hotkey Daemon & Optional Direct Native Crate Adapters:** global hotkey listener and optional direct in-process `UIA` (Windows), `AXUIElement` (macOS), and `AT-SPI2` (Linux) adapters in a dedicated crate without subprocesses (`unsafe` isolated outside L0–L2, zero JS/Electron/webview) if needed in the future
- [ ] **Full `WordCraft` Integration:** merge `storytold/craft-libs#3975` (`craft-proof` upstreaming), bump `craft-proof` in `wordcraft`, and integrate clarity, contextual paronyms, tone, readability, and live translation natively in `wordcraft`
- [ ] **Browser Extension (WASM + Native Messaging):** open RFC in `storytold/craft-contrib` (Rule 11) for a minimal browser extension shim consuming `grammarcraft-cli native-messaging` and the `wasm32` `grammarcraft-core`/`grammarcraft-rules` kernel

## Deliberately not planned (without dedicated RFC)

- **Handwritten JS/TS browser extension or Electron/webview shell without an approved `storytold/craft-contrib` RFC:** governed by Rust-only Rule 11.
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
| 2026-11-27 | M2/M3/M4 | PR #10 merged (`b002e5e`): `EngineRegistry`/`ModelRunner` with `sha2` SHA-256 verification and `local_engines` status, `crates/rules/src/upstream_ready.rs` (`check_text`), and `craft::ui` (`Divider`, `Checkbox`, `TextField`, `gamma_multiply`) migration; PR #11 merged (`449c02d`): `workflow_dispatch` `dry-run` input in `release.yml` |
| 2026-11-27 | M5 | PR #12 merged (`0af3dbf`): `-V`/`--version` in `grammarcraft-cli`, 12 app icons in `assets/app-icon/` (`ATTRIBUTION.md`), `packaging/env.sh`, `check-icons.sh`, Linux/Flatpak/Windows packaging scripts; PR #13 merged (`38c5d29`) and `release.yml` `dry-run=true` passed on `main` (run `46092799920`) |
| 2026-11-27 | M3/M5/M6 prep | PR #14 merged (`9bcc4f3`): `review.apply_all` (36 commands), floating `--quick-review` mode, `grammarcraft-cli native-messaging`, `wasm32-unknown-unknown` verification of `core`/`rules`, and M6+ roadmap |
| 2026-11-27 | M6-2 | PR #15 merged (`b44878d`): `grammarcraft-cli quick-review`, desktop `--quick-review --file`, `Apply All & Copy`, `Escape`, and Linux/Windows OS context-menu hooks |
| 2026-11-28 | M6-3 | PR #16 merged (`b8f76fe`): real-time live translation (`LiveTranslationConfig`, `TranslationResult`, `LocalOfflineTranslator` `pt`/`en`/`es`, `translate_with_cloud` under `ConsentLedger`, `translate.configure`/`live`/`status` — 39 commands, `native-messaging` allowlist, and `Live Translate` UI selector) |
| 2026-11-28 | M6-4 | Added pure-Rust direct OS overlay bridge (`os_bridge.rs`: `OsTextBridge`, `PlatformOsBridge`, `MockOsTextBridge`), `overlay.status`/`capture`/`apply` (42 commands), and anchored `--overlay` UI (`AlwaysOnTop`, `ViewportCommand::OuterPosition`, `Capture Selection`, `Replace in App`, combined with `translate.live`; `cargo xtask ci` green) |
