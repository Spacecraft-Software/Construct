---
name: steelbore-color-palette
description: >
  Single source of truth for the Steelbore color palette (Steelbore 2,
  Standard §11 v1.34) — canonical hex tokens, role semantics, the measured
  WCAG contrast matrix, the §11.1 theme contract, and the §11.1.1
  accessibility variants. ALWAYS consult whenever a color, hex value, palette
  token, theme, contrast ratio, or brand hue is needed for ANY Spacecraft
  Software or Steelbore OS artifact — UI code, TUI styling, editor/terminal
  themes, documents, diagrams, SVGs, charts, or CSS — even if the user never
  says "palette". Triggers: "Void Navy", "Plasma Orange", "Acid Lime",
  "steelbore theme", "brand colors", "high contrast variant", any request to
  color or style a Spacecraft deliverable, and any WCAG / EN 301 549 question
  about Spacecraft colors. Consumer skills (brand-guidelines, theme-factory,
  accessibility-support, document-format) defer here for color values — never
  restate hexes from memory; read them here.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Steelbore Color Palette

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

> **Authority chain:** The Steelbore Standard §11 (v1.34) is the normative
> text; this skill is its canonical machine-readable mirror and the **only**
> place palette hexes should be read from. If this skill and the Standard ever
> disagree, the Standard governs — and this skill must be fixed. Consumer
> skills (`spacecraft-brand-guidelines`, `spacecraft-theme-factory`,
> `spacecraft-accessibility-support`, `spacecraft-document-format`) define
> *application* rules and defer here for *values*.

## Canonical tokens — Steelbore 2

The **only** permitted colors for Spacecraft Software interfaces and documents:

| Token          | Hex       | RGB                | Class      | Role                            |
|----------------|-----------|--------------------|------------|---------------------------------|
| Void Navy      | `#000027` | RGB(0, 0, 39)      | Canvas     | **Background — all surfaces**   |
| Quantum Blue   | `#0E2A47` | RGB(14, 42, 71)    | Surface    | Elevated panels / cards         |
| Deep Matrix    | `#0B1A12` | RGB(11, 26, 18)    | Surface    | Code blocks / terminal wells    |
| Platinum Mist  | `#D9DEE5` | RGB(217, 222, 229) | Foreground | Body text / default readout     |
| Plasma Orange  | `#FF5E00` | RGB(255, 94, 0)    | Foreground | Primary accent / active readout |
| Pulse Violet   | `#8A6CFF` | RGB(138, 108, 255) | Foreground | Structure / links / borders     |
| Acid Lime      | `#B4FF00` | RGB(180, 255, 0)   | Foreground | Success / safe status / focus   |
| Mars Red       | `#FF3B3B` | RGB(255, 59, 59)   | Foreground | Error status                    |
| Plasma Magenta | `#E445FF` | RGB(228, 69, 255)  | Foreground | Warning / attention             |

**`#000027` (Void Navy) is the mandatory background for ALL Spacecraft
Software surfaces.** No alternative background is permitted. Non-negotiable.

### Class rules

- **Canvas** — Void Navy underlies everything. Every variant keeps it.
- **Surface** — fills placed *on* Void Navy, never replacements for it, and
  **never text colors** (Quantum Blue 1.40:1, Deep Matrix 1.14:1 — illegible
  as foregrounds). A surface's edge against the canvas is below the 3:1
  non-text floor: where the boundary is meaningful, draw it — the canonical
  edge is a Pulse Violet border. Surfaces never nest without a measured
  boundary.
- **Foreground** — the six text/accent tokens, verified per the matrix below.

## Contrast matrix (WCAG 2.2 AA · EN 301 549 clause 11)

Measured with the WCAG relative-luminance formula. Floors: **4.5:1** normal
text (1.4.3); **3:1** large text and non-text UI (1.4.3, 1.4.11). EN 301 549
V4.1.1 clause 11 inherits the same criteria for non-web software.

| Foreground     | vs Void Navy | vs Quantum Blue | vs Deep Matrix |
|----------------|--------------|-----------------|----------------|
| Platinum Mist  | 15.09:1      | 10.78:1         | 13.27:1        |
| Plasma Orange  | 6.66:1       | 4.76:1          | 5.85:1         |
| Pulse Violet   | 5.51:1       | **3.93:1** †    | 4.84:1         |
| Acid Lime      | 16.75:1      | 11.97:1         | 14.73:1        |
| Mars Red       | 5.77:1       | **4.12:1** †    | 5.07:1         |
| Plasma Magenta | 6.41:1       | 4.58:1          | 5.63:1         |

† **Restricted pairings.** On Quantum Blue, Pulse Violet and Mars Red are
limited to large text (≥18.66 px bold / ≥24 px regular), icons, and non-text
UI. Normal-size error prose on a surface is Platinum Mist carrying the
`[ERROR]` tag (§18.2), with Mars Red as border or icon accent only.

**Scope of the guarantee.** The eighteen pairings above are the *only*
verified pairs. Foreground-on-foreground mostly fails the 3:1 floor (Acid
Lime on Platinum Mist 1.11:1; Plasma Orange on Mars Red 1.15:1). Never place
palette-colored text on a palette-colored fill without measuring that specific
pair (≥4.5:1 text, ≥3:1 non-text). Color is never the sole carrier of meaning
— every colored status also carries `[OK]` `[WARN]` `[ERROR]` `[INFO]` or a
symbol. The visible **focus indicator** is Acid Lime (WCAG 2.2 §2.4.11,
passes on every background).

## Theme contract (§11.1)

Applications reference role tokens through a named `Steelbore` theme
(`steelbore` in snake_case) — bare hex literals in UI logic are forbidden for
new apps:

| Theme token   | Palette token  | Hex       |
|---------------|----------------|-----------|
| `background`  | Void Navy      | `#000027` |
| `surface`     | Quantum Blue   | `#0E2A47` |
| `surface-alt` | Deep Matrix    | `#0B1A12` |
| `foreground`  | Platinum Mist  | `#D9DEE5` |
| `accent`      | Plasma Orange  | `#FF5E00` |
| `structure`   | Pulse Violet   | `#8A6CFF` |
| `success`     | Acid Lime      | `#B4FF00` |
| `error`       | Mars Red       | `#FF3B3B` |
| `warning`     | Plasma Magenta | `#E445FF` |
| `focus`       | Acid Lime      | `#B4FF00` |
| `border`      | Pulse Violet   | `#8A6CFF` |

## Accessibility variants (§11.1.1)

`steelbore` is the **sole default**; variants are additive siblings selected
only by explicit user action or the §18.1 toggle. Void Navy remains the
background in every variant.

**`steelbore-high-contrast`** — every token ≥7:1 on Void Navy. Lifts only the
four tokens that need it; the lifted hexes are accessibility-derived, not
brand colors, and may not be used outside the variant:

| Theme token | Base hex  | Variant hex   | vs Void Navy |
|-------------|-----------|---------------|--------------|
| `accent`    | `#FF5E00` | **`#FF8A3D`** | 8.70:1       |
| `structure` | `#8A6CFF` | **`#B3A1FF`** | 9.19:1       |
| `error`     | `#FF3B3B` | **`#FF7A7A`** | 8.08:1       |
| `warning`   | `#E445FF` | **`#EE7BFF`** | 8.66:1       |

All other tokens carry over verbatim. In this variant every lifted token also
clears 4.5:1 on both surfaces (weakest: `#FF7A7A` on Quantum Blue, 5.77:1), so
the † restrictions do not apply — the variant is strictly safer.

**`steelbore-mono`** — 4-bit ANSI only, selected explicitly or via `NO_COLOR`.
Maps roles to conventional ANSI slots (success→green, error→red,
warning→yellow, structure→blue, focus→reverse-video), deferring hue entirely
to the user's terminal palette.

## Retired tokens (§11.2)

The v1.33 palette is retired and must not appear in new artifacts: Molten
Amber `#D98E32`, Steel Blue `#4B7EB0`, Radium Green `#50FA7B`, Red Oxide
`#FF5C5C`, Liquid Coolant `#8BE9FD`, and the old lifts `#7FAEDC` / `#FF8080`.
Existing artifacts are grandfathered until their next minor release.

## Typography companion (§12)

Colors ship with type: **Share Tech Mono** (headings) and **Inconsolata**
(body/code), both OFL; system `monospace` fallback. Never proprietary fonts.

## Shipped machine-readable artifacts

| File | What it is |
|------|------------|
| [`assets/steelbore.toml`](assets/steelbore.toml) | The canonical `Steelbore` theme contract — `[palette]`, `[themes.steelbore]` + measured contrast tables, `[themes.steelbore-high-contrast]`, `[themes.steelbore-mono]`, `[typography]`. Copy or parse it; never retype hexes. |
| [`assets/spacecraft.css`](assets/spacecraft.css) | The canonical Spacecraft HTML theme for `texi2any --css-include`. The copies in `standard/` and `spacecraft-texinfo-document/assets/` are synced derivatives — edit this one first, keep all three byte-identical. |

## Where application rules live

| Applying colors to… | Load |
|---------------------|------|
| Documents (DOCX/ODT/PDF heading map, page setup) | `spacecraft-document-format` + `spacecraft-brand-guidelines` |
| IDE / terminal / editor themes | `spacecraft-theme-factory` |
| Accessible mode, toggles, screen readers | `spacecraft-accessibility-support` |
| Full Standard compliance audit | `spacecraft-standard-constitution` |

*— Built by Spacecraft Software —*
