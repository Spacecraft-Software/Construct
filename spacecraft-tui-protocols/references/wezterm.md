# WezTerm profile

Back to [`SKILL.md`](../SKILL.md). WezTerm is the maintainer's daily driver,
so it is the first terminal every Spacecraft TUI is tested in.

**Checked:** 2026-11-06, against WezTerm's source at `main` commit
`e68a8ec` (2026-10-26) and its changelog. Paths below are in
<https://github.com/wez/wezterm>.

## Read this first: stable vs nightly

WezTerm's last versioned stable release is **20240203-110809-5046fc22**
(2024-02-03). Everything WezTerm has added since ships only in nightly or
git-snapshot builds. Several rows below depend on which one is running, so
always read the version:

- `TERM_PROGRAM=WezTerm`, `TERM_PROGRAM_VERSION=<YYYYMMDD-HHMMSS-hash>` are
  set for every spawned program (`config/src/config.rs`).
- XTVERSION (`CSI > 0 q`) answers `DCS > | WezTerm <version> ST`, with a
  **space** before the version, not kitty-style parentheses
  (`term/src/terminalstate/mod.rs`). Parse both forms.
- Compare versions by their leading `YYYYMMDD`.
- On NixOS, check what the installed package actually is with
  `wezterm --version`; distribution packages are often git snapshots rather
  than the 2024 stable. WezTerm's own flake (`nix/` in its repository) builds
  from `main`.

## Protocol by protocol

| Protocol | WezTerm status | Since | Config | Notes |
|----------|----------------|-------|--------|-------|
| Kitty keyboard | **Supported, off by default** | 20220624 | `config.enable_kitty_keyboard = true` | With it off, `CSI ? u` gets no reply and push/set are ignored, so the DA1-fenced probe correctly reports "unsupported". Separate stacks per screen; stack capped at 128. Builds before 2026-06-20 encoded `Esc` wrongly under the disambiguate flag (fix `aea9b9f`). |
| Kitty graphics | **Supported, on by default** | before 2024 | `config.enable_kitty_graphics` (default `true`) | `a=q` query, transmit, display and frames work. **No Unicode placeholders** (`U=1`), so no multiplexer-safe placement. Delete handles only by-id and all; every other `d=` target is logged and ignored, so delete by image id. When disabled, nothing is answered and the probe reports "unsupported". |
| Kitty animation | **Partial** | — | — | `a=f` (transmit frame) and `a=c` (compose) work, and frames with a gap play on their own. There is **no `a=a`** animation control, so a program cannot start, stop or set a loop count. Use app-driven frames (graphics.md, animation rule 2), which also keeps it stoppable for accessible mode. |
| Sixel | **Supported** | long-standing | — | DA1 advertises `4`. |
| iTerm2 inline images | **Supported** (single `File=` sequence) | long-standing | — | No `MultipartFile` / `FilePart` / `FileEnd`; send one `File=` sequence. Rung 3 is reachable, but rungs 1 and 2 come first. |
| OSC 52 write | **Supported** | long-standing | — | `c`, `p` and clear all work. DA1 advertises `52` since 2025-06-22 (`58141ad`). Since 2026-09-15 the clipboard goes to the pane's own window (`016b962`). |
| OSC 52 read | **Not supported** | — | — | `OSC 52 ; c ; ? ST` is parsed and silently ignored: no reply, so the DA1 fence ends the wait. Never wait on it without the fence. |
| OSC 9;4 progress | **Nightly only** | 2025-02-09 (`fb02395`, `44866cc`) | — | Shown in the tab bar by default since 2025-02-10, with a spinner for indeterminate since 2026-07-15. State 4 (paused) is accepted but not shown. Lua can read it with `pane:get_progress()`. **On the 20240203 stable, OSC 9 is a desktop notification**, so `OSC 9;4;1;50` pops a toast reading `4;1;50`. Emit only when the version is ≥ 20250210. |
| OSC 9 notification | **Supported** | — | `notification_handling` | Why the progress version gate above is a BLOCKER. |
| OSC 22 pointer shape | **Not supported** | — | — | OSC 22 is not in WezTerm's OSC table; it is dropped, and `?__current__` gets no reply. Send nothing; the probe handles it. |
| OSC 5522 clipboard | **Not supported** | — | — | Not parsed; no reply. Use OSC 52. |
| Cell size (`CSI 16 t`) | **Supported** | — | — | Use it to size images. |

## What a Spacecraft TUI gets in WezTerm

- **Keyboard:** full kitty keyboard once the user sets
  `enable_kitty_keyboard = true`; legacy keys otherwise. Recommend the setting
  in the program's docs and in `--verbose` diagnostics when the probe finds no
  support and `TERM_PROGRAM=WezTerm`.
- **Images:** kitty graphics (rung 1), deleted by id on exit. Inside tmux in
  WezTerm, half-blocks, because WezTerm lacks Unicode placeholders.
- **Animation:** app-driven frames only.
- **Clipboard:** OSC 52 write. No read, no rich MIME.
- **Progress:** OSC 9;4 on nightly or snapshot builds from 2025-02-10 on;
  nothing on the 2024 stable.
- **Pointer shapes:** none.

## Suggested user config

```lua
-- ~/.config/wezterm/wezterm.lua (or the Home Manager programs.wezterm.extraConfig)
local config = wezterm.config_builder()
config.enable_kitty_keyboard = true   -- off by default
config.enable_kitty_graphics = true   -- already the default; stated for clarity
return config
```
