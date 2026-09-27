#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prove shared reference blocks have not drifted, and cap `compatibility:`.

Some guidance has to reach an agent through more than one skill. The
execution-context block — how an agent measures which shell, OS, and package
environment will actually run a command — is needed by `spacecraft-cli-shell`
(which shell's syntax applies), `spacecraft-cli-preference` (which tools are
present), and `spacecraft-missing-pkg` (which provisioning route is open).
A bundle is a distribution in its own right (Standard §5.6): a consumer who
installs only one of those `.zip`s never sees the others, so a cross-skill
link would dangle. The block is therefore *copied* into each skill.

Copies are exactly how two texts stop being one text. §4.3 forbids two
independently maintained copies of a license for the same reason; the fix is
the same too — one canonical file, and enforced byte-equality for every copy.
Edit the canonical file, then copy it over each entry in SHARED.

Second check, same script because it is the same kind of drift — a value that
is valid in this repo but breaks at a consumer: the Agent Skills specification
caps the optional top-level `compatibility:` frontmatter field at 500
characters. Nothing else here measures it, and a loader that enforces the spec
rejects the whole skill, not the field.

Parsing is line-based, like check-skill-frontmatter.py, so a `compatibility:`
nested under `metadata:` is not mistaken for the top-level field. Single-line
values may be plain, 'single-' or "double-quoted" (quotes are not counted);
block (`>` / `|`) and multi-line plain scalars are folded the way YAML folds
them before measuring.

Usage:
    check-shared-blocks.py [ROOT]

Exits 1 and names every offender — a missing or differing copy, or an
over-length `compatibility:` — and exits 0 otherwise.
"""
import os
import sys

# canonical path -> the copies that must be byte-identical to it.
SHARED = {
    "spacecraft-cli-shell/references/execution-context.md": [
        "spacecraft-cli-preference/references/execution-context.md",
        "spacecraft-missing-pkg/references/execution-context.md",
    ],
}

COMPATIBILITY_MAX = 500  # Agent Skills specification limit

# Directories that are not root skills. Mirrors `excludedDirs` in flake.nix —
# if a directory is added there, add it here too.
NOT_ROOT_SKILLS = {
    "grok-skills", "android-skills", "orca-skills", "perplexity-skills",
    "Excluded", "construct-cli", "LICENSES", ".git", ".github", ".githooks",
    ".claude",
}


def read_bytes(path):
    try:
        with open(path, "rb") as f:
            return f.read()
    except OSError:
        return None


def check_shared(root):
    problems = []
    for canonical, copies in SHARED.items():
        want = read_bytes(os.path.join(root, canonical))
        if want is None:
            problems.append(f"{canonical}: canonical file is missing")
            continue
        for copy in copies:
            got = read_bytes(os.path.join(root, copy))
            if got is None:
                problems.append(f"{copy}: missing (copy of {canonical})")
            elif got != want:
                problems.append(
                    f"{copy}: differs from {canonical} — edit the canonical "
                    f"file and copy it over, never the copy")
    return problems


def _unquote(value):
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        inner = value[1:-1]
        if value[0] == "'":
            return inner.replace("''", "'")
        return inner.replace('\\"', '"').replace("\\\\", "\\")
    return value


def compatibility_value(path):
    """Return the rendered top-level `compatibility:` value, or None if absent."""
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError:
        return None
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    fm = lines[1:end]

    for i, line in enumerate(fm):
        if not line.startswith("compatibility:"):
            continue
        head = line.partition(":")[2].strip()
        # Continuation lines: indented or blank, up to the next top-level key.
        body = []
        for nxt in fm[i + 1:]:
            if nxt and nxt[0] not in " \t":
                break
            body.append(nxt.strip())
        while body and not body[-1]:
            body.pop()

        if head[:1] in (">", "|"):
            if head[0] == "|":
                return "\n".join(body) + "\n"
            # Folded: lines join with spaces, blank lines become newlines.
            out, buf = [], []
            for b in body:
                if b:
                    buf.append(b)
                else:
                    out.append(" ".join(buf))
                    buf = []
            if buf:
                out.append(" ".join(buf))
            return "\n".join(out) + "\n"
        if head[:1] in "'\"" and head:
            return _unquote(" ".join([head] + [b for b in body if b]))
        return " ".join([head] + [b for b in body if b]).strip()
    return None


def check_compatibility(root):
    problems = []
    for entry in sorted(os.listdir(root)):
        skill = os.path.join(root, entry)
        if entry in NOT_ROOT_SKILLS or not os.path.isdir(skill):
            continue
        path = os.path.join(skill, "SKILL.md")
        if not os.path.exists(path):
            continue
        value = compatibility_value(path)
        if value is not None and len(value) > COMPATIBILITY_MAX:
            problems.append(
                f"{entry}/SKILL.md: `compatibility:` is {len(value)} chars "
                f"(max {COMPATIBILITY_MAX}, over by "
                f"{len(value) - COMPATIBILITY_MAX})")
    return problems


def main(argv):
    root = argv[0] if argv else "."
    problems = check_shared(root) + check_compatibility(root)

    if problems:
        print("Shared-block / compatibility problems:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        print(
            "\nA shared block has one canonical file (the key in SHARED); every "
            "copy is byte-identical to it, because each bundle ships alone "
            "(§5.6) and cannot link to another skill. `compatibility:` is "
            f"capped at {COMPATIBILITY_MAX} characters by the Agent Skills "
            "specification.",
            file=sys.stderr,
        )
        return 1

    copies = sum(len(c) for c in SHARED.values())
    print(f"shared blocks OK ({len(SHARED)} canonical, {copies} copies); "
          f"compatibility OK")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
