# AGENTS.md — construct

`construct` is the Spacecraft Software **Construct** skills package manager (Rust
CLI + TUI) — the first executable in the Construct catalogue repository. It
conforms to the Spacecraft Software Dual-Mode Self-Documenting CLI Standard
(v1.1.0). This is the authoritative agent context file (Standard §5.7); `CLAUDE.md`
imports it and adds only Claude-Code-only notes.

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

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` are gated in CI by the **`cargo`** job in
`.github/workflows/ci.yml`, and a separate **`msrv`** job runs `cargo check`
against the `rust-version` declared in `Cargo.toml` (1.88). The gate runs on
stable, so a clippy or rustfmt change can turn a green PR red — fix the finding
rather than pinning the toolchain. `cargo audit` is **not** in CI: a new
advisory would redden `main` for a reason unrelated to the change under review,
so run it locally before adding a dependency (Standard §3.3).

## Architecture

- `main.rs` is thin: parse → build `Context` → `commands::dispatch` → render.
- `cli.rs` — the clap derive tree and the §3 global flags (`global = true`).
- `context.rs` — per-invocation resolved state (output mode, color, flags).
- `src/output/` is the **only** place that writes to stdout:
  - `mode.rs` — the §5 detection cascade + §6 color precedence
    (FORCE_COLOR overrides NO_COLOR).
  - `envelope.rs` — the `{ metadata, data }` JSON envelope.
  - `error.rs` — the structured `AppError` (machine: single-line `{"error":…}`;
    human: `[ERROR]`-tagged line + indented `hint:`). Never suppressible.
  - `diagnostic.rs` — non-error diagnostics (`Severity` ladder `[OK]`/`[WARN]`/
    `[INFO]`, machine: single-line `{"diagnostic":…}`, human: `[TAG]` line),
    gated by `Context::severity_floor` (`--quiet` → errors only, agent env →
    `warn`+, default → `ok`+, `--verbose` → `info`+). See the CLI Standard's
    `references/diagnostics.md`.
  - `render.rs` — json / jsonl / yaml / csv / human renderers; `--fields`.
  - `progress.rs` — progress bars and static step lines for long operations, **stderr
    only**, enabled only in human mode with a TTY stderr, no `--quiet`, and
    `TERM` ≠ `dumb` (resolved once into `Context::progress`); otherwise every
    handle is a no-op and not a byte changes. `--accessible` /
    `SPACECRAFT_A11Y` (flag wins; `Context::accessible`) swaps the animation
    for append-only `Working: <label>… 40%` lines, ≤ 1 per second, then
    `… done`. Every stderr writer (`Diagnostic::emit`, `emit_passthrough`,
    `AppError`) goes through `progress::write_stderr`, which erases a live line
    first — never write stderr around it.
  - `theme.rs` — the `steelbore` theme: the eleven Steelbore 2 role tokens of
    Standard §11.1 (no inline hex).
- `src/commands/` — one handler per command.
- `catalogue.rs` — root-skill discovery and `EXCLUDED_DIRS`, the one exclusion
  set for `skill build` and `skill ship` (pinned by a unit test to
  `.github/check-skill-frontmatter.py`'s `NOT_ROOT_SKILLS` and `flake.nix`'s
  `excludedDirs`). `install::plan::NON_SKILL_DIRS` is different on purpose: it
  serves arbitrary third-party sources.
- `gate.rs` — the Standard §5.6 frontmatter gate (description ≤ 1000,
  compatibility ≤ 500, strict YAML) shared by `ship`, `build`, and `vendor`. The counting
  rule stays `sources::skillmd::field_len`; `gate` owns the policy and the
  refusal shape.
- `src/bundle/` — the **pure** pipeline behind `skill build`: `collect` →
  `palette` → `frontmatter` (project + round-trip verify) → `consolidate`
  (Perplexity: a faithful port of the retired `perplexity-skills/build.py`,
  now the only generator of the committed
  `perplexity-skills/spacecraft-cli-preference.zip`, driven by
  `perplexity-skills/categories.toml`, reused for Gemini) / `gemini` (the
  Gemini app's allowed-extension renames + link rewrite + file-count gate) /
  `single` (single-file render) →
  `sink` (deterministic zips, atomic writes). `bundle::vendor_plan` runs the
  same gates and `claude` projection for `skill vendor`, and `tree` is its
  directory sink (ownership marker `.construct-vendor.toml`, stage → re-read →
  swap, symlinks never followed). It never sees a `Context`, never prints, and
  returns typed problems; `commands/build.rs` and `commands/vendor.rs` alone
  map them to exit codes.
  The five targets (`claude`, `grok`, `perplexity`, `gemini`, `single-file`), what each
  platform is for, and the `dist/<target>/` layout are described under
  *Distribution targets* in the repository-root `AGENTS.md`.
- `manifest.rs` — the single source of truth for `schema` and `describe`; the
  `tests::manifest_in_sync_with_cli` test fails if it drifts from the clap tree.

## Invariants (do not break)

- Printing to stdout happens ONLY in `src/output/`. No `println!` elsewhere.
- Data commands return `CommandOutput`; `main` renders it. Handlers never call
  `std::process::exit` — `main` owns the exit code.
- All timestamps go through `time::now_iso8601()` → ISO 8601 UTC with `Z`. Never
  local time, never `chrono::Local` / `NaiveDateTime`.
- Errors are `AppError` whose `hint` is a RUNNABLE command, not prose.
- Progress never reaches stdout and is silent off a human TTY. A countable
  in-process loop uses the animated `output::progress::Progress::bar`, never a
  bare write. Every subprocess wait uses `progress::{step, step_command}`:
  one static line before and a `done`/`failed` line after, no animation —
  git, `gh`, and nix can all prompt on the terminal (credentials,
  `accept-flake-config`), and a redraw would bury the prompt. `bundle::plan` reports its `(skill, target)` steps through a
  callback, so the pipeline stays free of `Context` and output.
- Every non-error stderr message goes through `output::diagnostic::Diagnostic`
  (or `emit_passthrough` for raw subprocess output) so the severity floor and
  `[TAG]` rendering apply — no bare `eprintln!` diagnostics.
- Exit codes follow the canonical map (0,1,2,3,4,5,127,…).
- Every `.rs` / `.toml` starts with the two-line SPDX header; license is
  `GPL-3.0-or-later`.
- Build artifacts carry no timestamps: every zip entry uses the fixed
  `sink::BUNDLE_EPOCH`, entries are byte-sorted, and deflate runs at a fixed
  level, so identical inputs and an identical `Cargo.lock` give identical
  bytes. `skill build` gates every selected skill before the first byte is
  written (all-or-nothing) and replaces only directories carrying its
  `.construct-build` marker unless `--force`.
- Bundle and vendored file modes are host-independent: `0755` exactly when
  the catalogue's git index records `100755` (one read-only
  `git ls-files -s -z` per run), else `0644` — never the working-tree exec
  bit, which `core.fileMode = false` makes unreliable.
- A partial `skill build` (named skills) never writes the `.construct-build`
  marker into a foreign directory, even under `--force`; only a directory it
  created, an empty one, or a full `--force` replacement is adopted.
- `skill vendor` never runs git in the consumer repository (its one git call
  is that read-only `ls-files` on the source catalogue) and never writes
  through a symlink (not even under `--force`); skill names are validated as
  Agent Skills ids before any path is built from them. Tests always pass `--into <tempdir>` — a defaulted
  `--into` from `construct-cli/` resolves to the Construct work tree itself.

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
