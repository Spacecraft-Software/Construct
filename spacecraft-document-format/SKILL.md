---
name: spacecraft-document-format
description: >
  Authoring rules for Spacecraft Software documents — the format router. Texinfo is
  canonical for prose (manuals, references, guides, reports, books): one .texi source
  compiles to plain text/Info, HTML, and PDF and converts to GFM, so structure and brand
  are defined once (Standard §8). ODF (.odt) is the secondary prose format and the primary
  format for spreadsheets (.ods) and presentations (.odp); Microsoft Office
  (.docx/.xlsx/.pptx) is the last resort, only when a consumer requires an MS-native file.
  Routes prose authoring to the spacecraft-texinfo skill and GFM rendering/companions to
  spacecraft-markdown-document. Every binary ODF/MS-Office deliverable MUST ship a
  same-named GFM (.md) companion; Texinfo is exempt (the .texi is already plain text). PDF
  is always an export, never hand-authored. Documents default to CC-BY-SA-4.0
  (GFDL-1.3-or-later permitted for Texinfo manuals). All rich-text outputs apply Void Navy
  (#000027) and the Standard §11 palette + §12 typography.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Software Document Format — Texinfo-First Documents & Office Suite

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

You are a document-generation assistant. Whenever you create or export documents, apply Spacecraft Software's theme exactly and consistently across the entire deliverable (page/slide background, styles, headings, tables, charts, shapes, and exported HTML/PDF appearance). The details for each format live in `references/`; load only the ones you need (see *Load-order index* below).

## §1 — Format priority (non-negotiable)

Choose by **document shape** first, then by format priority within that shape.

**Prose documents** — manuals, references, guides, reports, specs, books:

| Priority        | Format          | Extension | Role |
|-----------------|-----------------|-----------|------|
| 1 — **canonical** | **Texinfo**   | `.texi`   | Single source of truth. One file compiles to plain text/Info, HTML, and PDF, and converts to GFM Markdown. Standard §8. |
| 2 — secondary   | **ODF Text**    | `.odt`    | When a WYSIWYG/word-processor deliverable is required (LibreOffice, Google Docs). |
| 3 — last resort | **MS Office**   | `.docx`   | Proprietary fallback — only when a consumer explicitly requires an MS-native file. |

**Spreadsheets & presentations** — Texinfo cannot represent tabular workbooks or slide decks, so **ODF stays canonical** here:

| Shape         | Primary (ODF) | Last resort (MS Office, on explicit request only) |
|---------------|---------------|---------------------------------------------------|
| Spreadsheet   | `.ods`        | `.xlsx`                                           |
| Presentation  | `.odp`        | `.pptx`                                           |

**Generated & companion artifacts** (never the canonical source):

| Artifact                | What it is | Reference |
|-------------------------|------------|-----------|
| Texinfo outputs         | `.info` (+ plain text), `.html`, `.pdf` — **built** from the `.texi` via `make info` / `make html` / `make pdf`, never hand-authored | `texinfo-authoring.md` |
| GFM Markdown `.md`       | **Mandatory** companion for every binary ODF/MS-Office deliverable; **optional** for Texinfo (the `.texi` is already plain text) | `markdown-companion.md` |
| PDF from ODF/MS-Office   | Rendered via LibreOffice headless — only when the source is ODF/MS-Office rather than Texinfo | `pdf-export.md` |

**Rules:**

- **Default to Texinfo for any prose document** unless the user explicitly asks for a word-processor file (`.odt`/`.docx`), or the content is inherently a spreadsheet or slide deck. When unsure for prose, ask once and default to Texinfo.
- Texinfo is canonical precisely because **one `.texi` source compiles to text, HTML, and PDF — and converts to GFM** — so the structure and brand are defined once and every output stays in sync. Don't author the same prose document separately per output format.
- ODF is the **secondary** prose target and the **primary** spreadsheet/presentation target. MS Office is **always** the last resort, used only on explicit request.
- **PDF is an export, not an authored format.** Produce it from Texinfo via `texi2pdf`, or from an ODF/MS-Office source via LibreOffice headless. Never hand-author PDF.
- XLSX exception worth knowing: if the user's target is Google Sheets specifically, `.xlsx` round-trips with higher fidelity than `.ods`. The default is still ODS; the user makes the explicit exception. See `ms-office-authoring.md` §D.

## §2 — GFM Markdown companion (binary deliverables)

Every `.odt`/`.ods`/`.odp` and every `.docx`/`.xlsx`/`.pptx` deliverable lands with a sibling `.md` of the **same base name** in the **same directory**. Examples:

- `quarterly-report.odt` → `quarterly-report.md`
- `budget-2026.xlsx` → `budget-2026.md`
- `pitch.odp` → `pitch.md`

Why: the `.md` represents the document's content in **GitHub-Flavored Markdown (GFM)** so that diffs are reviewable, content is accessible to text-only tools (terminals, grep, screen readers, agent context windows), and agents can reason about the content without parsing binary office formats. The `.md` also serves as the authoritative content source — if the rich-text file and the markdown ever disagree, the markdown wins on regeneration.

**Texinfo is exempt from the pairing rule.** A `.texi` source is already plain text — diffable, greppable, and agent-readable — so it needs no separate `.md` companion. GFM is instead an **optional generated output** of Texinfo (via `pandoc -f texinfo -t gfm`); produce it only when a consumer wants a Markdown rendering. See `texinfo-authoring.md` §E.

Markdown cannot carry page colour, fonts, or page geometry. Do **not** try to render visual styling in `.md` (no inline HTML, no colour markup); the visual layer lives only in the source file. The companion `.md` carries an informational HTML comment at the top noting the source format and palette reference — see `markdown-companion.md`.

If both an ODF and an MS Office version of the same content are produced (rare), they share a single companion `.md`.

## §3 — Hard requirements (every deliverable)

These bind every format you produce. Violations are blockers, not preferences.

- **Page/slide/HTML background: Void Navy `#000027`** — Standard §11 mandate. Non-negotiable. The per-format recipes (`texinfo-authoring.md` §D for HTML/PDF, `odf-authoring.md` §C, `ms-office-authoring.md` §B) are not optional shortcuts; the colour silently drops in major readers if the layers aren't all applied.
- **Typography:** Share Tech Mono for all headings (and sheet headers, slide titles, chart titles). Inconsolata for body, cell content, code, slide bullets, captions. Standard §12.
- **Texinfo brand output.** Carry the palette and typography into Texinfo **HTML** via a bundled CSS (`makeinfo --css-include`); apply Void Navy as the **PDF** page colour where the TeX toolchain allows; `.info`/plain-text output inherits the reader/terminal theme (which already follows the palette on Steelbore OS). Details in `texinfo-authoring.md` §D.
- **Page geometry:**
  - Texinfo PDF / ODT / DOCX — ISO A4 portrait (210 × 297 mm). Never Letter, Legal, or any non-A4 size.
  - Spreadsheets (`.ods`/`.xlsx`) — A4 portrait print area; portrait orientation in print settings.
  - Presentations (`.odp`/`.pptx`) — 16:9 widescreen (1920 × 1080 px equivalent).
- **GFM companion is mandatory for binary deliverables.** Never ship a tier-1 ODF/MS-Office file without its `.md` sibling. (Texinfo is exempt — §2.) If you only have scope for one, ship the markdown alone — never the binary alone.
- **Licensing & REUSE coverage.** Document deliverables are **document-class** artifacts: they default to **`CC-BY-SA-4.0`** (use `CC-BY-4.0` only when maximal reuse is intended), per Standard §4.1.1. A **Texinfo manual distributed alongside GPL-licensed software MAY instead use `GFDL-1.3-or-later`** for compatibility with GNU documentation collections (Standard §8.5). Documents are **no longer SPDX-exempt** — §4.3 retired that carve-out:
  - A binary office file (`.odt`/`.ods`/`.odp`/`.docx`/`.xlsx`/`.pptx`/`.pdf`) cannot carry an inline header → cover it with a `.license` sidecar **or** a `REUSE.toml` entry.
  - A plain-text `.texi` or `.md` companion **can and should** carry the two-tag SPDX header inline (`SPDX-FileCopyrightText` + `SPDX-License-Identifier`). A Texinfo manual also states its licence in its `@copying` block.
  - Ship the chosen licence text in `LICENSES/`; `reuse lint` must pass.

## §4 — Palette cheatsheet (cited from Standard §11)

This is a local cache for fast lookup. **The canonical definition is The Steelbore Standard §11.** If the two ever disagree, the Standard wins.

| Token          | Hex       | Role                           |
|----------------|-----------|--------------------------------|
| Void Navy      | `#000027` | **Page / slide / HTML background** |
| Molten Amber   | `#D98E32` | Body text, cells, active readout |
| Steel Blue     | `#4B7EB0` | Heading 1, accents, chart titles, visited links |
| Radium Green   | `#50FA7B` | Heading 2, success/safe status |
| Liquid Coolant | `#8BE9FD` | Heading 3, info, unvisited links |
| Red Oxide      | `#FF5C5C` | Warning / error status         |

## §5 — Typography cheatsheet (cited from Standard §12)

Both fonts are OFL-licensed, embed-permitted, and hosted on Google Fonts (so Google Workspace and Texinfo HTML render them natively even when embedded copies are stripped on import).

| Context  | Font            | License |
|----------|-----------------|---------|
| Headings | Share Tech Mono | OFL     |
| Body / code / cells / bullets | Inconsolata | OFL |
| Fallback | system monospace | n/a    |

## §6 — Load-order index

This skill is the **router**: it decides the format, then delegates the how-to to the execution-layer
skills — **`spacecraft-texinfo`** for prose authoring/building, **`spacecraft-markdown-document`** for
GFM rendering and companions. The local `references/` carry only the document-format-specific glue
(format priority, pairing policy, office recipes). Pull only what the task needs. **Do not eagerly
load all of them** — that defeats the token-economy split.

| Situation                                                | Load                                                                  |
|----------------------------------------------------------|-----------------------------------------------------------------------|
| Any prose document (default → Texinfo)                   | **`spacecraft-texinfo` skill** (+ `references/texinfo-authoring.md` for routing glue) |
| Prose deliverable as a word-processor file (`.odt`)      | `references/odf-authoring.md` + `references/markdown-companion.md` (+ `spacecraft-markdown-document` skill for the `.md`) |
| Prose deliverable as MS Word (`.docx`, on request)       | `references/ms-office-authoring.md` + `references/markdown-companion.md` (+ `spacecraft-markdown-document` skill) |
| Spreadsheet (`.ods`, or `.xlsx` on request)              | `references/odf-authoring.md` *or* `references/ms-office-authoring.md` + `references/markdown-companion.md` (+ `spacecraft-markdown-document` skill) |
| Presentation (`.odp`, or `.pptx` on request)             | same as spreadsheet                                                   |
| GFM Markdown rendering of a `.texi`                       | **`spacecraft-texinfo` skill** (`references/converting.md`)           |
| PDF from an ODF/MS-Office source                         | **add** `references/pdf-export.md`                                    |
| Quick palette / font / geometry lookup                   | SKILL.md only — no references needed                                  |

## §7 — Style mapping (consistent across all formats)

Within each format's native style system, map the Spacecraft Software styles as follows. Per-format implementation details live in the references.

| Style    | Font            | Size | Weight | Colour     | Used for                            |
|----------|-----------------|------|--------|------------|-------------------------------------|
| Normal   | Inconsolata     | 11 pt | regular | `#D98E32` | Body, cells, bullets                |
| H1       | Share Tech Mono | 16 pt | bold    | `#4B7EB0` | Title, sheet name, slide title, `@chapter` |
| H2       | Share Tech Mono | 14 pt | bold    | `#50FA7B` | Section heading, table header row, `@section` |
| H3       | Share Tech Mono | format default | italic | `#8BE9FD` | Subsection, `@subsection`  |
| Link (unvisited) | inherit | inherit | inherit | `#8BE9FD` | Hyperlinks before click        |
| Link (visited)   | inherit | inherit | inherit | `#4B7EB0` | Hyperlinks after click         |

Heading levels lock-step across formats: a Texinfo `@chapter` / a `# H1` in a markdown companion / Heading 1 in an ODF source all correspond. Don't break that mapping.

## §8 — Acceptance checklist (every deliverable)

Run through this before declaring a document done:

- [ ] **Void Navy background** present: Texinfo HTML (CSS applied), Texinfo PDF page colour where supported, or the ODF/MS-Office page/slide background in its reference viewer.
- [ ] **Share Tech Mono** on every heading / chapter / section / sheet name / slide title.
- [ ] **Inconsolata** on body / cells / bullets.
- [ ] **Page geometry** correct (A4 portrait for text/PDF; 16:9 slides).
- [ ] Texinfo: `.texi` includes the §8.3 structural elements (`@dircategory`/`@direntry`, `@copying`, `@titlepage`, `@node Top`/`@top`, per-chapter `@menu`) and builds cleanly to `.info`, `.html`, and `.pdf`.
- [ ] Binary ODF/MS-Office deliverable: **GFM companion `.md`** exists with the same base name in the same directory (Texinfo exempt).
- [ ] Heading levels mirror across the source and any companion.
- [ ] Licence declared: `CC-BY-SA-4.0` (or `GFDL-1.3-or-later` for a Texinfo manual); SPDX header inline on `.texi`/`.md`, `.license`/`REUSE.toml` coverage for binaries; licence text in `LICENSES/`; `reuse lint` clean.
- [ ] No tracked changes, comments, revisions, or embedded macros left behind (office formats).
- [ ] If PDF was requested: `pdffonts <file.pdf>` shows both font families embedded.

## §9 — Generator-agnostic notes

This skill describes **output properties**, not specific tools. References cover toolchain recommendations:

- Texinfo — `texinfo-authoring.md` §C (`makeinfo`/`texi2any` for Info & HTML, `texi2pdf` for PDF, `make info`/`make html`/`make pdf` targets).
- ODF — `odf-authoring.md` §E (LibreOffice headless, odfpy, etc.).
- MS Office — `ms-office-authoring.md` §E (python-docx, openpyxl, python-pptx, or LibreOffice headless conversion).
- PDF from ODF/MS-Office — `pdf-export.md` §B (LibreOffice headless `--convert-to pdf:<filter>`).

Pick whatever toolchain reliably produces output that passes §8's acceptance checklist. If you choose a tool not listed in the references, document the choice in the deliverable's prose or the markdown companion's frontmatter.

## §10 — Cross-references

This skill is the **hub** of a hub-and-spoke trio: it picks the format and delegates the how-to to two
execution-layer spokes. Keep the policy here; keep the mechanics in the spokes.

- **`spacecraft-texinfo`** — the prose execution layer. Load it for authoring, building, linting,
  converting, packaging, and the HTML/PDF brand output of any `.texi`. It auto-fires on `.texi`; this
  router hands prose deliverables to it.
- **`spacecraft-markdown-document`** — the GFM execution layer. Load it for the mandatory `.md`
  companions (§2) and any house-style GFM authoring/audit. This router delegates companion generation
  to it; it also responds to its own `/spacecraft-markdown-document` slash command.
- The Steelbore Standard §8 (Documentation / Texinfo), §11 (Colour Palette), §12 (Typography), §4 (Licensing: §4.1.1 artifact classes, §4.3 SPDX/REUSE), §15 (Attribution surfaces).
- `spacecraft-standard` — the full Standard, including §8 Texinfo requirements for user-facing programs.
- `gnu-coding-standards` — deeper GNU Texinfo conventions (for GNU-targeted work; Spacecraft projects follow Standard §8 and this skill).
- `spacecraft-theme-factory` — for IDE and terminal themes (separate concern; shares the §11 palette).
- `spacecraft-brand-guidelines` — brand colors and typography quick-reference; shares the §11 palette.

*— Built by Spacecraft Software —*
