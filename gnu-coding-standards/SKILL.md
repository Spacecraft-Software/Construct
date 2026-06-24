---
name: gnu-coding-standards
description: >-
  Apply the GNU Coding Standards faithfully — including the free-software philosophy
  (GPL licensing, FSF copyright assignment, Texinfo-canonical documentation, the
  policy against promoting nonfree software) — extended so GNU's conventions are
  expressed idiomatically in C, Rust, GNU Guile, Go, and Python. The gate
  is GNU intent: consult this skill whenever the user is writing, reviewing,
  planning, or auditing software meant to be GNU-compliant, GNU-targeted, or a GNU
  package. Triggers include "GNU coding standards", "make this
  GNU-compliant", "GNU-standard", "I'm writing a GNU package/program", "should this
  conform to GNU conventions", or any request that code follow GNU practice. Do NOT
  use it for general-purpose coding, for Spacecraft Software / Steelbore projects
  (which have their own Standard), or for idiomatic-language questions that carry no
  GNU intent — a request to "write a Guile worker pool" or "make a Rust CLI" is not a
  GNU request unless GNU is named or clearly intended.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# GNU Coding Standards (C, Rust, Guile, Go, Python)

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

This skill encodes the GNU Coding Standards (Stallman et al., last updated April 2026)
and adapts them to five languages. It is **advisory**: when it triggers, write and
review code so the result would be accepted as a GNU package, expressing GNU's
intent idiomatically rather than transliterating C into other languages.

The GNU Coding Standards were written around C, and the source document predates the
prominence of Rust and Go in GNU contexts — it never mentions either. So this skill
distinguishes two things: GNU's **principles** (robustness, error-checking, the
user-facing CLI contract, the free-software philosophy), which are language-agnostic
and apply everywhere; and GNU's **C-specific mechanics** (brace placement, `getopt_long`,
Gnulib), which each language realizes through its own idioms. When in doubt, ask "what
is GNU trying to achieve here?" and satisfy *that* in the target language.

## When this applies — and when it does not

This is a narrow skill. It applies only when the work is GNU-targeted: the user has
said the software should be GNU-compliant, should follow the GNU Coding Standards,
is destined to become a GNU package, or is being audited against GNU practice.

It does **not** apply to general coding, and it does **not** apply to the user's own
Spacecraft Software / Steelbore projects — those have their own Standard, host on
GitHub, and make their own (different) policy choices. If there is no GNU framing,
this skill stays out of the way. The presence of Rust, Guile, Go, or Python is not a
trigger; GNU intent is the trigger.

## Two GNU postures (compatible vs. compliant)

GNU intent comes in two strengths:

- **GNU-compatible** — a Spacecraft/Steelbore project that adopts GNU's conventions where
  they aid interoperability while **staying itself**: on GitHub, under the Steelbore
  palette/brand and `Mohamed.Hammad@SpacecraftSoftware.org` attribution. `spacecraft-standard`
  stays primary; **this skill** supplies the GNU conventions for interop, and the
  free-software *political* mandates are advisory here — Steelbore identity stands. This is
  this skill's home turf.
- **GNU-compliant (full free software)** — the artifact's goal is to *be* a GNU package
  (official GNU, upstreamed to GNU/FSF, on Savannah), shedding Steelbore identity for GNU.
  Use the **`gnu-free-software`** skill — a self-sufficient, distributable skill that produces
  and enforces free software in the FSF/GNU tradition (free license + `COPYING`, the
  JavaScript Trap, GNU vocabulary, the full GNU-package contract). It stands alone and carries
  its own conventions, so you do **not** also load this skill under that posture.

The differentiator is identity-stripping: producing a real free-software/GNU package →
`gnu-free-software`; styling Spacecraft code to GNU conventions while staying Steelbore →
this skill (with `spacecraft-standard`).

## How to use this skill

1. **Always read `references/conventions.md` first.** It holds the language-agnostic
   mechanics that every GNU program must get right: the error-message grammar, the
   exact `--version` / `--help` output contract, the long-option conventions,
   internationalization, documentation (Texinfo), ChangeLog/NEWS style, and the
   release process (`configure`, Makefile conventions, directory variables, standard
   targets, `DESTDIR`).
2. **Then read the relevant language file(s)** in `references/`: `c.md`, `rust.md`,
   `guile.md`, `go.md`, `python.md`. Each shows how GNU's goals are met idiomatically
   in that language, and where the language's own canonical style legitimately
   overrides GNU's C-era mechanics.
3. For **Rust** and **Guile**, the language files cover only the GNU-package layer.
   For idiomatic Rust, also consult the `microsoft-rust-guidelines` skill; for idiomatic /
   concurrent Guile, also consult `spacecraft-guile-guidelines`. This skill governs
   *GNU conformance*; those skills govern *language craft*. They compose.

## Keeping free software free (this is not optional under full fidelity)

The user has asked for full GNU fidelity, so the free-software philosophy carries the
same weight as the technical conventions. A GNU package is a political artifact as much
as a technical one.

- **Licensing.** Use a GNU license, normally **GPL-3.0-or-later** for programs and the
  **GFDL** for manuals more than a few pages long. Put the full license in `COPYING`
  (and `COPYING.LESSER` if LGPL). Every source file carries a copyright notice and a
  short GPL header; SPDX identifiers are fine alongside, not instead of, the header.
- **Copyright assignment.** For a package whose copyright is held by the FSF, nontrivial
  contributions require signed legal papers *before* the code is merged — a few lines
  here and there do not, and an idea you reimplement yourself does not. When advising on
  accepting contributions to such a package, say this plainly rather than glossing over
  it. (The most damaging mistake is forgetting to record who a contributor was.)
- **Don't lean on proprietary programs.** Don't study proprietary source to write a GNU
  imitation; reorganize along different lines. Don't recommend, promote, or grant
  legitimacy to nonfree software or services. Mentioning a well-known nonfree system in
  passing (e.g. how to build on it) is fine; promoting it, or linking to a page that
  *requires* running nonfree JavaScript to use, is not. This is why GNU pages avoid
  linking to `github.com` (account creation needs nonfree JS) and to `wix.com`-style
  sites — quote excerpts instead of linking.
- **Don't recommend nonfree documentation** for free software, and don't recommend free
  programs (like `mplayer`) that strongly steer users toward nonfree codecs/add-ons.
- **Trademarks and terms.** Don't include trademark acknowledgments. Don't abbreviate
  Microsoft Windows as "win" (write it in full, or `w`/`w32` in symbols). Never use the
  word **"steward"** for anyone developing free software — the EU CRA gives it a legal
  meaning with painful obligations.
- **Plug-ins**, if the program has them, must affirm a GPL-compatible license through a
  programmatic check (the GNU convention is a `plugin_is_GPL_compatible` symbol or its
  equivalent in the host language).

Full detail and the rationale for each point live in `references/conventions.md` and the
GNU source; the bullets above are the operating summary.

## Choosing a language (GNU's stated hierarchy, kept faithfully)

GNU §3.1 has real opinions, and full fidelity means preserving them:

- For **compiled, high-speed** code, GNU's default is **C**. C++ is acceptable but avoid
  heavy template use; compiled Java is acceptable.
- When peak efficiency is not required, **Lisp, Scheme, Python, Ruby, and Java** are all
  acceptable.
- **GNU Guile (Scheme) is the preferred extension language** and "the path that will lead
  to overall consistency of the GNU system." When the task is to make a C/C++ program
  extensible, Guile is the sanctioned choice — prefer it over embedding Python/Perl.

**On Rust and Go:** the GNU Coding Standards document does not mention either language.
This skill applies GNU's *principles* to them — and Rust's memory safety and Go's explicit
error returns satisfy several GNU robustness goals naturally — but be honest that GNU has
not officially blessed them, and that for a *GNU extension language* specifically, Guile
remains the endorsed choice. Don't imply GNU endorses what it has not.

## Universal robustness goals (realized per-language)

These are goals, not C recipes. Each language file shows the idiomatic realization;
`references/conventions.md` gives the full statement. The core set:

- **No arbitrary limits.** Allocate dynamically; never silently truncate long lines,
  names, or inputs. (Vec/String, slices/maps, lists/vectors, lists/dicts — not fixed
  buffers.)
- **Check every error return**, and include the system error text plus the file name and
  the program name in the message. "cannot open foo.c" alone is not enough.
- **Handle allocation failure**: fatal in a noninteractive program; in an interactive one,
  abort the command and return to the command loop so the user can free memory and retry.
- **Handle the full byte/character range**: don't drop NUL or nonprinting characters; work
  correctly with UTF-8 / multibyte text.
- **On "impossible" conditions, abort** rather than printing a message — these are bugs;
  explain the invariant in a comment by the check.
- **Don't use an error count as the exit status** (status is 8 bits; 256 errors wraps to 0).
- **Behavior must not depend on the invocation name** (`argv[0]`) or on whether output is a
  terminal vs. a pipe — use options/config to select behavior, with narrow, documented
  exceptions (e.g. refusing to dump binary to a TTY, overridable with `-f`).
- **Operate with `/usr` and `/etc` read-only**; honor `TMPDIR`; create temp files safely.

## What lives where

- **Free-software philosophy, robustness goals, language choice** — this file.
- **Error-message grammar, `--version`/`--help` contract, long options, i18n,
  documentation/Texinfo, ChangeLog/NEWS, release process & Makefile/directory
  conventions** — `references/conventions.md`.
- **Per-language realization** (formatting tool, naming, error handling, option parsing,
  i18n binding, license header, build wrapper, gotchas) — `references/{c,rust,guile,go,python}.md`.
