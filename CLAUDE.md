# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A catalogue of agent skills loaded by Claude Code, Gemini CLI, and Codex from
`~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`. There is no build
system, runtime, or test suite — every artifact is markdown plus a small number
of templates/JSON. Edits are content edits; shipping is rezipping.

The authoritative governance document for everything produced in this repo is
[`spacecraft-standard/SKILL.md`](spacecraft-standard/SKILL.md) (The Spacecraft Software
Standard v1.2). Load it before any non-trivial edit — its §14 checklist is the
audit gate.

## Skill layout (per top-level directory)

```
<skill-name>/
├── SKILL.md           # frontmatter (name, description, license, maintainer, website) + body
├── LICENSE | LICENSE.md  # GPL-3.0-or-later (rust-guidelines is MIT — adapted from Microsoft)
├── CREDITS.md         # required when the skill builds on third-party work (Standard §13.3)
├── references/        # optional; loaded on demand by the agent
└── assets/            # optional; templates, JSON catalogs, etc.
```

Skill IDs (directory and frontmatter `name`) are **functional identifiers**,
not codenames — Standard §2.2 reserves codenames for projects/modules/utilities/
releases, not for skill identifiers. The README catalogue and the directory list
must stay in sync.

## Bundling (.zip and .skill)

Each skill ships as two bundles at the repo root: `<name>.zip` and
`<name>.skill`. They contain only `SKILL.md`, `LICENSE`, `CREDITS.md`, and
`references/` (plus `assets/` where present) — never tooling, generator
scripts, or raw upstream sources. Auxiliary inputs that don't belong in the
shipped skill live in `Excluded/` (e.g., `Rust-Guidelines.{md,txt}`,
`skill.ps1`).

Rebuild pattern (from `.claude/settings.local.json`):

```sh
rm -f <name>.zip <name>.skill
zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
```

Include each argument only when that file/dir exists in the skill — LICENSE
filenames vary (`LICENSE` vs `LICENSE.md`), `CREDITS.md` appears only where
§13.3 triggers fire (currently `rust-guidelines` and `spacecraft-cli-preference`),
and `references/` is optional.

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
   Add `<name>/assets` to both lines if the skill has an `assets/` dir.
   Omit any argument the skill doesn't have — `spacecraft-standard` is
   SKILL.md-only; LICENSE filenames vary (`LICENSE` vs `LICENSE.md`);
   `CREDITS.md` exists only where §13.3 applies (`rust-guidelines`,
   `spacecraft-cli-preference`). Run `ls <name>/` first when in doubt.
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
   Spacecraft Software Standard §12.2 forbids offset notation (`+0300`, `+00:00`); only
   `Z` / `+0000` is permitted. Signing is on globally
   (`commit.gpgsign=true`, `gpg.format=ssh`, `user.signingkey=~/.ssh/id_ed25519.pub`)
   — no extra flag needed.
4. **Push** to `origin/main` with no confirmation prompt — this repo is
   pre-authorised for auto-push on skill-directory changes.

The assistant's responsibility ends at push. **Local agent install dirs
refresh only after Home Manager rebuilds** — see *Local agent fan-out* below.
That rebuild is a user-initiated action; the assistant must not run it.

If multiple skills changed in one turn, rebuild **all** of their bundles in the
same commit. Never let `git status` show a skill-dir change without its
matching bundle change.

`git log -1 --show-signature` may report "No signature" locally because
`~/.ssh/allowed_signers` isn't populated; this is a verifier-side gap, not a
signing failure. GitHub validates the SSH signature independently and shows
"Verified" if the public key is registered as a **Signing** key in GitHub
account settings (Authentication-only keys won't validate signatures).

## Local agent fan-out (this host)

Local fan-out is managed by **Home Manager**, not by the assistant. Each
per-harness skill path is a real directory provisioned by Home Manager with
per-skill symlinks that chain through the Nix store to this repo:

```
~/.claude/skills/<skill>
  → /nix/store/<hash>-home-manager-files/.claude/skills/<skill>
  → /nix/store/<hash>-hm_<skill>
  → /spacecraft-software/construct/<skill>
```

Paths populated by Home Manager: `~/.claude/skills/`, `~/.codex/skills/`,
`~/.ai/skills/`, `~/.agent/skills/`. Gemini CLI's scan path is Home Manager's
responsibility on this host as well — the assistant does not provision it.

**After a push, Home Manager must be rebuilt** before per-harness paths
resolve to the new content. The maintainer runs the rebuild manually
(`home-manager switch …` or the equivalent flake command); the assistant
does **not** invoke it.

Verify after rebuild:

```sh
readlink -f ~/.claude/skills/spacecraft-standard
# → /spacecraft-software/construct/spacecraft-standard
```

If the symlink still resolves into a stale `/nix/store/<old-hash>-hm_*` path
(for example after a repo rename), the Home Manager config has not yet been
rebuilt against the new repo path. Until it is, agents read the previous
generation's content even though `origin/main` is current.

The assistant performs no `rsync`, no symlink setup, and no
`home-manager switch`. Its responsibility ends at `git push`.

## Editing rules specific to this repo

- **README §2 catalogue is load-bearing.** When adding a skill directory, add a
  matching alphabetical row to the table in `README.md`. When removing one,
  delete the row.
- **Dates are ISO 8601 UTC** anywhere they appear in SKILL.md, references, or
  changelogs (Standard §12). No AM/PM, no local-time strings.
- **Don't import skill content into `MEMORY.md` or CLAUDE.md.** The skills are
  the source of truth and are already loaded on demand.
- `Chat.txt` is a session export and is gitignored — never commit it.
- `Excluded/` is the holding pen for inputs that produce skill content but must
  not ship with it. Don't reference it from inside any `SKILL.md`.
- **Pre-existing dirty `.skill` files** (`spacecraft-agentic-cli.skill`,
  `spacecraft-cli-shell.skill`, `spacecraft-cli-standard.skill`,
  `spacecraft-missing-pkg.skill`) carry uncommitted changes from prior `-D`
  normalization passes. Do not commit them unless the task explicitly targets
  those files.

## Installation (what consumers do)

```sh
git clone git@github.com:Spacecraft-Software/Construct.git ~/.claude/skills
git clone git@github.com:Spacecraft-Software/Construct.git ~/.gemini/skills
git clone git@github.com:Spacecraft-Software/Construct.git ~/.codex/skills
```

The SSH remote is configured for [Gitway](https://github.com/Spacecraft-Software/Gitway).
