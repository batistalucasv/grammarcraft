# Roadmap

Status: **M0 in progress — repository instantiated from `craft-template` (`42f4fb7`) with clean-room records initialized; `Cargo.toml` pins, `deny.toml`, and first PR gate checks pending.**

A milestone is done only when every checkbox is verified in a fresh clone (and on GitHub Actions where noted), never when code is merely drafted.

## M0 — Repository instantiation & clean-room foundation

- [x] Copy `craft-template/template/`, rename `myapp` → `grammarcraft` before first commit (`42f4fb7`), verify `cargo xtask ci` locally, and publish `batistalucasv/grammarcraft` `main`
- [x] Initialize `clean-room/` (`README.md`, `assumptions.md`, `plan.md`, `source-log.md`), `docs/feature-names.md`, `docs/target-app-parity.md`, `docs/gaps.md`, and `docs/roadmap.md`
- [ ] Pin single `storytold/egui` fork revision matching `craft-libs` and `rust-version = "1.95"` in root `Cargo.toml`
- [ ] Configure `deny.toml` enforcing `craft-contrib/standards/licences.md` allowlist (blocking GPL/LGPL/AGPL and NC datasets)
- [ ] Complete USPTO, EUIPO/TMview, and WIPO Rule 4 trademark checks in `docs/feature-names.md` before first UI PR / public release
- [ ] Confirm `gate.yml@main` and `house-rules.yml@main` pass on first PR to `main` with branch protection enabled

## M1 — Core domain, persistence, undo/redo & confined CLI/MCP

- [x] Replace `Note` example with initial `grammarcraft-core` (`Document`, UTF-8-safe `TextSpan`, `WritingGoals`, `TextStats`, `CoreError`, `MAX_DOCUMENT_BYTES`), `grammarcraft-engine` (`Session`, `doc.set_text`, `doc.get`, `doc.clear`, `goals.set`, `stats.measure`, `hostile_parameters_never_panic`), and `grammarcraft-cli` integration tests
- [ ] Extend `grammarcraft-core` (L0) with `Issue`, `IssueCategory`, `StyleGuide`, `Snippet`, `ReadabilityMetrics`, `WritingScore`, and `MAX_*` bounds (zero I/O / zero JSON / zero panics)
- [ ] Implement `grammarcraft-rules` (L1) base pipeline wrapping `craft-proof` and `craft-text` with per-paragraph incremental caching and per-document ignore/dictionary isolation
- [ ] Implement `grammarcraft-engine` (L2): `Session`, bounded undo/redo stack, versioned atomic save/open (`.grammarcraft.json`, `.txt`, `.md`, `.docx` via `craft-docx`), `AutomationRoots` file confinement
- [ ] Register initial `doc.*`, `history.*`, `review.*`, and `goals.*` `CommandSpec` catalog with `hostile_parameters_never_panic` passing 100%
- [ ] Ship `grammarcraft-cli` + `grammarcraft-cli mcp` (`craft-mcp-server`) refusing startup without `--automation-read-root` and `--automation-write-root`

## M2 — Deterministic writing & style engine

- [ ] Add contextual spelling rules (homophones/paronyms in EN/PT-BR/ES) and regional variant consistency alerts (`en-US`/`en-GB`, `pt-BR`/`pt-PT`)
- [ ] Expand deterministic grammar rules (agreement, crasis/government in PT-BR, punctuation mechanics, comma splices) without copyleft code or rules
- [ ] Implement clarity & style analyzers: passive voice (modulated by `WritingGoals`), long/complex sentences, redundancies/pleonasms, weak adverbs, lexical repetition window, syntactic rhythm
- [ ] Implement open readability indices (Flesch Reading Ease, Flesch-Kincaid, PT-BR/Gulpease, reading/speaking time) and transparent 0–100 `WritingScore`
- [ ] Implement `PersonalDictionary` (`CUSTOM.DIC` UTF-16LE + UTF-8), declarative `StyleRules` (linear-time `regex`), and `SnippetCatalog` (`{{placeholder}}` expansion)

## M3 — Native `egui` desktop app & accessibility

- [ ] Build `apps/grammarcraft` using `craft-ui-shell` (`AppShell`), `craft-ui-patterns`, `craft-ui-widgets`, `craft-host`, and `craft-theme` / `craft-theme-store`
- [ ] Render category underlines with redundant encoding (theme colour + distinct stroke pattern + severity icon) and interactive suggestion cards/popovers
- [ ] Build bidirectional Review Pane (Alerts, Document Goals & Summary, Tone, Style Rules/Snippets) dispatching strictly via `Session::run`
- [ ] Pass `craft-design-lint`, full keyboard navigation (`Tab`/`F6`/`Escape`, zero traps), AccessKit screen-reader tree verification, and `egui_kittest` snapshot suite
- [ ] Enable authenticated `craft-automation` control channel (`--control`, confined to automation roots when requested)

## M4 — Multi-signal tone, hybrid AI assistance & local similarity

- [ ] Implement deterministic multi-signal `ToneProfile` (up to 3 tones with explicit span evidence), formality checks, and opt-in inclusive language rules
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
| 2026-11-27 | M1 | Replaced `Note` template domain with `Document`, `TextSpan`, `WritingGoals`, `TextStats`, `CoreError`, `doc.*`/`goals.set`/`stats.measure` commands, and CLI tests (`cargo xtask ci` green) |
