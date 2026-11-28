# Clean-Room Verification Record

> **Gate command:** `cargo xtask ci` (`fmt --check`, `clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`, `layers`, `assets`).  
> **Policy:** Accuracy and false-positive checks verify clean-room behavioral expectations and anti-false-positive guards only—never matching proprietary numerical scores.

## 1. Deep grammar & clarity verification (`crates/rules/src/tests.rs`)

| Test function | What it verifies | Anti-false-positive guard |
| --- | --- | --- |
| `deep_grammar_detects_en_pt_es_issues_and_avoids_false_positives` | Fires `grammar.comma_splice`, `grammar.subject_verb_agreement`, `grammar.less_vs_fewer`, `clarity.nominalization` (EN & PT-BR), `grammar.pt_crase`, `grammar.pt_existential`, `grammar.es_dequeismo`, `grammar.es_existential` with valid byte spans and replacements | Asserts zero `grammar.*` false positives on clean EN (`however` after `;`, singular `is`, `fewer errors`), clean PT-BR (`à medida que`, `a partir de`, `dia a dia`, `houve`, `faz 5 anos`), and clean ES (`pienso que`, `hubo`), plus strict language isolation |
| `monotonous_rhythm_triggers_at_three_sentences_and_relaxes_for_technical_specialized` | Fires `clarity.monotonous_rhythm` on 3 consecutive 6-word sentences under default `WritingGoals` | Suppresses `clarity.monotonous_rhythm` when `WritingGoals` sets `Audience::Specialized`, `Context::Technical`, `Purpose::Inform` (`min_run = 4`) |
| `analysis_engine_integrates_upstream_ready_punctuation_and_spacing_rules` | Verifies `upstream_ready::check_text` delimiter, quote, bracket-spacing, and comma/semicolon-spacing checks | Guards URLs, numbers, and contractions against false positives |
| `formality_and_inclusive_language_respect_goals_and_avoid_false_positives` | Verifies `inclusive_formality.rs` formality-mismatch and opt-in `inclusive_language` checks (EN/PT-BR/ES) | Ensures opt-in isolation when `inclusive_language == false` and clean prose passes without warnings |

## 2. Engine, ONNX (`rten`), hotkey & CLI verification (`crates/engine/src/tests.rs`, `apps/grammarcraft-cli/tests/cli.rs`)

| Test function | What it verifies |
| --- | --- |
| `rten_text_model_runner_verifies_sha256_loads_onnx_and_executes_inference` | Builds a valid ONNX `Identity` micro-graph, registers `RtenTextModelRunner` with matching SHA-256, verifies `local-engine:rten/grammar-rten-micro` inference, and asserts tampered `.onnx` bytes are rejected by SHA-256 verification before `rten::Model::load_file` |
| `hotkey_binding_validates_and_mock_listener_triggers_overlay_once` | Validates `HotkeyBinding::parse` (rejecting missing modifiers/keys) and `run_hotkey_overlay_once` with `MockHotkeyListener` + `MockOsTextBridge` |
| `hotkey_overlay_validates_binding_and_triggers_overlay_with_mock_once` | Drives built `grammarcraft-cli hotkey-overlay --hotkey Ctrl+Shift+G --once --mock-trigger --mock-text ...` binary and rejects invalid bindings with exit code `2` |
| `hostile_parameters_never_panic` | Sweeps all **42** `CommandSpec` entries against malformed/oversized/missing JSON inputs without panicking |
