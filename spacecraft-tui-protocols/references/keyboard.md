# Kitty keyboard protocol (Tier 1)

Back to [`SKILL.md`](../SKILL.md) §1. Upstream specification:
<https://sw.kovidgoyal.net/kitty/keyboard-protocol/>.

## Why it is Tier 1

Legacy terminal key encoding cannot tell `Ctrl+I` from `Tab`, `Ctrl+M` from
`Enter`, `Esc` from the start of an escape sequence, or report key release.
The kitty protocol fixes all of that, it is widely supported, and it degrades
to legacy encoding with no visible effect when it is absent. Standard §10
dual CUA + Vim bindings depend on distinguishing exactly these keys.

## Sequences

| Action | Sequence |
|--------|----------|
| Query current flags | `CSI ? u` → reply `CSI ? <flags> u` |
| Push flags (save old, set new) | `CSI > <flags> u` |
| Pop one entry | `CSI < u` (or `CSI < <n> u` to pop n) |
| Set flags without pushing | `CSI = <flags> ; <mode> u` (mode 1 set, 2 or, 3 and-not) |

Flags are a bitfield:

| Bit | Value | Meaning |
|-----|-------|---------|
| 0 | 1 | Disambiguate escape codes |
| 1 | 2 | Report event types (press, repeat, release) |
| 2 | 4 | Report alternate keys (shifted, base layout) |
| 3 | 8 | Report all keys as escape codes |
| 4 | 16 | Report associated text |

## Rules

1. **Push, never set.** Use `CSI > flags u` on entry and `CSI < u` on exit,
   so a parent program's flags come back. The main and alternate screens
   keep separate stacks: push after entering the alternate screen and pop
   before leaving it.
2. **Start with flag 1 only.** Add 2, 4, 8 or 16 only for a feature that
   needs them (release events for games or held-key actions, 8 for modifier
   keys alone). Flag 8 changes how plain text keys arrive and needs flag 16
   to recover typed text.
3. **Probe first** (`CSI ? u` fenced by DA1, `detection.md`). If there is no
   reply, do not push; read legacy keys.
4. **Pop on every exit path** (SKILL.md §2.3). This is the protocol where a
   missed restore hurts the user most.
5. **Suspend correctly.** On `Ctrl+Z` / `SIGTSTP` pop before stopping and
   push again on `SIGCONT`. When spawning `$EDITOR` or a shell, pop first and
   push again after it returns.
6. **Never require it.** Every binding must have a legacy-reachable
   equivalent, because tmux, GNOME Terminal and Terminal.app do not speak the
   protocol.

## Rust with crossterm

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::supports_keyboard_enhancement,
};

let kitty_keys = supports_keyboard_enhancement().unwrap_or(false);
if kitty_keys {
    execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;
    guard.keyboard_pushed = true; // detection.md TermGuard
}
// … event loop: crossterm parses both encodings into the same KeyEvent …
// On exit the guard writes `CSI < u`; the explicit form is:
// execute!(std::io::stdout(), PopKeyboardEnhancementFlags)?;
```

`KeyEvent::kind` is only meaningful (`Release`, `Repeat`) when
`REPORT_EVENT_TYPES` was pushed. Without it, filter on
`KeyEventKind::Press` so Windows, where crossterm always reports releases,
behaves the same as Unix.
