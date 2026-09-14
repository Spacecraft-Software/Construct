#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Verify AGENTS.md's inventories and §-references against the tree.

`check-skill-refs.py` proves every skill id *named* in prose resolves to a real
directory. It cannot see the failure this script exists for: a list whose every
entry resolves but whose **set** is wrong. "CREDITS.md: currently A, B, C" is a
claim about which skills carry the file, and A, B, and C all existing says
nothing about D, which also carries it and is missing from the list.

That is not hypothetical. Three CREDITS.md inventories in AGENTS.md had each
drifted differently; the one inside the rebuild procedure — the list an agent
actually consults at step 1 — omitted `spacecraft-steelbore-standard`, whose
bundles then shipped without CREDITS.md for exactly that reason. The `assets/`
inventory named one skill when three had the directory, which would have
dropped assets from the next rebuild of either of the other two. And AGENTS.md
pointed at "§14 checklist is the audit gate" for months after v2.00 renumbered
the checklist to §16.

All three share a shape: a stale pointer in the file agents are told to trust,
where following it lands on the wrong thing rather than failing loudly.

Three checks:

1. **CREDITS.md inventories** — every list must equal the set of root skills
   that actually carry CREDITS.md.
2. **assets/ inventories** — likewise for skills carrying an assets/ directory.
3. **§-references** — every §N.M cited in AGENTS.md must resolve to a heading
   in the Standard skill (SKILL.md or its references/, since long sections are
   split out for on-demand loading).

A NOTE ON BRITTLENESS, because it is the whole design problem here: these
inventories live in English prose, so locating them means pattern-matching
wording that a future edit may legitimately change. A parser that quietly
matches nothing would be worse than no gate at all — it would report success
over an unchecked file. So the site *counts* are asserted: if this script finds
fewer inventory sites than expected, it fails and says the wording moved. The
failure mode is therefore a loud "come update the checker", never a silent
pass. Raise EXPECTED_* when a genuinely new inventory is added to the prose.

Usage:
    check-agents-inventories.py [ROOT]
"""
import os
import re
import sys

# How many inventory sites each kind has in AGENTS.md today. Asserted so a
# reword that breaks the patterns fails loudly instead of checking nothing.
EXPECTED_CREDITS_SITES = 3   # layout diagram, bundling section, rebuild step 1
EXPECTED_ASSETS_SITES = 2    # layout diagram, rebuild step 1

STANDARD_SKILL = "spacecraft-steelbore-standard"

NOT_ROOT_SKILLS = {
    "grok-skills", "android-skills", "orca-skills", "perplexity-skills",
    "Excluded", "construct-cli", "LICENSES", ".git", ".github", ".githooks",
    ".claude",
}

# A list-introducing phrase: the inventory follows within the same sentence.
# Anchored on "CREDITS.md" plus an enumerating cue rather than on any single
# wording, so light rephrasing survives and heavy rephrasing trips the count
# assertion above.
#
# The gap allows periods and single newlines — the cue is routinely separated
# from the file name by a section number ("§15.3") and the list often starts on
# the next line — but never crosses a blank line, so a match cannot leap out of
# its own paragraph into an unrelated one.
# Each gap also refuses to cross the *other* inventory's file name, so the
# `CREDITS.md` in the zip recipe cannot reach forward into the `assets/`
# sentence three lines below it and be mistaken for a CREDITS inventory.
def _gap(exclude):
    return rf"(?:(?!\n\n)(?!{exclude}).){{0,200}}?"

CREDITS_SITE = re.compile(
    rf"CREDITS\.md\b{_gap('assets')}(?:currently|triggers fire|applies)\b",
    re.I | re.S)
ASSETS_SITE = re.compile(
    rf"assets/{_gap('CREDITS')}(?:currently|only|today)\b", re.I | re.S)

SECTION_REF = re.compile(r"§(\d+(?:\.\d+)*)")
HEADING = re.compile(r"^#+\s*§(\d+(?:\.\d+)*)", re.M)
HEADING_TEXT = re.compile(r"^#+\s*§(\d+(?:\.\d+)*)\s*—?\s*(.*)$", re.M)

# Existence alone is a weak check: the §14 → §16 drift went unnoticed for
# months precisely because §14 still existed, as Date, Time & Units. Where
# AGENTS.md makes a claim about *what a section is*, verify the claim against
# the heading text. Each entry is (phrase describing the section, word the
# heading must contain).
# `\s+` between every word, not a literal space: AGENTS.md is hard-wrapped, so
# the phrase routinely straddles a line break ("its §14 checklist is the\naudit
# gate"). A literal-space pattern silently matches nothing there.
SECTION_CLAIMS = (
    (re.compile(r"§(\d+(?:\.\d+)*)\s+checklist\s+is\s+the\s+audit\s+gate", re.I),
     "audit gate"),
)


def root_skills(root):
    out = set()
    for entry in os.listdir(root):
        if entry in NOT_ROOT_SKILLS or not os.path.isdir(os.path.join(root, entry)):
            continue
        if os.path.exists(os.path.join(root, entry, "SKILL.md")):
            out.add(entry)
    return out


def collect_inventory(text, match, known):
    """Collect the skill names listed immediately after an inventory cue.

    Scans forward from the cue to the end of the sentence (or a blank line, for
    the fenced layout diagram where there is no full stop) and keeps the tokens
    that name a real skill. Matching against `known` rather than a namespace
    pattern means `steelbore-color-palette` is picked up without hard-coding
    which prefixes count as skill ids.
    """
    tail = text[match.end():]
    stop = len(tail)
    # End of sentence, end of paragraph, or the next row of the layout diagram
    # (box-drawing characters) — the diagram's rows carry one inventory each and
    # are not separated by blank lines or full stops.
    for boundary in (". ", ".\n", "\n\n", "\n├", "\n└", "\n│"):
        i = tail.find(boundary)
        if i != -1:
            stop = min(stop, i)
    window = tail[:stop]
    found, seen = [], set()
    for tok in re.findall(r"[A-Za-z][A-Za-z0-9-]{3,}", window):
        if tok in known and tok not in seen:
            seen.add(tok)
            found.append(tok)
    return found


def standard_sections(root):
    """Every §-heading in the Standard skill, body and references/ alike."""
    base = os.path.join(root, STANDARD_SKILL)
    if not os.path.isdir(base):
        return None
    paths = [os.path.join(base, "SKILL.md")]
    refs = os.path.join(base, "references")
    if os.path.isdir(refs):
        paths += [os.path.join(refs, f) for f in os.listdir(refs)
                  if f.endswith(".md")]
    out, titles = set(), {}
    for p in paths:
        try:
            body = open(p, encoding="utf-8").read()
        except OSError:
            continue
        out |= set(HEADING.findall(body))
        for num, title in HEADING_TEXT.findall(body):
            titles.setdefault(num, title.strip())
    return out, titles


def check_inventory(text, pattern, actual, label, expected_sites, problems):
    sites = list(pattern.finditer(text))
    if len(sites) < expected_sites:
        problems.append(
            f"found {len(sites)} {label} inventory site(s), expected "
            f"{expected_sites} — the wording these patterns key on has "
            f"changed. Update CREDITS_SITE / ASSETS_SITE (and the EXPECTED_* "
            f"count) in this script; do not lower the expectation to match a "
            f"list that is no longer being checked.")
        return
    known = actual | {"spacecraft-agentic-cli"}  # names may appear as examples
    for m in sites:
        line = text[:m.start()].count("\n") + 1
        listed = set(collect_inventory(text, m, known))
        if not listed:
            continue  # a mention with no list after it (e.g. the zip recipe)
        missing = actual - listed
        extra = listed - actual
        if missing:
            problems.append(
                f"AGENTS.md:{line}: {label} list omits "
                f"{', '.join(sorted(missing))} — these carry it on disk. An "
                f"agent rebuilding from this list drops the file silently.")
        if extra:
            problems.append(
                f"AGENTS.md:{line}: {label} list names "
                f"{', '.join(sorted(extra))}, which do not carry it on disk.")


def main(argv):
    root = argv[0] if argv else "."
    path = os.path.join(root, "AGENTS.md")
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as e:
        print(f"cannot read {path}: {e}", file=sys.stderr)
        return 1

    skills = root_skills(root)
    problems = []

    check_inventory(
        text, CREDITS_SITE,
        {s for s in skills if os.path.exists(os.path.join(root, s, "CREDITS.md"))},
        "CREDITS.md", EXPECTED_CREDITS_SITES, problems)

    check_inventory(
        text, ASSETS_SITE,
        {s for s in skills if os.path.isdir(os.path.join(root, s, "assets"))},
        "assets/", EXPECTED_ASSETS_SITES, problems)

    found = standard_sections(root)
    if found is None:
        problems.append(f"{STANDARD_SKILL}/ not found — cannot check §-refs")
    else:
        sections, titles = found
        for m in SECTION_REF.finditer(text):
            if m.group(1) not in sections:
                line = text[:m.start()].count("\n") + 1
                problems.append(
                    f"AGENTS.md:{line}: §{m.group(1)} is not a section of the "
                    f"Standard skill. Section numbers move (v2.00 renumbered "
                    f"the audit-gate checklist from §14 to §16); cite the "
                    f"section that exists now.")
        for pattern, expected in SECTION_CLAIMS:
            for m in pattern.finditer(text):
                num = m.group(1)
                title = titles.get(num, "")
                if expected.lower() not in title.lower():
                    line = text[:m.start()].count("\n") + 1
                    problems.append(
                        f"AGENTS.md:{line}: describes §{num} as '{expected}', "
                        f"but §{num} is \"{title or '(unknown)'}\". The section "
                        f"exists, so an existence check passes — the number is "
                        f"pointing at the wrong section.")

    if problems:
        print("AGENTS.md inventory problems:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1

    print(f"AGENTS.md inventories OK ({len(skills)} root skills, "
          f"{len(found[0])} Standard sections)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
