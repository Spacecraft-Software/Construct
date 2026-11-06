# OSC 9;4 progress and OSC 22 pointer shapes (Tier 2)

Back to [`SKILL.md`](../SKILL.md) §1. Upstream specifications: ConEmu's
OSC 9;4 (as adopted by Windows Terminal), and kitty's pointer-shape protocol
<https://sw.kovidgoyal.net/kitty/pointer-shapes/>, which extends xterm's
OSC 22.

## OSC 9;4 progress

```text
OSC 9 ; 4 ; <state> ; <percent> ST
```

| State | Meaning |
|-------|---------|
| 0 | Remove the progress indicator |
| 1 | Normal, `percent` 0–100 |
| 2 | Error (red), `percent` optional |
| 3 | Indeterminate (spinner) |
| 4 | Paused / warning (yellow) |

Terminals show it in the tab, title bar, taskbar or dock. It is decorative:
the program must report progress in its own output too (Standard §17 for
development progress, §18 linear mode for accessibility).

### The collision rule

Many terminals implement `OSC 9 ; <text>` as "show a desktop notification".
On such a terminal, `OSC 9 ; 4 ; 1 ; 50` pops a notification reading
`4;1;50`, once per update. There is no query that tells the two apart, so
emission is **allow-list only**, never "send and hope":

| Signal | Terminal |
|--------|----------|
| `WT_SESSION` set | Windows Terminal |
| `ConEmuPID` or `ConEmuANSI=ON` | ConEmu |
| `TERM_PROGRAM` contains `ghostty`, `wezterm`, `iterm`, `vscode`, `rio`, `contour` (case-insensitive) | Ghostty, WezTerm, iTerm2 3.6+, VS Code, Rio, contour |
| `TERM=xterm-kitty` / `KITTY_WINDOW_ID` set **and** XTVERSION at or above the first release kitty's changelog lists OSC 9;4 for (record that version in the program's source when you implement this) | kitty |
| `TERM` starts with `foot` | foot |

For iTerm2 and kitty, older versions treat OSC 9 as a notification. Where a
version can be read (XTVERSION, `TERM_PROGRAM_VERSION`), require one known to
support progress; where it cannot, skip. Everything not on the list gets no
OSC 9;4. `--term-protocols=+progress` overrides.

### Emission rules

- Rate-limit to about 5 updates a second and only when the integer percent
  changes.
- Send `state 0` on completion, on error after a short delay, and in the
  restore guard (SKILL.md §2.3), or the bar stays stuck in the tab.
- Inside tmux, wrap in passthrough or skip.
- Write to the TTY on stderr, never stdout (Spacecraft CLI Standard: stdout
  carries data only).

## OSC 22 pointer shape

```text
OSC 22 ; <name> ST           set (a bare name works in xterm too)
OSC 22 ; ST                  reset to default
OSC 22 ; ><name> ST          push
OSC 22 ; < ST                pop
OSC 22 ; ?<name>,<name> ST   query support → OSC 22 ; 1,0 ST
OSC 22 ; ?__current__ ST     query current → OSC 22 ; <name or 0> ST
```

Names follow CSS `cursor`: `default`, `pointer`, `text`, `crosshair`, `move`,
`grab`, `grabbing`, `wait`, `progress`, `help`, `not-allowed`, `ew-resize`,
`ns-resize`, `nwse-resize`, `nesw-resize`, `zoom-in`, `zoom-out`, and the
other CSS names. xterm accepts X11 cursor-font names instead (`hand2`,
`xterm`, `watch`).

Rules:

1. Use it only where the TUI has mouse-interactive regions: `pointer` over
   links and buttons, `text` over editable text, `ew-resize` / `ns-resize`
   over split handles, `wait` during a blocking operation.
2. Probe with `?__current__`. If it answers, use push and pop; restore with
   pop (SKILL.md §2.3). If it does not, it is acceptable to send plain
   `OSC 22 ; <name>` only when identity says xterm, using X11 names, and to
   reset with `OSC 22 ; ST` on exit. Otherwise send nothing.
3. Check a name with the `?name` query before relying on it; fall back to
   `default`.
4. Change the shape only on hover transitions, not on every mouse-motion
   event.
