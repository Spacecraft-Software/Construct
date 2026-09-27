// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The per-invocation [`Context`]: the resolved output mode, color choice, and
//! the global-flag state every command handler needs. Built once in `main`
//! from the parsed [`Cli`] and passed by reference to each handler so behavior
//! stays consistent across the whole surface.

use std::io::IsTerminal as _;

use crate::cli::Cli;
use crate::output::diagnostic::Severity;
use crate::output::mode::{self, OutputMode};
use crate::output::progress;

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
    /// Standard §18.1 accessible mode, resolved once (flag > env > off).
    pub(crate) accessible: bool,
    /// Which source decided [`Self::accessible`]; reported under `--verbose`.
    pub(crate) accessible_source: A11ySource,
    /// How long-operation progress is drawn on stderr (never on stdout).
    pub(crate) progress: progress::Style,
}

/// Where the accessible-mode decision came from (Standard §18.1).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum A11ySource {
    /// `--accessible` / `--no-accessible`.
    Flag,
    /// `SPACECRAFT_A11Y`.
    Env,
    /// Nothing set: standard rendering, unchanged.
    Default,
}

impl A11ySource {
    /// Lowercase label for diagnostics.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::Env => "env",
            Self::Default => "default",
        }
    }
}

impl Context {
    /// Build the context from parsed CLI arguments, applying the output-mode
    /// detection cascade and color precedence chain.
    pub(crate) fn from_cli(cli: &Cli) -> Self {
        let g = &cli.global;
        let (mode, tui_fallback) = mode::resolve(g);
        let flag = if g.accessible {
            Some(true)
        } else if g.no_accessible {
            Some(false)
        } else {
            None
        };
        let (accessible, accessible_source) =
            resolve_accessible(flag, std::env::var("SPACECRAFT_A11Y").ok().as_deref());
        let gate = progress::enabled(
            matches!(mode, OutputMode::HumanWithColor | OutputMode::HumanNoColor),
            std::io::stderr().is_terminal(),
            g.quiet,
            std::env::var("TERM").as_deref() == Ok("dumb"),
        );
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
            accessible,
            accessible_source,
            progress: progress::style(gate, accessible, mode == OutputMode::HumanWithColor),
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

/// Resolve Standard §18.1 accessible mode. Precedence, first match wins:
/// `--accessible` / `--no-accessible` → `SPACECRAFT_A11Y` (`1`/`true`/`yes`/`on`
/// enable; `0`/`false`/`no`/`off`/empty disable; anything else is ignored) →
/// off. An explicit off at a higher level always wins. There is no config
/// layer and no auto-detect hint yet: unset everywhere means the default
/// presentation, unchanged.
fn resolve_accessible(flag: Option<bool>, env: Option<&str>) -> (bool, A11ySource) {
    if let Some(on) = flag {
        return (on, A11ySource::Flag);
    }
    let parsed = env.and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "" | "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    });
    match parsed {
        Some(on) => (on, A11ySource::Env),
        None => (false, A11ySource::Default),
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
    fn accessible_flag_beats_env() {
        assert_eq!(
            resolve_accessible(Some(false), Some("1")),
            (false, A11ySource::Flag)
        );
        assert_eq!(
            resolve_accessible(Some(true), Some("0")),
            (true, A11ySource::Flag)
        );
    }

    #[test]
    fn accessible_env_values() {
        assert_eq!(resolve_accessible(None, Some("1")), (true, A11ySource::Env));
        assert_eq!(
            resolve_accessible(None, Some("TRUE")),
            (true, A11ySource::Env)
        );
        assert_eq!(
            resolve_accessible(None, Some("0")),
            (false, A11ySource::Env)
        );
        assert_eq!(resolve_accessible(None, Some("")), (false, A11ySource::Env));
        assert_eq!(
            resolve_accessible(None, Some("maybe")),
            (false, A11ySource::Default)
        );
    }

    #[test]
    fn accessible_defaults_off() {
        assert_eq!(resolve_accessible(None, None), (false, A11ySource::Default));
    }

    #[test]
    fn severity_ordering_backs_the_floor_comparison() {
        assert!(Severity::Info < Severity::Ok);
        assert!(Severity::Ok < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
    }
}
