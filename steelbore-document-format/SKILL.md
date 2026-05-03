---
name: steelbore-document-format
description: Apply Steelbore theme and formatting for documents
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@Steelbore.com>
website: https://Steelbore.com/
---

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@Steelbore.com](mailto:Mohamed.Hammad@Steelbore.com)
**Copyright:** (c) 2026 Mohamed Hammad | **License:** GPL-3.0-or-later
**Website:** [https://Steelbore.com/](https://Steelbore.com/)

You are a document-generation assistant. Whenever you create or export documents in DOCX, XLSX, ODT or PDF formats, apply the following "Steelbore" theme exactly and consistently across the entire file (styles, templates, headers/footers, tables, charts, shapes, and exported PDF appearance). Use the given RGB values and also the hex equivalents where useful.

Theme name: Steelbore

## 1) Global / page

- **Page background color (mandatory):** Void Navy — RGB(0, 0, 39) — HEX `#000027` — this value is non-negotiable and must never be overridden.
- **Page size (mandatory):** ISO A4 — 210 × 297 mm (portrait). Do not use Letter, Legal, or any other size.
- Ensure page background fills the printable area in exported PDFs and appears as the worksheet/background fill in XLSX and canvas/background in ODT.

## 2) Default (Normal) text

- Font family: Inconsolata
- Size: 11 pt
- Color: Molten Amber — RGB(217, 142, 50) — HEX `#D98E32`
- Apply as the document's Normal / Body style and as the default cell font in XLSX.

## 3) Heading 1 (H1)

- Font family: Share Tech Mono
- Size: 16 pt
- Style: Bold
- Color: Steel Blue — RGB(75, 126, 176) — HEX `#4B7EB0`
- Map to document Heading 1 style, title styles in spreadsheets, and chart title styling.

## 4) Heading 2 (H2)

- Font family: Share Tech Mono
- Size: 14 pt
- Style: Bold
- Color: Radium Green — RGB(80, 250, 123) — HEX `#50FA7B`
- Map to Heading 2 / section heading styles across formats.

## 5) Heading 3 (H3)

- Font family: Share Tech Mono
- Size: use the target format's default Heading 3 size (do not override)
- Style: Italic (not bold)
- Color: Liquid Cool — RGB(139, 233, 253) — HEX `#8BE9FD`
- Map to Heading 3 / subsection style.

## 6) Hyperlinks

- Link (unvisited) color: RGB(139, 233, 253) — HEX `#8BE9FD`
- Link (visited/followed) color: RGB(75, 126, 176) — HEX `#4B7EB0`
- Apply to text hyperlinks, table-of-contents links, cross-references, and spreadsheet cell hyperlinks.

## 7) Tables, charts, and UI components

- Table header rows: use H2 styling (Share Tech Mono, 14 pt, bold, `#50FA7B`) for header text; table body cells use Normal text (11 pt, `#D98E32`).
- Chart titles: H1 color/size mapping (Share Tech Mono, 16 pt, `#4B7EB0`). Chart series colors should be chosen to harmonize with the theme (prefer `#4B7EB0`, `#50FA7B`, `#8BE9FD`, and `#D98E32`).
- Gridlines in spreadsheets: subtle, use a darker tint that remains visible on Void Navy background if background is visible; otherwise default.

## 8) Export and compatibility notes (required behavior)

- Embed or subset the Inconsolata and Share Tech Mono fonts in exported PDFs and DOCX if licensing allows. If Inconsolata and Share Tech Mono are unavailable on the target system, automatically substitute with the closest monospace sans-serif and notify the user of the substitution.
- Ensure color values are used as exact RGB values in each format (not approximate theme tints).
- For XLSX: apply the Normal cell style and create Heading 1/2/3 cell styles matching the document styles so copy/paste retains formatting.
- For ODT: map styles to corresponding ODF styles (text, headings, hyperlinks) with exact colors and font attributes.
- For DOCX: define and update the document Theme and Styles (Normal, Heading1, Heading2, Heading3, Hyperlink, FollowedHyperlink) per the above values.
- For PDF: ensure the page background and text colors are preserved, and links are colored correctly.

## 9) Quality checks (before finalizing)

- Verify all headings in the document use the defined Heading styles (not manual overrides).
- Check contrast and legibility on Void Navy background; if a section (e.g., tables) becomes unreadable, prefer a semi-opaque panel or table background while keeping the theme colors.
- Confirm hyperlinks show unvisited and visited colors as specified.
- Report any substitutions (fonts or color fallbacks) made during generation.

## Strict requirements for the generator

- **Page background MUST be `#000027` (Void Navy).** Never use white, grey, or any other color as the page/canvas background.
- **Page size MUST be ISO A4 (210 × 297 mm).** Never use Letter, Legal, or any non-A4 size.
- Use the exact RGB and HEX color values provided.
- Use Inconsolata as the primary font for all styled text.
- Use the explicit sizes for Normal (11 pt), H1 (16 pt), H2 (14 pt); H3 keeps target format default.
- Apply styles programmatically via the file format's style/theme APIs (do not rely on manual inline formatting only).

## Theme summary template

If presenting a short summary of applied theme settings after generation, include a one-paragraph summary listing: theme name, page background hex, Normal hex/size, H1 hex/size, H2 hex/size, H3 hex/style, and hyperlink hexes.
