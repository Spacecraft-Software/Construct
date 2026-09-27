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

Personal / Hobby posture (Standard §5). Implemented today — `construct --help`
and `construct describe` are the authority:

- **CLI skeleton** conforming to the Spacecraft Software Dual-Mode
  Self-Documenting CLI Standard — global flags, output-mode cascade,
  `{ metadata, data }` JSON envelope, structured errors, canonical exit codes,
  `construct schema` / `construct describe`.
- **Imperative installer** — `skill add | list | remove | update | find | use`
  across the agent registry (`construct agent list`), from a local path, git
  URL, or `owner/repo` source; `skill init` scaffolds a new skill.
- **Flake loop** — `skill sync` (flake-update only), `skill status` /
  `skill reset` (live skill tree against the flake pin).
- **Ship loop** — `skill ship`: gates, branch, signed commit, push, and a pull
  request; never pushes to the default branch, never merges.
- **Distribution** — `skill build` and `skill vendor` (below).
- **`--format explore` TUI** — also what a bare invocation on a terminal opens.

## Usage

```sh
construct skill sync                 # nix flake update construct (in the bravais flake)
construct skill sync --json          # machine-readable envelope
construct skill sync --dry-run       # show the plan, change nothing
construct skill build                # per-platform bundles under <repo>/dist/ (claude, grok, perplexity, gemini, single-file)
construct skill build --target claude,grok --dry-run   # gate + encode in memory, write nothing
construct skill vendor spacecraft-rust-guidelines --into ../repo   # .claude/skills/ for Claude Code cloud sessions; never commits
construct describe                   # capability manifest for agents
construct schema skill sync          # JSON Schema (Draft 2020-12) for one command
```

## Distribution targets

`construct skill build` writes release bundles to `<repo>/dist/<target>/`
(gitignored; `--out` moves it). Zips are deterministic — fixed mtimes, sorted
entries — so a rebuild of an unchanged tree is byte-identical. The source
`SKILL.md` is never edited; frontmatter is rewritten per target.

| Target | Platform / purpose |
|--------|--------------------|
| `claude` | Claude Code (local and web), claude.ai, Gemini CLI, Codex — nested `<name>/` layout, `maintainer` / `website` under `metadata:`, keeps `user-invocable`. |
| `grok` | Grok — flat zips (`SKILL.md` at the root) for every root skill plus the Grok-native skills; frontmatter `name` + `description` only. |
| `perplexity` | Perplexity — Claude layout without `user-invocable`; any skill over 100 files consolidated from `perplexity-skills/categories.toml`. The committed `perplexity-skills/spacecraft-cli-preference.zip` is a copy of this output. |
| `gemini` | The Gemini app — flat `<name>.zip` only (files only) for every root skill; frontmatter `name` + `description` only; members restricted to `.csv` / `.py` / `.txt` / `.md`, any other file shipped with `.txt` appended (`LICENSE.txt`, `assets/steelbore.toml.txt`) and in-skill links to it rewritten; consolidated like `perplexity`; refused over 100 files. |
| `single-file` | MiniMax and other platforms with no skill loader — one self-contained `<name>.md`, references inlined, in-skill links rewritten to anchors, and every `LICENSE` / `LICENSE.<TAG>` appended verbatim under a closing `## License` section (links to them anchor there); outside fenced blocks, links that leave the skill become plain text (a sibling skill is named, upstream paths keep their text). |

The build vendors the palette into `spacecraft-brand-guidelines` and
`spacecraft-accessibility-support`, and refuses any skill over the §5.6
description cap or the 500-character `compatibility` cap, writing nothing.

Claude Code **cloud sessions** load only a repository's committed
`.claude/skills/`. `construct skill vendor <skill>... [--into <repo>]
[--dry-run] [--force]` writes the `claude` tree there, never runs git in that
repository, and prints what to commit.

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
