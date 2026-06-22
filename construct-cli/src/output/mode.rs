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

/// Resolve the output mode from the global flags and environment.
pub(crate) fn resolve(g: &GlobalArgs) -> OutputMode {
    // 1. Explicit flag (`--json` is sugar for `--format json`).
    let explicit = if g.json {
        Some(FormatArg::Json)
    } else {
        g.format
    };
    if let Some(fmt) = explicit {
        return match fmt {
            FormatArg::Json => OutputMode::Json,
            FormatArg::Jsonl => OutputMode::Jsonl,
            FormatArg::Yaml => OutputMode::Yaml,
            FormatArg::Csv => OutputMode::Csv,
            FormatArg::Explore => resolve_explore(),
        };
    }

    // 2. Agent / CI environment.
    if is_agent_env() || is_ci() {
        return OutputMode::Json;
    }

    // 3 / 4. TTY detection.
    if std::io::stdout().is_terminal() {
        if should_use_color(g) {
            OutputMode::HumanWithColor
        } else {
            OutputMode::HumanNoColor
        }
    } else {
        OutputMode::Json
    }
}

/// Resolve `--format explore`. The interactive TUI runs only on a real
/// terminal (both stdout and stdin must be TTYs); for agents, CI, dumb
/// terminals, or pipes it falls back to JSON and warns — never trapping an
/// agent in a render loop (Standard §5).
fn resolve_explore() -> OutputMode {
    if is_agent_env()
        || is_ci()
        || is_dumb_term()
        || !std::io::stdout().is_terminal()
        || !std::io::stdin().is_terminal()
    {
        warn_tui_fallback("no interactive terminal available", "json");
        OutputMode::Json
    } else {
        OutputMode::Explore
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
    // 3. NO_COLOR (set + non-empty) disables.
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return false;
    }
    // 4. FORCE_COLOR (set + non-empty) enables, overriding NO_COLOR.
    if std::env::var_os("FORCE_COLOR").is_some_and(|v| !v.is_empty()) {
        return true;
    }
    // 5/6. CLICOLOR=0 and TERM=dumb disable.
    if std::env::var("CLICOLOR").as_deref() == Ok("0") || is_dumb_term() {
        return false;
    }
    // 7. TTY.
    std::io::stdout().is_terminal()
}

/// Emit a single-line JSON warning to stderr when the TUI cannot run.
fn warn_tui_fallback(reason: &str, fell_back_to: &str) {
    let warning = serde_json::json!({
        "warning": {
            "code": "TUI_FALLBACK",
            "message": "interactive explore mode unavailable; falling back",
            "reason": reason,
            "fell_back_to": fell_back_to,
            "timestamp": crate::time::now_iso8601(),
        }
    });
    eprintln!("{warning}");
}
