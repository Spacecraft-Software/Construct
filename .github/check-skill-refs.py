#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Catch cross-reference drift between the catalogue and the Standard.

Two checks, both for failures that no existing gate sees. The description cap
and `reuse lint` verify a skill in isolation; nothing verified that a skill
still *agrees* with the catalogue and the Standard around it.

1. **Skill references resolve.** A skill id named in prose (`` `spacecraft-foo` ``
   in Markdown, `@code{spacecraft-foo}` in Texinfo) must exist as a skill
   directory. The Standard shipped for months pointing at
   `spacecraft-accessibility` when the skill had always been
   `spacecraft-accessibility-support`. Nothing complained, because the failure
   is silent: a missing skill does not announce itself, it simply never loads,
   so work directed at it proceeds without it.

2. **No document-version pins.** A skill that says "The Steelbore Standard
   v1.42 (§11, §12)" is stale the moment any unrelated section changes — and
   ambiguous besides, since a reader cannot tell whether the version refers to
   the document or to the clause. Three skills carried such pins; two had gone
   stale within a single release of being written. Cite the section, and if a
   version is genuinely useful, say when that *section* last changed
   ("§11 — last amended in v1.39"), which only moves when the section moves.

Usage:
    check-skill-refs.py --catalogue <dir> [--no-version-check] <path>...

`--catalogue` is the directory holding skill directories (the Construct clone).
`--no-version-check` runs only check 1, for the Standard, whose own masthead
version is legitimate.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

# Namespaces that are unambiguously skill ids. `steelbore-*` is deliberately
# excluded: it is the palette and theme slug namespace (`steelbore-mono`,
# `steelbore-high-contrast`, `tokyonight-color-palette`), and those are values,
# not directories. The one real skill in that namespace,
# `steelbore-color-palette`, resolves anyway if it is ever referenced.
SKILL_NAMESPACES = ("spacecraft", "microsoft", "gnu")
_NS = "|".join(SKILL_NAMESPACES)

MARKDOWN_REF = re.compile(rf"`((?:{_NS})-[a-z0-9][a-z0-9-]*)`")
TEXINFO_REF = re.compile(rf"@code\{{((?:{_NS})-[a-z0-9][a-z0-9-]*)\}}")

# "Standard v1.42", "Standard (§11, §12), v1.39", "current as of Standard v1.42".
# Bounded to one clause so it cannot leap across sentences into an unrelated
# version number. The negative lookahead excludes three-component versions:
# The Steelbore Standard numbers itself v1.NN, while the Dual-Mode CLI Standard
# is a different document versioned v1.0.0 and legitimately pinned as such.
VERSION_PIN = re.compile(r"Standard\b[^.\n]{0,60}?\bv1\.\d+(?!\.\d)", re.IGNORECASE)

# Phrases that make a version a *clause* pin — when this section last changed —
# rather than a document pin. These are correct and must not be flagged.
CLAUSE_PIN = re.compile(
    r"last amended|introduced (in|at)|added (in|at)|retired (at|in)|reinstated"
    r"|un-retired|predat|superseded (in|at)|since v1\.|grandfathered|shipped v1\.",
    re.IGNORECASE,
)

# Opt out on a single line when neither rule fits. Keep it rare and explain why.
SUPPRESS = "skill-refs: allow"

# Never walked: vendored upstream (§4.2 forbids local edits), the holding pen,
# and generated trees that mirror a source elsewhere.
SKIP_DIRS = {".git", ".claude", "android-skills", "Excluded", "node_modules", "target"}

# Names inside a skill namespace that are not skills. `spacecraft-software` is
# the umbrella workspace and org, and appears in paths and prose.
NON_SKILL_NAMES = {"spacecraft-software"}

# Paths whose document-version pin is legitimate: the skill that encodes the
# Standard carries its version by design, and changelogs are history.
VERSION_EXEMPT_PARTS = ("spacecraft-standard-constitution",)
VERSION_EXEMPT_NAMES = ("CHANGELOG.md",)

# Changelogs also name things that have since been renamed or removed — that is
# what a history is for — so reference resolution is skipped there as well.
REF_EXEMPT_NAMES = ("CHANGELOG.md",)


def catalogue_skills(root: pathlib.Path) -> set[str]:
    """Every directory that is a skill, i.e. contains a SKILL.md.

    Covers top-level skills and the flat Grok variants one level down, which is
    where the two bundle layouts diverge.
    """
    found: set[str] = set()
    for skill_md in root.glob("*/SKILL.md"):
        found.add(skill_md.parent.name)
    for skill_md in root.glob("*/*/SKILL.md"):
        if skill_md.parent.parent.name not in SKIP_DIRS:
            found.add(skill_md.parent.name)
    return found


def iter_files(paths: list[str]):
    for raw in paths:
        p = pathlib.Path(raw)
        if p.is_dir():
            for child in sorted(p.rglob("*")):
                if child.is_file() and child.suffix in {".md", ".texi"}:
                    if not set(child.parts) & SKIP_DIRS:
                        yield child
        elif p.is_file():
            yield p


def version_exempt(path: pathlib.Path) -> bool:
    return path.name in VERSION_EXEMPT_NAMES or any(
        part in VERSION_EXEMPT_PARTS for part in path.parts
    )


def check(path: pathlib.Path, skills: set[str], check_versions: bool) -> list[str]:
    problems: list[str] = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as exc:
        return [f"{path}: unreadable ({exc})"]

    pattern = TEXINFO_REF if path.suffix == ".texi" else MARKDOWN_REF
    exempt = version_exempt(path)

    for n, line in enumerate(lines, 1):
        if SUPPRESS in line:
            continue

        for name in pattern.findall(line):
            if path.name in REF_EXEMPT_NAMES or name in NON_SKILL_NAMES:
                continue
            if name not in skills:
                near = min(
                    (s for s in skills if s.startswith(name) or name.startswith(s)),
                    key=len,
                    default=None,
                )
                hint = f" (did you mean `{near}`?)" if near else ""
                problems.append(
                    f"{path}:{n}: references `{name}`, which is not a skill{hint}"
                )

        if check_versions and not exempt:
            hit = VERSION_PIN.search(line)
            if hit and not CLAUSE_PIN.search(line):
                problems.append(
                    f"{path}:{n}: pins a document version ({hit.group(0).strip()!r}) "
                    f"— cite the section instead, or say when that section last changed"
                )

    return problems


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--catalogue", default=".", help="directory holding skill dirs")
    ap.add_argument(
        "--no-version-check",
        action="store_true",
        help="only verify that skill references resolve",
    )
    ap.add_argument("paths", nargs="+")
    args = ap.parse_args(argv)

    root = pathlib.Path(args.catalogue)
    skills = catalogue_skills(root)
    if not skills:
        print(
            f"error: no skills found under {root} — is --catalogue pointing at "
            f"the Construct clone?",
            file=sys.stderr,
        )
        return 2

    problems: list[str] = []
    for path in iter_files(args.paths):
        problems.extend(check(path, skills, not args.no_version_check))

    if problems:
        for p in problems:
            print(p, file=sys.stderr)
        print(
            f"\n{len(problems)} cross-reference problem(s). "
            f"Catalogue has {len(skills)} skills.",
            file=sys.stderr,
        )
        return 1

    print(f"skill references OK ({len(skills)} skills in catalogue)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
