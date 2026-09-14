#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Enforce Construct's SKILL.md `description` cap (<= 1000 rendered chars).

The skill loader's absolute limit is 1024; this repo caps the *rendered*
description at 1000 (a 24-char margin for encoding/trailing-newline edge
cases). YAML folded scalars (`description: >`) join their wrapped lines with
spaces, so the raw line count is not what the loader sees — this script
reproduces the folding so the count matches, exactly as documented in
CONTRIBUTING.md.

Three spellings reach the loader as the same kind of value, so all three are
measured:

  * block scalar      — `description: >` / `|`, used by most root skills
  * single-line       — plain or quoted on the key's own line, used by Grok
  * plain multi-line  — a bare `description:` followed by indented lines

The plain multi-line form went unmeasured until 2026-09-14: `rendered_length`
returned None for it, so the file was skipped *silently* by CI, by
`construct skill ship`, and by the pre-commit hook. A 1600-char description in
that form passed every gate and would then hard-fail at install time, which is
the worst place to discover it. It folds like a folded scalar but, being a
plain scalar, gains no trailing newline — hence the `trailing` distinction
below.

Usage:
    check-description-length.py FILE [FILE ...]

Prints every offender and exits 1 if any description exceeds the cap; exits 0
otherwise. Descriptions within WARN_MARGIN of the cap are reported on stderr
as warnings — they are compliant, and the exit code is unaffected, but the
remaining headroom is small enough that the next wording change is likely to
trip the gate. Files without frontmatter or a `description` key are skipped
silently.
"""
import sys

CAP = 1000

# Report (without failing) descriptions this close to the cap. A description at
# 998 is compliant but has four characters of room; surfacing that at authoring
# time is cheaper than a CI failure on an unrelated later edit.
WARN_MARGIN = 40


def _fold(fm, di, trailing):
    """Fold the indented continuation lines after `fm[di]` and return the length.

    Shared by the block-scalar and plain multi-line forms, which fold
    identically: wrapped lines join with a single space, a blank line becomes a
    newline. They differ only in `trailing` — whether the rendered value ends
    with a newline the loader also counts.
    """
    body = []
    for k in range(di + 1, len(fm)):
        l = fm[k]
        if l.strip() == "":
            body.append("")
        elif l.startswith((" ", "\t")):
            body.append(l.strip())
        else:
            break  # dedented line ends the scalar (next key)
    while body and body[-1] == "":
        body.pop()
    if not body:
        return None
    out, buf = [], []
    for b in body:
        if b == "":
            out.append(" ".join(buf))
            buf = []
        else:
            buf.append(b)
    if buf:
        out.append(" ".join(buf))
    return len("\n".join(out)) + (1 if trailing else 0)


def rendered_length(path):
    """Return the rendered description length, or None if there is none."""
    lines = open(path, encoding="utf-8").read().splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    fm = lines[1:end]

    di = next((k for k, l in enumerate(fm) if l.startswith("description:")), None)
    if di is None:
        return None
    head = fm[di][len("description:"):].strip()

    # Block scalar: `>` (folded) / `|` (literal), optional chomping (`-`/`+`).
    # Clip chomping (the default) keeps exactly one trailing newline; `-` strips
    # it.
    if head[:1] in ("|", ">"):
        return _fold(fm, di, trailing=not head.rstrip().endswith("-"))

    # Single-line plain or quoted scalar (Grok skills).
    if head:
        if len(head) >= 2 and head[0] == head[-1] and head[0] in ("'", '"'):
            head = head[1:-1]
        return len(head)

    # Plain multi-line scalar: a bare `description:` whose value is the indented
    # block beneath it. Folds like `>` but, as a plain scalar, gains no trailing
    # newline. Returning None here (the behaviour before 2026-09-14) let an
    # arbitrarily long description through every gate unmeasured.
    return _fold(fm, di, trailing=False)


def main(argv):
    offenders = []
    near = []
    for path in argv:
        try:
            n = rendered_length(path)
        except OSError as e:
            print(f"  ! cannot read {path}: {e}", file=sys.stderr)
            offenders.append((path, -1))
            continue
        if n is None:
            continue
        if n > CAP:
            offenders.append((path, n))
        elif n > CAP - WARN_MARGIN:
            near.append((path, n))
    if near:
        print(f"note: within {WARN_MARGIN} chars of the {CAP} cap "
              f"(compliant — not a failure):", file=sys.stderr)
        for path, n in sorted(near, key=lambda p: -p[1]):
            print(f"  {path}: {n} chars ({CAP - n} left)", file=sys.stderr)
    if offenders:
        print(f"SKILL.md description cap (<= {CAP} rendered chars) exceeded:",
              file=sys.stderr)
        for path, n in offenders:
            where = "unreadable" if n < 0 else f"{n} chars ({n - CAP} over)"
            print(f"  {path}: {where}", file=sys.stderr)
        print("Trim the `description` and re-stage. See CONTRIBUTING.md "
              "(Editing rules) for the folded-scalar rationale.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
