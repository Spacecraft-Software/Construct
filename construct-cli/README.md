# construct

`construct` is the Spacecraft Software **Construct** skills package manager — a
dual-mode (human + agent-native) CLI/TUI for installing, discovering, syncing,
and shipping agent *skills* across the ~70 AI coding agents Construct knows
about. It is the first executable in the [Construct](https://github.com/Spacecraft-Software/Construct)
catalogue repository.

It is modeled on [`vercel-labs/skills`](https://github.com/vercel-labs/skills)
(an imperative installer driven by a broad agent registry) and adds a
Construct-catalogue *ship-loop* (commit + push local skill edits, then refresh
the flake input that NixOS consumes).

## Status

Built in phases; this is an early cut.

- **Phase 1 (done):** CLI skeleton conforming to the Spacecraft Software
  Dual-Mode Self-Documenting CLI Standard — global flags, output-mode cascade,
  `{ metadata, data }` JSON envelope, structured errors, canonical exit codes,
  `construct schema` / `construct describe`, and `construct skill sync` (the
  flake-update-only operation).
- **Phase 2–5 (planned):** imperative installer + agent registry, general git
  sources, the ship-loop, and the `--format explore` TUI.

## Usage

```sh
construct skill sync                 # nix flake update construct (in the bravais flake)
construct skill sync --json          # machine-readable envelope
construct skill sync --dry-run       # show the plan, change nothing
construct describe                   # capability manifest for agents
construct schema skill sync          # JSON Schema (Draft 2020-12) for one command
```

## Output modes

`construct` auto-detects its audience (CLI Standard §5): an explicit
`--format`/`--json` flag wins, then the agent environment (`AI_AGENT`, `AGENT`,
`CI`) forces JSON, then a TTY gets colored human output, and a pipe gets JSON.
Machine output is a single `{ "metadata": …, "data": … }` document with ISO 8601
UTC timestamps; errors are single-line `{ "error": … }` objects on stderr with a
runnable `hint`.

## Build, test, lint

```sh
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Or via Nix, from the repository root:

```sh
nix build .#construct
./result/bin/construct --version
```

## License

GPL-3.0-or-later. See the repository root `LICENSES/` and `REUSE.toml`.
Maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>.
