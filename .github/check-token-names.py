#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Fail when a palette token name in steelbore.toml describes two colours.

Palette token names look per-palette, but every consumer that resolves a name
to a value treats them as one flat namespace. `check-palette-css.py` maps
`--void-navy` to "void navy" and walks every `[palettes.*]` table
first-writer-wins; a name bound to two hexes therefore resolves to whichever
palette appears earlier in the file, and a stylesheet declaring that token is
checked against the wrong palette's value while reporting success.

That is what happened to `Signal Green`, which named `#28C76F` in Steelbore
Blue and `#9ECE6A` in Tokyo Night from v1.39 until the v2.06 rename. It never
misfired only because no stylesheet happened to declare it. This gate makes the
next one a build failure instead of a latent one.

Shared tones are the reason this checks values rather than banning reuse
outright: `Ember Red`, `Solar Amber`, `Mint Signal` and `Ember Lift` are
deliberately carried across palettes at one value each, and that is correct.
What is never correct is one name meaning two colours.
"""

from __future__ import annotations

import pathlib
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parent.parent
TOML = REPO / "steelbore-color-palette" / "assets" / "steelbore.toml"


def main() -> int:
    try:
        data = tomllib.loads(TOML.read_text(encoding="utf-8"))
    except FileNotFoundError:
        print(f"error: {TOML} not found", file=sys.stderr)
        return 1
    except tomllib.TOMLDecodeError as exc:
        print(f"error: {TOML} is not valid TOML: {exc}", file=sys.stderr)
        return 1

    palettes = data.get("palettes")
    if not palettes:
        print("error: steelbore.toml has no [palettes.*] tables", file=sys.stderr)
        return 1

    # name -> hex -> [palette slugs]
    seen: dict[str, dict[str, list[str]]] = {}
    for slug, palette in palettes.items():
        for key, value in palette.items():
            if key == "reference" or not isinstance(value, str):
                continue
            seen.setdefault(key.strip().lower(), {}).setdefault(
                value.upper(), []
            ).append(slug)

    collisions = {name: v for name, v in seen.items() if len(v) > 1}
    if not collisions:
        print(f"token names OK ({len(seen)} distinct names across {len(palettes)} palettes)")
        return 0

    for name, by_hex in sorted(collisions.items()):
        print(f"`{name}` names {len(by_hex)} different colours:", file=sys.stderr)
        for hexval, slugs in sorted(by_hex.items()):
            print(f"    {hexval}  in  {', '.join(sorted(slugs))}", file=sys.stderr)
        print(
            "  A token name is a flat namespace across the whole file — rename one,\n"
            "  or make them the same colour if they were meant to be a shared tone.",
            file=sys.stderr,
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
