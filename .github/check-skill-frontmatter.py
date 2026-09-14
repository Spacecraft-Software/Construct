#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Require the distribution metadata every cross-platform SKILL.md carries.

A bundle is a distribution in its own right (Standard §5.6), so a consumer who
installs the `.zip` sees the frontmatter and nothing else of the repo around
it. `check-license-files.py` already proves the license *text* ships; this
proves the skill also *states* which license it is under, and who maintains it.

Until 2026-09-14 nothing checked these fields, and the fleet had drifted into
three encodings of the same facts:

  * 33 skills carried `maintainer` / `website` at the top level
  *  4 carried them nested under `metadata:` — same data, invisible to any
     consumer or tool reading top-level keys
  *  7 carried neither, and no `license` either

The fix is one encoding, top level, enforced here.

Scope — which skills must carry what:

  * Root (cross-platform) skills: name, description, license, maintainer,
    website. These install into ~/.claude, ~/.codex, ~/.gemini and are the
    repo's own work.
  * Grok skills: name and description only. Grok's loader does not consume the
    other three, and AGENTS.md documents the minimal frontmatter as
    deliberate — requiring them here would be inventing a rule the format does
    not have.
  * Vendored third-party trees (android-skills/, orca-skills/): nothing.
    Standard §4.2 preserves upstream's own layout and metadata verbatim;
    editing their frontmatter to satisfy a Spacecraft gate is exactly what
    §4.2 forbids.

Usage:
    check-skill-frontmatter.py [ROOT]

Exits 1 and names every offending field if any required key is missing or
empty; exits 0 otherwise.
"""
import os
import sys

REQUIRED_ROOT = ("name", "description", "license", "maintainer", "website")
REQUIRED_GROK = ("name", "description")

# Directories that are not root skills. Mirrors `excludedDirs` in flake.nix —
# if a directory is added there, add it here too.
NOT_ROOT_SKILLS = {
    "grok-skills", "android-skills", "orca-skills", "perplexity-skills",
    "Excluded", "construct-cli", "LICENSES", ".git", ".github", ".githooks",
    ".claude",
}

# gnu-free-software is free-standing free-software guidance under Mohamed
# Hammad's personal copyright with no Spacecraft branding — REUSE.toml carries
# an explicit attribution override for it so its distribution metadata stays
# honest ("not the @SpacecraftSoftware.org default"). Adding the Spacecraft
# maintainer and website here would contradict that override, so it is exempt
# from those two fields only; name, description, and license still apply.
#
# This is a deliberate exemption, not an oversight. Do not "fix" it by
# backfilling Spacecraft branding — read REUSE.toml's gnu-free-software
# annotation first.
BRANDING_EXEMPT = {"gnu-free-software"}
BRANDING_FIELDS = {"maintainer", "website"}


def top_level_keys(path):
    """Map top-level frontmatter keys to their raw value strings.

    Line-based on purpose: the only thing in question is which keys exist at
    the top level, and a YAML parser would additionally flatten `metadata:`
    nesting into something that looks compliant when it is not — which is the
    exact drift this gate exists to catch.
    """
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError as e:
        return None, f"cannot read: {e}"
    if not lines or lines[0].strip() != "---":
        return None, "no YAML frontmatter"
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None, "unterminated frontmatter"

    keys = {}
    for line in lines[1:end]:
        if not line or line[0] in " \t#":
            continue  # continuation, nested key, or comment
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        keys[key.strip()] = value.strip()
    return keys, None


def check(skill_dir, name, required):
    """Return a list of human-readable problems for one skill."""
    path = os.path.join(skill_dir, "SKILL.md")
    keys, err = top_level_keys(path)
    if err:
        return [f"{path}: {err}"]

    required = tuple(
        f for f in required
        if not (name in BRANDING_EXEMPT and f in BRANDING_FIELDS)
    )

    problems = []
    for field in required:
        if field not in keys:
            # Point at the nested copy when there is one — that is the common
            # case and the fix is a hoist, not new information.
            nested = any(
                line.strip().startswith(f"{field}:")
                for line in open(path, encoding="utf-8").read().splitlines()[:40]
            )
            hint = " (present under `metadata:` — hoist it to the top level)" if nested else ""
            problems.append(f"{path}: missing `{field}`{hint}")
        elif not keys[field] and field != "description":
            # `description:` legitimately has an empty value on the key's own
            # line when the text is a block or plain multi-line scalar beneath
            # it; the other fields are always single-line.
            problems.append(f"{path}: `{field}` is empty")

    declared = keys.get("name")
    if declared and declared != name:
        problems.append(
            f"{path}: `name: {declared}` does not match directory `{name}` "
            f"(skill ids are functional identifiers — Standard §2.2 — and the "
            f"directory is the id consumers install under)"
        )
    return problems


def main(argv):
    root = argv[0] if argv else "."
    problems, checked = [], 0

    for entry in sorted(os.listdir(root)):
        path = os.path.join(root, entry)
        if not os.path.isdir(path) or entry in NOT_ROOT_SKILLS:
            continue
        if os.path.exists(os.path.join(path, "SKILL.md")):
            problems += check(path, entry, REQUIRED_ROOT)
            checked += 1

    grok = os.path.join(root, "grok-skills")
    if os.path.isdir(grok):
        for entry in sorted(os.listdir(grok)):
            path = os.path.join(grok, entry)
            if os.path.isdir(path) and os.path.exists(os.path.join(path, "SKILL.md")):
                problems += check(path, entry, REQUIRED_GROK)
                checked += 1

    if problems:
        print("SKILL.md frontmatter problems:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        print(
            "\nEvery cross-platform skill carries name, description, license, "
            "maintainer, and website at the TOP level of its frontmatter — a "
            "bundle is a distribution in its own right (§5.6) and nested "
            "`metadata:` is invisible to consumers reading top-level keys.",
            file=sys.stderr,
        )
        return 1

    print(f"frontmatter OK ({checked} skills checked)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
