#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check spacecraft.css custom properties against the canonical palette.

`spacecraft.css` is the third mirror of `steelbore.toml`, after `steelbore.scm`
and the palette prose. The `.scm` mirror has been drift-gated since it shipped —
`generate-steelbore-scm.py --check` fails the build if it diverges. The CSS had
no gate at all, in either of the two repositories that carry it.

That asymmetry is not cosmetic. The CSS is what actually renders the Standard's
own HTML output: `texi2any --css-include=spacecraft.css` embeds it verbatim, so
a hex that drifts here is a wrong colour in the published document, silently and
with the document still building cleanly.

WHY THIS CHECKS RATHER THAN GENERATES, unlike the .scm mirror. The `.scm` file
is entirely derived, so regenerating it is lossless. `spacecraft.css` is nine
generated-looking lines of custom properties on top of roughly seventy-five
hand-authored lines of layout that reference them through `var(--token)`.
Generating the whole file would throw that layout away; generating only the
`:root` block and splicing it back is a rewrite of a hand-maintained file for no
gain over simply verifying it. Drift is what matters, and drift is detectable
without regeneration.

WHAT IT CHECKS. Every `--custom-property: #RRGGBB` declaration is matched to a
palette token by name — `--void-navy` to "Void Navy" — and its value compared to
the TOML's. This catches both directions of drift: a value edited in one place
and not the other, and a property whose name no longer corresponds to any token
(a rename that left the CSS behind).

A hex in the CSS that belongs to no declaration — inside a gradient, a shadow,
a border shorthand — is reported too, because a literal there is exactly the
restatement the single-source rule exists to prevent; it should be `var(--token)`.

BOTH COPIES. The file exists in Construct (shipped with the palette skill) and
in the Standard repository (used to build its HTML). They are byte-identical
today and must stay that way, so a second check compares any copies given to
each other. The Standard's CI runs this file out of its `.construct` checkout
against its own copy, the same way it already runs `check-skill-refs.py`.

Usage:
    check-palette-css.py [--toml PATH] [CSS ...]

With no CSS arguments every `spacecraft.css` under the working directory is
checked. `--toml` defaults to the palette skill's copy.
"""
import argparse
import os
import re
import sys
import tomllib

DEFAULT_TOML = os.path.join("steelbore-color-palette", "assets", "steelbore.toml")

DECL = re.compile(r"--([a-z0-9-]+)\s*:\s*(#[0-9A-Fa-f]{6})\b")
ANY_HEX = re.compile(r"#[0-9A-Fa-f]{6}\b")


def palette_values(toml_path):
    """Map 'Void Navy' -> '#000027' for every token the TOML defines."""
    with open(toml_path, "rb") as fh:
        data = tomllib.load(fh)
    out = {}

    def walk(node, key=""):
        if isinstance(node, dict):
            for k, v in node.items():
                walk(v, k)
        elif isinstance(node, str) and re.fullmatch(r"#[0-9A-Fa-f]{6}", node):
            out.setdefault(key.strip().lower(), node.upper())

    walk(data)
    return out


def kebab_to_name(prop):
    """`void-navy` -> `void navy`, the TOML's key modulo case and spacing."""
    return prop.replace("-", " ").strip().lower()


def check_file(path, values, problems):
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as e:
        problems.append(f"{path}: cannot read: {e}")
        return
    declared = {}
    for line_no, line in enumerate(text.splitlines(), 1):
        for m in DECL.finditer(line):
            declared[(line_no, m.group(1))] = m.group(2).upper()

    for (line_no, prop), hexval in declared.items():
        name = kebab_to_name(prop)
        canonical = values.get(name)
        if canonical is None:
            problems.append(
                f"{path}:{line_no}: `--{prop}` matches no palette token. Either "
                f"the token was renamed and this property was left behind, or "
                f"the property is not a palette colour and should not hold a "
                f"literal.")
        elif canonical != hexval:
            problems.append(
                f"{path}:{line_no}: `--{prop}` is {hexval}, but the palette "
                f"defines {name.title()} as {canonical}. steelbore.toml is the "
                f"single source (§11.4) — correct the CSS, not the TOML.")

    # Hexes outside a custom-property declaration: literals that should be var()
    declared_spans = set()
    for line_no, line in enumerate(text.splitlines(), 1):
        for m in DECL.finditer(line):
            declared_spans.add((line_no, m.start(2)))
    for line_no, line in enumerate(text.splitlines(), 1):
        for m in ANY_HEX.finditer(line):
            if (line_no, m.start()) not in declared_spans:
                problems.append(
                    f"{path}:{line_no}: literal {m.group(0)} outside a custom "
                    f"property — reference the token with var(--…) instead.")


def main(argv):
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--toml", default=DEFAULT_TOML)
    ap.add_argument("css", nargs="*")
    args = ap.parse_args(argv)

    if not os.path.exists(args.toml):
        print(f"canonical palette not found: {args.toml}", file=sys.stderr)
        return 1
    values = palette_values(args.toml)

    paths = args.css
    if not paths:
        for dirpath, dirnames, filenames in os.walk("."):
            dirnames[:] = [d for d in dirnames if d not in {".git", "node_modules"}]
            if "spacecraft.css" in filenames:
                paths.append(os.path.join(dirpath, "spacecraft.css"))
    if not paths:
        print("no spacecraft.css found — nothing to check")
        return 0

    problems = []
    for p in sorted(paths):
        check_file(p, values, problems)

    # Every copy must be identical; divergence between them is drift too.
    if len(paths) > 1:
        blobs = {}
        for p in sorted(paths):
            try:
                blobs[p] = open(p, "rb").read()
            except OSError:
                pass
        first = next(iter(blobs), None)
        for p, blob in blobs.items():
            if first and blob != blobs[first]:
                problems.append(
                    f"{p}: differs from {first}. The copies are byte-identical "
                    f"by design; update them together.")

    if problems:
        print("spacecraft.css palette drift:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        print(
            "\nspacecraft.css is embedded verbatim into the Standard's HTML "
            "output, so a drifted value is a wrong colour in the published "
            "document with the build still passing. Values come from "
            "steelbore.toml.", file=sys.stderr)
        return 1

    print(f"spacecraft.css OK ({len(paths)} file(s), "
          f"{len(values)} palette tokens available)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
