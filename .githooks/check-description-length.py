#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Enforce Construct's SKILL.md `description` cap (<= 1000 rendered chars).

The skill loader's absolute limit is 1024; this repo caps the *rendered*
description at 1000 (a 24-char margin for encoding/trailing-newline edge
cases). YAML folded scalars (`description: >`) join their wrapped lines with
spaces, so the raw line count is not what the loader sees — this script
reproduces the folding so the count matches, exactly as documented in
CONTRIBUTING.md. Both the block form (`description: >`, used by root skills)
and the single-line plain/quoted form (used by Grok skills) are handled.

Usage:
    check-description-length.py FILE [FILE ...]

Prints every offender and exits 1 if any description exceeds the cap;
exits 0 otherwise. Files without frontmatter or a `description` key are
skipped silently.
"""
import sys

CAP = 1000


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
    if head[:1] in ("|", ">"):
        clip = not head.rstrip().endswith("-")  # `-` strips the trailing newline
        body = []
        for k in range(di + 1, len(fm)):
            l = fm[k]
            if l.strip() == "":
                body.append("")
            elif l.startswith((" ", "\t")):
                body.append(l.strip())
            else:
                break  # dedented line ends the block (next key)
        while body and body[-1] == "":
            body.pop()
        out, buf = [], []
        for b in body:
            if b == "":
                out.append(" ".join(buf))
                buf = []
            else:
                buf.append(b)
        if buf:
            out.append(" ".join(buf))
        return len("\n".join(out)) + (1 if clip else 0)

    # Single-line plain or quoted scalar (Grok skills).
    if head:
        if len(head) >= 2 and head[0] == head[-1] and head[0] in ("'", '"'):
            head = head[1:-1]
        return len(head)
    return None


def main(argv):
    offenders = []
    for path in argv:
        try:
            n = rendered_length(path)
        except OSError as e:
            print(f"  ! cannot read {path}: {e}", file=sys.stderr)
            offenders.append((path, -1))
            continue
        if n is not None and n > CAP:
            offenders.append((path, n))
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
