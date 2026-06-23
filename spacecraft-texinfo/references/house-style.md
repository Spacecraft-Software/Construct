# House Style for Texinfo

Spacecraft Software conventions for `.texi` source, with copy-paste templates.
The Steelbore Standard is canonical; where this file and the Standard disagree,
the Standard wins. Section numbers below cite the Steelbore Standard: §4
licensing/REUSE, §8 Documentation (Texinfo), §11 palette, §12 typography,
§14 dates, §15 attribution.

## 1. The inline SPDX/REUSE header (§4.3)

The old "documents are exempt from SPDX" rule is gone. Every file carries two
REUSE tags. A `.texi` is plain text and **can** carry them inline as Texinfo
comments, so it does — no `.license` sidecar, no `REUSE.toml` entry needed for the
source itself. Put them on the first lines, right after `\input texinfo`:

```texinfo
\input texinfo   @c -*- texinfo -*-
@c SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
@c SPDX-License-Identifier: CC-BY-SA-4.0
```

Rules:

- Use the **full** address `Mohamed.Hammad@SpacecraftSoftware.org` here. `reuse
  lint` parses this line; the `[at]` obfuscation form breaks it and is forbidden
  in SPDX headers (§15.2).
- The `SPDX-License-Identifier` must match the licence declared in `@copying`
  (CC-BY-SA-4.0 *or* GFDL-1.3-or-later — pick one, keep them in sync).
- A `Makefile`, `build.sh`, or other **software** shipped beside the manual keeps
  its own header with `SPDX-License-Identifier: GPL-3.0-or-later`.
- The licence's verbatim text lives in the project's `LICENSES/` directory
  (`LICENSES/CC-BY-SA-4.0.txt` or `LICENSES/GFDL-1.3-or-later.txt`).

## 2. The `@copying` block — CC-BY-SA-4.0 (house default)

`@copying` declares the permissions once; `@insertcopying` drops the text into the
title page and the Top node. Year in ISO form; version/date via `@set` flags.

```texinfo
@set VERSION 1.0
@set UPDATED 2026-06-19

@copying
This manual is for @value{PROJECT} (version @value{VERSION},
updated @value{UPDATED}).

Copyright @copyright{} 2026 Mohamed Hammad & Spacecraft Software.

@quotation
This document is licensed under the Creative Commons
Attribution-ShareAlike 4.0 International License (CC-BY-SA-4.0). To view a
copy of this licence, visit
@url{https://creativecommons.org/licenses/by-sa/4.0/}.
@end quotation
@end copying
```

## 3. The `@copying` block — GFDL-1.3-or-later (GNU-upstream alternative)

Use this only when the manual must align with an upstream GNU package. Swap **both**
the SPDX tag (`@c SPDX-License-Identifier: GFDL-1.3-or-later`) and this paragraph.
The GFDL requires its standard permission notice; keep it verbatim:

```texinfo
@copying
This manual is for @value{PROJECT} (version @value{VERSION},
updated @value{UPDATED}).

Copyright @copyright{} 2026 Mohamed Hammad & Spacecraft Software.

@quotation
Permission is granted to copy, distribute and/or modify this document
under the terms of the GNU Free Documentation License, Version 1.3 or
any later version published by the Free Software Foundation; with no
Invariant Sections, no Front-Cover Texts, and no Back-Cover Texts. A
copy of the license is included in the section entitled ``GNU Free
Documentation License''.
@end quotation
@end copying
```

When GFDL is used, include the licence text as an appendix
(`@node GNU Free Documentation License` / `@appendix …`) — the standard
`fdl.texi` include from the Texinfo distribution does this; `@include fdl.texi`.
Default house choice (CC-BY-SA-4.0) needs no such appendix.

## 4. Title page & attribution (§15)

```texinfo
@titlepage
@title @value{PROJECT} Manual
@subtitle For version @value{VERSION}
@author Mohamed Hammad
@page
@vskip 0pt plus 1filll
@insertcopying

Maintained by Mohamed Hammad
@email{Mohamed.Hammad@@SpacecraftSoftware.org}.
@uref{https://PROJECT.SpacecraftSoftware.org/}
@end titlepage
```

- **Escaping `@`:** a literal at-sign is always written `@@` in Texinfo, including
  inside `@email{...}`. So the address is `@email{Mohamed.Hammad@@SpacecraftSoftware.org}`
  (GNU's own example is `@email{bug-texinfo@@gnu.org}`). The SPDX `@c` header is
  the one place to write the address *un*-escaped — comment lines are not parsed
  for `@`-commands, and `reuse lint` needs the raw address.
- **Email obfuscation (§15.2):** the `[at]` form is for *plain running prose*
  where the address is not a live link, e.g. `Mohamed.Hammad [at]
  SpacecraftSoftware.org`. Do not use it inside `@email{}` (which builds a real
  `mailto:` link) or in the SPDX header.
- Replace `PROJECT` with the project subdomain from Standard §15.1 (e.g.
  `https://Gitway.SpacecraftSoftware.org/`).
- The copyright year is the first-release year or current year, or a range
  (`2025-2026`) for multi-year projects.

## 5. Dates (§14)

- ISO 8601 only: `YYYY-MM-DD`. Today is rendered `2026-06-19`.
- **Never `@today{}`** — it emits "19 June 2026"-style output, not ISO. If a
  build-time date is unavoidable, set it explicitly with `@set UPDATED` and
  reference `@value{UPDATED}`.
- Durations (rare in manuals) use ISO 8601 (`PT1H30M`); units metric-primary.

## 6. Palette & typography → applied at OUTPUT (§11/§12)

Texinfo source is format-agnostic; the brand is applied when rendering.

| Output | Brand application |
|--------|-------------------|
| **Info** | Inherently plain text — no theming. This is correct; do not fight it. |
| **plain text** (`--plaintext`) | Plain — no theming. |
| **HTML** | Full theme via `assets/spacecraft.css` (`--css-include`). Void Navy `#000027` background, Share Tech Mono headings, Inconsolata body, palette links/code. |
| **PDF** (TeX) | A4 via `@afourpaper`; font/colour theming is limited in `texinfo.tex` — see `building.md` §PDF for the supported subset (paper size + body font note). Do not promise full palette parity in PDF. |
| **DocBook / EPUB** | Structural; downstream tools apply their own styling. |

Palette tokens (Standard §11, cached): Void Navy `#000027` (background),
Molten Amber `#D98E32` (body), Steel Blue `#4B7EB0` (H1/accent/visited link),
Radium Green `#50FA7B` (H2/success), Liquid Coolant `#8BE9FD` (H3/info/link),
Red Oxide `#FF5C5C` (warning/error). Fonts (Standard §12): Share Tech Mono
(headings), Inconsolata (body/code) — both OFL.

## 7. File naming

- Source file: lowercase project name, `.texi` extension (`gitway.texi`,
  `mawaqit.texi`). For multi-file manuals, a top file `@include`s chapter files.
- `@setfilename` names the **Info** output: `@setfilename gitway.info`.
- Keep the `.texi` at the documentation root (e.g. `doc/gitway.texi`) with
  `LICENSES/` and any `Makefile` beside it.

## 8. Quick compliance gate

Before shipping a `.texi`, confirm: inline SPDX header (full address, both tags) ·
`@copying` matches the SPDX licence · `@documentencoding UTF-8` · ISO dates, no
`@today{}` · attribution block (maintainer, contact, subdomain, ISO year) · builds
clean · HTML themed, PDF A4 · `LICENSES/<id>.txt` present.
