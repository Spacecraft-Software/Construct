# Contributing to `construct` (the CLI)

This covers the **`construct` binary** under `construct-cli/`. For the skill
*catalogue* workflow (editing skills, rebuilding `.zip`/`.skill` bundles), see
the repository-root [`CONTRIBUTING.md`](../CONTRIBUTING.md).

Construct is a personal hobby project (Spacecraft Software Standard §5, Personal
posture): external input is welcome but discretionary, with no warranty or SLA.

## Development workflow

```sh
cd construct-cli
cargo build
cargo test
cargo clippy --all-targets -- -D warnings   # must be clean (no -D-warnings escapes)
cargo fmt                                    # before committing
cargo audit                                  # no vulnerabilities
```

All four gates (clippy, fmt, test, audit) must pass before a change lands. The
binary also builds reproducibly through Nix: `nix build .#construct` from the
repo root.

## House rules

- **SPDX on every source file.** `.rs` and `.toml` start with the two-line
  header (`SPDX-FileCopyrightText` + `SPDX-License-Identifier: GPL-3.0-or-later`).
  Markdown is covered by the repo `REUSE.toml` default. `reuse lint` must pass.
- **The CLI Standard is law.** Read `construct-cli/CLAUDE.md` (= `AGENTS.md`) for
  the invariants: stdout writes only in `src/output/`, ISO 8601 UTC timestamps,
  structured errors with runnable hints, the canonical exit-code map, and the
  `manifest.rs`-as-single-source-of-truth rule for `schema`/`describe`.
- **Keep `CLAUDE.md` and `AGENTS.md` identical.** They are peers.
- **Dates are ISO 8601 UTC** (`YYYY-MM-DD`), no local time.

## Commits

Commits to a Spacecraft Software remote must be cryptographically signed and show
"Verified" on GitHub (Standard §6.3; Ed25519 SSH signing). Assistant-driven
commits add a `Co-Authored-By:` trailer. Stage explicitly by name — never
`git add -A` at the repo root (other root bundles may carry unrelated changes;
see the root CONTRIBUTING).

Maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
