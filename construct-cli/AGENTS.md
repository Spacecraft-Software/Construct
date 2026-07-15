# construct — AI Coding-Agent Context

`construct` is the Spacecraft Software **Construct** skills package manager (Rust
CLI + TUI) — the first executable in the Construct catalogue repository. It
conforms to the Spacecraft Software Dual-Mode Self-Documenting CLI Standard
(v1.0.0). This file and `CLAUDE.md` are peers; keep them identical.

## Build / test / lint

```sh
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo audit
# Via Nix, from the repository root:
nix build .#construct && ./result/bin/construct --version
```

## Architecture

- `main.rs` is thin: parse → build `Context` → `commands::dispatch` → render.
- `cli.rs` — the clap derive tree and the §3 global flags (`global = true`).
- `context.rs` — per-invocation resolved state (output mode, color, flags).
- `src/output/` is the **only** place that writes to stdout:
  - `mode.rs` — the §5 detection cascade + §6 color precedence.
  - `envelope.rs` — the `{ metadata, data }` JSON envelope.
  - `error.rs` — the structured `AppError` (machine: single-line `{"error":…}`).
  - `render.rs` — json / jsonl / yaml / csv / human renderers; `--fields`.
  - `theme.rs` — the Steelbore six-token palette (no inline hex).
- `src/commands/` — one handler per command.
- `manifest.rs` — the single source of truth for `schema` and `describe`; the
  `tests::manifest_in_sync_with_cli` test fails if it drifts from the clap tree.

## Invariants (do not break)

- Printing to stdout happens ONLY in `src/output/`. No `println!` elsewhere.
- Data commands return `CommandOutput`; `main` renders it. Handlers never call
  `std::process::exit` — `main` owns the exit code.
- All timestamps go through `time::now_iso8601()` → ISO 8601 UTC with `Z`. Never
  local time, never `chrono::Local` / `NaiveDateTime`.
- Errors are `AppError` whose `hint` is a RUNNABLE command, not prose.
- Exit codes follow the canonical map (0,1,2,3,4,5,127,…).
- Every `.rs` / `.toml` starts with the two-line SPDX header; license is
  `GPL-3.0-or-later`.

## Forbidden

- `println!` / `eprintln!` outside `src/output/` (and the no-subcommand help path).
- `chrono::Local`, naive/offset timestamps in any output.
- Hand-rolled argument parsing — use clap.
- Adding a command without a matching `manifest.rs` entry (the sync test fails).

## Adding a command

1. Add the clap sub-command in `cli.rs`.
2. Add a handler under `src/commands/`.
3. Add a `CommandSpec` in `manifest.rs`.
4. Wire it in `commands::dispatch`.
5. Add black-box tests in `tests/cli.rs`.

Maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org> ·
Project: https://Construct.SpacecraftSoftware.org/
