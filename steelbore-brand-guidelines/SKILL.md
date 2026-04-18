---
name: steelbore-brand-guidelines
description: Applies Steelbore's official brand colors and typography to any sort of artifact that may benefit from having Steelbore's look-and-feel. Use it when brand colors or style guidelines, visual formatting, or project design standards apply.
license: GPL-3.0-or-later
---

# Steelbore Brand Guidelines

> **Source of truth:** The Steelbore Standard v1.0 (§8 Colors, §9 Typography).
> All values here are canonical. Do not use any other color or font values for Steelbore artifacts.

## Color Palette (WCAG 2.1 AA Compliant)

All colors are verified for contrast against the Void Navy (`#000027`) background.

| Token          | Hex       | RGB                | Role                          |
|----------------|-----------|--------------------|-------------------------------|
| Void Navy      | `#000027` | RGB(0, 0, 39)      | **Background / Canvas**       |
| Molten Amber   | `#D98E32` | RGB(217, 142, 50)  | Primary Text / Active Readout |
| Steel Blue     | `#4B7EB0` | RGB(75, 126, 176)  | Primary Accent / Structural   |
| Radium Green   | `#50FA7B` | RGB(80, 250, 123)  | Success / Safe Status         |
| Red Oxide      | `#FF5C5C` | RGB(255, 92, 92)   | Warning / Error Status        |
| Liquid Coolant | `#8BE9FD` | RGB(139, 233, 253) | Info / Links                  |

**`#000027` (Void Navy) is the mandatory background for ALL Steelbore surfaces** —
documents, terminals, editor themes, application UIs. No alternative background is permitted.

## Typography

Only FOSS-licensed fonts are permitted. Acceptable licenses: OFL, Apache 2.0, Ubuntu Font License, CC0-1.0.

| Context        | Font              | License | Source       |
|----------------|-------------------|---------|--------------|
| Headings       | Share Tech Mono   | OFL     | Google Fonts |
| Body / Code    | Inconsolata       | OFL     | Google Fonts |
| Fallback       | monospace (system)| N/A     | System       |

Never use proprietary fonts. Outfit, Inter, Roboto, and similar non-OFL fonts are **not permitted**.

## Document Creation (DOCX / PDF)

For full document styling rules, load the `steelbore-document-format` skill.
Quick reference:

- **Page background:** `#000027` (Void Navy) — mandatory, non-negotiable
- **Page size:** ISO A4 (210 × 297 mm)
- **Body text:** Inconsolata, 11 pt, Molten Amber `#D98E32`
- **H1:** Share Tech Mono, 16 pt, bold, Steel Blue `#4B7EB0`
- **H2:** Share Tech Mono, 14 pt, bold, Radium Green `#50FA7B`
- **H3:** Share Tech Mono, default size, italic, Liquid Coolant `#8BE9FD`
- **Links:** Liquid Coolant `#8BE9FD` (unvisited), Steel Blue `#4B7EB0` (visited)

## UI / Visual Design

- Apply the palette to Material Design components (the required UI system for Steelbore GUIs).
- All new color pairings must pass WCAG 2.1 Level AA contrast verification before adoption.
- For IDE and terminal themes, load the `steelbore-theme-factory` skill.