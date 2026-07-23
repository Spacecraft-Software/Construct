# CLI & TUI Accessibility — implementation reference

Standard §18.2. Read [`../SKILL.md`](../SKILL.md) first for the activation
contract and the always-on rules.

## Why terminals are different

There is no accessibility tree in a terminal. No ARIA, no roles, no live
regions, no way to say "announce this." A screen reader (Orca, NVDA) reads the
terminal emulator's **character grid**. Consequences that drive every rule
below:

- **Repainting a region re-reads it.** A TUI that redraws a status bar every
  100 ms produces a speech loop. The user cannot get past it.
- **Column alignment conveys nothing.** A table read aloud is a run-on
  sentence; the reader has no notion of "column 3."
- **Cursor-positioning tricks are invisible.** Anything whose meaning comes
  from *where* it was drawn is lost entirely.
- **No library rescues you.** ratatui, crossterm, notcurses, Textual — none
  exposes an accessibility tree, because the terminal has nowhere to put one.
  The linear fallback is the application's job.

## Rules that apply in every mode

Not gated behind the toggle. These are correctness requirements for all
output, always:

| Rule | Bad | Good |
|------|-----|------|
| Status carries a text tag | `\x1b[31mfailed to connect\x1b[0m` | `[ERROR] failed to connect` |
| Diagnostics to `stderr`, results to `stdout` | both to `stdout`, interleaved | separated |
| No text on unverified colored fills | amber text on a red chip (1.13:1) | amber text on Void Navy (7.64:1) |
| `NO_COLOR` family honored | color forced on | per `spacecraft-cli-standard` precedence |

## Rules that apply in accessible mode

### No animation

Spinners, marquees, blinking, and progress animations are replaced by a single
static line with **monotonic** progress, rewritten at most once per second.

```
Working… 40%
Working… 70%
Working… done
```

Never emit a braille/dot spinner (`⠋⠙⠹⠸`) in accessible mode — screen readers
pronounce the glyphs. GitHub hit exactly this and replaced their spinner with a
static `Working…` line for `gh a11y`.

### No decorative art

ASCII art, banners, figlet headers, and box-drawing decoration are suppressed.
Where the art is **informational** rather than decorative — a chart, a
topology diagram — emit an equivalent text description. Do not silently drop
information:

```
[INFO] Dependency graph: caliper-core depends on caliper-trace and caliper-io;
       caliper-trace depends on caliper-io.
```

### Linear, append-only output

Output reads correctly top to bottom. Separate logical sections with blank
lines so the reader can navigate by paragraph.

### Tabular fallback

Every table offers a non-columnar rendering — one `field: value` per line:

```
name: caliper-core
version: 0.4.1
status: [OK] built

name: caliper-trace
version: 0.4.1
status: [ERROR] link failed
```

### Prompt legibility

State the question, the choices, and the default in plain text *before*
awaiting input. Prompts that convey state through cursor positioning or
redraw are non-compliant.

```
Overwrite existing output? [y/N]
  y = overwrite
  N = keep existing (default)
>
```

## TUI linear mode (§18.2.3)

Any full-screen TUI must additionally provide a **non-redraw, append-only
stream mode** reachable through the same toggle:

- Write new state as **new lines**; never repaint regions.
- Do **not** enter the alternate screen buffer.
- Do not hide the cursor or install mouse-tracking escape sequences.

Where a TUI would otherwise be the only route to an operation, an equivalent
**non-interactive CLI path** must exist — flags plus `--json` — so the
operation stays scriptable and reachable without navigating a visual grid.
Per Standard §8, document that path in the Texinfo manual alongside the
interactive one.

## Rust sketch

The toggle, resolved once, threaded as a value — not read from the environment
at each call site:

```rust
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

/// How accessible mode was decided — reported under `--verbose` so the user
/// can tell *why* the mode is on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A11ySource {
    Flag,
    Env,
    Config,
    Hint,
    Default,
}

#[derive(Debug, Clone, Copy)]
pub struct Accessibility {
    pub enabled: bool,
    pub source: A11ySource,
}

impl Accessibility {
    /// Standard §18.1 precedence: flag > env > config > hints > default-off.
    /// An explicit `false` at any level wins over every lower level.
    pub fn resolve(
        flag: Option<bool>,
        env: Option<bool>,
        config: Option<bool>,
        hint: bool,
    ) -> Self {
        match (flag, env, config) {
            (Some(v), _, _) => Self { enabled: v, source: A11ySource::Flag },
            (_, Some(v), _) => Self { enabled: v, source: A11ySource::Env },
            (_, _, Some(v)) => Self { enabled: v, source: A11ySource::Config },
            _ if hint => Self { enabled: true, source: A11ySource::Hint },
            // Silence is never consent to change the default presentation.
            _ => Self { enabled: false, source: A11ySource::Default },
        }
    }

    /// Only unambiguous signals. Never infer from `$LANG`, terminal size, or
    /// the mere absence of a TTY.
    pub fn detect_hint() -> bool {
        std::env::var_os("NO_COLOR").is_some()
            || matches!(std::env::var("TERM").as_deref(), Ok("dumb"))
            || std::env::var("GTK_MODULES")
                .is_ok_and(|m| m.split(':').any(|p| p == "gail" || p == "atk"))
    }

    pub fn theme(self) -> &'static str {
        if self.enabled { "steelbore-high-contrast" } else { "steelbore" }
    }
}
```

Status tags belong to the renderer, not to call sites, so the tag can never be
forgotten:

```rust
pub enum Status { Ok, Error, Warn, Info }

impl Status {
    /// Always emitted — §18.2.1 is not gated behind the toggle.
    pub fn tag(self) -> &'static str {
        match self {
            Status::Ok => "[OK]",
            Status::Error => "[ERROR]",
            Status::Warn => "[WARN]",
            Status::Info => "[INFO]",
        }
    }
}
```

Per Standard §3.1, load `microsoft-rust-guidelines` before writing the real
implementation.

## Verifying by ear

Reading the output is not verification — listening to it is:

```sh
your-tool --accessible build 2>&1 | espeak-ng -s 160
```

If you cannot follow what happened, neither can a user who has no other
channel. Provision `espeak-ng` ephemerally per `spacecraft-missing-pkg`.
