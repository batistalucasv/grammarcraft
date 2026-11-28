# Clean-Room Specification Freeze

> **Frozen baseline:** `feat/m6-5-hotkey-grammar-clarity-onnx` (`585f9cd`)  
> **Scope:** L0 (`grammarcraft-core`), L1 (`grammarcraft-rules`), L2 (`grammarcraft-engine`), and front ends (`apps/grammarcraft`, `apps/grammarcraft-cli`).

## Frozen clean-room invariants

1. **Observer / Implementer & licensing boundary:**
   - Zero `GPL`, `LGPL`, `AGPL`, or non-commercial (`NC`) code, lexicons, or model weights are linked or embedded (`deny.toml` enforces `standards/licences.md`).
   - All deterministic proofing, deep grammar (`grammar_deep.rs`), clarity/nominalization/rhythm (`clarity.rs`, `scoring.rs`), inclusive/formality (`inclusive_formality.rs`), tone (`tone.rs`), and similarity (`similarity.rs`) rules derive strictly from public linguistic references and ArtCraft specifications recorded in [`source-log.md`](source-log.md).
2. **No proprietary oracle calibration:**
   - `ReadabilityMetrics` (Flesch Reading Ease, Flesch-Kincaid Grade, Gulpease, reading/speaking minutes), `WritingScore` (0–100), and `ToneProfile` use open, documented formulas and span-anchored lexical/structural counts; none are tuned or calibrated against proprietary product outputs.
3. **Never-crash & layer boundaries:**
   - Workspace production crates forbid `unsafe` (`unsafe_code = "forbid"`) and deny `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/`unreachable!`.
   - `cargo xtask layers` enforces strict downward dependencies (`apps/*` → `engine` → `rules` → `core`), keeping `grammarcraft-core` and `grammarcraft-rules` `wasm32-unknown-unknown`-clean.
4. **Local-first AI, ONNX & OS overlay safety:**
   - `RtenTextModelRunner` (`rten` `0.28.0`) verifies streaming `sha2::Sha256` digests in `Session::register_local_runner` before loading `.onnx` weights.
   - `PlatformOsBridge` and `PlatformHotkeyListener` use bounded, `unsafe`-free platform probes/commands; `translate_with_cloud` and remote AI providers fail closed unless `ConsentLedger` grants explicit consent.
