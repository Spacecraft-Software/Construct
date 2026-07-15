---
name: spacecraft-texinfo-document
description: >
  The how-to layer for authoring, building, linting, and converting GNU Texinfo
  (.texi/.texinfo/.txi) in Spacecraft Software house style. Texinfo is the
  CANONICAL prose format for Spacecraft manuals, guides, and references — one
  source compiles to Info, HTML, PDF, DocBook, text, and EPUB.
  ALWAYS consult this skill when a .texi file is involved, or the user mentions
  Texinfo, makeinfo, texi2any, texi2pdf, an Info or GNU-style manual,
  @deffn/@node/@menu, or asks to write or build a manual, document a
  crate/library/CLI API, generate Info/HTML/PDF from one source, fix a .texi that
  won't compile, or convert Markdown/README into a manual. Prefer it over generic
  Markdown/office-doc skills for long-form prose — Spacecraft prose is Texinfo;
  office/Markdown are downstream. Do NOT use it for chat answers,
  code comments, or GFM companions (that is spacecraft-markdown-document).
license: GPL-3.0-or-later
metadata:
  maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
  website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Software Texinfo — Single-Source Documentation

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later (skills are software-class, Standard §4.1.1)
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

You are a documentation assistant working in **GNU Texinfo**. Texinfo is a markup
language in which a single `.texi` source compiles to many output formats — Info,
HTML, PDF, DocBook, plain text, EPUB — so structure and house brand are defined
**once** and rendered everywhere. In the Spacecraft Software document hierarchy
(`spacecraft-document-format`), **Texinfo is canonical for prose**: manuals,
references, guides, reports, and books. ODF/MS-Office are downstream office
formats; Markdown is a companion, not a prose authoring target. This skill is the
deep *how-to* beneath that declaration.

This SKILL.md is the map. It carries the **house-style essentials** and the
**load-order index**; the mechanics live in `references/` and ready-to-use files
live in `assets/`. Pull only what the task needs — do not eagerly load every
reference.

## §1 — The four workflows (and what to load)

Identify which job you are doing, then load only the matching pieces. Every job
shares the house-style core in §2.

| Workflow | Trigger | Load |
|----------|---------|------|
| **Author** a new manual | "write a manual", "document this API", starting a `.texi` | `references/authoring.md`; for API/reference docs also `references/definition-commands.md`; start from `assets/template.texi` (prose) or `assets/software-manual.texi` (API) |
| **Build** to output formats | "build the docs", "make Info/HTML/PDF", `makeinfo`, `texi2any` | `references/building.md`; `assets/spacecraft.css` (HTML theme); `assets/build.sh` (all-formats helper) |
| **Lint / fix** existing `.texi` | "won't compile", node/menu errors, warnings | `references/linting.md` |
| **Convert** Markdown/other → Texinfo (or back) | "turn this README into a manual", "Texinfo from Markdown" | `references/converting.md`; then `references/authoring.md` to house-polish |

Need a specific `@`-command's exact syntax at any point? → `references/command-reference.md`
(the complete command map, organized by category, with a table of contents).

The toolchain (`texi2any`/`makeinfo`, `texi2pdf`) may not be installed. Provision
it ephemerally per `spacecraft-missing-pkg` — Guix (`guix shell texinfo`) or Nix
(`nix shell nixpkgs#texinfo`) first; never a system package manager. Details in
`references/building.md`.

## §2 — House-style essentials (the non-negotiables)

These bind every `.texi` you write or touch. They come from The Steelbore
Standard; the full rationale and copy-paste templates are in
`references/house-style.md`. The highlights:

- **SPDX/REUSE header, inline (Standard §4.3).** A `.texi` is plain text, so it
  carries the two REUSE tags as Texinfo comments at the very top — never a
  `.license` sidecar. Use the **full** contact address here (so `reuse lint`
  parses it):

  ```texinfo
  \input texinfo   @c -*- texinfo -*-
  @c SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
  @c SPDX-License-Identifier: CC-BY-SA-4.0
  ```

- **Document license (Standard §4.1.1).** A manual is document-class →
  **`CC-BY-SA-4.0`** by default, declared in an `@copying` block and surfaced via
  `@insertcopying`. **`GFDL-1.3-or-later`** is the documented alternative for
  manuals that must align with an upstream GNU package — swap the SPDX tag and the
  `@copying` permission paragraph together (both templates in `house-style.md`).
  Software-class artifacts shipped *alongside* a manual (a `Makefile`, `build.sh`)
  stay `GPL-3.0-or-later`.

- **Encoding & language.** Always declare `@documentencoding UTF-8` and
  `@documentlanguage en` (or the project's locale) in the header. UTF-8 is the
  default but stating it is house style.

- **ISO 8601 dates only (Standard §14).** Write dates as `YYYY-MM-DD`
  (e.g., `2026-06-19`). **Do not use `@today{}`** — it renders the non-ISO
  "1 Jan 2000" style. Put the date in an `@set UPDATED 2026-06-19` flag and
  reference `@value{UPDATED}`, or write it literally.

- **Attribution (Standard §15).** The `@copying` block and `@titlepage` carry the
  maintainer (Mohamed Hammad), the contact `Mohamed.Hammad@SpacecraftSoftware.org`,
  the project subdomain `https://<Project>.SpacecraftSoftware.org/`, and an ISO
  copyright year. In running prose (not the SPDX header) the scraper-resistant
  `Mohamed.Hammad [at] SpacecraftSoftware.org` form is permitted.

- **Palette & typography apply to OUTPUT, not source (Standard §11/§12).** Info
  and plain text are inherently unstyled — that is fine. **HTML** gets the full
  Spacecraft theme via `assets/spacecraft.css` (Void Navy `#000027` background,
  Share Tech Mono headings, Inconsolata body, palette links/code). **PDF** is
  produced through TeX; apply A4 (`@afourpaper`, per house ISO-A4 geometry) and
  the documented font/colour caveats in `references/building.md`.

- **No `.md` companion required.** The GFM-companion rule that binds ODF/MS-Office
  deliverables does **not** apply here: the `.texi` *is* the reviewable plain-text
  source. (You may still *generate* Markdown as an output via `--plaintext`/pandoc
  when asked — see `converting.md` — but it is not a mandatory sibling.)

## §3 — The canonical skeleton (always start here)

Every Spacecraft manual follows GNU's prescribed file structure. The full,
compilable, house-styled starting points are `assets/template.texi` (prose) and
`assets/software-manual.texi` (API reference). The required ordering, top to
bottom:

1. `\input texinfo` line **with** the inline SPDX `@c` header (§2).
2. **Header block** between `@c %**start of header` / `@c %**end of header`:
   `@setfilename project.info`, `@documentencoding UTF-8`,
   `@documentlanguage en`, `@settitle Project Manual <version>`.
3. **`@copying` … `@end copying`** — the permission/licence text (CC-BY-SA-4.0).
4. **`@dircategory` + `@direntry`** — the Info directory entry.
5. **`@titlepage`** — `@title`, `@subtitle`, `@author`, then `@vskip 0pt plus
   1filll` + `@insertcopying`.
6. **`@contents`** — the table of contents.
7. **`@ifnottex` `@node Top` `@top <Name>` `@insertcopying` `@end ifnottex`** —
   the Top node (suppressed in print/DocBook).
8. **Master `@menu`** listing the chapters.
9. The body: one `@node` immediately followed by its sectioning command
   (`@chapter`, `@section`, …) for each unit.
10. **`@bye`** on its own line.

The cardinal correctness rule that prevents most build failures: **every `@node`
is paired one-to-one with a sectioning command, and the node graph is mirrored by
the menus.** If `makeinfo` complains about nodes or menus, this pairing is almost
always the cause — see `references/linting.md`.

## §4 — Build & verify (the short version)

Treat `makeinfo` as both compiler **and** linter: a manual is not done until it
builds with **zero errors and zero warnings**. The minimum verification loop
(full flag reference in `references/building.md`):

```sh
# Info (the native format) — also the strictest validator
makeinfo --no-split project.texi

# HTML, single page, house-themed
texi2any --html --no-split --css-include=spacecraft.css project.texi

# PDF (via TeX), A4 paper without editing the source
texi2pdf --texinfo=@afourpaper project.texi
```

`makeinfo` and `texi2any` are the same program. Prefer `texi2any` when selecting
output formats explicitly (`--html`, `--docbook`, `--plaintext`, `--epub3`,
`--info`); `makeinfo` defaults to Info. The all-formats helper `assets/build.sh`
wraps these (POSIX `sh`; a Nushell variant is noted inline, per the user's shell
preference).

## §5 — Acceptance checklist (run before declaring done)

- [ ] Inline SPDX `@c` header present with **both** tags and the full address.
- [ ] `@copying` block present, `@insertcopying` used on the title page **and** in
      the Top node; licence text matches the SPDX tag (CC-BY-SA-4.0 or GFDL).
- [ ] `@documentencoding UTF-8` and `@documentlanguage` declared.
- [ ] Dates are ISO 8601; no `@today{}`.
- [ ] Every `@node` has a matching sectioning command; menus mirror the node tree.
- [ ] Builds with `makeinfo --no-split` at **zero errors, zero warnings**.
- [ ] HTML build uses `spacecraft.css`; PDF build uses A4.
- [ ] Attribution (maintainer, contact, project subdomain, ISO year) present.
- [ ] `reuse lint` would pass for the `.texi` (header is REUSE-valid).

## §6 — Cross-references

- `spacecraft-document-format` — the format hierarchy that declares Texinfo
  canonical for prose; load it when the deliverable also needs ODF/MS-Office
  renderings or a generated Markdown companion.
- `spacecraft-standard-constitution` — The Steelbore Standard: §4 licensing/REUSE, §11 palette,
  §12 typography, §14 dates/UTC, §15 attribution. The Standard always wins on
  conflict.
- `gnu-coding-standards` — when the manual targets an actual GNU package; that
  skill names Texinfo as canonical and this skill is its execution layer.
- `spacecraft-missing-pkg` — to provision `texinfo`/`texlive` ephemerally.
- `spacecraft-markdown-document` — for GFM companions and chat-bound Markdown
  (the *non*-prose-deliverable case).
- `spacecraft-brand-guidelines` — palette/typography quick reference for the
  HTML/PDF theme.

*— Built by Spacecraft Software —*
