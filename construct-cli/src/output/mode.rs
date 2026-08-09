// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Output-mode detection and the color-precedence chain (CLI Standard §5, §6).
//!
//! The cascade is, first match wins: explicit `--format`/`--json` flag → agent
//! environment (`AI_AGENT`/`AGENT`/`CI`) → stdout is a TTY (human + color) →
//! piped stdout (JSON). `--format explore` additionally refuses to trap an
//! agent: it falls back to JSON (warning on stderr) when no interactive TTY is
//! available.

use std::io::IsTerminal as _;

use crate::cli::{ColorArg, FormatArg, GlobalArgs};

/// The resolved rendering mode for an invocation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum OutputMode {
    /// Human terminal output with ANSI color.
    HumanWithColor,
    /// Human terminal output without color.
    HumanNoColor,
    /// Single JSON document with the envelope.
    Json,
    /// Newline-delimited JSON.
    Jsonl,
    /// YAML 1.2.
    Yaml,
    /// RFC 4180 CSV.
    Csv,
    /// Interactive ratatui TUI.
    #[allow(
        dead_code,
        reason = "constructed once the explore TUI lands in a later phase"
    )]
    Explore,
}

impl OutputMode {
    /// True for the machine-readable formats (no color, pure data on stdout).
    pub(crate) fn is_machine(self) -> bool {
        matches!(self, Self::Json | Self::Jsonl | Self::Yaml | Self::Csv)
    }
}

/// Resolve the output mode from the global flags and environment. The second
/// element is the `--format explore` fallback reason, when the TUI could not
/// run — the caller emits it as a `TUI_FALLBACK` warn diagnostic once a full
/// `Context` exists, so it honors the output mode and severity floor instead
/// of being raw JSON on a human terminal.
pub(crate) fn resolve(g: &GlobalArgs) -> (OutputMode, Option<&'static str>) {
    // 1. Explicit flag (`--json` is sugar for `--format json`).
    let explicit = if g.json {
        Some(FormatArg::Json)
    } else {
        g.format
    };
    if let Some(fmt) = explicit {
        return match fmt {
            FormatArg::Json => (OutputMode::Json, None),
            FormatArg::Jsonl => (OutputMode::Jsonl, None),
            FormatArg::Yaml => (OutputMode::Yaml, None),
            FormatArg::Csv => (OutputMode::Csv, None),
            FormatArg::Explore => resolve_explore(),
        };
    }

    // 2. Agent / CI environment.
    if is_agent_env() || is_ci() {
        return (OutputMode::Json, None);
    }

    // 3 / 4. TTY detection.
    let mode = if std::io::stdout().is_terminal() {
        if should_use_color(g) {
            OutputMode::HumanWithColor
        } else {
            OutputMode::HumanNoColor
        }
    } else {
        OutputMode::Json
    };
    (mode, None)
}

/// Resolve `--format explore`. The interactive TUI runs only on a real
/// terminal (both stdout and stdin must be TTYs); for agents, CI, dumb
/// terminals, or pipes it falls back to JSON with the reason — never trapping
/// an agent in a render loop (Standard §5).
fn resolve_explore() -> (OutputMode, Option<&'static str>) {
    let reason = if is_agent_env() {
        Some("agent environment (AI_AGENT/AGENT) is set")
    } else if is_ci() {
        Some("running under CI")
    } else if is_dumb_term() {
        Some("TERM=dumb")
    } else if !std::io::stdout().is_terminal() {
        Some("stdout is not a TTY")
    } else if !std::io::stdin().is_terminal() {
        Some("stdin is not a TTY")
    } else {
        None
    };
    match reason {
        Some(r) => (OutputMode::Json, Some(r)),
        None => (OutputMode::Explore, None),
    }
}

/// Whether the process is being driven by an AI agent (forces machine output).
pub(crate) fn is_agent_env() -> bool {
    ["AI_AGENT", "AGENT"]
        .iter()
        .any(|var| std::env::var(var).is_ok_and(|v| !v.is_empty() && v != "0" && v != "false"))
}

/// Whether running under continuous integration.
fn is_ci() -> bool {
    std::env::var("CI").is_ok_and(|v| v == "true" || v == "1")
}

/// Whether `TERM=dumb` (no color, no cursor movement).
fn is_dumb_term() -> bool {
    std::env::var("TERM").as_deref() == Ok("dumb")
}

/// Early machine-mode probe used before argument parsing succeeds, so clap usage
/// errors can still be reported as structured JSON for agents and pipelines.
pub(crate) fn is_machine_early() -> bool {
    is_agent_env() || is_ci() || !std::io::stdout().is_terminal()
}

/// The color-precedence chain (CLI Standard §6). First match wins.
fn should_use_color(g: &GlobalArgs) -> bool {
    // 1/2. Explicit flags.
    if g.no_color {
        return false;
    }
    match g.color {
        Some(ColorArg::Never) => return false,
        Some(ColorArg::Always) => return true,
        Some(ColorArg::Auto) | None => {}
    }
    // 3–7. Environment + TTY, as a pure decision for testability.
    color_env_decision(
        std::env::var_os("FORCE_COLOR").is_some_and(|v| !v.is_empty()),
        std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
        std::env::var("CLICOLOR").as_deref() == Ok("0"),
        is_dumb_term(),
        std::io::stdout().is_terminal(),
    )
}

/// Steps 3–7 of the §6 color-precedence chain, decoupled from the live
/// environment. First match decides; `FORCE_COLOR` is checked before
/// `NO_COLOR` so it overrides it (per force-color.org).
fn color_env_decision(
    force_color: bool,
    no_color: bool,
    clicolor_zero: bool,
    dumb_term: bool,
    tty: bool,
) -> bool {
    if force_color {
        return true;
    }
    if no_color {
        return false;
    }
    if clicolor_zero || dumb_term {
        return false;
    }
    tty
}

#[cfg(test)]
mod tests {
    use super::color_env_decision;

    #[test]
    fn force_color_overrides_no_color() {
        assert!(color_env_decision(true, true, false, false, false));
    }

    #[test]
    fn no_color_disables_on_a_tty() {
        assert!(!color_env_decision(false, true, false, false, true));
    }

    #[test]
    fn clicolor_zero_and_dumb_term_disable() {
        assert!(!color_env_decision(false, false, true, false, true));
        assert!(!color_env_decision(false, false, false, true, true));
    }

    #[test]
    fn tty_decides_when_no_env_signal() {
        assert!(color_env_decision(false, false, false, false, true));
        assert!(!color_env_decision(false, false, false, false, false));
    }
}
