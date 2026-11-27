# Roadmap

Status: **M0 — template compiles and its gates pass.**

A milestone is done when its checks are true in a fresh clone, not when the code is written.

## M0 — the template itself

- [x] Workspace with `crates/*`, `apps/*` and `xtask`
- [x] Two layers with the rule enforced (`cargo xtask layers`)
- [x] Every action a `CommandSpec`, with a hostile-parameter test
- [x] Never-crash lints in every production crate root
- [x] `cargo xtask ci` green locally
- [x] CI calls the shared `gate.yml` and `house-rules.yml`
- [ ] First `main` push: confirm both workflows pass on the remote

## M1 — persistence

- [ ] A versioned file format and a round-trip test
- [ ] `file.save` / `file.open` commands, with paths as hostile input
- [ ] Recovery: a crash while writing must not destroy the previous file

## M2 — undo

- [ ] Every mutation goes through `Session::run` (already true — keep it that way)
- [ ] Undo/redo commands and a test that restores the exact previous document
- [ ] A bound on history size

## M3 — a second front end

- [ ] A UI that dispatches by command id and reads only `Session`'s public API
- [ ] The same command ids produce the same result from the CLI and the UI
- [ ] A headless screenshot or snapshot test

## Deliberately not planned

- Plugins, scripting or a network service. Each changes the threat model; add one only with a
  written decision and a place to enforce it.

## Progress log

| Date | Milestone | What changed |
|---|---|---|
| 2026-11-13 | M0 | Template created; local gate green |