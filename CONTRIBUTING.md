# Contributing to Construct

Construct is the **skill catalogue** for the Spacecraft Software ecosystem — a
collection of agent skills (markdown plus a few templates/JSON) loaded by Claude
Code, Gemini CLI, Codex, and Grok. There is no build system, runtime, or test
suite: every artifact is content, and "shipping" is rebuilding the bundle.

This document is the **version-controlled workflow** for changing a skill. It is
the contributor-facing companion to the terse [`AGENTS.md`](AGENTS.md) digest.
(A maintainer-local `CLAUDE.md` overlay also exists on the maintainer's host with
host-specific notes; it is intentionally gitignored and not required to
contribute.)

For project stance and general etiquette, see the umbrella
[CONTRIBUTING](https://github.com/Spacecraft-Software) guidance — Spacecraft
Software is a personal hobby project; external input is welcome but discretionary.

## The one rule that matters: bundles are the install surface

Each skill ships as two bundles at the repo root: `<name>.zip` and `<name>.skill`.
Consumers install from these. **A bundle that lags its `SKILL.md` / `references/`
ships broken content to every consumer.** So the contract is mechanical: after
*any* edit inside a `<skill-name>/` directory, rebuild both bundles **in the same
commit** as the source change.

## Skill layout

```
<skill-name>/
├── SKILL.md              # frontmatter (name, description, …) + body
├── LICENSE | LICENSE.md  # optional in-dir copy; ~half the skills omit it and
│                         #   fall back on the repo-root LICENSE
├── CREDITS.md            # required when the skill builds on third-party work
│                         #   (Standard §15.3)
├── references/           # optional; loaded on demand by the agent
└── assets/               # optional
```

Skill IDs (directory name and frontmatter `name`) are **functional identifiers**,
not codenames (Standard §2.2 reserves codenames for projects, not skill IDs).

## Bundling (`.zip` and `.skill`)

Bundles contain only `SKILL.md`, `LICENSE`, `CREDITS.md`, and `references/` (plus
`assets/` where present) — never tooling, generator scripts, or raw upstream
sources (those live in `Excluded/`, which is gitignored).

```sh
rm -f <name>.zip <name>.skill
zip -qr  <name>.zip   <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
zip -qrD <name>.skill <name>/SKILL.md <name>/LICENSE <name>/CREDITS.md <name>/references
```

Include each argument only when that file/dir exists in the skill — run
`ls <name>/` first when unsure. Add `<name>/assets` to both lines if the skill has
one. The `.skill` bundle uses `-D` to drop directory entries; the `.zip` keeps
them. The two are built from the **same args in the same commit**, so they never
diverge. Verify with `unzip -l <name>.zip` before committing.

## Workflow: every skill-directory change

1. **Rebuild both bundles** for the changed skill (commands above). If multiple
   skills changed in one turn, rebuild **all** of their bundles.
2. **Stage explicitly by name** — the skill files **and** both bundles together:
   ```sh
   git add <name>/SKILL.md <name>.zip <name>.skill
   ```
   **Never `git add -A` / `git add .`** — other root `.skill` files may carry
   unrelated uncommitted changes that must not be swept into your commit.
3. **Commit in UTC.** The Steelbore Standard §14.2 forbids offset notation
   (`+0300`); only `Z` / `+0000` is permitted:
   ```sh
   TZ=UTC GIT_COMMITTER_DATE="$(TZ=UTC date)" \
     git commit --date "$(TZ=UTC date)" -m "..."
   ```
4. **Sign every commit (mandatory).** Commits to a Spacecraft Software remote must
   be cryptographically signed and show "Verified" on GitHub (Standard §6.3).
   Ed25519 SSH signing is the default (`commit.gpgsign=true`, `gpg.format=ssh`,
   with the signing key registered as a **Signing** key on GitHub — auth-only keys
   do not validate signatures). Assistant-driven commits add a
   `Co-Authored-By: …` trailer; human commits do not.
5. **Push** to `origin/main`:
   ```sh
   git push https://github.com/Spacecraft-Software/Construct.git main
   ```

Never let `git status` show a skill-dir change without its matching bundle change.

## Detecting already-committed drift

A clean working tree does **not** prove the bundles are current: a past commit can
bump `SKILL.md` while forgetting the bundle, leaving a committed `.zip`/`.skill`
that silently lags its source. `git status` can't see it. Before trusting the
install surface, sweep:

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

Any `DRIFT:` line means rebuild that skill's bundles and commit. The sweep checks
`.zip` as the canonical surface; `.skill` is built in lockstep, so a drifted `.zip`
implies a drifted `.skill`.

## Grok skills (`grok-skills/`)

Grok uses a **flat** bundle format — `SKILL.md` and any `assets/` / `references/`
live at the **root** of the `.zip`, not inside a `<skill-name>/` directory. Build
from inside the skill directory so the paths land at the zip root:

```sh
cd grok-skills/<name>
rm -f ../<name>.zip ../<name>.skill
zip -qr  ../<name>.zip   SKILL.md [assets] [references]
zip -qrD ../<name>.skill SKILL.md [assets] [references]
```

Verify the zip top level is `SKILL.md` (not `<name>/SKILL.md`) before committing —
a nested layout breaks Grok's loader. The same staging/commit contract applies.
Grok bundles ship **inside** `grok-skills/`, and `grok-skills/README.md` must stay
in sync with the subdirectory listing.

## Editing rules

- **`SKILL.md` frontmatter `description` — hard cap 1000 characters.** The loader's
  absolute limit is 1024 (it rejects anything longer), but this repo caps the
  *rendered* description at **1000** — a description MUST NOT exceed 1000 chars.
  YAML folded scalars (`description: >`) join lines with spaces and add a trailing
  newline, so the **rendered** length is what counts. Re-check after any
  description edit:
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
  The [pre-commit hook](#pre-commit-hook) enforces this automatically on every
  staged skill — no need to remember the snippet, but do activate the hook.
- **`microsoft-rust-guidelines` is intentionally `user-invocable: false`.** It is
  the mandatory auto-load Rust base — `spacecraft-standard-constitution` mandates loading it
  before any Rust and `spacecraft-rust-guidelines` defers to it as "load first," so
  it fires automatically and is hidden from the `/` menu on purpose. Do **not**
  remove the field to "fix" a perceived load failure — that is by design.
- **`README.md` §2 catalogue is load-bearing.** Adding a skill directory means
  adding a matching **alphabetical** row to the table; removing one means deleting
  the row. The README catalogue and the directory list must always agree.
- **Dates are ISO 8601 UTC** wherever they appear (SKILL.md, references,
  changelogs) — `YYYY-MM-DD`, no AM/PM or local-time strings (Standard §14).
- **Licensing / REUSE.** Skills are software-class → `GPL-3.0-or-later` by default
  (Standard §4.1.1); third-party-derived skills preserve their upstream license
  (`microsoft-rust-guidelines` is MIT, `gnu-coding-standards` is GFDL-1.3-or-later).
  Coverage is via `REUSE.toml` (the `**` default plus per-skill overrides); ship
  license texts in `LICENSES/`. `reuse lint` must pass.
- **Don't import skill content into other docs.** The skills are the source of
  truth and load on demand; don't copy their bodies into README, AGENTS.md, or
  CONTRIBUTING.md.
- **`Excluded/`** is the holding pen for inputs that produce skill content but must
  not ship with it; never reference it from inside a `SKILL.md`. Session exports
  (`Chat*.txt`) are gitignored — never commit them.

## Pre-commit hook

A tracked hook enforces the description cap so a stale or over-long count can
never reach a commit. Git does **not** honour tracked hooks automatically, so
activate it **once per clone**:

```sh
git config core.hooksPath .githooks
```

On every `git commit`, `.githooks/pre-commit` checks the **staged** `SKILL.md`
files (root and Grok) with `.githooks/check-description-length.py`, which
reproduces the folded-scalar rendering above and aborts the commit if any
rendered description exceeds **1000** characters. It validates the *staged blob*,
not the working tree, so a fixup you forgot to re-stage is still caught. The
only dependency is `python3`.

Run the same checker by hand over the whole catalogue any time:

```sh
python3 .githooks/check-description-length.py */SKILL.md grok-skills/*/SKILL.md
```

It exits non-zero and lists each offender (with how many chars over) when any
description is too long, and is silent for files with no frontmatter or no
`description` key.

## Nix flake

The repo is also a Nix flake. Skill auto-detection is by `SKILL.md` presence, so
**adding a new skill directory needs no flake edit**. `flake.lock` is tracked and
must be committed when it changes.

*— Built by Spacecraft Software —*
