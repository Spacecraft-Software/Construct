---
name: steelbore-cli-shell
description: >
  Syntax-compliance guard for shell commands. Companion to steelbore-cli-preference
  (which picks the tool; this one checks the syntax around it). ALWAYS consult the
  first time in a conversation Claude is about to run, write, or suggest a shell
  command — one-liners, scripts, `.nu`/`.ion`/`.sh` files, CI blocks, README snippets,
  documentation. Also consult whenever the user mentions Nushell, Ion, Redox, POSIX,
  bashisms, shell portability, or a `$SHELL`. Detects the target shell (Nushell / Ion
  / POSIX sh / Bash), blocks Bash-only patterns that silently break elsewhere
  (`[[ ]]`, `(( ))`, `<(...)` process substitution, `${var^^}`, Bash arrays,
  `function` keyword, `source` for POSIX), and routes to the correct per-shell
  reference. Syntax priority — POSIX sh first, then shell-native (Ion or Nushell)
  where POSIX diverges, Bash extensions last.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@Steelbore.com>
website: https://Steelbore.com/
---

# Steelbore CLI Shell — Syntax Compliance Guard

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)
**Copyright:** (c) 2026 Mohamed Hammad | **License:** GPL-3.0-or-later
**Website:** [https://Steelbore.com/](https://Steelbore.com/)

Sibling skill to `steelbore-cli-preference`. That skill decides **which tool** to run (`rg` over `grep`, `eza` over `ls`); this skill makes sure the **syntax around it** parses in the target shell.

Two shells matter most here: **Nushell** (Mohamed's primary interactive + Lattice/Steelbore default) and **Ion** (Redox default, secondary). Both are Rust-written and neither accepts Bash scripts as-is. Bashisms that "always worked" in Bash will fail — sometimes loudly, sometimes silently — in either.

## When to consult

- **First shell command in a conversation** — always, regardless of how trivial it looks.
- Any time work involves `.nu`, `.ion`, or `.sh` files (reading, writing, editing).
- Any CI block, Makefile/`justfile` recipe, README snippet, or doc example containing shell.
- Any time the user names a shell (Nushell, Ion, bash, dash, zsh) or says "POSIX", "portable script", "bashism".

After the first consult in a session, trust the decision and proceed — don't re-load on every command unless the target shell changes or a new script file is opened.

## Step 1 — Detect the target shell

**Auto-detect first, ask only as last resort.** Try these signals in order, stop at the first that resolves. The goal is to never pause the conversation for a question that context already answers.

Signals 1–3 are **direct evidence** — commit silently, proceed.
Signal 4 is **inference** — commit *and announce the assumption in one line* so the user can correct if wrong.
Signal 5 is the only one that asks.

1. **File extension** *(evidence)* — `.nu` → Nushell; `.ion` → Ion; `.sh` → POSIX; `.bash` → Bash.
2. **Shebang** *(evidence)* — `#!/usr/bin/env nu` → Nushell; `#!/usr/bin/env ion` → Ion; `#!/bin/sh` or `#!/usr/bin/env sh` → POSIX; `#!/bin/bash` → Bash.
3. **Explicit user mention** *(evidence)* — "in Nushell", "my Ion script", "POSIX-compatible", "bash one-liner".
4. **Environmental inference** *(announce)* — state the assumption in a short sentence before/alongside the command, e.g. "Assuming Nushell (your primary) — say the word for Ion or POSIX.":
   - `bash_tool` in *this* Claude environment runs **Bash**. Commands executed here and now should target **POSIX** (Bash accepts all POSIX).
   - Steelbore / Lattice / "my shell" context with no other signal → **Nushell** (Mohamed's primary). When "my shell" is used explicitly, consider offering an **Ion** secondary since Mohamed runs both.
   - Redox OS context → **Ion**.
   - GitHub Actions `run:` block without `shell:` override → **Bash** on Linux/macOS runners, `pwsh` on Windows; default to POSIX-compliant for cross-platform.
5. **Still ambiguous** *(ask)* — one short question, commit to the answer for the rest of the conversation.

## Step 2 — Apply the priority order

Within the detected shell, emit constructs in this preference order:

| Rank | Syntax family | Use when |
|------|---------------|----------|
| 1 | **POSIX sh** | The target shell accepts POSIX (sh, dash, Bash, zsh, partially Ion). Maximally portable. |
| 2 | **Ion native** | Target is Ion *and* POSIX diverges (variable binding, arrays, functions, control flow). |
| 3 | **Nushell native** | Target is Nushell. Nushell rejects POSIX wholesale — skip straight to native. |
| 4 | **Bash extensions** | Last resort. Only when the target is confirmed Bash *and* no POSIX or native equivalent works. |

"POSIX first" means *prefer constructs that happen to be both POSIX and valid in the target shell* — not *write POSIX into a `.nu` file*. Nushell scripts get Nushell syntax; Ion scripts get Ion syntax; the POSIX preference applies to sh / bash / dash / zsh targets and to any portability crossroads.

**When the target is Bash, still write POSIX.** Bash accepts every POSIX construct, so a Bash target is not a license to reach for `[[ ]]`, `(( ))`, arrays, or `${var^^}`. Stay at rank 1 by default; drop to rank 4 only when POSIX genuinely cannot express the operation (e.g., `$PIPESTATUS`, process substitution, `shopt -s globstar`).

**When you do use a rank-4 Bash extension, announce it.** One short inline note is enough: "Using `$PIPESTATUS` here — Bash-only; no POSIX equivalent." This makes the deviation auditable and gives the user a chance to request a rewrite or accept the trade-off.

## Step 3 — Load the right reference before emitting

Once the target is known, read the matching file **before** writing the command. These files carry the gotchas — variable syntax, string interpolation, redirection, command substitution — that this SKILL.md deliberately does not duplicate.

- **Nushell** → `references/nushell.md`
- **Ion** → `references/ion.md`
- **POSIX sh / dash / bash-in-POSIX-mode** → `references/posix-safe.md`
- **Bashism translation** (when converting an existing Bash snippet to any other target) → `references/bashisms.md`

## Top bashisms that break silently in Nushell and Ion

These are the high-frequency offenders. Catching them up front saves a reference-file lookup:

- **`[[ cond ]]`** — Bash-only. Use `test` / `[ ]` in POSIX, `exists`/`test` in Ion, `if $var == ... { }` in Nushell.
- **`(( arith ))`** — Bash-only. POSIX: `$(( ... ))`. Ion: `let a += 1` style arithmetic on `let`. Nushell: direct expressions like `$x + 1`.
- **`${var^^}` / `${var,,}` / `${var//pat/rep}`** — Bash-only. No portable equivalent; use `tr` / `sed` (POSIX), Ion string methods (`$upper(var)`, `$replace(var ...)`), or Nushell (`$var | str upcase`, `$var | str replace ...`).
- **`arr=(a b c)` with `${arr[@]}`** — Bash-only array syntax. Ion: `let arr = [a b c]`, expand with `@arr`. Nushell: `let arr = [a b c]`, expand with `$arr`.
- **`<(cmd)` process substitution** — Bash/zsh only. Not in POSIX, Ion, or Nushell. Use a pipe or a tempfile instead.
- **`function name { ... }`** — Bash syntax. POSIX: `name() { ... }`. Ion: `fn name; ...; end`. Nushell: `def name [] { ... }`.
- **`&>file`** — Bash. POSIX: `>file 2>&1`. Ion supports `&>`. Nushell: `out+err> file`.
- **`source file`** — Bash. POSIX: `. file`. Ion and Nushell both accept `source`.
- **`$RANDOM`, `$SECONDS`, `$LINENO`** — Bash-only specials. Use `shuf -i`, `date +%s`, or shell-native equivalents.

Full table: `references/bashisms.md`.

## Handoff

This skill stops at syntax. Once the command parses in the target shell, `steelbore-cli-preference` takes over for tool substitutions (`grep` -> `rg`, `ls` -> `eza`, `cat` -> `bat`, etc.). Both skills are expected to be active simultaneously — neither duplicates the other.

If `nix-shell-provisioner` is also active and a tool needs to be provisioned, syntax compliance applies to the `nix-shell` invocation itself: the *inside* of `nix-shell -p pkg --run '...'` runs in Bash (it's `nix-shell`'s contract), so the `--run` payload follows POSIX/Bash rules regardless of the user's outer shell.
