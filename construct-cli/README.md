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
construct skill build                # per-platform bundles under <repo>/dist/ (claude, chatgpt, grok, perplexity, gemini, single-file)
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
| `claude` | Claude Code (local and web), claude.ai, Gemini CLI, Codex — nested `<name>/` layout, `maintainer` / `website` under `metadata:`, keeps `user-invocable` and `disable-model-invocation`. |
| `chatgpt` | ChatGPT web — the `claude` layout without `user-invocable` or `disable-model-invocation` (the Agent Skills fields only), never consolidated; refused over 25 MB per file, 50 MB per archive, or 500 files. |
| `grok` | Grok — flat zips (`SKILL.md` at the root) for every root skill plus the Grok-native skills; frontmatter `name` + `description` only. |
| `perplexity` | Perplexity — Claude layout without `user-invocable` or `disable-model-invocation`; any skill over 100 files consolidated from `perplexity-skills/categories.toml`. The committed `perplexity-skills/spacecraft-cli-preference.zip` is a copy of this output. |
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

### Progress and accessible mode

Long operations (`skill build`, the git clone/pull behind `skill add` /
`skill update`, `nix flake update` and `nix build` behind `skill sync`, and
the push / pull-request steps of `skill ship`) show progress on **stderr
only**, and only when every condition holds: human output mode (no
`--json`/`--format`, no agent or CI environment, stdout a TTY), stderr a TTY,
no `--quiet`, and `TERM` not `dumb`. Everywhere else nothing is drawn, so
piped and machine output is byte-for-byte what it was without progress. A
diagnostic printed mid-operation erases the progress line first, so the two
never share a row. `skill build` shows an animated bar; every subprocess
step (git clone/pull, git push, `gh pr create`, `nix flake update`,
`nix build`) is never animated — each can prompt on the terminal
(credentials, nix's `accept-flake-config`), so it prints one static
`Working: <step>…` line before and `… done` / `… failed` after, and a prompt
stays readable.

`--accessible` (or `SPACECRAFT_A11Y=1`) turns on Standard §18 accessible mode:
no spinner, no animation, no cursor movement — progress becomes plain,
append-only lines such as `Working: building bundles… 40%`, written only when
the percentage rises and at most once per second, ending in
`Working: building bundles… done`. Precedence: `--accessible` /
`--no-accessible` beat `SPACECRAFT_A11Y` (`1`/`true`/`yes`/`on` on,
`0`/`false`/`no`/`off` off), and unset everywhere means off. `--verbose`
reports the resolved state and its source as an `ACCESSIBLE_MODE` info
diagnostic.

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
