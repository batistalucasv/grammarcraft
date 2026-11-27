# Security policy

## Reporting a vulnerability

Please report security problems privately: open the repository's **Security** tab and choose
**Report a vulnerability** (GitHub private advisories). Do not open a public issue or pull request
for a vulnerability before a fix is released.

Useful to include:

- the version or commit, and the platform;
- what an attacker can do, and what they need (a crafted file, a JSON request, a local account…);
- steps or a sample file that reproduce it.

## Supported versions

Fixes go into `main` and ship in the next release. Older releases do not get backported fixes.

## Scope

In scope: anything that lets a document, a file, a command parameter or a network response read or
write files it should not, run code, leak data, or crash the process and lose the user's work.

Crashes from hostile input count, because this project treats every panic as a bug — see
[Never crash](AGENTS.md).