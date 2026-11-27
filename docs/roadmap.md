# Roadmap

Status: **M1 `.docx` support (`craft-docx` / `craft-wordmodel`) and M2 Spanish (`es*`) contextual paronym rules added on `feat/docx-and-es-contextual` (PR #4 pending merge); Rule 4 trademark searches, corpus verification, desktop UI (M3), and hybrid AI (M4) remain open.**

A milestone is done only when every checkbox is verified in a fresh clone (and on GitHub Actions where noted), never when code is merely drafted.

## M0 — Repository instantiation & clean-room foundation

- [x] Copy `craft-template/template/`, rename `myapp` → `grammarcraft` before first commit (`42f4fb7`), verify `cargo xtask ci` locally, and publish `batistalucasv/grammarcraft` `main`
- [x] Initialize `clean-room/` (`README.md`, `assumptions.md`, `plan.md`, `source-log.md`), `docs/feature-names.md`, `docs/target-app-parity.md`, `docs/gaps.md`, and `docs/roadmap.md`
- [x] Set `rust-version = "1.95"` in root `Cargo.toml` and migrate `AGENTS.md` to the `craft-agents` block (`RFC-0072`)
- [x] Configure `deny.toml` enforcing `craft-contrib/standards/licences.md` allowlist (blocking GPL/LGPL/AGPL and NC datasets; `cargo-deny` binary not installed locally for offline execution)
- [ ] Pin single `storytold/egui` fork revision matching `craft-libs` when introducing desktop UI (`M3`)
- [ ] Complete USPTO, EUIPO/TMview, and WIPO Rule 4 trademark checks in `docs/feature-names.md` before first UI PR / public release
- [ ] Confirm `gate.yml@main` and `house-rules.yml@main` pass together on PR with branch protection enabled on `main`

## M1 — Core domain, persistence, undo/redo & confined CLI/MCP

- [x] Replace `Note` example with `grammarcraft-core` (L0): `Document`, UTF-8-safe `TextSpan`, `WritingGoals`, `TextStats`, `IssueId`, `IssueCategory`, `Severity`, `Confidence`, `Replacement`, `Issue`, `IssueList`, `ReadabilityMetrics`, `WritingScore`, `ToneProfile`, `PersonalDictionary`, `IgnoredIssueSet`, `CoreError`, and `MAX_*` bounds (zero I/O / zero `serde_json` / zero panics)
- [x] Implement `grammarcraft-rules` (L1) base pipeline wrapping `craft-proof` with per-paragraph incremental caching, per-document ignore/dictionary isolation, initial clarity/style rules, open readability metrics, `WritingScore`, and span-anchored `ToneProfile`
- [x] Implement `grammarcraft-engine` (L2): `Session`, bounded undo/redo stack (`MAX_UNDO_STEPS = 200`), versioned atomic save/open (`.grammarcraft.json` v1, `.docx` via `craft-docx` + `craft-wordmodel`, `.txt`, `.md` via `craft-atomic`), and `AutomationRoots` file confinement (`file_access.rs`)
- [x] Register initial `CommandSpec` catalog (`doc.*`, `history.*`, `goals.set`, `stats.measure`, `review.*`, `dict.*`) with `hostile_parameters_never_panic` passing 100%
- [x] Ship `grammarcraft-cli` (`check`, `stats`, `run`, `commands`) + `grammarcraft-cli mcp` refusing startup without `--automation-read-root` and `--automation-write-root`
- [ ] Integrate `craft-text` UAX #14/#29 segmentation

## M2 — Deterministic writing & style engine

- [x] Add EN, PT-BR, and ES (`es*`) contextual spelling rules (homophones/paronyms) and regional variant consistency alerts (`en-US`/`en-GB`, `pt-BR`/`pt-PT`)
- [x] Implement `PersonalDictionary` (`case_sensitive` + `CUSTOM.DIC` UTF-16LE BOM and UTF-8), declarative `StyleGuide` (`PreferredTerm`, `BrandCasing`, linear-time `PatternRule` with `regex = "1"`), and `SnippetCatalog` (`{{placeholder}}` expansion) across L0/L1/L2 + 9 `style.*`/`snippet.*`/`dict.*` commands (23 total)
- [x] Implement initial clarity & style analyzers (EN/PT-BR passive voice modulated by `WritingGoals`, long sentences, common redundancies, sliding-window lexical repetition), open readability indices (Flesch, Flesch-Kincaid, Gulpease, reading/speaking times), and transparent 0–100 `WritingScore`
- [ ] Expand deterministic grammar rules (agreement, crasis/government in PT-BR, punctuation mechanics, comma splices) and vague-verb/syntactic-rhythm analyzers without copyleft code or rules
- [ ] Configure `clean-room/derivation/`, `freeze.md`, and `verification.md` corpus accuracy checks plus `SARIF` CLI output

## M3 — Native `egui` desktop app & accessibility

- [ ] Build `apps/grammarcraft` using `craft-ui-shell` (`AppShell`), `craft-ui-patterns`, `craft-ui-widgets`, `craft-host`, and `craft-theme` / `craft-theme-store`
- [ ] Render category underlines with redundant encoding (theme colour + distinct stroke pattern + severity icon) and interactive suggestion cards/popovers
- [ ] Build bidirectional Review Pane (Alerts, Document Goals & Summary, Tone, Style Rules/Snippets) dispatching strictly via `Session::run`
- [ ] Pass `craft-design-lint`, full keyboard navigation (`Tab`/`F6`/`Escape`, zero traps), AccessKit screen-reader tree verification, and `egui_kittest` snapshot suite
- [ ] Enable authenticated `craft-automation` control channel (`--control`, confined to automation roots when requested)

## M4 — Multi-signal tone, hybrid AI assistance & local similarity

- [ ] Expand `ToneProfile` with formality-mismatch checks, treatment/convention consistency, and opt-in inclusive language rules
- [ ] Integrate `craft-ai-client`, `craft-ai-native` (`SecretStore`, `ConsentFile`, `HistoryFile`), `craft-ai-openai` (`text.complete`), and `craft-ai-egui` with explicit `Local (Offline)` vs `Cloud` status and mandatory remote consent
- [ ] Add local NLP/tone/GEC `ModelRunner` integration path in `craft-ai-engine-rten` / `craft-ai-engine-candle` (weights downloaded on demand with SHA-256 verification)
- [ ] Ship `ai.rewrite`, `ai.adjustTone`, `ai.summarize`, `ai.expand`, `ai.draft` with word-level diff preview and atomic single-step undo via `ai.applyResult`
- [ ] Ship local N-gram/MinHash `Similarity Check` over authorized local corpora and 12-style citation validation/formatting (`CSL`/`BibTeX`)

## M5 — Ecosystem integration, localization & release packaging

- [ ] Upstream generic contextual/clarity rules and reusable suggestion-card widgets to `craft-proof` / `craft-libs` (`wordcraft` integration PR)
- [ ] Register `grammarcraft` in `craft-ai-agents` (`KNOWN_APPS`) for `craft-ai-mcp` launcher discovery
- [ ] Author English catalog and coordinate `storytold/craft-i18n` translations (`grammarcraft.tsv`)
- [ ] Add `packaging/` assets (`ATTRIBUTION.md` verified by `cargo xtask assets`) and wire `storytold/craft-common-actions/.github/workflows/release.yml@main` with green `dry-run`

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
| 2026-11-27 | M1/M2 | Added `.docx` read/write via `craft-docx` + `craft-wordmodel` (`doc.open`, `doc.save`, `check`), Spanish (`es*`) contextual paronym rules, and clarity language isolation (`cargo xtask ci` green) |
