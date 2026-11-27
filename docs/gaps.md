# Gaps

Ranked work the template does **not** do yet. Each row is an extension point with a known seam, so
a new project can pick what it needs instead of growing organically.

| # | Gap | Where the seam already is | What "done" looks like |
|---|---|---|---|
| 1 | Nothing is persisted | `Document` is in memory; `Session::new()` starts empty | `io` module in L1: save/load with a format you can version, plus a round-trip test |
| 2 | No undo | `Session::run` is the single mutation point | Snapshot or command log around `run`, with a test that undo restores the exact document |
| 3 | A command list is not a UI | `Registry::describe` already emits JSON | A front end that dispatches by id and never touches `Document` directly |
| 4 | No structured logging | errors are returned, not recorded | `log` + a subscriber in the binary; no `println!` in libraries |
| 5 | No localisation | strings are inline | a catalog keyed by command id; the `label` field becomes a key |
| 6 | No packaging | `cargo build --release` only | a release workflow calling `craft-common-actions`, and a version source of truth |
| 7 | No benchmarks | tests prove behaviour, not cost | a criterion or divan bench for the hot path once there is one |

Do not start a gap before the one it depends on: undo is cheap once every mutation goes through
`run`, and expensive before.

When a gap closes, delete its row and record the change in `docs/architecture.md`'s revision
history. A gap list that only grows stops being read.