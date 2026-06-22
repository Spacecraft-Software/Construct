// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The per-invocation [`Context`]: the resolved output mode, color choice, and
//! the global-flag state every command handler needs. Built once in `main`
//! from the parsed [`Cli`] and passed by reference to each handler so behavior
//! stays consistent across the whole surface.

use crate::cli::Cli;
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
}

impl Context {
    /// Build the context from parsed CLI arguments, applying the output-mode
    /// detection cascade and color precedence chain.
    pub(crate) fn from_cli(cli: &Cli) -> Self {
        let g = &cli.global;
        let mode = mode::resolve(g);
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
        }
    }

    /// True when the resolved mode is one of the machine-readable formats.
    pub(crate) fn is_machine(&self) -> bool {
        self.mode.is_machine()
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
