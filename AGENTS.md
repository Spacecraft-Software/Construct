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
`~/.agents/skills/` location plus per-harness symlinks. **`flake.lock` is
tracked and must be committed.** Skill auto-detection is by `SKILL.md`
presence — adding a new skill directory is enough; no flake edit needed.
`grok-skills`, `android-skills`, `Excluded`, `.claude`, `.git`, and
`construct-cli` are explicitly excluded from top-level skill auto-detection
(see `excludedDirs` in `flake.nix`).

The authoritative governance document for everything produced in this repo is
[`spacecraft-steelbore-standard/SKILL.md`](spacecraft-steelbore-standard/SKILL.md), which encodes
The Steelbore Standard — it carries the current version in its own masthead, so
none is repeated here to go stale. Load it before any non-trivial edit — its §14 checklist is the
audit gate. The skill is the upstream of the published `standard/` document;
changes flow skill → published standard, so this `SKILL.md` may lead it.

## Skill layout (per top-level directory)

```
<skill-name>/
├── SKILL.md           # frontmatter (name, description, license, maintainer, website) + body
├── LICENSE            # REQUIRED (Standard §5.6 license carriage). Verbatim license text, byte-identical to the matching `LICENSES/` file, a regular file, no extension (§4.3). Almost always GPL-3.0-or-later; `gnu-coding-standards` carries the GFDL-1.3-or-later text instead. Multi-licensed skills carry `LICENSE.<TAG>` in its place — `microsoft-rust-guidelines` ships `LICENSE.GPL` + `LICENSE.MIT`.
├── CREDITS.md         # required when the skill builds on third-party work (Standard §15.3); currently microsoft-rust-guidelines, gnu-coding-standards, spacecraft-cli-preference, spacecraft-rust-guidelines, spacecraft-ada-guidelines
├── references/        # optional; loaded on demand by the agent
└── assets/            # optional; only spacecraft-agentic-cli has one today
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
`microsoft-rust-guidelines`, `gnu-coding-standards`, `spacecraft-cli-preference`,
`spacecraft-rust-guidelines`, `spacecraft-ada-guidelines`). `references/` and `assets/` are optional.
Run `ls <name>/` first whenever you're unsure.

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
   (today only `spacecraft-agentic-cli` does). `SKILL.md` and the license file
   are always present; omit any other argument the skill doesn't have.
   `microsoft-rust-guidelines` is dual-licensed and passes
   `<name>/LICENSE.GPL <name>/LICENSE.MIT` instead of `<name>/LICENSE`.
   `CREDITS.md` exists only where §15.3 applies (`microsoft-rust-guidelines`,
   `gnu-coding-standards`, `spacecraft-cli-preference`,
   `spacecraft-rust-guidelines`, `spacecraft-ada-guidelines`). Run `ls <name>/`
   first when in doubt.
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

Currently implemented: `construct skill sync` (runs `nix flake update
construct` in a consumer flake, default `/spacecraft-software/bravais`),
`construct skill ship`, `construct describe`, `construct schema`. Planned
phases add an imperative installer across ~70 agent registries, general git
sources, and a `--format explore` TUI.

`construct skill ship` implements the branch+PR workflow above end-to-end:
it enforces bundle-drift and the §5.6 description cap, switches to a feature
branch (generated from the shipped skills, or `--branch`), stages by explicit
name, makes the signed UTC commit, pushes the branch, and opens the PR with
`gh`. It **never** pushes to the default branch, and it never merges — that
stays the maintainer's call. `--dry-run` reports the whole plan, including the
branch it would use, without touching anything.

Because nothing lands on `main`, `ship` no longer runs `skill sync`; run
`construct skill sync` after the PR merges. `--no-sync` is retained as a hidden
no-op so existing invocations keep working. Read `construct-cli/AGENTS.md` before working inside
this subdirectory — it governs that subtree, not this file.

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
The module never clobbers a real directory it did not create, and prunes only
symlinks pointing into its own tree.

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

## Local agent fan-out (Home Manager hosts)

Local fan-out is managed by **Home Manager**, not by the assistant. Every
per-harness path is a symlink to the canonical `~/.agents/skills`, which under
`mutablePointer` chains through a mutable pointer into the store:

```
~/.claude/skills            → ~/.agents/skills
~/.agents/skills            → ~/.local/state/construct/current
~/.local/state/construct/current → …/pinned → /nix/store/<hash>-construct-skills
```

With `perSkillLinks.enable = true` the middle step changes shape: `~/.agents/skills`
is a **real directory** whose entries are per-skill symlinks into
`…/construct/current/<skill>`. Same content, but names the module does not carry
stay free for another installer to own — which is what an Orca host needs (see
*Vendored Orca skills* above).

The same option renders `~/.grok/skills` the same way (links straight into the
store — the Grok tree has no mutable pointer). Both trees go through one shared
renderer in `flake.nix`, so what gets clobbered and what gets pruned cannot
diverge between them.

Paths populated by Home Manager: `~/.claude/skills/`, `~/.codex/skills/`,
`~/.ai/skills/`, `~/.agent/skills/`. Gemini CLI's scan path is Home Manager's
responsibility on this host as well — the assistant does not provision it.

The Nix flake's `homeManagerModules.default` (in `flake.nix`) provides a
unified path layout for new consumers: install once to `~/.agents/skills/` and
symlink every per-harness path (`~/.claude/skills`, `~/.gemini/skills`,
`~/.codex/skills`, `~/.ai/skills`, `~/.agent/skills`) to that canonical
location. Grok skills install separately to `~/.grok/skills/` because of
their different bundle layout. This host is already on the module
(`spacecraft.construct` in `bravais/users/mj/home.nix`, with `mutablePointer`
and a longer `agentPaths` list).

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
