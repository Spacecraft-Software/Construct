---
name: spacecraft-tui-protocols
description: >
  Implements modern terminal protocols in Spacecraft Software TUIs and CLIs so
  they light up where the terminal supports them and stay silent where it does
  not. Covers the kitty keyboard protocol, kitty graphics with a sixel, iTerm2
  inline-image and half-block fallback ladder, OSC 52 clipboard, OSC 9;4
  progress, and OSC 22 mouse pointer shapes, with kitty animation and the
  OSC 5522 clipboard as optional kitty-only extras and OSC 1337 clipboard
  ruled out. Mandates one detection pattern for all of them: gate on a TTY and
  on agent mode, probe once at startup fenced by a DA1 query, never block on a
  reply, and restore every mode on exit, panic and signal through a Rust
  guard. Consult when adding images, rich key handling, copy to clipboard,
  taskbar progress or pointer shapes to a terminal program, when reviewing
  escape-sequence output, or when a terminal prints garbage, leaves keys
  garbled after exit, or pops stray notifications.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Software TUI Protocols

**Version:** 1.0.1 | **Date:** 2026-11-08 | **Author:** Mohamed Hammad
**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

This skill tells you which modern terminal protocols a Spacecraft Software
program should speak, and how to speak them without breaking terminals that
don't. It sits beside `spacecraft-cli-standard` (output modes, TTY and agent
detection, `--format explore`) and `spacecraft-accessibility-support`
(Standard §18). Rust is the reference language; load
`microsoft-rust-guidelines` and `spacecraft-rust-guidelines` before writing
any of the Rust shown here.

---

## §1 — Scope: what to support

Support is tiered by how many users benefit and how safely it degrades.
"Implement" means the full detect → use → fall back → restore cycle in §2.

| Tier | Protocol | Sequence family | Rule |
|------|----------|-----------------|------|
| 1 | Kitty keyboard protocol | `CSI > flags u` / `CSI ? u` / `CSI < u` | **Implement** in every TUI that reads keys. [`references/keyboard.md`](references/keyboard.md) |
| 1 | Kitty graphics protocol | `APC G … ST` | **Implement** in any TUI that shows images, always behind a positive probe, with the fallback ladder sixel → iTerm2 `OSC 1337;File=` → Unicode half-blocks → alt text. [`references/graphics.md`](references/graphics.md) |
| 1 | OSC 52 clipboard (write) | `OSC 52 ; c ; base64 ST` | **Implement** as the baseline copy path. Reading is optional and usually denied. [`references/clipboard.md`](references/clipboard.md) |
| 2 | OSC 9;4 progress | `OSC 9 ; 4 ; state ; pct ST` | **Implement** for long operations, gated on terminal identity (§2.4). [`references/progress-pointer.md`](references/progress-pointer.md) |
| 2 | OSC 22 pointer shape | `OSC 22 ; name ST` | **Implement** when the TUI has mouse-interactive regions. [`references/progress-pointer.md`](references/progress-pointer.md) |
| 3 | Kitty graphics animation | `APC G a=f/a=a/a=c` | **Optional.** Part of the graphics module, not a separate feature. Without it, the app swaps frames itself. |
| 3 | OSC 5522 clipboard | `OSC 5522 ; type=… ST` | **Optional.** Only for rich MIME copy (images, HTML). Always keep OSC 52 as the fallback. |
| — | OSC 1337 clipboard (iTerm2) | `OSC 1337 ; Copy=…` | **Do not implement.** iTerm2 supports OSC 52. iTerm2's `OSC 1337;File=` *inline images* are a separate feature and stay in the graphics ladder. |

Tier 3 items are never a reason to block a release, and no Tier 1 or 2 code
may depend on them.

---

## §2 — The five rules every protocol follows (BLOCKER)

Unknown but well-formed escape sequences are normally parsed and discarded
(ECMA-48), which is why careful use is safe. Every way these protocols go
wrong comes from breaking one of these rules.

### §2.1 — Gate before emitting anything

Emit no protocol sequence, and send no probe, when any of these holds:

- the output stream is not a TTY (`std::io::IsTerminal`);
- agent or CI mode is active (`AI_AGENT` or `AGENT` set to any non-empty
  value, or `CI` truthy; see `spacecraft-cli-standard` §agent detection);
- `TERM` is `dumb` or unset;
- the user turned the protocol off (§2.5).

### §2.2 — Probe once, fenced by DA1, never block

Where a protocol has a query, detect support by asking, not by guessing:

1. Enter raw mode. Write every probe for the features you need **plus a
   trailing `CSI c` (DA1)** in one write.
2. Read replies until the DA1 reply (`CSI ? … c`) arrives. Every terminal
   answers DA1, and terminals answer in order, so a feature that has not
   replied by then is unsupported.
3. Cap the whole exchange with a deadline (default 200 ms, configurable). On
   timeout, treat every unanswered feature as unsupported and **drain** any
   late bytes before handing the TTY back, so no reply leaks into the shell
   as typed text.
4. Probe once at startup. Cache the result for the session.

Identity hints (`TERM_PROGRAM`, `TERM`, `WT_SESSION`, XTVERSION `CSI > q`)
are used only where no query exists (OSC 52 write, OSC 9;4). They are hints,
never proof. Full probe set and a Rust reader: [`references/detection.md`](references/detection.md).

### §2.3 — Restore on every exit path

Each mode that changes terminal state gets undone on normal exit, error
return, panic, and SIGTERM/SIGHUP (in raw mode, Ctrl-C arrives as a key, not
SIGINT). In Rust this is a guard whose `Drop` restores, plus a panic hook
that runs the same restore before the panic message prints, plus a signal
handler (`signal-hook`). Restore order is the reverse of setup:

pop keyboard flags (`CSI < u`) → pop pointer shape (`OSC 22 ; <`) → clear
progress (`OSC 9 ; 4 ; 0 ST`) → delete images placed on the main screen →
disable mouse reporting → leave the alternate screen → leave raw mode.

A missed keyboard pop leaves the user's shell receiving `CSI … u` for every
key until `reset`. That is a BLOCKER, not a cosmetic bug.

### §2.4 — Never send a sequence another meaning can catch

Some sequences collide with older meanings. The known case is OSC 9:
terminals that treat `OSC 9 ; text` as a desktop notification will pop a
notification reading `4;1;50` for every progress update. Emit OSC 9;4 only
when identity says the terminal handles it (allow-list in
[`references/progress-pointer.md`](references/progress-pointer.md)), never on
an unknown terminal. Large APC image payloads also count: send graphics only
after a positive probe, because the few terminals that print unknown APC
print megabytes of base64.

### §2.5 — User override

Every protocol has a user override that beats detection, as a flag and an
environment variable, so a user can force a feature on in a terminal that
mis-reports or off in one that misbehaves:

```text
--term-protocols=<list>          e.g. --term-protocols=-graphics,+progress
SPACECRAFT_TERM_PROTOCOLS=<list> same syntax; the flag wins
```

Names: `keyboard`, `graphics`, `animation`, `clipboard`, `clipboard-rich`,
`progress`, `pointer`, and `all`. `+name` forces on, `-name` forces off.
List the resolved state in `--verbose` diagnostics and in `describe`/`schema`
output where the program has one.

---

## §3 — Multiplexers

tmux, zellij and GNU screen sit between the program and the terminal. Treat
"inside a multiplexer" as its own capability tier, detected by `TMUX`,
`ZELLIJ` or `STY`:

- Probes go through the multiplexer. A missing reply means unsupported, as
  everywhere else; never assume the outer terminal's features.
- Keyboard: tmux does not answer `CSI ? u`. It has its own `extended-keys`
  setting; the program falls back to legacy keys.
- Graphics: kitty graphics in tmux needs `allow-passthrough on`, every
  sequence wrapped in `DCS tmux; … ST` with inner `ESC` doubled, and
  **Unicode placeholders** (`U=1`) so images move with the text. If the
  placeholder path is not implemented, fall back to half-blocks in tmux.
- OSC 52: tmux handles it itself with `set-clipboard on`; send it unwrapped.
- OSC 9;4 and OSC 22: wrap in passthrough or skip. Skipping is acceptable.

---

## §4 — Accessibility and agent mode

- **Agent / CI / non-TTY:** no sequences, no probes (§2.1). Structured output
  carries no image data; describe images by path or alt text.
- **Accessible mode (`--accessible` / `SPACECRAFT_A11Y=1`, Standard §18):**
  no animation; every image is preceded by a one-line text description;
  progress is also reported as text (the §18 linear mode), with OSC 9;4
  allowed as a supplement; pointer shapes may stay.
- **Reduced motion:** treat it like accessible mode for animation.

---

## §5 — Rust building blocks

Prefer these over hand-rolled escape handling:

| Need | Crate | Notes |
|------|-------|-------|
| Keyboard flags, mouse, alt screen, raw mode | `crossterm` | `PushKeyboardEnhancementFlags`, `PopKeyboardEnhancementFlags`, `terminal::supports_keyboard_enhancement()` (which does the DA1-fenced probe) |
| Images with probing and fallbacks | `ratatui-image` | Probes kitty / sixel / iTerm2 / half-blocks and font size; renders as a ratatui widget |
| OSC 52 write | `crossterm` (`clipboard::CopyToClipboard`) or a 10-line encoder with `base64` | |
| Low-level parsing of replies | `vte` or `termwiz` | For the §2.2 reader when not using a crate that probes for you |
| Signals for §2.3 | `signal-hook` | |

Pin versions in `Cargo.toml` and check each crate's current API before use;
these crates move quickly.

---

## §6 — Audit checklist

Run, don't read. Each item is checked by running the program in the named
environment.

- [ ] `prog | cat` and `AI_AGENT=x prog` emit no escape sequences on
      either stream: `prog 2>&1 | tr -cd '\033' | wc -c` and
      `AI_AGENT=x prog 2>&1 | tr -cd '\033' | wc -c` both print 0. Count
      the ESC byte itself. Never grep `od -c` output for `033`: its octal
      offset column contains `033` too (`0000330`, `0003300`), so clean
      output of 1728 bytes or more, and some shorter lengths, fails.
- [ ] Startup in a terminal that answers nothing but DA1 (e.g. `TERM=xterm`
      inside `script`) completes within the probe deadline and leaves no reply
      bytes in the shell.
- [ ] `kill -TERM` and a forced panic both leave the shell typing normally
      (no `CSI u` garbage), the pointer shape reset, and no stuck progress bar.
- [ ] Images: kitty, Ghostty or WezTerm shows the kitty path; foot the sixel
      path; tmux without passthrough shows half-blocks; GNOME Terminal (VTE)
      shows half-blocks.
- [ ] OSC 9;4 is not sent in a terminal outside the allow-list, including
      WezTerm's 2024 stable release, where it would pop a notification.
- [ ] WezTerm (the reference terminal, [`references/wezterm.md`](references/wezterm.md))
      is tested both with and without `enable_kitty_keyboard = true`.
- [ ] `--term-protocols=-all` turns every protocol off, `SPACECRAFT_TERM_PROTOCOLS`
      is honoured, and the flag beats the variable.
- [ ] Accessible mode shows image descriptions and text progress, with no
      animation.

---

## §7 — References

| File | Read when |
|------|-----------|
| [`references/detection.md`](references/detection.md) | Writing the startup probe, the DA1-fenced reader, or the restore guard |
| [`references/keyboard.md`](references/keyboard.md) | Handling keys with the kitty keyboard protocol |
| [`references/graphics.md`](references/graphics.md) | Showing images or animations; the fallback ladder; tmux |
| [`references/clipboard.md`](references/clipboard.md) | Copy and paste: OSC 52, OSC 5522, why not OSC 1337 |
| [`references/progress-pointer.md`](references/progress-pointer.md) | OSC 9;4 progress and OSC 22 pointer shapes |
| [`references/support-matrix.md`](references/support-matrix.md) | Which terminals support what, with sources and the date checked |
| [`references/wezterm.md`](references/wezterm.md) | Testing in WezTerm, the maintainer's daily driver: per-protocol status checked against its source, the stable-vs-nightly split, and the config to enable |
| [`CREDITS.md`](CREDITS.md) | The upstream specifications this skill is based on (Standard §15.3) |
