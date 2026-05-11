# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A catalogue of agent skills loaded by Claude Code, Gemini CLI, and Codex from
`~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`. There is no build
system, runtime, or test suite — every artifact is markdown plus a small number
of templates/JSON. Edits are content edits; shipping is rezipping.

The authoritative governance document for everything produced in this repo is
[`steelbore-standard/SKILL.md`](steelbore-standard/SKILL.md) (The Steelbore
Standard v1.1). Load it before any non-trivial edit — its §14 checklist is the
audit gate.

## Skill layout (per top-level directory)

```
<skill-name>/
├── SKILL.md           # frontmatter (name, description, license, maintainer, website) + body
├── LICENSE | LICENSE.md  # GPL-3.0-or-later (rust-guidelines is MIT — adapted from Microsoft)
├── references/        # optional; loaded on demand by the agent
└── assets/            # optional; templates, JSON catalogs, etc.
```

Skill IDs (directory and frontmatter `name`) are **functional**, not
metallurgical — Standard §2 reserves codenames for projects/modules/utilities,
not for skill identifiers. The README catalogue and the directory list must
stay in sync.

## Bundling (.zip and .skill)

Each skill ships as two bundles at the repo root: `<name>.zip` and
`<name>.skill`. They contain only `SKILL.md`, `LICENSE`, and `references/`
(plus `assets/` where present) — never tooling, generator scripts, or raw
upstream sources. Auxiliary inputs that don't belong in the shipped skill live
in `Excluded/` (e.g., `Rust-Guidelines.{md,txt}`, `skill.ps1`).

Rebuild pattern (from `.claude/settings.local.json`):

```sh
rm -f <name>.zip <name>.skill
zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/references
zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/references
```

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
   zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/references
   zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/references
   ```
   Add `<name>/assets` to both lines if the skill has an `assets/` dir.
   Omit `<name>/LICENSE` or `<name>/references` if the skill doesn't have them
   (`steelbore-standard` is SKILL.md-only, for example).
2. **Stage** the skill directory **and** both bundles in the same commit —
   never separately.
3. **Commit with UTC timestamps**:
   ```sh
   TZ=UTC GIT_COMMITTER_DATE="$(TZ=UTC date)" \
     git commit --date "$(TZ=UTC date)" -m "..."
   ```
   Steelbore Standard §12.2 forbids offset notation (`+0300`, `+00:00`); only
   `Z` / `+0000` is permitted. Signing is on globally
   (`commit.gpgsign=true`, `gpg.format=ssh`, `user.signingkey=~/.ssh/id_ed25519.pub`)
   — no extra flag needed.
4. **Push** to `origin/main` with no confirmation prompt — this repo is
   pre-authorised for auto-push on skill-directory changes.
5. **Fan out to local agent install dirs** so every agent CLI on this host
   sees the pushed state:
   ```sh
   rsync -a --delete \
     --exclude='.git' --exclude='.claude' --exclude='Excluded' \
     --exclude='Chat.txt' --exclude='CLAUDE.md' --exclude='.gitignore' \
     --exclude='*.zip' --exclude='*.skill' \
     /steelbore/construct/ ~/.agents/skills/
   ```
   `~/.agents/skills/` is the canonical local install. `~/.ai/skills`,
   `~/.agent/skills`, `~/.claude/skills`, and `~/.codex/skills` are
   **symlinks** to it (set up once per host — see the *Local agent fan-out*
   section below). They pick up the new content automatically; no per-target
   rsync needed. `~/.gemini/skills` is deliberately **not** a symlink — Gemini
   CLI scans `~/.agents/skills/` directly and would emit duplicate-skill
   warnings if it saw both.

If multiple skills changed in one turn, rebuild **all** of their bundles in the
same commit. Never let `git status` show a skill-dir change without its
matching bundle change.

`git log -1 --show-signature` may report "No signature" locally because
`~/.ssh/allowed_signers` isn't populated; this is a verifier-side gap, not a
signing failure. GitHub validates the SSH signature independently and shows
"Verified" if the public key is registered as a **Signing** key in GitHub
account settings (Authentication-only keys won't validate signatures).

## Local agent fan-out

A single canonical skills directory at `~/.agents/skills/` feeds every agent
CLI on this host. Step 5 of the workflow rsyncs the repo into it; the
per-harness conventional paths are symlinks pointing back at it.

| Path                | Kind                       | Target              | Why |
|---------------------|----------------------------|---------------------|-----|
| `~/.agents/skills/` | directory (canonical)      | (content)           | Source of truth — rsync target in step 5 |
| `~/.ai/skills`      | symlink                    | `~/.agents/skills`  | Generic AI-tool convention |
| `~/.agent/skills`   | symlink                    | `~/.agents/skills`  | Generic single-`agent` convention |
| `~/.claude/skills`  | symlink                    | `~/.agents/skills`  | Claude Code reads from here |
| `~/.codex/skills`   | symlink                    | `~/.agents/skills`  | OpenAI Codex reads from here |
| `~/.gemini/skills`  | **must not exist**         | —                   | Gemini CLI is configured to scan `~/.agents/skills/` natively; a symlink at this path would cause Gemini to enumerate every skill twice and emit duplicate-skill warnings. |

### One-time setup (per host)

Run once on a fresh box, or after retiring a real `~/.<harness>/skills` clone.
If any of the four target paths is currently a real directory (e.g. from the
README install instructions), it gets renamed aside before being replaced
with a symlink:

```sh
mkdir -p ~/.agents
for d in ~/.ai/skills ~/.agent/skills ~/.claude/skills ~/.codex/skills; do
  if [ -e "$d" ] && [ ! -L "$d" ]; then
    mv "$d" "${d}.pre-symlink.$(TZ=UTC date -u +%Y%m%dT%H%M%SZ)"
  fi
  mkdir -p "$(dirname "$d")"
  ln -sfn ~/.agents/skills "$d"
done
```

Verify: `ls -la ~/.ai/skills ~/.agent/skills ~/.claude/skills ~/.codex/skills`
should print four `... -> /home/<you>/.agents/skills` lines. `ls ~/.gemini/skills`
should error with *No such file or directory* — if it doesn't, remove whatever
is there before Gemini next loads.

The `--delete` in step 5's rsync means anything sitting in `~/.agents/skills/`
that isn't in the repo gets removed on the next sync. If you're prototyping a
skill locally before adding it to the repo, keep the work tree somewhere
**outside** `~/.agents/skills/` until it lands in `/steelbore/construct/`.

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

## Installation (what consumers do)

```sh
git clone git@github.com:Steelbore/Construct.git ~/.claude/skills
git clone git@github.com:Steelbore/Construct.git ~/.gemini/skills
git clone git@github.com:Steelbore/Construct.git ~/.codex/skills
```

The SSH remote is configured for [Gitway](https://github.com/Steelbore/Gitway).
