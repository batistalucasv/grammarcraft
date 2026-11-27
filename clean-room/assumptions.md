# Starting assumptions

> **Immutability rule:** Per `storytold/craft-libs` (`docs/clean-room.md`), these starting assumptions are written before deriving linguistic tables or consulting reference corpora, marked `[unverified]`, committed once, and never edited. Reviewers diff later derivation and verification records against this baseline.

## Architecture and ecosystem assumptions

- **A-01 `[unverified]`:** `craft-proof` (`storytold/craft-libs`) provides clean-room Rust spellchecking (`SCOWL`, `Moby`, Hunspell `.aff`/`.dic` reader without `unsafe`), dialect variants (`en-variants.txt`), `packs`, `user_dict`, and conservative base grammar checks for 11 locales, sufficient as the L1 lexical/base-grammar foundation once wrapped with per-document ignore/dictionary scoping.
- **A-02 `[unverified]`:** `craft-text` (`storytold/craft-libs`) provides UAX #14 / #29 segmentation, UAX #9 BiDi, linear-time `regex` search, and `hypher` hyphenation (excluding the `slovak-gpl` feature) suitable for syllable counting and sentence boundary detection across target languages.
- **A-03 `[unverified]`:** `craft-ai-integrations` (`craft-ai-client`, `craft-ai-openai`, `craft-ai-native`, `craft-ai-egui`) can drive cloud and OpenAI-compatible `text.complete` endpoints with explicit consent ledger and OS keyring storage, while local CPU inference for NLP/GEC/tone tasks can be added as `ModelRunner` implementations in `craft-ai-engine-rten` or `craft-ai-engine-candle`.

## Linguistic and algorithmic assumptions

- **A-04 `[unverified]`:** Contextual confusable disambiguation (homophones and paronyms in English, Portuguese, and Spanish) can achieve low false-positive rates using deterministic part-of-speech / local token-window rules and permissively licensed frequency/n-gram tables without requiring `LGPL` rule sets (`LanguageTool`) or proprietary heuristics.
- **A-05 `[unverified]`:** Passive voice, long/complex sentence detection, pleonasms/verbose phrases, lexical repetition in a sliding window, and syntactic rhythm checks can be implemented deterministically per language in pure Rust with goal-based (`WritingGoals`) severity modulation.
- **A-06 `[unverified]`:** Classic open readability formulas (Flesch Reading Ease, Flesch-Kincaid Grade, Portuguese Flesch adaptation, Gulpease) combined with hyphenation-based syllable estimation deliver consistent, explainable metrics without needing proprietary scoring curves.
- **A-07 `[unverified]`:** A transparent 0–100 `WritingScore` computed purely from open-issue severity density normalized by word/sentence count and `WritingGoals` alignment provides actionable writer feedback without calibrating against any proprietary product's numerical score.
- **A-08 `[unverified]`:** Multi-signal tone classification (combining modal lexicons, hedges, imperatives, punctuation patterns, and optional permissive local ONNX classifiers) can produce up to 3 explainable tone signals anchored to exact text spans.
- **A-09 `[unverified]`:** Local document similarity checks over user-authorized directories can operate deterministically using N-gram shingling plus MinHash/SimHash with exact span verification, keeping network requests strictly opt-in.

## Platform and surface assumptions

- **A-10 `[unverified]`:** A standalone native desktop editor (`egui` via `craft-ui-shell`), `grammarcraft-cli` with stdio `mcp` (`craft-mcp-server`), and upstreaming generic rules/widgets to `craft-libs` cover core writing workflows under the Rust-only rule without handwritten JavaScript/TypeScript or Electron/webview shells.
- **A-11 `[unverified]`:** System-wide OS overlay assistance (post-M5) can be implemented in pure Rust via isolated platform accessibility adapters (Windows UIA, macOS `AXUIElement`, Linux `AT-SPI2`) without introducing `unsafe` code into L0–L2.
