# Accessibility Audit — verification reference

Standard §18.4. An accessibility claim is **not satisfied by inspection**.
Every item below is something you run, not something you read.

## Gate 0 — Is this a registered game?

- [ ] Check the §18.5 registry (today: **Ironway**).

**If yes: stop.** §18 and §10 do not apply. There is nothing to audit, no gate
to pass, and no remediation entry to file. Report the project as exempt, not
as failing. Do not run gates 1–9.

**If no:** continue.

## Gate 1 — Activation (§18.1)

- [ ] `--accessible` turns it on; `--no-accessible` turns it off.
- [ ] `SPACECRAFT_A11Y=1` / `=0` respected.
- [ ] `[accessibility] enabled` respected from config.
- [ ] **Precedence holds**: `--no-accessible` beats `SPACECRAFT_A11Y=1`, which
      beats config, which beats hints.
- [ ] With nothing set anywhere, output is byte-identical to the pre-§18
      default. *(This is the constraint that protects the Steelbore default —
      test it explicitly, do not assume it.)*
- [ ] `--verbose` reports the resolved state **and its source**.
- [ ] Hints fire only on `TERM=dumb`, `NO_COLOR`, `GTK_MODULES=…gail:atk` —
      not on `$LANG`, terminal size, or absence of a TTY.

## Gate 2 — Always-on output rules (§18.2.1)

Verify in **default** mode, not just accessible mode:

- [ ] Every status line carries `[OK]` / `[ERROR]` / `[WARN]` / `[INFO]`.
- [ ] Piping through `cat` loses no information (nothing meaningful lives in
      color alone):
      ```sh
      your-tool build 2>&1 | cat
      ```
- [ ] Diagnostics go to `stderr`, results to `stdout`:
      ```sh
      your-tool build 2>/dev/null   # results only
      your-tool build 1>/dev/null   # diagnostics only
      ```
- [ ] `NO_COLOR=1` suppresses color; `FORCE_COLOR=1` restores it.
- [ ] No text is drawn on a palette-colored fill without a measured ratio.

## Gate 3 — Accessible-mode output (§18.2.2)

- [ ] No spinner, marquee, or blink survives `--accessible`.
- [ ] Progress is static, monotonic, and rewritten at most once per second.
- [ ] No ASCII art, banners, or box-drawing decoration.
- [ ] Informational art is replaced by a text description, not dropped.
- [ ] Every table has a `field: value` per-line fallback.
- [ ] Prompts state question, choices, and default before awaiting input.

## Gate 4 — TUI linear mode (§18.2.3)

- [ ] Linear mode reachable through the same toggle.
- [ ] Does **not** enter the alternate screen buffer.
- [ ] Appends lines; never repaints a region.
- [ ] Every interactive operation has a non-interactive CLI equivalent with
      `--json` output.
- [ ] That path is documented in the Texinfo manual (§8).

## Gate 5 — Listen to it

The decisive test. Reading output is not verification:

```sh
your-tool --accessible build 2>&1 | espeak-ng -s 160
```

- [ ] The sequence of events is followable by ear alone.
- [ ] No glyph is pronounced as noise (`⠋`, `▰`, box-drawing characters).
- [ ] Errors are comprehensible as sentences, not as punctuation salad.

Provision `espeak-ng` ephemerally per `spacecraft-missing-pkg`.

## Gate 6 — GUI with a real screen reader (§18.3)

Emulators do not count. Use the real thing for the platform:

| Platform | Screen reader |
|----------|---------------|
| Linux | Orca |
| Windows | NVDA |
| macOS | VoiceOver |

- [ ] Every interactive element announces a **name** and a **role**.
- [ ] No element announces as bare "button" / "unlabelled".
- [ ] Decorative elements are skipped, not read.
- [ ] State changes are announced, not merely repainted.
- [ ] Custom-drawn surfaces publish nodes (AccessKit / `QAccessibleInterface`).
- [ ] System reduced-motion and high-contrast preferences take effect
      **without** the §18.1 toggle.

## Gate 7 — Keyboard (§10)

- [ ] Every primary task completable with no pointing device.
- [ ] Focus order linear; focused element visibly indicated.
- [ ] Bindings remappable through config.
- [ ] `Insert`, `CapsLock`, `KP_Insert`, and `Ctrl`+`Option` are **not**
      captured — verify with a screen reader actually running, since the
      conflict only manifests then.

## Gate 8 — Contrast (§11)

Measure the pairings **actually used**, not just foreground-on-background, and
record the ratios. Reference values (Steelbore 2, §11.0.2 matrix) — each
foreground against all three legal backgrounds, plus its high-contrast lift:

| Token | vs Void Navy | vs Quantum Blue | vs Deep Matrix | High-contrast variant |
|-------|--------------|-----------------|----------------|-----------------------|
| Platinum Mist  | 15.09:1 | 10.78:1        | 13.27:1 | `#D9DEE5` — 15.09:1 (verbatim) |
| Plasma Orange  | 6.66:1  | 4.76:1         | 5.85:1  | `#FF8A3D` — 8.70:1  |
| Pulse Violet   | 5.51:1  | **3.93:1** †   | 4.84:1  | `#B3A1FF` — 9.19:1  |
| Acid Lime      | 16.75:1 | 11.97:1        | 14.73:1 | `#B4FF00` — 16.75:1 (verbatim) |
| Mars Red       | 5.77:1  | **4.12:1** †   | 5.07:1  | `#FF7A7A` — 8.08:1  |
| Plasma Magenta | 6.41:1  | 4.58:1         | 5.63:1  | `#EE7BFF` — 8.66:1  |

† On Quantum Blue fills, Pulse Violet and Mars Red are restricted to large
text, icons, and non-text UI (≥3:1); they miss the 4.5:1 normal-text floor.
In the high-contrast variant all four lifted tokens clear 4.5:1 on both
surfaces, so the restriction vanishes there.

Surface tokens (Quantum Blue 1.40:1, Deep Matrix 1.14:1 vs Void Navy) are
**never text colors**, and a meaningful surface boundary must be drawn —
Pulse Violet (5.51:1 vs canvas) is the canonical edge (§11.0.1).

- [ ] Every text pairing used measures ≥4.5:1 (AA), or ≥7:1 in the
      high-contrast variant.
- [ ] Every non-text UI boundary measures ≥3:1.
- [ ] No surface token is used as a text color, and no normal-size Mars Red
      or Pulse Violet text sits on a Quantum Blue fill.
- [ ] The recorded claim states **which pairing** was measured (§13).

Contrast is deterministic — compute it rather than eyeballing it:

```python
def _lin(c: float) -> float:
    c /= 255
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

def luminance(hex_color: str) -> float:
    h = hex_color.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return 0.2126 * _lin(r) + 0.7152 * _lin(g) + 0.0722 * _lin(b)

def contrast(a: str, b: str) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)
```

## Gate 9 — Remediation record (§18.4)

Registered games (Gate 0) are excluded — they owe no remediation entry,
because they owe no conformance. For any other project not yet conforming:

- [ ] A **dated remediation entry** exists in `PROJECTS.md` recording the
      current accessibility state and the intended remediation.
- [ ] The date is ISO 8601 / UTC per §14.

An absent entry is itself a compliance failure. A project may be unfinished;
it may not be *silently* unfinished.

## Reporting

Report audit results with §17 progress format when the audit is part of an
implementation effort:

```
[Progress: ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▱▱▱▱▱▱] 70%
Milestones: M0: 100% | M1: 100% | M2: 70% | M3: 0%
Product Status: MVP: 90% | PRD: 70%
```
