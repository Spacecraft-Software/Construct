# AGENTS.md — Construct skill catalogue

Authoritative agent context (Standard §5.7). Every agent reads this file —
Claude, Codex, Gemini, Grok, Cursor, opencode. `CLAUDE.md` imports it and adds
only Claude-Code-only notes. The full, version-controlled procedure (bundling
commands, drift sweep, branch/PR flow) lives in
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## What this repo is

A catalogue of agent skills loaded by Claude Code, Gemini CLI, and Codex from
`~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`, plus Grok-specific
skills in [`grok-skills/`](grok-skills/) loaded from `~/.grok/skills/` and
vendored Android skills in [`android-skills/`](android-skills/). Skill content
is markdown plus a small number of templates/JSON — there is no build system,
runtime, or test suite for it; edits there are content edits, shipping is
rezipping. The one exception is [`construct-cli/`](construct-cli/), a real
Rust binary (see below) with its own build/test/lint surface.

Current skills (authoritative list is `README.md §2`; run `ls -d spacecraft-*
gnu-* microsoft-*` or check the README table if this drifts) span three
groups: infra/tooling (`spacecraft-agentic-cli`, `spacecraft-brand-guidelines`,
`spacecraft-cli-preference`, `spacecraft-cli-shell`, `spacecraft-cli-standard`,
`spacecraft-document-format`, `spacecraft-markdown-document`,
`spacecraft-missing-pkg`, `spacecraft-steelbore-standard`,
`spacecraft-texinfo-document`, `spacecraft-theme-factory`, `gnu-coding-standards`,
`gnu-free-software`), the governing Standard skill
(`spacecraft-steelbore-standard`), and one `spacecraft-<language>-guidelines`
skill per supported language (currently: ada, carbon, chez, clang, clojure,
commonlisp, cpp, dartflutter, elixir, erlang, gleam, golang, guile, java,
kotlin, lua, nickel, nim, nix, nu, ocamel, python, rust, swift, typescript,
zig — plus `microsoft-rust-guidelines`, the mandatory Rust base skill). New
language/skill directories land here often; treat the inline list as a rough
map, not ground truth — `README.md §2` and the directory listing are.

The repo is also a Nix flake (`flake.nix` + `flake.lock`). The flake exposes
each detected skill as `packages.${system}.${skill-name}` (each Grok skill as
`packages.${system}.grok-${skill-name}`, each Android skill similarly
namespaced) and ships `homeManagerModules.default` that wires up the canonical
`~/.agents/skills/` location plus per-agent directories. **`flake.lock` is
tracked and must be committed.** Skill auto-detection is by `SKILL.md`
presence — adding a new skill directory is enough; no flake edit needed.
`grok-skills`, `android-skills`, `Excluded`, `.claude`, `.git`, and
`construct-cli` are explicitly excluded from top-level skill auto-detection
(see `excludedDirs` in `flake.nix`).

The authoritative governance document for everything produced in this repo is
[`spacecraft-steelbore-standard/SKILL.md`](spacecraft-steelbore-standard/SKILL.md), which encodes
The Steelbore Standard — it carries the current version in its own masthead, so
none is repeated here to go stale. Load it before any non-trivial edit — its §16 checklist is the
audit gate. The skill is the upstream of the published `standard/` document;
changes flow skill → published standard, so this `SKILL.md` may lead it.

## Skill layout (per top-level directory)

```
<skill-name>/
├── SKILL.md           # frontmatter (name, description, license, maintainer, website) + body
├── LICENSE            # REQUIRED (Standard §5.6 license carriage). Verbatim license text, byte-identical to the matching `LICENSES/` file, a regular file, no extension (§4.3). Almost always GPL-3.0-or-later; `gnu-coding-standards` carries the GFDL-1.3-or-later text instead. Multi-licensed skills carry `LICENSE.<TAG>` in its place — `microsoft-rust-guidelines` ships `LICENSE.GPL` + `LICENSE.MIT`.
├── CREDITS.md         # required when the skill builds on third-party work (Standard §15.3); currently microsoft-rust-guidelines, gnu-coding-standards, gnu-free-software, spacecraft-cli-preference, spacecraft-rust-guidelines, spacecraft-ada-guidelines, spacecraft-steelbore-standard
├── references/        # optional; loaded on demand by the agent
└── assets/            # optional; currently spacecraft-agentic-cli, spacecraft-texinfo-document, steelbore-color-palette
```

Skill IDs (directory and frontmatter `name`) are **functional identifiers**,
not codenames — Standard §2.2 reserves codenames for projects/modules/utilities/
releases, not for skill identifiers. The README catalogue and the directory list
must stay in sync.

## Bundling (.zip and .skill)

Each skill ships as two bundles at the repo root: `<name>.zip` and
`<name>.skill`. They contain only `SKILL.md`, the license file(s), `CREDITS.md`,
and `references/` (plus `assets/` where present) — never tooling, generator
scripts, or raw upstream sources. Auxiliary inputs that don't belong in the
shipped skill live in `Excluded/` (e.g., `Rust-Guidelines.{md,txt}`,
`skill.ps1`).

Rebuild pattern (from `.claude/settings.local.json`):

```sh
rm -f <name>.zip <name>.skill
zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
```

**`LICENSE` is never optional** — Standard §5.6 makes the bundle the unit of
distribution, so every bundle ships the license text. The only variation is the
name: `microsoft-rust-guidelines` is dual-licensed and passes
`microsoft-rust-guidelines/LICENSE.GPL microsoft-rust-guidelines/LICENSE.MIT`
in place of a single `LICENSE`. Include the other arguments only when they
exist. `CREDITS.md` appears only where §15.3 triggers fire (currently
`microsoft-rust-guidelines`, `gnu-coding-standards`, `gnu-free-software`,
`spacecraft-cli-preference`, `spacecraft-rust-guidelines`,
`spacecraft-ada-guidelines`, `spacecraft-steelbore-standard`). `references/`
and `assets/` are optional. Run `ls <name>/` first whenever you're unsure.

The `.skill` bundle uses `-D` to drop directory entries; the `.zip` keeps them.
Verify with `unzip -l <name>.zip` before committing. After editing any file
inside a skill directory, **rebuild both bundles in the same commit** — a stale
bundle ships broken content to every agent that installs from the zip.

## Workflow: every skill-directory change

The bundles are the install surface. A bundle that lags its `SKILL.md` /
`references/` / `assets/` ships broken content to every consumer. The contract
is mechanical — apply it after **any** edit inside a `<skill-name>/` directory:

1. **Rebuild both bundles** for the changed skill:
   ```sh
   rm -f <name>.zip <name>.skill
   zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
   zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
   ```
   Add `<name>/assets` to both lines if the skill has an `assets/` dir
   (currently `spacecraft-agentic-cli`, `spacecraft-texinfo-document`, and
   `steelbore-color-palette`). Prefer `ls <name>/` over this list — omitting
   an `assets/` dir that exists is silent: the rebuild succeeds and the
   bundle simply ships without it. `SKILL.md` and the license file are always
   present; omit any other argument the skill doesn't have.
   `microsoft-rust-guidelines` is dual-licensed and passes
   `<name>/LICENSE.GPL <name>/LICENSE.MIT` instead of `<name>/LICENSE`.
   `CREDITS.md` exists only where §15.3 applies (`microsoft-rust-guidelines`,
   `gnu-coding-standards`, `gnu-free-software`, `spacecraft-cli-preference`,
   `spacecraft-rust-guidelines`, `spacecraft-ada-guidelines`,
   `spacecraft-steelbore-standard`).

   **Run `ls <name>/` and pass every file you see — do not rebuild from the
   lists above.** They are a map, not ground truth, and an omission here is
   silent in the worst way: the `zip` succeeds, the working tree still looks
   correct, and only the shipped bundle is missing a file. This list omitted
   `spacecraft-steelbore-standard` until 2026-09-14, and its bundles shipped
   without `CREDITS.md` for exactly that reason.
2. **Stage** the skill directory **and** both bundles in the same commit —
   never separately. Always stage by explicit name:
   ```sh
   git add <name>/SKILL.md <name>.zip <name>.skill
   ```
   Never use `git add -A` or `git add .` — other `.skill` files at the
   repo root carry pre-existing uncommitted changes from prior normalization
   passes and must not be swept into unrelated commits.
3. **Commit with UTC timestamps**:
   ```sh
   TZ=UTC GIT_COMMITTER_DATE="$(TZ=UTC date)" \
     git commit --date "$(TZ=UTC date)" -m "..."
   ```
   The Steelbore Standard §14.2 forbids offset notation (`+0300`, `+00:00`); only
   `Z` / `+0000` is permitted. Signing is on globally
   (`commit.gpgsign=true`, `gpg.format=ssh`, `user.signingkey=~/.ssh/id_ed25519.pub`)
   — no extra flag needed. Assistant-driven commits end with a
   `Co-Authored-By: Claude …` trailer; human commits do not.
4. **Branch + PR — never push to `main`.** Every change, including a one-line
   version bump, goes through a feature branch → pull request → squash-merge →
   delete branch. This matches `/spacecraft-software/standard/AGENTS.md`, which
   states the rule for **both** the Standard and Construct repos, and matches
   this repo's own recent history (#11, #21, #22 are all squash-merged PRs).
   ```sh
   git switch -c <short-topic-branch>
   # …rebuild bundles, stage by name, commit (steps 1–3)…
   git push -u https://github.com/Spacecraft-Software/Construct.git <branch>
   gh pr create --repo Spacecraft-Software/Construct --base main --head <branch> \
     --title "…" --body "…"
   ```
   Use HTTPS on this host — the SSH remote
   (`git@github.com:Spacecraft-Software/Construct.git`) is intermittently
   unreachable here. The HTTPS push still carries the locally-made signed
   commit, so GitHub's "Verified" status is preserved.

   There is **no auto-push pre-authorisation.** A prior note in this file
   claimed skill-directory changes could be pushed straight to `main` without a
   prompt; that contradicted the Standard repo's cross-repo rule and is
   withdrawn. Opening the PR is where the assistant stops — **merging is the
   maintainer's call**, and the assistant never merges its own PR.

The assistant's responsibility ends at opening the PR. **Local agent install dirs
refresh only after Home Manager rebuilds** — see *Local agent fan-out* below.
That rebuild is a user-initiated action; the assistant must not run it.

If multiple skills changed in one turn, rebuild **all** of their bundles in the
same commit. Never let `git status` show a skill-dir change without its
matching bundle change.

**Detecting already-committed drift.** A clean working tree does *not* prove the
bundles are current: a past commit can bump `SKILL.md` while forgetting the
bundle, leaving a committed `.zip`/`.skill` that silently lags its source (this
has happened — `spacecraft-steelbore-standard` shipped v1.12 against a v1.18 `SKILL.md`).
`git status` can't see it. Before trusting the install surface, sweep:

```sh
for d in */; do n="${d%/}"; [ -f "$n/SKILL.md" ] || continue
  case "$n" in grok-skills|android-skills|Excluded|construct-cli) continue;; esac
  inzip="$(unzip -Z1 "$n.zip" 2>/dev/null | grep -v '/$')"
  # (a) content drift: every file inside the bundle must match the working tree
  printf '%s\n' "$inzip" | while read -r f; do [ -n "$f" ] || continue
    unzip -p "$n.zip" "$f" 2>/dev/null | diff -q - "$f" >/dev/null \
      || echo "DRIFT (content): $n.zip :: $f"
  done
  # (b) missing from bundle: every shippable file on disk must be in the bundle
  find "$n" -type f \( -name SKILL.md -o -name 'LICENSE*' -o -name CREDITS.md \
      -o -path "$n/references/*" -o -path "$n/assets/*" \) 2>/dev/null | while read -r f; do
    printf '%s\n' "$inzip" | grep -qxF "$f" \
      || echo "DRIFT (missing): $n.zip lacks $f"
  done
done
```

Any `DRIFT:` line means rebuild that skill's bundles and commit. The sweep now
walks the **whole** bundle: `(a)` content-diffs every file the `.zip` contains
(`SKILL.md`, `references/**`, `LICENSE`, `CREDITS.md`, `assets/**`) against the
working tree, and `(b)` flags any shippable file on disk that the bundle is
missing — so adding a `references/` file (e.g. `spacecraft-steelbore-standard`'s
`CHANGELOG.md`) without rebuilding is caught too. It checks `.zip` as the
canonical surface; `.skill` is built in lockstep from the same args in the same
commit, so a drifted `.zip` implies a drifted `.skill`.

`git log -1 --show-signature` may report "No signature" locally because
`~/.ssh/allowed_signers` isn't populated; this is a verifier-side gap, not a
signing failure. GitHub validates the SSH signature independently and shows
"Verified" if the public key is registered as a **Signing** key in GitHub
account settings (Authentication-only keys won't validate signatures).

## `construct-cli/` — the `construct` Rust binary

[`construct-cli/`](construct-cli/) is a separate, real Rust project (its own
`Cargo.toml`/`Cargo.lock`, `src/`, `tests/`, and per-directory context files) —
the Spacecraft Software Construct skills package manager, conforming to the
Dual-Mode Self-Documenting CLI Standard. It is excluded from the flake's skill
auto-detection (`excludedDirs`) but is itself buildable via `nix build
.#construct`. Standard commands:

```sh
cd construct-cli
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Currently implemented (`construct --help` / `construct describe` are the
authority): the imperative installer `construct skill add | list | remove |
update | find | use` across the agent registry (`construct agent list`), from a
local path, git URL, or `owner/repo` source; `construct skill init`;
`construct skill sync` (runs `nix flake update construct` in a consumer flake,
default `/spacecraft-software/bravais`); `construct skill status` / `skill
reset` (the live skill tree against the flake pin); `construct skill ship`;
`construct skill build`; `construct skill vendor`; `construct describe`;
`construct schema`; and the `--format explore` TUI (also what a bare
invocation on a terminal opens).

`construct skill ship` implements the branch+PR workflow above end-to-end:
it enforces bundle-drift and the §5.6 description gate (the 1000-character
cap, and a frontmatter every strict YAML parser accepts — a plain-scalar
`description:` containing `: ` is a mapping, not a string, and is refused),
switches to a feature branch (generated from the shipped skills, or
`--branch`), stages by explicit name, makes the signed UTC commit, pushes the
branch, and opens the PR with `gh`. It **never** pushes to the default branch, and it never merges — that
stays the maintainer's call. `--dry-run` reports the whole plan, including the
branch it would use, without touching anything.

Because nothing lands on `main`, `ship` no longer runs `skill sync`; run
`construct skill sync` after the PR merges. `--no-sync` is retained as a hidden
no-op so existing invocations keep working.

`construct skill build [SKILL...] [--target claude,chatgpt,grok,perplexity,gemini,single-file,minimax]`
generates per-platform release bundles under `dist/<target>/` (gitignored —
release artifacts, never committed). It rewrites frontmatter per target (the
source `SKILL.md` stays as is), vendors the palette into the brand and
accessibility skills, consolidates any skill over Perplexity's 100-file limit
from `perplexity-skills/categories.toml` (for the `perplexity` and `gemini`
targets), and writes deterministic zips. It
refuses — writing nothing — on any §5.6 gate failure. It does **not** touch the
committed root `<name>.zip` / `<name>.skill` bundles, and ship's drift check is
unchanged.

`construct skill vendor <skill>... [--into <repo>] [--dry-run]` writes the
`claude` target's tree for the named skills into
`<repo>/.claude/skills/<name>/` — the only place Claude Code cloud sessions
load skills from. It never runs git in that repository (only a read-only
`git ls-files` on the source catalogue, for file modes): it prints the
`git add` / `git commit` step for the user. It replaces only directories carrying its
`.construct-vendor.toml` marker (no timestamp, no source path), refuses
anything else unless `--force`, and never writes through a symlinked
`.claude/`, `.claude/skills/`, or skill directory, even with `--force`.
Long operations (`skill build`, clone/fetch in `skill add` / `skill update`,
`skill sync`, `skill ship`) draw progress on stderr in human TTY mode only —
never under `--json`, a pipe, an agent/CI env, `--quiet`, or `TERM=dumb`.
`--accessible` / `SPACECRAFT_A11Y=1` (flag wins) switches it to static,
append-only `Working: …` lines at most once per second (Standard §18).

Read `construct-cli/AGENTS.md` before working inside
this subdirectory — it governs that subtree, not this file.

### Distribution targets

`construct skill build` writes one directory per target under `dist/` at the
repo root (`--out <dir>` moves it). `dist/` is gitignored — release artifacts,
never committed. Zips are deterministic (fixed member mtimes, sorted entries),
so two builds of the same tree are byte-identical and CI release artifacts
reproduce. The source `SKILL.md` is never edited; frontmatter is rewritten per
target.

| Target | Output | Platform / purpose |
|--------|--------|--------------------|
| `claude` | `dist/claude/<name>.zip` + `.skill`, nested `<name>/` layout | Claude Code (local and web), claude.ai, Gemini CLI, Codex. Spec-clean frontmatter: `name`, `description`, `license`, `compatibility` / `allowed-tools` when present, `maintainer` / `website` moved under `metadata:`; Claude Code's invocation controls `user-invocable` and `disable-model-invocation` kept (this target only). |
| `chatgpt` | `dist/chatgpt/<name>.zip` + `.skill`, nested `<name>/` layout | ChatGPT web (Skills → Create → Upload from your computer). The open Agent Skills layout ChatGPT validates against: the `claude` tree and frontmatter minus `user-invocable` and `disable-model-invocation`, never consolidated. Refused when over ChatGPT's published limits — 25 MB per uncompressed file, 50 MB per archive, 500 files (`construct-cli/src/bundle/chatgpt.rs`). A manual-only skill (`disable-model-invocation: true`) loses that control here, as on Perplexity. |
| `grok` | `dist/grok/<name>.zip` + `.skill`, **flat** (`SKILL.md`, `LICENSE`, `references/`, `assets/` at the zip root) | Grok. Every root skill plus the Grok-native `grok-skills/*`; frontmatter `name` + `description` only. |
| `perplexity` | `dist/perplexity/<name>.zip`, nested layout | Perplexity. Claude frontmatter minus `user-invocable` and `disable-model-invocation`; any skill over 100 files is consolidated from `perplexity-skills/categories.toml`. |
| `gemini` | `dist/gemini/<name>.zip` only (no `.skill`), **flat**, files only (no directory entries) | The Gemini app's skill upload. Every root skill (no Grok-native skills); frontmatter `name` + `description` only, the description byte-identical to the source. The app accepts only `.csv`, `.py`, `.txt`, and `.md` members, so any other file gets `.txt` appended — `LICENSE` → `LICENSE.txt`, `LICENSE.MIT` → `LICENSE.MIT.txt` (§5.6 license carriage; §4.3's no-extension rule governs repository files, not this platform bundle), `assets/steelbore.toml` → `assets/steelbore.toml.txt` — and every markdown link inside the skill that points at a renamed file is rewritten to follow it (links only, outside fenced code). Skills over 100 files are consolidated exactly as for `perplexity`; a bundle still over 100 files, a rename that collides with an existing file, or a `name` that is not kebab-case is refused. |
| `minimax` | `dist/minimax/<name>.zip` + `.skill`, nested `<name>/` layout | MiniMax Agent (web): Create → Upload a skill → "Saved to My skills" (personal; never press **Publish skill** — that is a public listing, gated by Standard §6.4). Byte-identical to `chatgpt`, which the uploader was verified to accept on 2026-10-06. Gated on the uploader's one stated rule, exactly one `SKILL.md` per `.zip` / `.skill`: a second one anywhere under `references/` or `assets/` is refused. MiniMax publishes no size limits for this uploader, so none is enforced. Not a MiniMax marketplace plugin (`.minimax-plugin/plugin.json`) — that route is a public submission. |
| `single-file` | `dist/single-file/<name>.md` | Platforms with no skill loader: a short frontmatter header, the `SKILL.md` body, every `references/` file inlined under its own heading with links rewritten to in-document anchors, text assets in fenced blocks, `CREDITS.md`, then a closing `## License` section carrying every `LICENSE` / `LICENSE.<TAG>` verbatim in fenced `text` blocks (§5.6 license carriage — `microsoft-rust-guidelines.md` ships both the GPL text and the MIT permission notice). No relative link survives outside a fenced block: a reference's `../SKILL.md` back-link points at the body's anchor, a link to a license file (`LICENSE.MIT`, a reference's `../LICENSE`) points at its subsection, a sibling-skill link becomes plain text naming that skill, and any other link that leaves the skill (upstream paths in `microsoft-rust-guidelines`) keeps its text and loses the link. Fenced content is verbatim. |

Every non-source target vendors `steelbore-color-palette/assets/steelbore.toml`
into `spacecraft-brand-guidelines` and `spacecraft-accessibility-support` as
`assets/steelbore.toml` (`assets/steelbore.toml.txt` for `gemini`) and
verifies it byte-identical to the source. The build
enforces the §5.6 description cap (the same check `ship` uses) and the spec's
500-character `compatibility` cap, emitting nothing for an offending skill.
Scope is root skills (the `excludedDirs` set; `android-skills/` and
`orca-skills/` are never rewritten) plus `grok-skills/*` for the `grok` target.
The committed root `<name>.zip` / `<name>.skill` and `grok-skills/*.zip`
bundles are separate and untouched by `build`.

**Claude Code cloud sessions** load only a repository's committed
`.claude/skills/`, not `~/.claude/skills`. To give a cloud session a skill, run
`construct skill vendor <skill>... --into <repo>` and commit what it prints.

### Releases

`.github/workflows/release.yml` publishes the `construct skill build` output as
a GitHub release. Nobody runs it by hand in the normal flow:

- **Trigger.** `workflow_run` on CI completing — it proceeds only when CI
  succeeded on a `push` to `main` of `Spacecraft-Software/Construct`, so a
  release never ships a tree the gates refused and a fork never cuts one
  (§6.4). `workflow_dispatch` (input `ref`, default `main`) is the manual
  backfill for a run that never fired; it is accepted only when dispatched
  from `main`, refuses any commit not on `main`, and does not re-check CI:
  `gh workflow run release.yml --ref main -f ref=<full-40-char-sha>` (a
  7-character sha fails checkout).
- **Two jobs, split on the token.** `build` (`contents: read`) compiles
  `construct` from a clean `cargo build --release --locked` — no rust-cache,
  so no restored artifact can reach the shipped binary — builds every target
  twice and fails on any byte difference, then packages. `publish`
  (`contents: write`, plus `id-token`/`attestations` for provenance) runs no
  repository code: it takes `build`'s artifact and talks to the Releases API
  with `gh`. Every action is pinned to a full commit sha.
- **Tag.** `bundles-YYYY-MM-DD-<sha7>`, dated by the commit's UTC committer
  date, not the build — the tag is a function of the sha, so a re-run lands on
  the same tag. A tag on another commit, or any tag lookup failure other than a
  clean 404, fails the run. `--latest` is set unless the current latest
  release's commit already descends from this one.
- **Never rewrites a published byte.** On a re-run, every asset the release
  already holds is checked against this build by digest; any difference fails
  the run. Only missing or cut-off assets are uploaded (`manifest.json` is
  refreshed only alongside them), and a draft left by a killed job is
  completed and published.
- **Assets.** `construct-bundles-<target>.zip` per target — `claude`, `chatgpt`,
  `grok`, `perplexity`, `gemini`, `single-file`, `minimax` (the per-skill `.zip` /
  `.skill` / `.md` files; `claude`, `chatgpt`, `grok`, and `minimax` carry a `.zip` and a
  `.skill` per skill, the
  others one file per skill; the outer archives are themselves
  byte-stable), `SHA256SUMS` over those archives, and `manifest.json` (commit,
  UTC commit and build times, construct version, per-target skill counts).
  Each archive also carries a Sigstore build-provenance attestation binding it
  to `release.yml` on `main`; verify with `gh attestation verify <archive>
  --repo Spacecraft-Software/Construct --signer-workflow
  Spacecraft-Software/Construct/.github/workflows/release.yml --source-ref
  refs/heads/main` (`--repo` alone accepts any workflow on any branch). The
  attestation is minted only after the digest checks pass; `manifest.json`,
  not the attestation, names the source commit. The notes carry a
  per-platform install table.
- **No new commits.** The tag is created by Actions through the Releases API
  on the squash commit already on `main`, which is signed and Verified under
  Standard §6.3.

## Vendored Android skills (`android-skills/`)

`android-skills/` is Google's official [android/skills](https://github.com/android/skills)
catalogue, vendored **verbatim and unmodified** (Apache-2.0, third-party —
Standard §4.2 upstream-preservation). Never hand-edit content inside it;
upstream changes come in via re-vendoring, not local patches. It is flattened
one level from upstream (`<category>/<skill>/SKILL.md` → `<skill>/SKILL.md`)
to match how Construct's loader and flake discover skills, and its own
`README.md`/`CREDITS.md` track provenance and the flattening rule. It is
excluded from the root flake's `excludedDirs` and packaged separately.

## Vendored Orca skills (`orca-skills/`)

`orca-skills/` vendors three skills from [Orca](https://github.com/stablyai/orca)
— `computer-use`, `orca-cli`, `orchestration` — **verbatim and unmodified**
(MIT, third-party — Standard §4.2). Never hand-edit them; a new revision comes
in by re-copying upstream's `SKILL.md` and updating `CREDITS.md`. The generic
leaf names `computer-use` and `orchestration` are reserved: Orca's own CLI looks
its skills up by exact leaf name, so a future Spacecraft skill must not claim
either.

**They are OPT-IN in the flake (`spacecraft.construct.enableOrca`, default off),
and must stay off on any host that runs the Orca app.** Orca installs and
updates its own copies; a copy served from the Nix store makes its updater fail
in a way no amount of re-vendoring fixes:

- Orca's scanner (`observeSkillPackage`, in the app's `app.asar`) throws
  `skill-package-link` on any file with `nlink != 1`. Store optimisation
  hardlinks identical files, so every store-served `SKILL.md` eventually has
  nlink > 1 — ours sat at 5–7. The throw is caught and reported as status
  `unrecognized`: the "The copy here doesn't match the official version" row in
  Settings → Update skills.
- Even past that check, store files are mode 444 and
  `classifyHomeSkillTopology` marks an unwritable path `read-only`, which the
  updater also skips.
- **Byte-identity does not help.** `computer-use` and `orchestration` were
  byte-for-byte the official revisions and were flagged just the same. Any note
  claiming a revision pin clears the warning is wrong.

On an Orca host the arrangement is: `enableOrca = false`, plus
`spacecraft.construct.perSkillLinks.enable = true` so `~/.agents/skills` is a
real directory with room for `orca skills install` (`npx skills add`) to own
`computer-use/`, `orca-cli/`, `orchestration/` as real, writable directories.
The module never replaces a real directory or a symlink it did not make, and
prunes only symlinks pointing into its own tree.

Turn `enableOrca` on only where nothing else provides these skills — no Orca
app, an air-gapped host, a container image. `packages.skills-with-orca` builds
that merged tree.

## Grok skills (`grok-skills/`)

Grok uses a **flat** bundle format — `SKILL.md` and any `assets/` / `references/`
live at the **root** of the `.zip`, not inside a `<skill-name>/` directory.
Grok-specific skills therefore live under `grok-skills/` with their own
catalogue (`grok-skills/README.md`) and their bundles ship **inside**
`grok-skills/`, not at the repo root.

Source layout is the same as cross-platform skills:

```
grok-skills/<name>/
├── SKILL.md
├── assets/        (optional)
└── references/    (optional)
```

Bundle layout is **flat** (different from root skills):

```
grok-skills/<name>.zip
├── SKILL.md
├── assets/...
└── references/...
```

Build from inside the skill directory so the paths land at the zip root:

```sh
cd grok-skills/<name>
rm -f ../<name>.zip ../<name>.skill
zip -qr  ../<name>.zip   SKILL.md [assets] [references]
zip -qrD ../<name>.skill SKILL.md [assets] [references]
```

Verify the zip top level is `SKILL.md` (not `<name>/SKILL.md`) before
committing — a nested layout will break Grok's loader.

`construct skill build --target grok` produces flat Grok bundles for **every**
root skill (frontmatter reduced to `name` + `description`, `LICENSE` at the zip
root) plus the Grok-native skills here, under `dist/grok/` — see *Distribution
targets* above. Its `gfm-markdown` output is content-identical to the committed
`grok-skills/gfm-markdown.zip`; the committed bundles here are still rebuilt by
the recipe above.

The same workflow contract applies as for root skills: rebuild both bundles
whenever any file inside a Grok skill changes, stage the directory and both
bundles in the same commit, never use `git add -A`. The
`grok-skills/README.md` catalogue table must stay in sync with the
subdirectory listing.

**Installing a third-party Grok skill on a Home-Manager host.** `~/.grok/skills`
is module-managed. While it is a whole-directory store link, `npx skills add`
cannot write a leaf into it: the store is mounted read-only, so the install
fails with `EROFS` — or with `ENOENT`, which looks like a missing directory and
is not, when it runs in the window during a rebuild where the old generation has
been garbage-collected and the new link is not yet in place. Set
`spacecraft.construct.perSkillLinks.enable = true` and the directory becomes
real, leaving every name this module does not carry free for an imperative
install to own. The alternative is to vendor the skill into `grok-skills/` and
let the flake ship it, which is the right answer when the skill should be
declarative on every host rather than installed on one.

Frontmatter is also minimal for Grok — just `name` and `description`. No
`license`, `maintainer`, `website` fields (Grok's loader does not consume
them). A Grok skill still carries its own `LICENSE`, because Standard §5.6
license carriage is about the *bundle*, not the frontmatter — and because the
flat layout puts it at the zip root rather than under `<name>/`, the recipe
above passes a bare `LICENSE`. The repo-root `LICENSE` remains the canonical
GPL-3.0-or-later text as a regular file, with `LICENSES/GPL-3.0-or-later.txt`
a symlink back to it (§4.3, v1.38 direction), and every skill copy is
byte-identical to it.

## Perplexity bundle (`perplexity-skills/`)

Perplexity rejects an uploaded skill zip containing **more than 100 files**.
Every skill here is comfortably under that except `spacecraft-cli-preference`,
which ships 110 per-tool `references/` files so an agent can lazy-load exactly
the one tool it is about to run. Perplexity accepts the *layout*; it fails only
on the count.

`perplexity-skills/` therefore holds a **generated**, consolidated bundle for
that one skill — `construct skill build --target perplexity` merges the 110
per-tool files into 14 category files and rewrites `SKILL.md`'s links to
`references/<category>.md#<tool>`, driven by the map in
`perplexity-skills/categories.toml`.
It is excluded from the flake's skill auto-detection (`excludedDirs`) and is
not a skill directory: there is no `SKILL.md` at its top level.

**The contract that matters when editing:**

- **Never hand-edit** the category files or the zipped `SKILL.md`. They are
  build output. Edit the canonical `spacecraft-cli-preference/`.
- **Regenerate and commit `perplexity-skills/spacecraft-cli-preference.zip` in
  the same commit** as any change to `spacecraft-cli-preference/SKILL.md` or
  its `references/` — the same install-surface rule that governs the root
  bundles. This is easy to miss precisely because the file lives in a
  different directory from the skill that determines its contents:

  ```sh
  construct skill build --target perplexity spacecraft-cli-preference
  cp dist/perplexity/spacecraft-cli-preference.zip perplexity-skills/
  ```

  The build self-checks that every rewritten `#anchor` resolves to a real
  `## <tool>` heading, and asserts the map in
  `perplexity-skills/categories.toml` covers exactly the canonical tool set —
  so adding or removing a tool fails the build until the map is updated. The
  map is data, not code; `construct` is its only reader. The committed zip is
  the Perplexity target's output verbatim: spec-clean frontmatter
  (`maintainer` / `website` under `metadata:`) and a `LICENSE` (§5.6). The
  former `perplexity-skills/build.py` generator is retired.

Full rationale, the category table, and the regeneration rules live in
[`perplexity-skills/README.md`](perplexity-skills/README.md) and the
"Perplexity bundle" section of [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Local agent fan-out (Home Manager hosts)

Local fan-out is managed by **Home Manager**, not by the assistant. The
canonical tree is `~/.agents/skills`, which under `mutablePointer` chains
through a mutable pointer into the store, and which most agents (Codex, Gemini
CLI, Goose, Kimi, OpenCode, Kilo, Mimo, Cursor, Grok, Copilot, Orca) read
directly. An agent that only reads its own directory gets it populated per
`spacecraft.construct.agentPaths`, where each entry carries a `mode`:

```
~/.agents/skills                 → ~/.local/state/construct/current            # the hub
~/.local/state/construct/current → …/pinned → /nix/store/<hash>-construct-skills
~/.claude/skills/<skill>         → ~/.local/state/construct/current/<skill>    # mode = "per-skill": a REAL directory, one link per skill
~/.agent/skills                  → ~/.agents/skills                            # mode = "dir-symlink": the default, what a bare string means
~/.codex/skills                    (untouched)                                 # mode = "none": the agent reads the hub itself
```

With `perSkillLinks.enable = true` the hub takes the same shape as a
`per-skill` agent directory: `~/.agents/skills` is a **real directory** whose
entries are per-skill symlinks into `…/construct/current/<skill>`. Same
content, but names the module does not carry stay free for another installer
to own — which is what an Orca host needs (see *Vendored Orca skills* above).

`per-skill` exists because `dir-symlink` leaks: through a directory symlink,
an agent's private writes (Claude Code's `synced/`, Codex's `.system/`) land
inside the shared hub, where every other agent sees them. The per-skill
renderer is the hub's own — it never replaces a real entry or a symlink it did
not make (a Vercel relative link, a user's override), and prunes only links
into its own tree — so the agent's directory keeps whatever else lives there.
A regular file or a foreign symlink at the path itself skips that entry with a
stderr note; the activation carries on. `none` mirrors the Vercel `skills`
CLI's "universal" agents: nothing to render, and the only write is removing a
hub symlink an earlier generation of the module left behind. `dir-symlink`
stays the default for compatibility, never removes a real directory it finds
at the path, and re-points only a symlink it made itself. A trailing `/` on a
path is stripped; an absolute, empty or duplicated path fails evaluation.

The same renderer draws `~/.grok/skills` (links straight into the store — the
Grok tree has no mutable pointer). Every tree goes through that one function in
`flake.nix`, so what gets clobbered and what gets pruned cannot diverge.

Which paths Home Manager populates on this host is stated by `agentPaths` in
`bravais/users/mj/home.nix` (`spacecraft.construct`, with `mutablePointer`,
`perSkillLinks` and a mode per agent). Gemini CLI's scan path is Home
Manager's responsibility on this host as well — the assistant does not
provision it.

The Nix flake's `homeManagerModules.default` (in `flake.nix`) provides that
layout for new consumers: install once to `~/.agents/skills/`, then populate
only the directories of agents that cannot read it. Grok skills install
separately to `~/.grok/skills/` because of their different bundle layout.

**After a PR merges, Home Manager must be rebuilt** before per-harness paths
resolve to the new content. The maintainer runs the rebuild manually
(`home-manager switch …` or the equivalent flake command); the assistant
does **not** invoke it.

Verify after rebuild:

```sh
readlink -f ~/.claude/skills/spacecraft-steelbore-standard
# → /nix/store/<hash>-construct-skills/spacecraft-steelbore-standard
sha256sum ~/.claude/skills/spacecraft-steelbore-standard/SKILL.md \
          /spacecraft-software/construct/spacecraft-steelbore-standard/SKILL.md
```

The tree is a store copy, not a link back into the checkout, so the check is
byte-equality against the working tree rather than the resolved path. If the
two hashes differ (or the path still resolves into a stale
`/nix/store/<old-hash>-hm_*` from an earlier layout), Home Manager has not been
rebuilt against the current commit — agents read the previous generation's
content even though `origin/main` is ahead.

The assistant performs no `rsync`, no symlink setup, and no
`home-manager switch`. Its responsibility ends at opening the PR.

## Editing rules specific to this repo

- **`SKILL.md` frontmatter `description` — hard cap 1000 characters.** The skill
  loader's absolute limit is 1024 (`field 'description' in SKILL.md must be at
  most 1024 characters`), but this repo caps the *rendered* `description` at
  **1000** — a description MUST NOT exceed 1000 chars, full stop. The 24-char
  margin absorbs loader/encoding edge cases and the trailing newline YAML adds.
  YAML folded scalars (`description: >`) join lines with spaces, turn blank lines
  into newlines, and add one trailing newline — so the rendered length is what
  counts, not the raw line count. Re-check after any description edit; every
  skill is currently ≤1000 (closest: `gnu-coding-standards` and
  `spacecraft-guile-guidelines`, just under).
  Folded-aware check before committing:
  ```sh
  python3 - "$skill/SKILL.md" <<'PY'
  import sys
  L=open(sys.argv[1]).read().splitlines(); i=L.index('---',1); fm=L[1:i]
  j=[k for k,l in enumerate(fm) if l.startswith('description:')][0]
  body=[fm[k].strip() for k in range(j+1,len(fm)) if fm[k].startswith(' ') or not fm[k].strip()]
  out=[]; buf=[]
  for b in body:
    (out.append(' '.join(buf)),buf.clear()) if b=='' else buf.append(b)
  if buf: out.append(' '.join(buf))
  print(len('\n'.join(out))+1)
  PY
  ```
  The cap is normative (**Standard §5.6**) and enforced at three points, in
  descending authority:
  1. **CI** — the `SKILL.md description cap` step in `.github/workflows/ci.yml`
     runs `.githooks/check-description-length.py` over every `SKILL.md` found in
     the tree (`find`, so `grok-skills/` and `android-skills/` are covered). This
     is the gate; it cannot be skipped.
  2. **`construct skill ship`** — refuses to stage, commit, or open a PR for a skill whose
     rendered description exceeds 1000 (exit 5, `CONFLICT`, with an
     `oversized_skills` array naming each offender and its overage). This is the
     pre-pack gate §5.6 requires.
  3. **Pre-commit hook** — `.githooks/pre-commit` runs the same checker against
     the *staged* `SKILL.md` blobs (root + Grok, block *and* single-line forms).
     Tracked hooks aren't auto-honoured; this host is already activated
     (`git config core.hooksPath .githooks`), a fresh clone needs that once.
     Convenience, not the gate.

  Run the checker over everything by hand exactly as CI does:
  ```sh
  find . -name SKILL.md -not -path './.git/*' -print0 \
    | xargs -0 python3 .githooks/check-description-length.py
  ```
- **`microsoft-rust-guidelines` is intentionally `user-invocable: false`.** It is
  the mandatory auto-load Rust base — `spacecraft-steelbore-standard` mandates loading it
  before any Rust, `spacecraft-rust-guidelines` defers to it as "load first," and
  `gnu-coding-standards` / `spacecraft-cli-standard` / `spacecraft-agentic-cli`
  chain to it. It fires automatically from its own description and those chains,
  so it's hidden from the `/` menu on purpose (Claude Code docs: "background
  knowledge users shouldn't invoke directly"). Do **not** remove the field to
  "fix" a perceived load failure — its absence from the menu is by design.
- **License files are named `LICENSE`, with no extension** (Standard §4.3).
  `LICENSE.md` and `LICENSE.txt` are non-compliant, and a skill offered under
  more than one license carries `LICENSE.<TAG>` per license — never a dash
  (`LICENSE-MIT`) and never a combined file. Every skill has one, it is a
  regular file, and it is byte-identical to the matching text in `LICENSES/`
  (§5.6). `.github/check-license-files.py` is the gate and reads which license
  applies from `REUSE.toml`, so there is no second list to maintain; run
  `python3 .github/check-license-files.py .` before pushing. Third-party
  vendored trees (`android-skills/`, `orca-skills/`) are exempt — §4.2 keeps
  upstream's own layout and filenames verbatim.
- **Rebuild BOTH bundles after any skill-dir edit**, in the same commit:
  `<name>.zip` (`zip -qr`, keeps dir entries) and `<name>.skill` (`zip -qrD`,
  drops them). A bundle that lags its `SKILL.md`/`references/` ships broken
  content to every consumer.
- **Stage explicitly by name — never `git add -A` / `git add .`.** Other root
  `.skill` files carry unrelated uncommitted changes that must not be swept in.
- **Commit in UTC, signed** (signing is global, no flag needed); assistant
  commits add a `Co-Authored-By: Claude …` trailer.
- **Branch + PR — never push to `main`.** Every change, including a one-line
  version bump, goes through a feature branch → pull request → squash-merge →
  delete branch. This is a two-repo rule: `/spacecraft-software/standard/`
  states it for the Standard *and* Construct, and it applies to human and
  assistant-driven changes alike. There is **no auto-push exemption** for
  skill-directory edits. An agent's work ends at opening the PR — **merging is
  the maintainer's call**, and an agent never merges its own PR.
  `construct skill ship` implements this end-to-end — branch, signed commit,
  push, `gh pr create` — and never pushes to the default branch.
- **`execution-context.md` is a shared block.** `spacecraft-cli-shell`,
  `spacecraft-cli-preference`, and `spacecraft-missing-pkg` each ship
  `references/execution-context.md` — the Step −1 classifier (local-host /
  disposable-sandbox / no-execution) that lets them run outside the user's own
  machine. The canonical copy is `spacecraft-cli-shell`'s; edit it there, `cp`
  it over the other two, and rebuild all three skills' bundles plus the
  Perplexity zip in the same commit. `.github/check-shared-blocks.py` (CI)
  fails on any byte difference, and also caps a `compatibility:` frontmatter
  value at the spec's 500 characters.
- **README §2 catalogue is load-bearing.** When adding a skill directory, add a
  matching alphabetical row to the table in `README.md`. When removing one,
  delete the row.
- **Dates are ISO 8601 UTC** anywhere they appear in SKILL.md, references, or
  changelogs (Standard §12). No AM/PM, no local-time strings.
- **Don't import skill content into a memory file or a context file.** The
  skills are the source of truth and are already loaded on demand.
- `Chat.txt` / `Chat2.txt` / `Chat3.txt` are session exports and are gitignored
  (`Chat*.txt`) — never commit them.
- `Excluded/` is the holding pen for inputs that produce skill content but must
  not ship with it. Don't reference it from inside any `SKILL.md`.
- **This file is the single agent-facing source of truth** (Standard §5.7).
  There is no second copy to keep in sync: `CLAUDE.md` imports it, and the full
  procedure lives in [`CONTRIBUTING.md`](CONTRIBUTING.md).
- **REUSE compliance** (`LICENSES/` + `REUSE.toml`) applies repo-wide per
  Standard §4.3 — every shipped file needs SPDX tags or `REUSE.toml` coverage.
  `reuse lint` should pass before pushing if you touched licensing metadata.

## Installation (what consumers do)

```sh
git clone git@github.com:Spacecraft-Software/Construct.git ~/.claude/skills
git clone git@github.com:Spacecraft-Software/Construct.git ~/.gemini/skills
git clone git@github.com:Spacecraft-Software/Construct.git ~/.codex/skills
git clone git@github.com:Spacecraft-Software/Construct.git ~/.grok/skills
```

Or, via Nix flake:

```nix
inputs.construct.url = "github:Spacecraft-Software/Construct";
# then in HM modules:
construct.homeManagerModules.default
{ spacecraft.construct.enable = true; spacecraft.construct.enableGrok = true; }
```

`nix flake update construct` in the consumer flake bumps to the latest commit.

The SSH remote is configured for [Gitway](https://github.com/Spacecraft-Software/Gitway).
