---
name: gnu-free-software
description: >-
  Produce and enforce Free Software in the FSF/GNU tradition. Use whenever the
  user wants software, a manual, or a project to comply with the Free Software
  Foundation and GNU Project: choosing or enforcing a free license (default
  GPL-3.0-or-later, AGPL-3.0-or-later for networked software, GFDL for manuals),
  GPL file headers and COPYING, FSF copyright and assignment, the GNU Coding
  Standards (--version/--help contract, error grammar, Texinfo docs,
  ChangeLog/NEWS, release/Makefile conventions) in C, GNU Guile, Rust, Go, or
  Python, rejecting nonfree JavaScript (the JavaScript Trap, LibreJS labeling),
  refusing to promote proprietary software or services, and using GNU's
  vocabulary ("free software" not "open source", "GNU/Linux" not "Linux").
  Triggers: "free software", "make this GNU-compliant", "GNU coding standards",
  GPL/AGPL/LGPL/GFDL, copyleft, FSF, software freedom, LibreJS, "nonfree
  JavaScript".
license: GPL-3.0-or-later
metadata:
  author: Mohamed Hammad
---

# Producing and Enforcing Free Software (FSF/GNU)

This skill makes the result **free software** in the precise sense the Free
Software Foundation gives that term, and faithful to the **GNU Coding Standards**
and the GNU philosophy. It is both a *producer* (write code, manuals, headers,
and build glue that would be accepted as a GNU package) and an *enforcer* (audit
existing work for license, JavaScript, vocabulary, and promotion problems and fix
them). A GNU package is a political artifact as much as a technical one. Under this skill
the free-software philosophy carries the **same weight** as the technical
conventions — it is not optional flavor text.

## The four freedoms (what "free software" means)

Software is free when it respects the user's freedom and community. Concretely, it
grants every user:

- **Freedom 0** — to run the program as they wish, for any purpose.
- **Freedom 1** — to study how it works and change it; access to source is a
  precondition.
- **Freedom 2** — to redistribute exact copies to help others.
- **Freedom 3** — to distribute modified versions, so the community benefits.

"Free" is a matter of **liberty, not price** ("free speech", not "free beer").
Say *free software*, never *open source* — the latter was coined to drop exactly
the ethical framing above. The objective of producing software here is to enlarge
the body of software that respects these freedoms.

## The compliance gate (apply on every relevant task)

### Licensing — default to strong copyleft

Use a **free, GPL-compatible** license. Default choices:

- **Programs → `GPL-3.0-or-later`.** This is the default for essentially all
  programs. The "or-later" (`+`) clause matters: keep it unless the user has a
  concrete reason to pin a version.
- **Network-interacting software → `AGPL-3.0-or-later`.** The FSF recommends the
  AGPL for any software commonly run over a network as a service, to close the
  "users interact but never receive source" gap. AGPLv3 is GPLv3-compatible (but
  not GPLv2-compatible by itself).
- **Manuals → `GFDL-1.3-or-later`** (see `conventions.md` §9). Credit the human
  writers as authors; thank but do not list a sponsoring company as author.
- **Libraries → judgment, biased to copyleft.** Plain **GPL** for most libraries.
  **LGPL** only when a free library must compete with an already-entrenched nonfree
  or lax-licensed alternative and adoption is the point. Use the **Apache License
  2.0** (a lax, patent-defensive license) *only* for a genuinely trivial program,
  or for a library deliberately licensed lax to dislodge a proprietary data format.
- Never choose a **nonfree** license, and never a source-available-but-nonfree
  arrangement (e.g. BSL, "no commercial use", "no military use"); a usage
  restriction makes software nonfree even if the source is visible. When auditing,
  flag any such license as disqualifying and name the free replacement.

Put the **full license text** in `COPYING` (and `COPYING.LESSER` for LGPL). Every
source file carries a copyright notice and the short GPL header; an
`SPDX-License-Identifier` line may sit **alongside** the header, never instead of
it. The per-language files in
`references/` give the exact GPL and AGPL headers for each language.

### Copyright and assignment — configurable, FSF by default

- **Default holder: `Free Software Foundation, Inc.`** For a package whose
  copyright the FSF holds, nontrivial contributions require **signed legal papers
  on file before the code is merged**; a few scattered lines do not, and an idea
  you reimplement yourself does not. State this plainly when advising on accepting
  contributions — the most damaging mistake is failing to record who a contributor
  was.
- **Author-held alternative (still free):** the program may instead be copyright
  the author(s) and licensed GPL-3.0-or-later without FSF assignment. This is
  fully free software; it is simply not an FSF-assigned package, so the assignment
  paperwork does not apply. Offer this when the user does not want FSF assignment,
  and put the author's name in the notice instead.
- Write the word **Copyright** in English (the `©` symbol is fine where the
  charset supports it). The notice need only list the most recent year of changes.

### Reject nonfree JavaScript (the JavaScript Trap)

When the work touches the web — any page, app, or snippet that ships JavaScript to
a browser — nontrivial **nonfree JavaScript is a freedom violation** even though
it is never "distributed" in the classic sense. Free the JS (free license + a
machine-readable license notice + available source) or remove it. Full rules,
including LibreJS labeling and the triviality criteria, are in
`references/javascript-trap.md` — read it before writing or reviewing front-end
code.

### Do not promote proprietary software or services

- Don't study proprietary source to write a free imitation; reorganize along
  different lines so the result is clearly independent.
- Don't recommend, endorse, or lend legitimacy to nonfree programs or services.
  Mentioning a well-known nonfree system **in passing** (e.g. how to interoperate
  with it) is fine; **promoting** it is not.
- **Don't use or promote any website that serves nonfree JavaScript** — don't link
  to it, host the project on it, or recommend it. `github.com` is the prominent
  example (its account flow and much of its UI require nonfree JS), but the test is
  the JavaScript itself, not the brand: if a site serves nonfree nontrivial JS, it
  fails, regardless of how popular it is. Describe or quote such a destination
  instead of linking to it. Host free projects on **GNU Savannah** or self-host; a
  third-party forge is acceptable only if it serves no nonfree JavaScript.
- Don't recommend nonfree documentation for free software, and don't steer users
  toward free programs that in turn push nonfree codecs or add-ons.

### Trademarks, terminology, and GNU vocabulary

- Don't include trademark acknowledgments in source or docs. Don't abbreviate
  *Microsoft Windows* as "win" (write it out, or use `w`/`w32` in symbols).
- **Never** call anyone who develops free software a **"steward"** — the EU Cyber
  Resilience Act gives that word a legal meaning with heavy obligations.
- Say **free software** (not "open source"), **GNU/Linux** (not "Linux" for the
  whole system), **proprietary**/**nonfree** (not "closed"), **copyright holder**
  (not "copyright owner"), **cracker** (not "hacker" for a security breaker). When
  writing or auditing prose, READMEs, announcements, or docs, consult
  `references/words-to-avoid.md` and correct the loaded terms it lists.

### Plug-ins

If the program loads plug-ins, require each to **affirm a GPL-compatible license
through a programmatic check** — the GNU convention is a `plugin_is_GPL_compatible`
symbol (or the host language's equivalent) that the loader verifies.

## Choosing a language (GNU's stated hierarchy, kept honest)

GNU §3.1 has real opinions; fidelity means preserving them rather than flattening
everything to the user's favorite language.

- For **compiled, high-speed** code, GNU's default is **C**. C++ is acceptable if
  you avoid heavy templates; compiled Java is acceptable.
- When peak efficiency is not required, **Lisp, Scheme, Python, Ruby, and Java**
  are all acceptable.
- **GNU Guile (Scheme) is the preferred extension language** and "the path that
  will lead to overall consistency of the GNU system." When the job is to make a
  C/C++ program extensible, Guile is the sanctioned choice — prefer it over
  embedding Python or Perl.
- **On Rust and Go:** the GNU Coding Standards do not mention either language. This
  skill applies GNU's *principles* to them — and Rust's memory safety and Go's
  explicit error returns satisfy several GNU robustness goals naturally — but be
  honest that GNU has **not** officially blessed them, and that for a GNU
  *extension language* specifically, Guile remains the endorsed choice. Don't imply
  GNU endorses what it has not.

## Universal robustness goals (realized per language)

These are goals, not C recipes; each language file shows the idiomatic realization
and `conventions.md` gives the full statement.

- **No arbitrary limits.** Allocate dynamically; never silently truncate long
  lines, names, or inputs.
- **Check every error return**, and put the system error text, the file name, and
  the program name in the message — "cannot open foo.c" alone is not enough.
- **Handle allocation failure**: fatal in a noninteractive program; in an
  interactive one, abort the command and return to the command loop.
- **Handle the full byte/character range**: don't drop NUL or nonprinting bytes;
  work correctly with UTF-8 / multibyte text.
- **On "impossible" conditions, abort** (with a comment naming the invariant)
  rather than printing a message — these are bugs.
- **Don't use an error count as the exit status** (status is 8 bits; 256 errors
  wraps to 0).
- **Behavior must not depend on the invocation name** (`argv[0]`) or on whether
  output is a terminal vs. a pipe — select behavior with options/config, with
  narrow documented exceptions (e.g. refusing to dump binary to a TTY, overridable
  with `-f`).
- **Operate with `/usr` and `/etc` read-only**; honor `TMPDIR`; create temp files
  safely.

## How to use this skill

1. **Read `references/conventions.md` first.** It holds the language-agnostic
   mechanics every free/GNU program must get right: the error-message grammar, the
   exact `--version` / `--help` output contract, long-option conventions, i18n
   (gettext), Texinfo documentation, ChangeLog/NEWS style, and the release process
   (`configure`, Makefile conventions, directory variables, standard targets,
   `DESTDIR`).
2. **Then read the relevant language file** in `references/`:
   - `c.md` — C, where the GNU rules are literal (the canonical case).
   - `guile.md` — GNU Guile, the endorsed extension language (cleanest mapping).
   - `other-languages.md` — Rust, Go, and Python: a brief conformance layer over
     each language's own canonical style, with the honest "principles apply, not
     officially blessed" framing.
3. **When the work involves the web or any JavaScript**, read
   `references/javascript-trap.md`.
4. **When writing or auditing prose** (READMEs, manuals, announcements, commit
   messages), apply `references/words-to-avoid.md`.
5. Attribution for the FSF/GNU source material this skill encodes is in
   `CREDITS.md` and `ATTRIBUTION.md`.

## What lives where

- **Four freedoms, the compliance gate, language hierarchy, robustness goals** —
  this file.
- **Error grammar, `--version`/`--help` contract, long options, i18n,
  Texinfo/NEWS, ChangeLog, release & Makefile/directory conventions** —
  `references/conventions.md`.
- **Per-language realization** (formatter, naming, error handling, option parsing,
  i18n binding, license header, build wrapper, gotchas) — `references/c.md`,
  `references/guile.md`, `references/other-languages.md`.
- **Nonfree-JavaScript rules and LibreJS labeling** —
  `references/javascript-trap.md`.
- **GNU vocabulary corrections** — `references/words-to-avoid.md`.
