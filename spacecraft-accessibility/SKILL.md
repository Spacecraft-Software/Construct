---
name: spacecraft-accessibility
description: >
  Implements and audits Standard §18 accessibility for Spacecraft Software
  applications — CLI, TUI, and GUI. ALWAYS consult when adding, reviewing, or
  auditing accessibility support; when wiring the `--accessible` /
  `SPACECRAFT_A11Y` toggle; when a TUI needs a linear (non-redraw) fallback
  mode; when selecting a screen-reader bridge (AccessKit, GtkAccessible,
  Flutter Semantics, QAccessible); when emitting or checking the
  `steelbore-high-contrast` / `steelbore-mono` theme variants; or when
  verifying WCAG 2.2 AA / EN 301 549 clause 11 conformance. Triggers include
  "accessibility", "a11y", "screen reader", "NVDA", "Orca", "VoiceOver",
  "AccessKit", "AT-SPI", "high contrast", "color contrast", "reduced motion",
  "keyboard navigation", "no-animation", "NO_COLOR", "linear mode", "WCAG",
  "EN 301 549", and "EAA". Every Spacecraft Software application must ship a
  working accessible mode (§18), so consult this the moment user-facing output
  — terminal or graphical — appears on the horizon.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Software Accessibility

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

> **Source of truth:** The Steelbore Standard **§18** (Accessibility), with §10
> (key bindings), §11/§11.1.1 (palette and theme variants), and §13 (design
> system). This skill is the *how*; §18 is the *what*. Where they appear to
> diverge, §18 governs — fix this skill.

## The rule that governs everything else

Accessibility support is **mandatory for the developer to implement** and
**optional for the user to activate**.

- **Every** Spacecraft Software application must ship a working accessible
  mode — new and existing projects alike, no phase-in.
- Accessible mode is **off by default**. The default experience is the
  `Steelbore` theme and standard rendering, entirely unchanged.

If a change you are making would alter what a user sees *without* them opting
in, you have left the accessibility layer and are editing the default
experience. Stop and reconsider.

## Which reference to load

| You are… | Load |
|----------|------|
| Wiring the toggle, or working on CLI/TUI output | [`references/cli-tui.md`](references/cli-tui.md) |
| Working on a GUI, or picking a screen-reader bridge | [`references/gui.md`](references/gui.md) |
| Auditing, verifying, or signing off a release | [`references/audit.md`](references/audit.md) |

## §18.1 — The activation contract

One toggle. Four sources. Resolve once at startup; never re-resolve mid-run.

| Precedence | Source | Form |
|---|---|---|
| 1 (highest) | Command-line flag | `--accessible` / `--no-accessible` |
| 2 | Environment | `SPACECRAFT_A11Y=1` / `SPACECRAFT_A11Y=0` |
| 3 | Configuration | `[accessibility] enabled = true` |
| 4 (lowest) | Auto-detect hints | `TERM=dumb`, `NO_COLOR`, `GTK_MODULES` containing `gail:atk` |

Non-negotiables:

- **Unset at every source ⇒ standard `Steelbore` rendering, unchanged.**
  Silence is never consent to change the default presentation.
- An explicit `0` / `--no-accessible` **always wins**, including over hints.
- Hints may only fire on unambiguous signals. Do not guess from `$LANG`,
  terminal size, or the absence of a TTY.
- Report the resolved state *and the source that decided it* under `--verbose`.
- It is **one switch** covering every §18.2 and §18.3 behavior. Per-feature
  accessibility flags (`--no-spinner`, `--no-emoji`, …) may exist as
  conveniences but never as a substitute for the single toggle.

## The three things that most often go wrong

**1. Color used as the only signal.** A red line reading `failed to connect`
is non-compliant in *every* mode, not just accessible mode. Ship the tag:

```
[ERROR] failed to connect
```

Applies to `[OK]`, `[ERROR]`, `[WARN]`, `[INFO]`. This is an always-on rule —
it is not gated behind the toggle.

**2. Assuming the TUI framework handles it.** A terminal has no accessibility
tree — no ARIA, no roles, no live regions. A screen reader reads the
emulator's character grid, so a redraw-based interface produces re-reads and
speech loops. No terminal UI library (ratatui included) provides
accessibility. **The application must supply the linear fallback itself.**

**3. Text on a palette-colored fill.** §11 verifies palette tokens against
Void Navy, *not* against each other. Molten Amber on Red Oxide is 1.13:1;
Radium Green on Liquid Coolant is 1.01:1 — indistinguishable. Never place
palette-colored text on a palette-colored fill without measuring that
specific pair (≥4.5:1 text, ≥3:1 non-text boundaries).

## Theme variants (§11.1.1)

`steelbore` is and remains the **sole default**. The variants are additive
siblings, never replacements:

| Variant | Selected by | Behavior |
|---------|-------------|----------|
| `steelbore` | **Default** | Canonical §11 palette, unchanged |
| `steelbore-high-contrast` | Accessible mode, or explicit | Every token ≥7:1 (AAA) on Void Navy |
| `steelbore-mono` | Explicit, or `NO_COLOR` | 4-bit ANSI only — defers to the user's terminal palette |

High contrast lifts **only the two tokens that need it**:

| Token | Base | Variant | Contrast |
|-------|------|---------|----------|
| `background` | Void Navy `#000027` | `#000027` | (canvas) |
| `foreground` | Molten Amber | `#D98E32` | 7.64:1 |
| `accent` | Steel Blue `#4B7EB0` | **`#7FAEDC`** | 8.73:1 |
| `success` | Radium Green | `#50FA7B` | 14.87:1 |
| `error` | Red Oxide `#FF5C5C` | **`#FF8080`** | 8.41:1 |
| `info` | Liquid Coolant | `#8BE9FD` | 14.74:1 |

**Void Navy remains the background in every variant** — high contrast comes
from lifting foregrounds, never from abandoning the canvas. The two shifted
hexes are accessibility-derived lifts of existing role tokens, not new brand
colors, and may not be used outside the variant.

`steelbore-mono` deliberately gives up exact brand color: mapping to the 16
ANSI colors hands control to the user's own terminal theme, which is the only
way to honor a contrast setup the application cannot see. This is the
mechanism GitHub adopted for `gh a11y`.

## Key bindings (§10)

- Every binding **must be user-remappable** through the project's config
  layer. A fixed keymap is non-compliant.
- These modifiers belong to screen readers and **must not be captured**:

  | Chord | Claimed by |
  |-------|------------|
  | `Insert` / `CapsLock` | NVDA (Windows) |
  | `Insert` / `KP_Insert` | Orca (GNOME/Linux) |
  | `Ctrl`+`Option` | VoiceOver (macOS) |

- Every pointer-reachable action must be keyboard-reachable; focus order
  linear, focused element visibly indicated.

## Normative targets

**WCAG 2.2 Level AA** where the success criteria apply, and **EN 301 549
clause 11 (non-web software)** for CLI and TUI — the only normative text that
addresses terminal software. The European Accessibility Act has been
enforceable since 2025-06-28, and EN 301 549 V4.1.1 folds in WCAG 2.2.

## Related skills

| Task | Skill |
|------|-------|
| CLI structure, `--json`, `NO_COLOR` precedence, TTY detection | `spacecraft-cli-standard` |
| Emitting theme files for editors/terminals | `spacecraft-theme-factory` |
| Full Standard compliance | `spacecraft-standard-constitution` |
| Documenting the accessible path in a manual | `spacecraft-texinfo-document` |

*— Built by Spacecraft Software —*
