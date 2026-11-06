# Detection, probing and restore

Back to [`SKILL.md`](../SKILL.md) §2.

Notation: `ESC` = `0x1b`, `CSI` = `ESC [`, `OSC` = `ESC ]`, `APC` = `ESC _`,
`DCS` = `ESC P`, `ST` = `ESC \`. Prefer `ST` over `BEL` (`0x07`) as the OSC
terminator in what you send, and accept both in what you read.

## The probe set

Send only the probes for features the program uses, then DA1 last.

| Feature | Probe | Positive reply | Notes |
|---------|-------|----------------|-------|
| Kitty keyboard | `CSI ? u` | `CSI ? <flags> u` | Flags currently in effect |
| Kitty graphics | `APC G i=31,s=1,v=1,a=q,t=d,f=24;AAAA ST` | `APC G i=31;OK ST` | A 1×1 RGB query image; `a=q` means nothing is displayed or stored. An error reply (`APC G i=31;E… ST`) also proves the protocol is spoken |
| Sixel | (DA1 itself) | `4` among the DA1 parameters | |
| Cell size in pixels | `CSI 16 t` | `CSI 6 ; <h> ; <w> t` | Needed to size images. Fall back to `CSI 14 t` (window pixels) divided by rows and columns, then to a default of 8×16 |
| Pointer shape | `OSC 22 ; ?__current__ ST` | `OSC 22 ; <name or 0> ST` | |
| Terminal identity | `CSI > 0 q` (XTVERSION) | `DCS > \| <name(version)> ST` or `DCS > \| <name> <version> ST` (WezTerm) | A hint only (SKILL.md §2.2). Parse both forms |
| OSC 5522 | `OSC 5522 ; type=read ; Lg== ST` | `OSC 5522 ; type=read:status=OK ST` … | `Lg==` is base64 of `.`, which lists available MIME types without a permission prompt. `status=ENOSYS` or no reply means unsupported |
| Fence | `CSI c` (DA1) | `CSI ? <params> c` | Always last |

OSC 52 write and OSC 9;4 have no query. Use identity (`progress-pointer.md`,
`clipboard.md`).

## Reader algorithm

```text
enable raw mode
write(probes ++ "\x1b[c")
deadline = now + 200 ms
loop:
    wait for input until deadline        # poll(2), never a blocking read
    if timed out: break
    read available bytes, feed the parser
    for each complete reply:
        record it against its probe
        if it is the DA1 reply: done = true
    if done: break
drain: read with a ~20 ms wait until nothing arrives   # swallow late replies
restore the previous terminal mode
```

The parser must handle replies split across reads and interleaved with real
key presses. Hand any non-reply bytes to the normal input path instead of
dropping them.

## Rust sketch

This reads from `/dev/tty`, so it works when stdin is redirected. It uses
`crossterm` for raw mode, `rustix` for `poll`, and leaves parsing to a small
state machine (or the `vte` crate). Check the current `rustix` `poll`
signature for your pinned version before copying; it changed between 0.38
and 1.x.

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// What the startup probe found. Everything defaults to `false`.
#[derive(Debug, Default, Clone)]
pub struct Caps {
    pub kitty_keyboard: bool,
    pub kitty_graphics: bool,
    pub sixel: bool,
    pub pointer_shape: bool,
    pub cell_px: Option<(u16, u16)>,
    pub xtversion: Option<String>,
}

pub fn probe(deadline: Duration) -> std::io::Result<Caps> {
    let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    enable_raw_mode()?;
    let _raw = scopeguard::guard((), |()| {
        let _ = disable_raw_mode();
    });

    tty.write_all(
        b"\x1b[?u\
          \x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\
          \x1b[16t\
          \x1b]22;?__current__\x1b\\\
          \x1b[>0q\
          \x1b[c",
    )?;
    tty.flush()?;

    let mut parser = ReplyParser::default(); // your state machine or a vte::Perform impl
    let mut caps = Caps::default();
    let end = Instant::now() + deadline;
    let mut buf = [0u8; 4096];

    while let Some(left) = end.checked_duration_since(Instant::now()) {
        if !wait_readable(&tty, left)? {
            break; // deadline hit: unanswered features stay false
        }
        let n = tty.read(&mut buf)?;
        if parser.feed(&buf[..n], &mut caps) {
            break; // DA1 reply seen
        }
    }
    drain(&mut tty, Duration::from_millis(20))?;
    Ok(caps)
}
```

`wait_readable` wraps `poll` on the TTY fd with the remaining time;
`drain` reads and discards until a short `poll` times out. Keep `probe`
fallible and treat any error as "nothing supported"; a failed probe must
never stop the program from starting.

If the program uses `crossterm::terminal::supports_keyboard_enhancement()` or
`ratatui_image::picker::Picker::from_query_stdio()`, those already run a
fenced probe for their own feature. Run them one after the other at startup,
before the event loop starts reading input, never concurrently with it.

## Restore guard

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::Write;

/// Records which modes were turned on, and undoes them in reverse on drop.
#[derive(Default)]
pub struct TermGuard {
    pub keyboard_pushed: bool,
    pub pointer_pushed: bool,
    pub progress_active: bool,
    pub mouse: bool,
    pub alt_screen: bool,
}

impl TermGuard {
    pub fn restore(&mut self, out: &mut impl Write) {
        if std::mem::take(&mut self.keyboard_pushed) {
            let _ = out.write_all(b"\x1b[<u");
        }
        if std::mem::take(&mut self.pointer_pushed) {
            let _ = out.write_all(b"\x1b]22;<\x1b\\");
        }
        if std::mem::take(&mut self.progress_active) {
            let _ = out.write_all(b"\x1b]9;4;0;0\x1b\\");
        }
        if std::mem::take(&mut self.mouse) {
            let _ = out.write_all(b"\x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l");
        }
        if std::mem::take(&mut self.alt_screen) {
            let _ = out.write_all(b"\x1b[?1049l");
        }
        let _ = out.flush();
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        self.restore(&mut std::io::stdout());
    }
}
```

Share the guard state with the panic hook and the signal handler (for
example through a `static` `Mutex<TermGuard>` or atomics per mode), so all
three paths run the same `restore`, and `std::mem::take` makes a second run a
no-op. Install the panic hook with `std::panic::take_hook` and chain to the
previous hook after restoring, so the panic message prints on a sane
terminal. Images that should outlive the program (inline output in the main
screen) are left alone; images drawn in a full-screen TUI are deleted with
`APC G a=d ST` before leaving the alternate screen.

## Identity hints

Use these only for features without a query, or to choose between two
probed-positive paths:

| Variable / reply | Meaning |
|------------------|---------|
| `TERM_PROGRAM` | `ghostty`, `WezTerm`, `iTerm.app`, `vscode`, `WarpTerminal`, `rio`, … |
| `TERM` | `xterm-kitty`, `xterm-ghostty`, `foot`, `wezterm`, `alacritty`, … |
| `KITTY_WINDOW_ID` | kitty |
| `WT_SESSION` | Windows Terminal |
| `ConEmuPID` / `ConEmuANSI` | ConEmu |
| `TMUX`, `ZELLIJ`, `STY` | tmux, zellij, GNU screen (SKILL.md §3) |
| XTVERSION reply | Name and version as the terminal reports it; survives SSH, unlike environment variables |

Over SSH the environment variables usually do not travel; XTVERSION and the
probes do. Prefer them when both are available.
