---
name: steelbore-color-palette
description: >
  Single source of truth for the Steelbore palette family (§11, last amended v1.45)
  — nine palettes, their hex tokens, WCAG contrast matrices, the §11.1
  role-token contract, every §11.1.1 accessibility variant, and the §11.6
  system-theme contract. Modern is the
  default; Classic, Blue, BlackPinkPanther, MatrixGreen, NavyWhite, and Tokyo
  Night are opt-in (§11.4); Solarized Dark and Light are §11.5 fidelity
  palettes — verbatim, non-conforming, never adoptable. ALWAYS consult whenever
  a color, hex, palette token, theme, contrast ratio, or brand hue is needed for
  ANY Spacecraft Software or Steelbore OS artifact — UI code, TUI styling,
  editor/terminal themes, documents, diagrams, SVGs, or CSS — even if the user
  never says "palette". Triggers: "Void Navy", "Plasma Orange", "Tokyo Night",
  "Solarized", "brand colors", "high contrast", "system theme", "light mode",
  "SPACECRAFT_THEME", and any WCAG / EN 301 549 question. Consumer skills
  defer here for values — never restate hexes.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Steelbore Color Palette Family

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

> **Authority chain:** The Steelbore Standard **§11** — last amended in v1.45 —
> is the normative text; this skill is its canonical machine-readable mirror and
> the **only** place palette hexes should be read from. The version cited is the
> one in which §11 last *changed*, not the current document version: a release
> that does not touch §11 leaves this skill current. If this skill and the Standard ever
> disagree, the Standard governs — and this skill must be fixed. Consumer
> skills (`spacecraft-brand-guidelines`, `spacecraft-theme-factory`,
> `spacecraft-accessibility-support`, `spacecraft-document-format`) define
> *application* rules and defer here for *values*.

## The family (§11)

Nine palettes — seven conforming, two §11.5 fidelity. **Steelbore Modern is the default** — use it unless the project
has declared an alternate in its `README.md` (§11.4). A project uses **exactly
one** palette; tokens are never mixed across palettes, because every contrast
guarantee is computed per-palette.

| Theme slug | Palette | Canvas | Status |
|---|---|---|---|
| `steelbore` | Steelbore Modern | `#000027` Void Navy | **Default** |
| `steelbore-classic` | Steelbore Classic | `#000027` Void Navy | Legacy six-role contract (§11.2) |
| `steelbore-blue` | Steelbore Blue | `#0A1024` Orbit Navy | Alternate (§11.3.1) |
| `steelbore-blackpinkpanther` | Steelbore BlackPinkPanther | `#141418` Core Black | Alternate (§11.3.2) |
| `steelbore-matrixgreen` | Steelbore MatrixGreen | `#0C1A2B` Circuit Navy | Alternate (§11.3.3) |
| `steelbore-navywhite` | Steelbore NavyWhite | `#E7E5E0` Pearl Silver | Alternate (§11.3.4) — **light canvas** |
| `tokyonight` | Tokyo Night | `#1A1B26` Night | Alternate (§11.3.5) — upstream theme, verbatim, no restricted pairings |
| `solarized-dark` | Solarized Dark | `#002B36` base03 | **Fidelity (§11.5) — non-conforming**, not adoptable |
| `solarized-light` | Solarized Light | `#FDF6E3` base3 | **Fidelity (§11.5) — non-conforming**, not adoptable; body text 4.13:1 |

Each conforming palette has a `<slug>-high-contrast` sibling for §18.1
accessible mode. The §11.5 fidelity palettes have **none** — lifting their
tokens would change the values they exist to reproduce, so `steelbore-mono`
(palette-independent) is the accessible path for them.

**Reference names (§11.4.1)** are additive prose labels; the slug above stays
the machine identifier: `steelbore-color-palette`,
`steelboreclassic-color-palette`, `blue-color-palette`,
`blackpinkpanther-color-palette`, `matrixgreen-color-palette`,
`navywhite-color-palette`, `tokyonight-color-palette`,
`solarizeddark-color-palette`, `solarizedlight-color-palette`. Each is carried
in the TOML as its palette's `reference` key.

**All values for every palette are in
[`assets/steelbore.toml`](assets/steelbore.toml) — read them there.** The
tables below cover Modern (the default); §11.2–§11.3 of the Standard and the
TOML carry the others.

## Canonical tokens — Steelbore Modern

The permitted colors for Steelbore Modern, the default palette:

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

**`#000027` (Void Navy) is the mandatory canvas for every surface under
Steelbore Modern** — and Modern is the default, so it is the background of
every artifact that has not declared an alternate (§11.4). Within a palette
the canvas is non-negotiable; a declared alternate uses *its* canvas instead
(NavyWhite's is light).

### Class rules

- **Canvas** — the palette's canvas underlies everything, and every variant
  of that palette keeps it (Void Navy for Modern and Classic).
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

**`steelbore-high-contrast`** — every foreground token ≥7:1 on Void Navy.
Lifts only the four tokens that need it; the lifted hexes are
accessibility-derived, not brand colors, and may not be used outside the
variant:

| Theme token | Base hex  | Variant hex   | vs Void Navy |
|-------------|-----------|---------------|--------------|
| `accent`    | `#FF5E00` | **`#FF8A3D`** | 8.70:1       |
| `structure` | `#8A6CFF` | **`#B3A1FF`** | 9.19:1       |
| `error`     | `#FF3B3B` | **`#FF7A7A`** | 8.08:1       |
| `warning`   | `#E445FF` | **`#EE7BFF`** | 8.66:1       |

All other tokens carry over verbatim, and alias tokens follow their bases:
`focus` stays Acid Lime `#B4FF00`; `border` follows `structure` to `#B3A1FF`
(see `assets/steelbore.toml`). In this variant every lifted token also
clears 4.5:1 on both surfaces (weakest: `#FF7A7A` on Quantum Blue, 5.77:1), so
the † restrictions do not apply — the variant is strictly safer.

**`steelbore-mono`** — 4-bit ANSI only, selected explicitly or via `NO_COLOR`.
Maps roles to conventional ANSI slots (success→green, error→red,
warning→yellow, structure→blue, focus→reverse-video), deferring hue entirely
to the user's terminal palette.

## System theme resolution & declaration (§11.6)

Added in v1.45. §11 says *which colors*; §11.6 says **which member of the family
renders on this machine, right now**. Binds GUI, TUI and CLI alike — a CLI
already honors `NO_COLOR`, so it already has a theme.

**Register thirteen, author against one.** `[meta] registered-set` in
`steelbore.toml` is the authoritative list: the six conforming palettes, each
with its `-high-contrast` sibling, plus `steelbore-mono`. Classic is **not** in
it — it binds the legacy six-role contract and carries an `info` token that is
not one of §11.1's eleven roles. Registering is not defaulting: the default stays
the project's §11.4 palette.

**Two-stage resolution.** Stage 1 picks a base palette; stage 2 picks a variant
*of that palette*. An accessibility signal therefore chooses a sibling, never a
palette, and can never silently change the brand.

| Stage 1 — base palette | Stage 2 — variant overlay |
|---|---|
| 1. `--theme=<slug>` / `[theme] name` in config | 1. A pinned `-high-contrast`/`-mono` slug |
| 2. `SPACECRAFT_THEME=<slug>` (a slug, never a boolean) | 2. `NO_COLOR` ⇒ `steelbore-mono` |
| 3. the §11.6.4 declaration file's `active` key | 3. §18.1 accessible mode ⇒ `<base>-high-contrast` |
| 4. platform light/dark preference (GUI only) | 4. platform high contrast ⇒ `<base>-high-contrast` |
| 5. the project's §11.4 palette — `steelbore` if none | 5. the stage-1 palette itself |

An unusable slug is **skipped, never fatal** — resolution falls one source lower,
so a typo in `/etc` never leaves a machine without a working interface. Fidelity
slugs resolve at stage-1 source 1 only. Resolved theme, deciding source and
overlay are reported under `--verbose`.

**Light and dark.** `steelbore-navywhite` is the family's only light canvas, so a
light preference is answered by rendering a *different palette*. That is legal
because §11.4 forbids **combining** tokens, and a switch combines nothing: the
whole set is replaced at once and the canvas travels with the palette. The switch
must be **atomic and whole-surface** — an app that cannot do that resolves once
at startup and holds. Read polarity and counterparts from
`[resolution.polarity]` / `[resolution.pair]`; never hardcode the mapping.

**The declaration is a file**, so a CLI in a text console can read it —
`$XDG_CONFIG_HOME/steelbore/theme.toml` over `/etc/steelbore/theme.toml`, keys
merged from the highest-precedence file that supplies them. It carries **slugs,
never colors**: the OS declares *which* theme, values come from this skill.
Absence of both files is *no declaration*; `active = "steelbore"` is a
declaration *of Modern*. Paths and the variable name are in `[resolution]` —
`SPACECRAFT_THEME`, never `STEELBORE_THEME` (already a boolean shell flag).

## Steelbore Classic (§11.2)

The pre-v1.34 six-token palette, **preserved as a named family member** — it
was briefly retired at v1.34 and reinstated at v1.35. Shares Void Navy with
Modern, keeps its **legacy six-role contract** (`background`, `foreground`,
`accent`, `success`, `error`, `info`), and defines **no surface class**, so
§11.0.1 does not apply and every foreground is measured against Void Navy
alone.

| Role | Token | Hex | vs Void Navy |
|------|-------|-----|--------------|
| `background` | Void Navy | `#000027` | (canvas) |
| `foreground` | Molten Amber | `#D98E32` | 7.64:1 |
| `accent` | Steel Blue | `#4B7EB0` | 4.77:1 |
| `success` | Radium Green | `#50FA7B` | 14.87:1 |
| `error` | Red Oxide | `#FF5C5C` | 6.74:1 |
| `info` | Liquid Coolant | `#8BE9FD` | 14.74:1 |

`steelbore-classic-high-contrast` lifts `accent` → `#7FAEDC` (8.73:1) and
`error` → `#FF8080` (8.41:1). Classic's token-on-token failures are severe —
Molten Amber on Red Oxide 1.13:1, Radium Green on Liquid Coolant 1.01:1.

## Alternate palettes (§11.3)

Four alternates, each anchored on two fixed colors that never change. Full
role tables and three-background matrices are in
[`assets/steelbore.toml`](assets/steelbore.toml); anchors and headline
characteristics:

| Palette | Canvas anchor | Accent anchor | Notes |
|---------|---------------|---------------|-------|
| Blue | Orbit Navy `#0A1024` | Electric Blue `#0066FF` | Electric Blue is **†restricted** — 3.91:1, large text / icons / non-text UI only |
| BlackPinkPanther | Core Black `#141418` | Plasma Magenta `#E445FF` | No restricted pairings; accent shares a hex with Modern's `warning` |
| MatrixGreen | Circuit Navy `#0C1A2B` | Solar Lime `#B6FF3B` | `surface-alt` is *darker* than canvas (Ambient Black); Lime Shadow `#8AC22A` is the pressed state, not a role token |
| NavyWhite | Pearl Silver `#E7E5E0` | Lunar Navy `#111827` | **Light canvas.** Accent/status hues are deepened for AA; high contrast *darkens*. Source tints (`#3A6EA5`, `#4C8C6F`, `#B94A48`, `#D9A441`) are non-text fills only |

## Typography companion (§12)

Colors ship with type: **Share Tech Mono** (headings) and **Inconsolata**
(body/code), both OFL; system `monospace` fallback. Never proprietary fonts.

## Shipped machine-readable artifacts

| File | What it is |
|------|------------|
| [`assets/steelbore.toml`](assets/steelbore.toml) | **The canonical contract for the whole family** — `[palettes.*]` for all nine, `[themes.*]` (17 themes: seven conforming palettes with their `-high-contrast` siblings, two §11.5 fidelity palettes, and `steelbore-mono`) with measured per-background contrast tables and per-palette rules, plus `[resolution]` / `[resolution.polarity]` / `[resolution.pair]` for §11.6 and `[typography]`. `[meta] registered-set` is the §11.6.1 must-register list. Copy or parse it; never retype hexes. |
| [`assets/spacecraft.css`](assets/spacecraft.css) | The canonical Spacecraft HTML theme for `texi2any --css-include`. The copies in `standard/` and `spacecraft-texinfo-document/assets/` are synced derivatives — edit this one first, keep all three byte-identical. |

## Where application rules live

| Applying colors to… | Load |
|---------------------|------|
| Documents (DOCX/ODT/PDF heading map, page setup) | `spacecraft-document-format` + `spacecraft-brand-guidelines` |
| IDE / terminal / editor themes | `spacecraft-theme-factory` |
| Accessible mode, toggles, screen readers | `spacecraft-accessibility-support` |
| Full Standard compliance audit | `spacecraft-standard-constitution` |

*— Built by Spacecraft Software —*
