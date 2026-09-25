# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Validate that every JSON / JSONC / TOML / YAML config template parses cleanly,
and that every skill's SKILL.md frontmatter is YAML a strict parser accepts.

Run from the repo root:  python3 .github/validate-configs.py

Walks the tree (skipping .git), parses each config by extension, and exits
non-zero listing every malformed file. New host templates are picked up
automatically — no list to maintain.

SKILL.md frontmatter is the one config that hides in a .md file, and it is
the one every skill loader parses. Until 2026-09-25 nothing here parsed it:
`check-skill-frontmatter.py` is line-based on purpose (its docstring says why)
and `check-description-length.py` folds the text itself, so a plain-scalar
`description:` containing `: ` — `… the house look: slide decks …`, which is
the start of a nested mapping to a YAML parser, not a string — passed every
gate. Lenient readers still showed the skill; strict ones (PyYAML, Nushell's
`from yaml`, `construct`'s serde_yaml) rejected the whole frontmatter, so
`construct skill find` listed the skill with no description and `construct
skill ship` exempted it from the §5.6 cap as "nothing to measure". This step
parses the frontmatter of every root and Grok skill — the same walk as
`check-skill-frontmatter.py`; vendored trees are §4.2 upstream content — and
fails on the first thing a strict parser refuses.
"""
from __future__ import annotations

import json
import os
import re
import sys
import tomllib

import yaml

SKIP_DIRS = {".git"}
EXTS = (".json", ".jsonc", ".toml", ".yaml", ".yml")

# Directories that are not root skills. Mirrors NOT_ROOT_SKILLS in
# check-skill-frontmatter.py (which mirrors `excludedDirs` in flake.nix) — if a
# directory is added there, add it here too.
NOT_ROOT_SKILLS = {
    "grok-skills", "android-skills", "orca-skills", "perplexity-skills",
    "Excluded", "construct-cli", "LICENSES", ".git", ".github", ".githooks",
    ".claude",
}


def strip_jsonc(text: str) -> str:
    """Drop /* */ blocks and whole-line // comments.

    Only *whole-line* // comments are removed, so URLs such as
    https://example inside string values are left untouched.
    """
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return "\n".join(l for l in text.splitlines() if not l.lstrip().startswith("//"))


def validate(path: str) -> str | None:
    try:
        if path.endswith(".jsonc"):
            with open(path, encoding="utf-8") as f:
                json.loads(strip_jsonc(f.read()))
        elif path.endswith(".json"):
            with open(path, encoding="utf-8") as f:
                json.load(f)
        elif path.endswith(".toml"):
            with open(path, "rb") as f:
                tomllib.load(f)
        elif path.endswith((".yaml", ".yml")):
            with open(path, encoding="utf-8") as f:
                yaml.safe_load(f)
    except Exception as exc:  # report any parse error, keep checking the rest
        return f"{path}: {type(exc).__name__}: {exc}"
    return None


def frontmatter(path: str) -> str | None:
    """The text between the opening `---` line and the next one, or None.

    Presence and termination are `check-skill-frontmatter.py`'s findings; a
    file without a frontmatter block is skipped here rather than reported
    twice.
    """
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    return "\n".join(lines[1:end])


def validate_skill_frontmatter(path: str) -> str | None:
    """Parse a SKILL.md frontmatter with a strict YAML parser; None if it is fine."""
    try:
        text = frontmatter(path)
        if text is None:
            return None
        data = yaml.safe_load(text)
    except Exception as exc:  # any parse error — one line, so CI output stays greppable
        detail = " ".join(str(exc).split())
        return f"{path}: SKILL.md frontmatter is not valid YAML: {detail}"
    if not isinstance(data, dict):
        return f"{path}: SKILL.md frontmatter is not a YAML mapping"
    return None


def skill_mds(root: str = ".") -> list[str]:
    """Every SKILL.md the frontmatter gates cover: root skills and grok-skills/.

    The same walk as `check-skill-frontmatter.py`. Vendored third-party trees
    (android-skills/, orca-skills/) are excluded for the same reason they are
    excluded there — Standard §4.2 preserves upstream content verbatim, so a
    finding in one could not be acted on here.
    """
    found = []
    for entry in sorted(os.listdir(root)):
        if entry in NOT_ROOT_SKILLS:
            continue
        path = os.path.join(root, entry, "SKILL.md")
        if os.path.isfile(path):
            found.append(path)
    grok = os.path.join(root, "grok-skills")
    if os.path.isdir(grok):
        for entry in sorted(os.listdir(grok)):
            path = os.path.join(grok, entry, "SKILL.md")
            if os.path.isfile(path):
                found.append(path)
    return found


def main() -> int:
    failures: list[str] = []
    checked = 0
    for root, dirs, files in os.walk("."):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in files:
            if not name.endswith(EXTS):
                continue
            checked += 1
            if (err := validate(os.path.join(root, name))) is not None:
                failures.append(err)

    skills = skill_mds()
    for path in skills:
        if (err := validate_skill_frontmatter(path)) is not None:
            failures.append(err)

    for err in failures:
        print(f"FAIL  {err}", file=sys.stderr)
    print(
        f"Checked {checked} config files and {len(skills)} SKILL.md frontmatters, "
        f"{len(failures)} failed."
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
