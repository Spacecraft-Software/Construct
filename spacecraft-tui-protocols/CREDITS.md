# CREDITS

The `spacecraft-tui-protocols` skill tells programs how to use terminal
protocols designed by other projects. Those specifications form the
conceptual basis of the skill, so they are credited here in accordance with
[The Steelbore Standard §15.3](../spacecraft-steelbore-standard/SKILL.md).
All prose in this skill is original writing; no specification text is
reproduced. Escape-sequence syntax is cited for interoperability.

## Upstream specifications

| Name | Author(s) | License | Source URL | Used for |
|------|-----------|---------|------------|----------|
| Kitty keyboard protocol | Kovid Goyal and kitty contributors | GPL-3.0 (kitty docs) | https://sw.kovidgoyal.net/kitty/keyboard-protocol/ | `references/keyboard.md` |
| Kitty graphics protocol (incl. animation, Unicode placeholders) | Kovid Goyal and kitty contributors | GPL-3.0 (kitty docs) | https://sw.kovidgoyal.net/kitty/graphics-protocol/ | `references/graphics.md`, `references/detection.md` |
| Kitty clipboard protocol (OSC 5522) | Kovid Goyal and kitty contributors | GPL-3.0 (kitty docs) | https://sw.kovidgoyal.net/kitty/clipboard/ | `references/clipboard.md` |
| Kitty pointer shapes (OSC 22 extension) | Kovid Goyal and kitty contributors | GPL-3.0 (kitty docs) | https://sw.kovidgoyal.net/kitty/pointer-shapes/ | `references/progress-pointer.md` |
| XTerm Control Sequences (OSC 52, OSC 22, DA1, XTVERSION, sixel) | Thomas E. Dickey | MIT-style (xterm) | https://invisible-island.net/xterm/ctlseqs/ctlseqs.html | `references/detection.md`, `references/clipboard.md` |
| ConEmu / Windows Terminal progress sequence (OSC 9;4) | ConEmu project; Microsoft | Per project | https://learn.microsoft.com/windows/terminal/tutorials/progress-bar-sequences | `references/progress-pointer.md` |
| iTerm2 inline images (OSC 1337 File=) | George Nachman and iTerm2 contributors | GPL-2.0-or-later (iTerm2) | https://iterm2.com/documentation-images.html | `references/graphics.md` |
| terminfo.dev support tables | terminfo.dev | Per site | https://terminfo-dev.pages.dev/ | `references/support-matrix.md` (cross-check only) |

## License of this skill

`SKILL.md` and reference text: © 2026 Mohamed Hammad —
**GPL-3.0-or-later**. Protocol names and sequences remain the work of their
respective authors and are cited for interoperability.

## Maintainer

Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
