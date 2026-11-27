# Clean-room governance for GrammarCraft

This directory holds the permanent clean-room record for `GrammarCraft`, following `standards/clean-room.md`, `standards/licences.md`, and `standards/naming.md` in `storytold/craft-contrib`, as well as the process model in `docs/clean-room.md` in `storytold/craft-libs`.

## Roles and boundaries

1. **Observer vs. Implementer separation:** An observer operating any commercial writing assistant uses only project-generated or contributor-owned inputs, captures only isolated window screenshots into the gitignored `plan/` folder, and writes behavioural specifications only. An implementer writes code strictly from behavioural specifications, public documentation, open standards, academic literature, and permissively licensed corpora—never operating the proprietary app or reading its installers, browser extensions, bundles, or internal strings.
2. **No numerical oracle calibration (*Measurements*):** Per `standards/clean-room.md` (§ *Observing a proprietary app*, item 3), numerical outputs of proprietary apps must not be measured to fit or tune our own parameters. `ReadabilityMetrics` and `WritingScore` use open, documented, deterministic formulas only.
3. **Prohibited sources:**
   - No `GPL`, `LGPL`, `AGPL`, or `CDDL` code, rules, or dictionaries linked, embedded, or copied (explicitly excluding `LanguageTool` and `libhunspell` code/rules). External copyleft tools may run only as separate processes for independent test fixtures or verification if ever needed, never shipped or linked.
   - No proprietary browser extensions (`.crx`, `.xpi`), desktop bundles (`.asar`), SDKs, UI copy, icons, fonts, or non-commercial (`NC`) datasets/model weights.
   - Only SPDX licences on the `storytold/craft-contrib` `standards/licences.md` allowlist (plus unmodified `MPL-2.0` third-party dependencies with required notices, and legacy `craft-i18n` linguistic datasets under their documented exceptions).

## Record structure

- [`assumptions.md`](assumptions.md): Initial assumptions marked `[unverified]`, committed once and never edited.
- [`plan.md`](plan.md): Allowed sources, corpus plan, derivation phases, freeze criteria, and open maintainer decisions.
- [`source-log.md`](source-log.md): Chronological log of public specifications, standards, and corpora consulted.
- `derivation/` *(future, M2+)*: Experiment logs, stdlib-only `python -I` scripts, and literature citations supporting linguistic tables.
- `freeze.md` *(future, M2+)*: Frozen derived data tables (JSON) and generated Rust bindings.
- `verification.md` *(future, M2+)*: Post-freeze accuracy and hostile-input verification against open corpora without post-freeze tuning.
- `plan/` *(gitignored)*: Local observation notes and isolated window captures; never committed, linked, or attached to issues or pull requests.

All third-party assets, dictionaries, corpora, and model weights used by the repository must be recorded in [`ATTRIBUTION.md`](../ATTRIBUTION.md) and [`NOTICE`](../NOTICE) in the pull request that introduces them.
