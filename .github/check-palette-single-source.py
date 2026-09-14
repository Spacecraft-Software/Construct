#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Keep colour values in exactly one place.

Standard §11.4 already says it — "values are read, never retyped" — and names
`steelbore-color-palette/assets/steelbore.toml` as the canonical machine-readable
form. Nothing enforced it, and the fleet had drifted into 49 restated hex values
across four consumer skills plus 35 in the palette skill's own prose, every one
of them a copy of a value the TOML already owned.

Nothing had gone wrong yet: at the time of writing every restated value still
matched. That is exactly the state worth locking in, because the failure mode is
silent — a hex edited in the TOML and missed in a consumer ships a wrong colour
with no error anywhere, and a contrast ratio quoted beside it becomes a false
accessibility claim.

What counts as a value, and why all three:

  * hex literals      (#RRGGBB)
  * RGB triples       (RGB(0, 0, 39)) — the same value in another base

CONTRAST RATIOS ARE NOT CHECKED, and that is a deliberate limitation rather than
an oversight. A measured ratio (15.09:1, Platinum Mist on Void Navy) really is
derived data that goes stale the moment either endpoint moves. But a *threshold*
(4.5:1 for WCAG AA text, 3:1 for non-text, 7:1 for AAA) is a constant of the
specification being applied, and the two are textually identical. Gating the
pattern flagged 60 occurrences in spacecraft-accessibility-support, a large
share of them thresholds — removing those would delete the rule the skill exists
to enforce.

No regex distinguishes "a number this palette produced" from "a number WCAG
requires", so the check is not attempted. Measured ratios therefore remain a
known residual duplication: if a palette value changes, the ratios quoted
alongside it must be re-derived by hand from the TOML, which carries them.

Skills name *tokens* ("Void Navy", "Plasma Orange Lift") and the roles they
play. Token names are stable identifiers, not values; they are the vocabulary
this rule exists to protect.

FENCED CODE BLOCKS ARE EXEMPT, and the reason is not convenience. An authoring
recipe contains markup the reader pastes verbatim:

    fo:background-color="#000027"

`fo:background-color="Void Navy"` is not valid ODF. The literal is required for
the snippet to work at all, so a rule that stripped it would break the recipe
rather than improve it. The prose *around* the block still has to name the
token, which is where a reader learns which value to look up — the block shows
the mechanism, the prose carries the meaning.

This is a real hole: a value inside a fence can still drift. It is accepted
deliberately, because a broken example costs more than a stale one that sits
beside correct prose naming its token.

ALLOWED to carry values:

  * steelbore-color-palette/assets/**  — the canonical TOML and its generated
    mirrors (steelbore.scm is drift-gated by generate-steelbore-scm.py; the CSS
    is a rendering of the same data)

The Standard skill was exempt until v2.03. §11 was the normative palette
specification and carried the values itself, so they could not be removed here
alone — that took a two-repo change at a matching version. v2.03 inverted §11.4:
the TOML now governs every value and §11 governs the contract around it, so the
exemption is gone and the skill is checked like everything else.

Usage:
    check-palette-single-source.py [ROOT]
"""
import os
import re
import sys

ALLOWED_PREFIXES = (
    os.path.join("steelbore-color-palette", "assets"),
)

# CHANGELOG.md anywhere is exempt, and this one is not a convenience either. A
# changelog is a historical record: the v1.45 entry describing Tokyo Night's
# arrival quotes the canvas that version introduced, and that quotation is a
# statement about what was true then. Editing it to name a token instead would
# falsify the record — the entry would claim a wording the release never had.
# Those values are also inert: nothing reads a changelog to resolve a colour, so
# a stale one misleads no implementation. Live prose is what this gate is for.
EXEMPT_BASENAMES = {"CHANGELOG.md"}

# Third-party trees keep upstream's content verbatim under Standard §4.2, and
# construct-cli is code with its own fixtures rather than palette prose.
SKIP_DIRS = {".git", ".github", ".githooks", "android-skills", "orca-skills",
             "construct-cli", "Excluded", "LICENSES", "node_modules"}

HEX = re.compile(r"#[0-9A-Fa-f]{6}\b")
RGB = re.compile(r"\bRGB\(\s*\d{1,3}\s*,\s*\d{1,3}\s*,\s*\d{1,3}\s*\)", re.I)
PATTERNS = (("hex literal", HEX), ("RGB triple", RGB))


def allowed(rel):
    return any(rel == p or rel.startswith(p + os.sep) for p in ALLOWED_PREFIXES)


def main(argv):
    root = argv[0] if argv else "."
    problems = []
    scanned = 0

    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if not fn.endswith(".md"):
                continue
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, root)
            if allowed(rel) or fn in EXEMPT_BASENAMES:
                continue
            try:
                text = open(full, encoding="utf-8").read()
            except (OSError, UnicodeDecodeError):
                continue
            scanned += 1
            fenced = False
            for line_no, line in enumerate(text.splitlines(), 1):
                if line.lstrip().startswith("```"):
                    fenced = not fenced
                    continue
                if fenced:
                    continue  # markup pasted verbatim — see the module docstring
                for label, pat in PATTERNS:
                    for m in pat.finditer(line):
                        problems.append(
                            f"{rel}:{line_no}: {label} {m.group(0)} — name the "
                            f"token instead; the value belongs in "
                            f"steelbore-color-palette/assets/steelbore.toml")

    if problems:
        print("Colour values found outside the single source:", file=sys.stderr)
        for p in problems[:60]:
            print(f"  {p}", file=sys.stderr)
        if len(problems) > 60:
            print(f"  ... and {len(problems) - 60} more", file=sys.stderr)
        print(
            "\nStandard §11.4: values are read, never retyped. A skill names the "
            "token and the role it plays; the value is read from the TOML at "
            "authoring time. Restating one here means a future palette edit has "
            "to find every copy — and the copy it misses fails silently.",
            file=sys.stderr)
        return 1

    print(f"palette single-source OK ({scanned} markdown files carry no colour values)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
