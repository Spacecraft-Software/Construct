# Terminal support matrix

Back to [`SKILL.md`](../SKILL.md) §1.

**Checked:** 2026-11-06. **Status:** indicative, not normative. Programs
detect support at runtime (SKILL.md §2.2); this table exists to choose tiers
and plan tests, never to hard-code behaviour by terminal name. Re-check a row
against the terminal's own documentation before relying on it, and update
the date when you do.

Legend: **Y** supported · **P** partial or opt-in · **N** not supported ·
**?** unverified.

| Terminal | Kitty keyboard | Kitty graphics | Kitty animation | Sixel | OSC 52 write | OSC 9;4 | OSC 22 | OSC 5522 |
|----------|---------------|----------------|-----------------|-------|--------------|---------|--------|----------|
| kitty | Y | Y | Y | N | Y | Y (recent) | Y | Y |
| Ghostty | Y | Y | ? | N | Y | Y | Y | ? |
| WezTerm (see `wezterm.md`) | P (off until `enable_kitty_keyboard = true`) | P (no Unicode placeholders) | P (frames play, no `a=a` control) | Y | Y (read: N) | P (nightly ≥ 2025-02-10 only; stable shows a notification) | N | N |
| foot | Y | N | N | Y | Y | Y | Y | ? |
| iTerm2 3.6 | Y | Y (reported) | ? | Y | Y | Y | Y | ? |
| Alacritty | Y | N | N | N | Y | ? | ? | N |
| Konsole | N | P | ? | Y | ? | Y (reported) | ? | ? |
| Windows Terminal | ? | N | N | Y (1.22+) | Y | Y | ? | N |
| GNOME Terminal / VTE | N | N | N | P (build option) | **N** | ? | ? | N |
| xterm | N (uses modifyOtherKeys) | N | N | Y (VT340 mode) | P (config) | ? | Y (X11 names) | N |
| VS Code (xterm.js) | Y (recent) | N | N | P (opt-in) | Y | Y | Y | N |
| Warp | Y | Y | ? | ? | Y | Y | Y | ? |
| Terminal.app | N | ? | ? | N | Y | ? | ? | N |
| tmux | N (own `extended-keys`) | P (passthrough + placeholders) | N | P (3.4+, build option) | Y (`set-clipboard on`) | passthrough | passthrough | N |

## What the table means for each tier

- **Kitty keyboard:** most current terminals; the legacy fallback covers the
  rest with no visible cost.
- **Kitty graphics:** kitty and Ghostty fully, others partially. Sixel covers
  foot, WezTerm, Windows Terminal, xterm and Konsole. Half-blocks cover the
  rest, notably GNOME Terminal and Alacritty.
- **Kitty animation:** only kitty is confirmed. This is why it is Tier 3 and
  app-driven frames are the portable path.
- **OSC 52:** near-universal for write. GNOME Terminal (VTE) is the notable
  gap, which is why SKILL.md requires a visible fallback.
- **OSC 9;4:** broad and growing, but the notification collision on older
  iTerm2 and kitty and on unknown terminals is why emission is allow-list
  only.
- **OSC 5522:** kitty is the only implementation confirmed here, which is why
  it is Tier 3.

## Sources

- kitty protocol specifications: keyboard
  <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>, graphics
  <https://sw.kovidgoyal.net/kitty/graphics-protocol/>, clipboard
  <https://sw.kovidgoyal.net/kitty/clipboard/>, pointer shapes
  <https://sw.kovidgoyal.net/kitty/pointer-shapes/>
- iTerm2 inline images: <https://iterm2.com/documentation-images.html>
- terminfo.dev extension pages (per-terminal test results; its summary counts
  and tables sometimes disagree, so treat it as one input):
  <https://terminfo-dev.pages.dev/extensions/kitty-keyboard-protocol>,
  `/kitty-graphics-protocol`, `/kitty-graphics-animation`,
  `/osc52-clipboard-write`, `/osc9-progress-bar`, `/osc-22-pointer-shape`,
  `/osc5522-clipboard`
- `termpulse` crate's OSC 9;4 detection list (`src/detect.rs`), which is the
  basis of the allow-list in `progress-pointer.md`
- Each terminal's own documentation and changelog for the version-qualified
  rows
