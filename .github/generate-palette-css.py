#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Regenerate the generated prologue of spacecraft.css from steelbore.toml.

`spacecraft.css` is two files in one. The prologue — the header comment, the
@font-face rules, and the `:root` custom properties — restates facts that live
in `steelbore.toml`: nine hexes, the palette's role bindings, and the Standard
version §11 was last amended at. The body below it is hand-authored layout that
references those properties through `var()` and has no business being generated.

§11.4 says values are read, never retyped. Before this script the prologue
retyped them, and the `Palette: … v1.34` line in the header had been wrong
since v1.35. `check-palette-css.py` caught a value that drifted; nothing caught
the version pin, because nothing generated it.

So: everything between the sentinels is derived, and `--check` fails the build
when the file no longer matches what the TOML implies. The body is left exactly
as found.

Regenerate with:   python3 .github/generate-palette-css.py
Verify with:       python3 .github/generate-palette-css.py --check

Both the canonical copy and its synced derivatives are written, so the three
stay byte-identical by construction rather than by discipline.
"""

from __future__ import annotations

import pathlib
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parent.parent
TOML = REPO / "steelbore-color-palette" / "assets" / "steelbore.toml"

# The canonical copy first; the rest are synced derivatives (§11.4). Paths
# outside this repository are handled by their own repo's CI — the Standard's
# workflow already compares its copy against the canonical one.
TARGETS = [
    REPO / "steelbore-color-palette" / "assets" / "spacecraft.css",
    REPO / "spacecraft-texinfo-document" / "assets" / "spacecraft.css",
]

BEGIN = "/* >>> generated from steelbore.toml — do not edit below this line <<< */"
END = "/* >>> end generated — hand-authored layout follows <<< */"

# Theme whose tokens the stylesheet exposes. Modern is the §11.4 default and
# the palette every document uses absent a declaration.
THEME = "steelbore"

# Role tokens in the order they should appear, canvas and surfaces first.
ROLE_ORDER = [
    "background",
    "surface",
    "surface-alt",
    "foreground",
    "accent",
    "structure",
    "success",
    "error",
    "warning",
    "focus",
    "border",
]


def kebab(name: str) -> str:
    """`Void Navy` -> `void-navy`, the inverse of check-palette-css.py's mapping."""
    return "-".join(name.strip().lower().split())


def build_prologue(data: dict) -> str:
    meta = data["meta"]
    palette = data["palettes"][THEME]
    theme = data["themes"][THEME]

    # Map hex -> palette token name, so the CSS property is named after the
    # token rather than after the role (a hex may serve two roles).
    by_hex = {v.upper(): k for k, v in palette.items() if k != "reference"}

    # Collect the distinct hexes this theme actually binds, in role order, and
    # record every role each one carries so the comment is derived, not typed.
    roles_for: dict[str, list[str]] = {}
    order: list[str] = []
    for role in ROLE_ORDER:
        hexval = theme[role].upper()
        if hexval not in roles_for:
            order.append(hexval)
            roles_for[hexval] = []
        roles_for[hexval].append(role)

    rows = []
    for hexval in order:
        token = by_hex.get(hexval)
        if token is None:
            raise SystemExit(
                f"error: {hexval} is bound by [themes.{THEME}] but is not a named "
                f"token in [palettes.{THEME}] — every role must resolve to a token"
            )
        rows.append((f"--{kebab(token)}:", hexval, ", ".join(roles_for[hexval]), token))

    prop_w = max(len(r[0]) for r in rows)
    body = "\n".join(
        f"  {prop:<{prop_w}} {hexval};  /* {token} — {roles} */"
        for prop, hexval, roles, token in rows
    )

    return f"""{BEGIN}

/* Palette: {meta['standard']}.  Typography: §12 (Share Tech Mono / Inconsolata,
   both OFL).  Values are generated from steelbore.toml — §11.4 requires they be
   read, never retyped.  Regenerate with .github/generate-palette-css.py.

   Fonts are resolved locally and never fetched (§9.1): an installed copy is
   used where one exists, and the generic monospace fallback covers the rest.
   No third-party subresource is requested at render time. */

@font-face {{
  font-family: 'Share Tech Mono';
  src: local('Share Tech Mono'), local('ShareTechMono-Regular');
  font-display: swap;
}}

@font-face {{
  font-family: 'Inconsolata';
  src: local('Inconsolata'), local('Inconsolata-Regular');
  font-display: swap;
}}

:root {{
{body}
  color-scheme: dark;
}}

{END}"""


def render(path: pathlib.Path, prologue: str) -> str:
    text = path.read_text(encoding="utf-8")
    head, sep, rest = text.partition(BEGIN)
    if not sep:
        raise SystemExit(
            f"error: {path} has no generated-section sentinel.\n"
            f"  Expected a line reading:\n    {BEGIN}\n"
            "  Add the sentinels around the header/@font-face/:root block first."
        )
    _, sep2, tail = rest.partition(END)
    if not sep2:
        raise SystemExit(f"error: {path} has an opening sentinel but no closing {END!r}")
    return head + prologue + tail


def main() -> int:
    check = "--check" in sys.argv[1:]
    data = tomllib.loads(TOML.read_text(encoding="utf-8"))
    prologue = build_prologue(data)

    stale = []
    for path in TARGETS:
        if not path.exists():
            print(f"error: {path} not found", file=sys.stderr)
            return 1
        want = render(path, prologue)
        if check:
            if path.read_text(encoding="utf-8") != want:
                stale.append(path)
        else:
            path.write_text(want, encoding="utf-8")
            print(f"wrote {path.relative_to(REPO)} from steelbore.toml")

    if check:
        if stale:
            for path in stale:
                print(
                    f"`{path.relative_to(REPO)}` is stale — it no longer matches "
                    "`steelbore.toml`.",
                    file=sys.stderr,
                )
            print(
                "Run `python3 .github/generate-palette-css.py` and commit the result. "
                "steelbore.toml is the single source (§11.4) — correct the TOML, "
                "never the CSS.",
                file=sys.stderr,
            )
            return 1
        print(f"spacecraft.css is in sync with steelbore.toml ({len(TARGETS)} copies)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
