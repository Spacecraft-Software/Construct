---
name: spacecraft-accessibility-support
description: >
  Implements and audits Standard §18 accessibility for Spacecraft Software
  applications — CLI, TUI, and GUI. ALWAYS consult when adding, reviewing, or
  auditing accessibility; when wiring the `--accessible` / `SPACECRAFT_A11Y`
  toggle; when a TUI needs a linear (non-redraw) fallback mode; when selecting
  a screen-reader bridge (AccessKit, GtkAccessible, Flutter Semantics,
  QAccessible); when emitting or checking the `steelbore-high-contrast` /
  `steelbore-mono` theme variants; or when verifying WCAG 2.2 AA / EN 301 549
  clause 11 conformance. Triggers include "accessibility", "a11y", "screen
  reader", "NVDA", "Orca", "VoiceOver", "AccessKit", "AT-SPI", "high contrast",
  "reduced motion", "keyboard navigation", "NO_COLOR", "linear mode", "WCAG",
  "EN 301 549", and "EAA". Every Spacecraft Software application other than a
  registered game must ship a working accessible mode (§18). Also consult when
  working on a game (§18.5 exempts games from §18 and §10 entirely;
  accessibility there is optional).
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Software Accessibility

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

> **Source of truth:** The Steelbore Standard **§18** (Accessibility), with §10
> (key bindings), §11/§11.1.1 (palette family and theme variants — v1.35;
> Steelbore Modern is the default), and §13 (design system). This skill is the
> *how*; §18 is the *what*. Where they appear to diverge, §18 governs — fix
> this skill.

## Stop here if this is a game

**Projects registered as games under §18.5 are exempt from §18 in full and
from §10 in full.** Accessibility in a game is **optional, nothing is
enforced**, and its absence is never a compliance failure.

If you are working on a registered game — today that is **Ironway** — do not
apply the rest of this skill as requirements. Do not add compliance gates, do
not flag missing accessibility as a defect, and do not ask for a remediation
entry. Jump to [Games](#games) for the recommendations a game may freely
decline, then stop.

Everything below this section applies to **non-game** projects.

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

**3. Text on a palette-colored fill.** §11 verifies foreground tokens against
Void Navy and the two surface tokens (§11.0.2 matrix), *not* against each
other. Acid Lime on Platinum Mist is 1.11:1; Plasma Orange on Mars Red is
1.15:1 — indistinguishable. Never place palette-colored text on a
palette-colored fill without measuring that specific pair (≥4.5:1 text, ≥3:1
non-text boundaries). Two measured restrictions already exist: on Quantum Blue
surfaces, Mars Red (4.12:1) and Pulse Violet (3.93:1) are large-text/UI only —
normal-size error prose on a surface is Platinum Mist carrying the `[ERROR]`
tag, with Mars Red as border or icon accent.

## Theme variants (§11.1.1)

`steelbore` (Steelbore Modern) is and remains the **sole default**. The
variants are additive siblings, never replacements.

**If the project has declared an alternate palette** under §11.4
(`steelbore-classic`, `steelbore-blue`, `steelbore-blackpinkpanther`,
`steelbore-matrixgreen`, `steelbore-navywhite`), accessible mode selects
*that palette's* `<slug>-high-contrast` sibling — take its lifts from
`steelbore-color-palette`'s `assets/steelbore.toml`, not from the Modern
table below. `steelbore-mono` is palette-independent and serves all of them.
Note `steelbore-navywhite` has a **light canvas**, so its high-contrast
variant darkens foregrounds instead of lightening them; the principle holds —
contrast comes from moving the foreground, never from abandoning the canvas.

| Variant | Selected by | Behavior |
|---------|-------------|----------|
| `steelbore` | **Default** | Canonical §11 palette, unchanged |
| `steelbore-high-contrast` | Accessible mode, or explicit | Every foreground token ≥7:1 (AAA) on Void Navy |
| `steelbore-mono` | Explicit, or `NO_COLOR` | 4-bit ANSI only — defers to the user's terminal palette |

**How this composes with §11.6 (added in v1.45).** §11.6.3 resolves a theme in
two stages, and accessible mode lives entirely in the second. Stage 1 picks the
**base palette**; stage 2 picks a **variant of that palette** — so an
accessibility signal chooses a *sibling* and can never change which palette is in
force. Overlay precedence, highest first: a variant slug the user pinned
explicitly, then `NO_COLOR` ⇒ `steelbore-mono`, then §18.1 accessible mode ⇒
`<base>-high-contrast`, then the **platform's** high-contrast preference ⇒
`<base>-high-contrast` (read from the OS independently of the §18.1 toggle, per
§18.3 — the user already expressed it system-wide).

Three consequences worth stating:

- **`NO_COLOR` outranks accessible mode**, because mono is the only variant that
  surrenders color outright and a user who asked for no color may not be handed a
  colored sheet. `NO_COLOR` is a *color* instruction: it selects mono whether or
  not it also enabled accessible mode as a source-4 hint, and
  `SPACECRAFT_A11Y=0` does **not** undo it.
- **A §11.6.4 declaration file's `high-contrast` key is not an accessible-mode
  switch.** It enters at overlay signal 4 and selects a theme sibling only. §18.1
  remains the sole switch for §18's behavioral requirements — announcements,
  linear mode, motion suppression — none of which are a color setting.
- §18.1's four-source precedence is unchanged by §11.6; §11.6 consumes its
  resolved boolean and adds no accessibility rule of its own.

High contrast lifts **only the four tokens that need it**:

| Token | Base | Variant | Contrast |
|-------|------|---------|----------|
| `background` | Void Navy `#000027` | `#000027` | (canvas) |
| `foreground` | Platinum Mist | `#D9DEE5` | 15.09:1 |
| `accent` | Plasma Orange | **`#FF8A3D`** | 8.70:1 |
| `structure` | Pulse Violet | **`#B3A1FF`** | 9.19:1 |
| `success` | Acid Lime | `#B4FF00` | 16.75:1 |
| `error` | Mars Red | **`#FF7A7A`** | 8.08:1 |
| `warning` | Plasma Magenta | **`#EE7BFF`** | 8.66:1 |

Alias tokens follow their bases: `focus` stays Acid Lime `#B4FF00`; `border`
follows `structure` to `#B3A1FF`. In the variant, all four lifted tokens also
clear 4.5:1 on Quantum Blue and Deep Matrix (weakest pairing: `#FF7A7A` on
Quantum Blue, 5.77:1), so the §11.0.2 large-text restrictions do not apply
under high contrast — the variant is strictly safer than the default.

**Void Navy remains the background in every variant** — high contrast comes
from lifting foregrounds, never from abandoning the canvas. The shifted
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
  linear, focused element visibly indicated. The visible focus indicator is
  Acid Lime `#B4FF00` (16.75:1 on Void Navy) — WCAG 2.2 §2.4.11 compliant on
  every background.

## Games

Standard §18.5. Games are exempt from §18 and §10 in full — **nothing here is
required**, and a game that ships no accessibility features at all is fully
compliant.

**Why the carve-out exists.** §18 assumes a character grid, or a widget tree
with roles and names. Games are neither: they are real-time simulations
rendering custom, non-widget interfaces where play itself is the purpose. The
accessibility techniques that suit games are a genuinely different discipline
from the one §18 codifies, so applying §18 to a game would enforce the wrong
requirements at disproportionate cost.

**Registry** (§18.5.1) — a project is a game when it declares so in its
`README.md` *and* appears here:

| Project | Class |
|---------|-------|
| Ironway | **Game** — exempt from §18 and §10 |
| (all other projects) | Standard — §18 and §10 apply in full |

The carve-out is narrow. It covers registered projects, not any project that
merely has a playful or game-like interface. A TUI with animated ASCII art is
not a game.

### Recommended for games (never required)

Offer these; never require them. A game may decline any or all without
justification:

- **Remappable controls** — already standard practice in games, independent of
  accessibility.
- **Leave screen-reader chords alone** — `Insert`, `CapsLock`, `KP_Insert`,
  `Ctrl`+`Option` belong to NVDA, Orca, and VoiceOver. Capturing them collides
  with a screen reader the player may be running.
- **Colorblind-safe signalling** — pair hue with shape, icon, or text.
- **Subtitles and captions** for spoken or plot-critical audio.
- **Honor the system reduced-motion preference** where the engine exposes it.

### If a game does ship a toggle

Use the §18.1 names (`--accessible`, `SPACECRAFT_A11Y`) and the §11.1.1
variant names rather than inventing new ones. This constrains only the
*naming* of something the game already chose to build — it requires no feature
to exist.

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
| Full Standard compliance | `spacecraft-steelbore-standard` |
| Documenting the accessible path in a manual | `spacecraft-texinfo-document` |

*— Built by Spacecraft Software —*
