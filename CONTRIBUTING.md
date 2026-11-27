# Contributing

## Before you write code

1. Read [`AGENTS.md`](AGENTS.md) — it applies to people as much as to agents.
2. Check whether the change is a bug, a feature or a refactor, and open an issue for anything that
   is not a small fix. Say what you observed, what you expected and where.
3. For a bug: reproduce it first, then add a test that fails without the fix.

## The gates

```bash
cargo xtask ci
```

That runs, in order, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, the workspace tests, `cargo xtask layers` and `cargo xtask assets`. It stops at the first
failure, which is the one to fix.

Report what you actually ran. "CI will check it" is not a result; if a gate could not run locally,
say which and why.

## Changing a rule

The shared house rules at the top of `AGENTS.md` come from
[`storytold/craft-contrib`](https://github.com/storytold/craft-contrib). Do not edit the copy here:
propose the change there, and this repository picks it up with the next sync. Rules specific to this
project live below the second `craft-house-rules:end` marker.

## Commits and pull requests

- `main` changes only through a pull request.
- Keep a pull request to one thing; unrelated formatting makes the diff unreviewable.
- The description says what changed, why, and what you ran.
- No agent attribution in the pull request.

## Adding a dependency

Check its licence first. The suite allows `ISC`, `MIT`, `BSD-2-Clause`, `BSD-3-Clause`,
`Apache-2.0`, `OFL-1.1`, `Ubuntu-font-1.0`, `Bitstream-Vera`, `CC0-1.0`, `CC-BY-4.0`,
`CC-BY-SA-4.0` and `LicenseRef-Public-Domain`. A dependency outside that list is a decision for a
maintainer, not a detail of a feature pull request.

## Contributor credits

You are credited by your GitHub username automatically. If you want a real or display name in the
About window, add **your own** entry to [`contributors/people.toml`](contributors/people.toml) in
your pull request. Never add someone else's.