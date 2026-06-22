// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct` — the Spacecraft Software Construct skills package manager.
//!
//! A dual-mode (human + agent-native) command-line tool for installing,
//! discovering, syncing, and shipping agent *skills* across the ~70 AI coding
//! agents Construct knows about. It is the first executable artifact in the
//! Construct catalogue repository and conforms to the Spacecraft Software
//! Dual-Mode Self-Documenting CLI Standard (SFRS v1.0.0).
//!
//! The binary is intentionally thin: argument parsing lives in [`cli`], the
//! output contract (envelope, structured errors, mode detection) lives in
//! [`output`], and each sub-command's behavior lives under [`commands`]. Every
//! data-returning command speaks the `{ metadata, data }` envelope in machine
//! mode and a colored human rendering on a TTY.

// `AppError` deliberately carries the full structured-error contract (message,
// hint, timestamp, command, extensions). Boxing every `Result` to shave bytes
// off the cold error path is churn with no benefit for a short-lived CLI.
#![allow(
    clippy::result_large_err,
    reason = "AppError carries the full structured-error contract; boxing the cold error path is needless churn"
)]

mod cli;
mod commands;
mod context;
mod install;
mod manifest;
mod output;
mod registry;
mod time;

use std::panic::AssertUnwindSafe;

use clap::Parser as _;

use crate::cli::Cli;
use crate::context::Context;
use crate::output::error::AppError;

fn main() {
    // UTF-8 console + ANSI enablement on Windows; a no-op elsewhere.
    platform::init_console();
    let code = real_main();
    std::process::exit(code);
}

/// Parse, dispatch, and translate the result into a process exit code.
///
/// All panics in command handlers are caught here and reported as a structured
/// `INTERNAL_ERROR` so an agent never sees a bare Rust panic on a broken pipe.
fn real_main() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => return cli::handle_parse_error(&err),
    };

    let ctx = Context::from_cli(&cli);

    let dispatched = std::panic::catch_unwind(AssertUnwindSafe(|| commands::dispatch(&cli, &ctx)));

    match dispatched {
        Ok(Ok(Some(output))) => {
            output::render::emit(&ctx, &output);
            0
        }
        Ok(Ok(None)) => 0,
        Ok(Err(err)) => output::error::report(&err, ctx.mode),
        Err(_) => {
            let err = AppError::internal(
                &ctx,
                "an unexpected internal error occurred",
                "re-run with --verbose and report the failure at the project URL",
            );
            output::error::report(&err, ctx.mode)
        }
    }
}

/// Platform-specific startup shims.
mod platform {
    /// Startup console setup.
    ///
    /// A no-op on the platforms Construct targets (Linux and macOS, see the
    /// flake's `systems` list), where stdout is UTF-8 with ANSI support by
    /// default. If Windows support is added, set the console output code page
    /// to 65001 (UTF-8) and enable `ENABLE_VIRTUAL_TERMINAL_PROCESSING` here.
    pub(crate) fn init_console() {}
}
