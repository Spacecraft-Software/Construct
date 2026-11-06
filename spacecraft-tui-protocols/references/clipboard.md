# Clipboard: OSC 52 (Tier 1), OSC 5522 (Tier 3), OSC 1337 (not used)

Back to [`SKILL.md`](../SKILL.md) §1. Upstream specifications: OSC 52 as
defined by xterm's control sequences document, and kitty's clipboard
protocol <https://sw.kovidgoyal.net/kitty/clipboard/>.

## OSC 52 (baseline)

Write the clipboard:

```text
OSC 52 ; c ; <base64 of UTF-8 text> ST
```

`c` is the clipboard; `p` is the primary selection (X11 / Wayland); `s` is
also accepted by many terminals. Clear with an empty or invalid payload
(`OSC 52 ; c ; ! ST`).

Rules:

1. **Write is the feature.** It works across SSH and tmux, which is its whole
   value over calling `wl-copy` or a platform API.
2. **No query exists for write support**, so send it when §2.1 gates pass and
   the user asked to copy. A terminal without support ignores it. Because
   that failure is silent, always confirm in the UI with the copied text or a
   way to see it ("Copied 42 characters; press `v` to view"), so a user on
   GNOME Terminal (VTE has no OSC 52) can still copy by hand.
3. **Keep payloads small.** Several terminals and tmux cap the sequence (often
   around 100 KB of base64, some much lower). Over the cap, tell the user and
   offer to write to a file instead.
4. **Read (`OSC 52 ; c ; ? ST`) is optional and usually denied.** Many
   terminals disable it or prompt each time, for good security reasons. Never
   rely on it; if used, it goes through the §2.2 fenced probe with a
   deadline and treats silence as "denied".
5. **tmux:** with `set -g set-clipboard on`, tmux receives the sequence and
   forwards it to the outer terminal itself. Send it unwrapped.
6. **Locally, a platform clipboard is fine too.** When not over SSH and a
   native clipboard is reachable (`arboard` crate on Wayland/X11/macOS/
   Windows), the program may use it, and fall back to OSC 52.

Rust:

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
use base64::Engine as _;
use std::io::Write;

pub fn osc52_copy(out: &mut impl Write, text: &str) -> std::io::Result<()> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    write!(out, "\x1b]52;c;{b64}\x1b\\")?;
    out.flush()
}
```

`crossterm` also ships `clipboard::CopyToClipboard`
(`CopyToClipboard::to_clipboard_from(text)`, `::to_primary_from(text)`);
check whether your pinned version gates it behind a cargo feature.

## OSC 5522 (Tier 3, optional)

Kitty's newer clipboard protocol carries any MIME type (images, HTML, rich
text), can read with a permission prompt, and chunks large payloads.
Implement it only when the program genuinely copies non-text content, and
only after the probe in `detection.md` gets `status=OK`.

Write flow (each packet is `OSC 5522 ; <metadata> ; <base64 payload> ST`,
metadata is colon-separated `key=value`):

```text
OSC 5522 ; type=write ST
OSC 5522 ; type=wdata:mime=text/plain ; <base64 chunk> ST     # repeat per chunk
OSC 5522 ; type=wdata:mime=image/png ; <base64 chunk> ST      # next MIME type
OSC 5522 ; type=wdata ST                                     # end of data
← OSC 5522 ; type=write:status=DONE ST   (or EPERM, EIO, EINVAL, ENOSYS, EBUSY)
```

Chunks are at most 4096 bytes before base64. Send all chunks of one MIME
type before the next. Add `loc=primary` on the opening `type=write` for the
primary selection. Always include a `text/plain` representation, and fall
back to OSC 52 with that text when the reply is an error or never arrives.

## OSC 1337 clipboard (not implemented)

iTerm2's own copy sequence (`OSC 1337 ; Copy=…`) predates its OSC 52 support.
iTerm2 now supports OSC 52, so the iTerm2-specific form adds a code path with
no user benefit. Do not implement it. This exclusion does not cover iTerm2's
`OSC 1337 ; File=` inline images, which are rung 3 of the image ladder.
