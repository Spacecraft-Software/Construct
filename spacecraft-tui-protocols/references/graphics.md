# Images and animation (Tier 1, animation Tier 3)

Back to [`SKILL.md`](../SKILL.md) §1. Upstream specifications: kitty graphics
<https://sw.kovidgoyal.net/kitty/graphics-protocol/>, iTerm2 inline images
<https://iterm2.com/documentation-images.html>, and the DEC sixel format as
implemented by xterm.

## The fallback ladder

Pick the first rung the probe confirms. Never skip to a rung that was not
confirmed, except the last two, which work everywhere.

| Rung | Protocol | Confirmed by | Typical terminals |
|------|----------|--------------|-------------------|
| 1 | Kitty graphics | `a=q` query reply (`detection.md`) | kitty, Ghostty, WezTerm, Konsole (partial) |
| 2 | Sixel | `4` in the DA1 reply | foot, WezTerm, xterm (VT340 mode), mlterm, Konsole, Windows Terminal, contour |
| 3 | iTerm2 inline images (`OSC 1337 ; File=`) | Identity: `TERM_PROGRAM=iTerm.app` or `WezTerm` (no query in common use) | iTerm2, WezTerm |
| 4 | Unicode half-blocks (`▀` with fg/bg colours) | Truecolor or 256-colour output | Any colour terminal |
| 5 | Alt text | Always | Non-colour, `NO_COLOR`, accessible mode |

In accessible mode, rung 5 text is printed *before* the image on every rung
(SKILL.md §4). `ratatui-image` implements rungs 1 to 4 with probing; use it
in ratatui TUIs rather than writing a renderer.

## Kitty graphics essentials

- Transmit once, place many times. Give each image an id (`i=`), transmit
  with `a=t`, display with `a=p`, or do both with `a=T`.
- Prefer `f=100` (PNG) or `f=32` (RGBA) with `o=z` (zlib) to cut bytes.
- Chunk payloads at 4096 base64 bytes with `m=1` on every chunk but the last.
- Use `q=2` on transmit and place commands to suppress replies the program
  does not read. Replies it does not read end up as input.
- Where the terminal and program run on the same machine, `t=f` (file) or
  `t=s` (shared memory) avoids sending pixels through the PTY at all. Check
  `SSH_CONNECTION` / `SSH_TTY`; over SSH use `t=d` (direct).
- Delete what you placed: `APC G a=d,d=i,i=<id> ST` per image, or `a=d` for
  all visible placements, before leaving the alternate screen (SKILL.md §2.3).
- Size from the cell-pixel probe (`CSI 16 t`) and place with `c=` / `r=`
  (columns, rows) so the image fits the layout regardless of font size.

### Inside tmux

Kitty graphics works in tmux only with `allow-passthrough on`, every
sequence wrapped as `ESC P tmux; <sequence with each ESC doubled> ESC \`, and
**Unicode placeholders**: create a virtual placement with `U=1`, then print
the placeholder character `U+10EEEE` with row/column diacritics and the image
id in the foreground colour. Without placeholders the image does not scroll
or move with its pane. If the program does not implement placeholders, use
rung 4 in tmux.

## Animation (Tier 3)

Kitty's animation extends the same protocol: `a=f` adds frames to an image,
`a=a` controls playback (`s=3` run, `s=1` stop, `v=` loop count), and `a=c`
composes frames. Only kitty is confirmed to implement it fully (see
`support-matrix.md`), and there is no separate probe beyond trying it.

Rules:

1. Animation lives inside the graphics module. It is never its own
   user-facing feature or flag beyond `animation` in `--term-protocols`.
2. The portable path is app-driven: re-place frames on a timer by replacing
   the image data under the same id (kitty, Ghostty) or redrawing sixel or
   half-block frames. Use native `a=f` only where `KITTY_WINDOW_ID` or
   XTVERSION says kitty, as an optimisation.
3. No animation in accessible mode, under reduced motion, or in agent mode;
   show the first frame and its alt text instead.
4. Cap frame rate (≤ 15 fps for app-driven frames over SSH) and stop when the
   image scrolls out of view.

## Sixel and iTerm2 notes

- Sixel is palette-based; quantise to 256 colours or fewer and expect banding.
  Ask for colour registers with `CSI ? 1 ; 1 S` (XTSMGRAPHICS) where needed.
- Sixel images are cleared by overwriting the cells, not by an id.
- iTerm2: `OSC 1337 ; File=inline=1;size=<bytes>;width=<n>;height=<n>;preserveAspectRatio=1 : <base64> ST`.
  For large images, and always inside tmux, use the multipart form that
  iTerm2 3.5+ provides: `OSC 1337 ; MultipartFile=<args> ST`, then one or
  more `OSC 1337 ; FilePart=<base64 chunk> ST`, then `OSC 1337 ; FileEnd ST`.
  iTerm2 caps a single sequence at 1 MiB, and older tmux far lower.
