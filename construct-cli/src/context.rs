// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The per-invocation [`Context`]: the resolved output mode, color choice, and
//! the global-flag state every command handler needs. Built once in `main`
//! from the parsed [`Cli`] and passed by reference to each handler so behavior
//! stays consistent across the whole surface.

use crate::cli::Cli;
use crate::output::diagnostic::Severity;
use crate::output::mode::{self, OutputMode};

/// Resolved runtime settings for a single invocation.
#[allow(
    dead_code,
    reason = "global-flag state; several fields (color/quiet/print0/yes/absolute_time) are consumed by later phases"
)]
#[derive(Debug, Clone)]
pub(crate) struct Context {
    /// Full invocation with the executable normalized to `construct`
    /// (the `metadata.command` / `error.command` value).
    pub(crate) command: String,
    /// The resolved output mode after the §5 detection cascade.
    pub(crate) mode: OutputMode,
    /// Whether ANSI color is enabled (only ever true in human mode).
    pub(crate) color: bool,
    /// `--dry-run`: plan only, no side effects.
    pub(crate) dry_run: bool,
    /// `--quiet`: suppress non-error diagnostics.
    pub(crate) quiet: bool,
    /// `--verbose` count.
    pub(crate) verbose: u8,
    /// `--fields`: optional output projection.
    pub(crate) fields: Option<Vec<String>>,
    /// `--print0`: NUL-delimit list output.
    pub(crate) print0: bool,
    /// `--yes` / `--force`: assume yes for confirmations.
    pub(crate) yes: bool,
    /// `--absolute-time`: render absolute timestamps in human mode.
    pub(crate) absolute_time: bool,
    /// The minimum severity emitted to stderr (diagnostics.md §4), resolved
    /// once per invocation from `--quiet` / `--verbose` / the agent env.
    pub(crate) severity_floor: Severity,
    /// Why `--format explore` fell back to JSON, when it did. Emitted as a
    /// `TUI_FALLBACK` warn diagnostic by `main` once the context exists.
    pub(crate) tui_fallback: Option<&'static str>,
}

impl Context {
    /// Build the context from parsed CLI arguments, applying the output-mode
    /// detection cascade and color precedence chain.
    pub(crate) fn from_cli(cli: &Cli) -> Self {
        let g = &cli.global;
        let (mode, tui_fallback) = mode::resolve(g);
        Self {
            command: invocation_string(),
            mode,
            color: mode == OutputMode::HumanWithColor,
            dry_run: g.dry_run,
            quiet: g.quiet,
            verbose: g.verbose,
            fields: g.fields.clone(),
            print0: g.print0,
            yes: g.yes,
            absolute_time: g.absolute_time,
            severity_floor: resolve_floor(g.quiet, g.verbose, mode::is_agent_env()),
            tui_fallback,
        }
    }

    /// Whether a diagnostic of `severity` clears the floor and is emitted.
    /// Errors always do — `AppError` never consults the floor.
    pub(crate) fn allows(&self, severity: Severity) -> bool {
        severity >= self.severity_floor
    }
}

/// Resolve the severity floor (diagnostics.md §4). Explicit flags beat the
/// environment: `--quiet` → errors only; `--verbose` → everything; a detected
/// agent env → failures and degradations (`warn`+); default → `ok`+.
fn resolve_floor(quiet: bool, verbose: u8, agent_env: bool) -> Severity {
    if quiet {
        Severity::Error
    } else if verbose > 0 {
        Severity::Info
    } else if agent_env {
        Severity::Warn
    } else {
        Severity::Ok
    }
}

/// The full command line with `argv[0]` normalized to the canonical binary name
/// so the recorded command is stable regardless of how the binary was invoked.
fn invocation_string() -> String {
    let mut args: Vec<String> = std::env::args().collect();
    if let Some(first) = args.first_mut() {
        "construct".clone_into(first);
    }
    args.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_table_matches_diagnostics_spec() {
        // --quiet → errors only; beats the agent env.
        assert_eq!(resolve_floor(true, 0, true), Severity::Error);
        // --verbose → everything; beats the agent env.
        assert_eq!(resolve_floor(false, 1, true), Severity::Info);
        // agent env → failures and degradations.
        assert_eq!(resolve_floor(false, 0, true), Severity::Warn);
        // default → ok and up.
        assert_eq!(resolve_floor(false, 0, false), Severity::Ok);
    }

    #[test]
    fn severity_ordering_backs_the_floor_comparison() {
        assert!(Severity::Info < Severity::Ok);
        assert!(Severity::Ok < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
    }
}
