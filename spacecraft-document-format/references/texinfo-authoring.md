# Texinfo Authoring — Canonical Prose Format

Reference for authoring Spacecraft Software prose documents in **Texinfo** — the canonical format for manuals, references, guides, reports, specs, and books (SKILL.md §1, Standard §8). Load this for any prose deliverable unless the user explicitly asked for a word-processor file.

**Why Texinfo is canonical:** one `.texi` source compiles to plain text/Info, HTML, and PDF — and converts to GFM Markdown — from a single file. Structure and brand are defined once; every output stays in sync. No other format gives a diffable, plain-text source that fans out to all the distribution formats at once.

Docs: GNU Texinfo manual (<https://www.gnu.org/software/texinfo/manual/texinfo/>). Texinfo 7.x+ is assumed.

## §A — When a Texinfo manual is the deliverable

Mirrors Standard §8.1:

| Project type | Requirement |
|---|---|
| CLI / TUI / GUI app with substantive user-facing functionality | **MUST** ship a Texinfo manual (invocation, options, concepts, examples) |
| Library with a public API | **SHOULD** ship a Texinfo reference manual covering all public interfaces |
| Simple script / internal tooling | **MAY** skip — a well-structured `README.md` suffices |

For software projects, the manual lives at `doc/<project>.texi` (Standard §8.2). For standalone document deliverables (a report, a book), put the `.texi` wherever the deliverable is organized; the build targets and brand rules below still apply.

## §B — Minimal `.texi` skeleton (required structural elements)

Every manual includes the Standard §8.3 elements: `@dircategory`/`@direntry`, `@copying`, `@titlepage`, `@node Top` + `@top`, and a `@menu` per chapter.

```texinfo
\input texinfo
@c %**start of header
@setfilename project.info
@documentencoding UTF-8
@settitle Project Manual 1.0
@c %**end of header

@c SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
@c SPDX-License-Identifier: CC-BY-SA-4.0

@dircategory Spacecraft Software
@direntry
* Project: (project).        One-line description for the Info directory.
@end direntry

@copying
This manual is for Project version 1.0.

Copyright @copyright{} 2026 Mohamed Hammad & Spacecraft Software.

@quotation
Released under CC-BY-SA-4.0. (Use GFDL-1.3-or-later instead when distributing
alongside GPL software for GNU-collection compatibility — Standard §8.5.)
@end quotation
@end copying

@titlepage
@title Project Manual
@subtitle Version 1.0
@author Mohamed Hammad
@page
@vskip 0pt plus 1filll
@insertcopying
@end titlepage

@contents

@node Top
@top Project

@insertcopying

@menu
* Introduction::    What Project does.
* Invocation::      Command-line usage.
* Index::
@end menu

@node Introduction
@chapter Introduction
Project is@dots{}

@node Invocation
@chapter Invocation
@cindex invoking
Each documented program needs an @code{Invoking @var{program}} node.

@node Index
@unnumbered Index
@printindex cp

@bye
```

Conventions worth keeping: structure by the user's concepts (one coherent topic per node), document every option with examples, provide an Index (`@cindex`/`@printindex`), use active voice and present tense. (These are the GNU documentation conventions; `gnu-coding-standards` covers them in depth for GNU-targeted work.)

## §C — Build toolchain & targets

Standard §8.2 requires three `Makefile` targets; §8.4 fixes the output set.

| Output | Tool | Command |
|--------|------|---------|
| `.info` (+ plain text) | `makeinfo` / `texi2any` | `makeinfo project.texi` · plain text: `makeinfo --plaintext project.texi` |
| `.html` | `makeinfo --html` | `makeinfo --html --css-include=steelbore.css project.texi` |
| `.pdf` | `texi2pdf` (or HTML→PDF, §D) | `texi2pdf project.texi` |

Minimal `Makefile`:

```make
project.info: project.texi ; makeinfo $<
html:         project.texi ; makeinfo --html --css-include=steelbore.css $<
pdf:          project.texi ; texi2pdf $<
info: project.info
.PHONY: html pdf info
```

Build `.info`, `.html`, and `.pdf` and ship all three (Standard §8.4). Register `.info` at install time with `install-info` (§G).

## §D — Brand application (palette §11 + typography §12)

Texinfo's outputs theme differently; cover each:

### HTML (full brand — the primary branded output)

Pass a bundled CSS with `--css-include=steelbore.css`. Void Navy background, Molten Amber body, Share Tech Mono headings, Inconsolata body, palette-mapped headings and links (SKILL.md §7):

```css
/* steelbore.css — Standard §11 palette + §12 typography */
@import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&family=Inconsolata&display=swap');
body            { background: #000027; color: #D98E32;
                  font-family: 'Inconsolata', monospace; }
h1, h2, h3, h4  { font-family: 'Share Tech Mono', monospace; }
h1, .chapter    { color: #4B7EB0; }    /* Steel Blue   */
h2, .section    { color: #50FA7B; }    /* Radium Green */
h3, .subsection { color: #8BE9FD; }    /* Liquid Coolant */
a:link          { color: #8BE9FD; }    /* unvisited */
a:visited       { color: #4B7EB0; }    /* visited   */
code, pre, samp { font-family: 'Inconsolata', monospace; }
```

Verify Void Navy is visible in a browser and both font families load.

### PDF

`texi2pdf` (stock Texinfo/TeX) produces a structurally correct PDF but does **not** support a page background colour through the standard path — accept the default page and rely on HTML for the fully-branded rendering, **or** take the brand-faithful route:

- **HTML → PDF (brand-faithful):** render the branded HTML to PDF so the Void Navy CSS carries through — `weasyprint project.html project.pdf`, or a headless browser (`chromium --headless --print-to-pdf`). This keeps the §11 palette and §12 fonts in the PDF.

Pick `texi2pdf` when structure/printability is what matters; pick HTML→PDF when the PDF must carry the full brand. Either way, A4 portrait geometry (SKILL.md §3).

### Info / plain text

`.info` and `--plaintext` output carry no embedded colour — they render in the reader/terminal, which already follows the palette on Steelbore OS (and via `spacecraft-theme-factory` themes elsewhere). No action needed beyond clean structure.

## §E — Markdown & plain-text conversion

A `.texi` is already plain text, so it needs **no** mandatory GFM companion (SKILL.md §2). Produce Markdown only when a consumer wants it:

```sh
pandoc -f texinfo -t gfm project.texi -o project.md     # GFM rendering
makeinfo --plaintext project.texi -o project.txt        # plain-text rendering
```

When you do emit a `.md` from a `.texi`, it follows the GFM house style in `markdown-companion.md` (ATX headings, GFM tables, no raw HTML beyond the metadata comment).

## §F — Licensing

Texinfo manuals are **document-class** (Standard §4.1.1):

- **Default `CC-BY-SA-4.0`.** Use `CC-BY-4.0` only when maximal reuse is intended.
- **`GFDL-1.3-or-later` is the permitted alternative** when the manual ships alongside GPL-licensed software and GNU-documentation-collection compatibility is wanted (Standard §8.5).
- Declare the licence in the `@copying` block (§B) **and** as an inline two-tag SPDX header in the `.texi` (`@c SPDX-FileCopyrightText:` / `@c SPDX-License-Identifier:`).
- Ship the licence text in `LICENSES/<id>.txt`; `reuse lint` must pass (Standard §4.3).

## §G — Packaging integration (Standard §8.6)

Package manifests install the `.info` and register it with `install-info`:

| Package manager | Requirements |
|---|---|
| **Guix** (`packaging/guix.scm`) | add `texinfo` as a native input; run `install-info` in the install phase |
| **Nix** (`packaging/default.nix`) | add `texinfo` to `nativeBuildInputs`; the standard Autoconf/Make `installPhase` runs `install-info` automatically |
| **PKGBUILD** (`packaging/PKGBUILD`) | add `texinfo` to `makedepends`; `install -Dm644` the `.info`; call `install-info` in `post_install` |

## §H — Acceptance checklist (Texinfo)

In addition to SKILL.md §8 (general acceptance):

- [ ] `.texi` carries the §8.3 structural elements (`@dircategory`/`@direntry`, `@copying`, `@titlepage`, `@node Top`/`@top`, per-chapter `@menu`).
- [ ] Inline SPDX two-tag header present in the `.texi`; licence stated in `@copying`.
- [ ] `make info`, `make html`, `make pdf` all succeed; `.info`/`.html`/`.pdf` produced.
- [ ] HTML: Void Navy background visible, Share Tech Mono headings + Inconsolata body loaded via the bundled CSS.
- [ ] PDF: A4 portrait; brand carried (HTML→PDF route) or structure-only (`texi2pdf`) as chosen.
- [ ] `install-info` hook present in all three package manifests (Standard §5.5 / §8.6).
- [ ] Licence text in `LICENSES/`; `reuse lint` clean.
