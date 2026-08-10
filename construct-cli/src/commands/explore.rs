// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct --format explore` (and a bare TTY invocation): run the [`crate::tui`]
//! browser, then execute the action it returns once the terminal is restored,
//! rendering the result as normal human output.

use crate::cli::SyncArgs;
use crate::commands::{skill, sync};
use crate::context::Context;
use crate::install::{self, plan::AddOptions, plan::Scope};
use crate::output::error::AppError;
use crate::output::mode::OutputMode;
use crate::output::{render, CommandOutput};
use crate::tui::{self, TuiAction};

/// Launch the explore TUI and carry out the chosen action.
pub(crate) fn run(ctx: &Context) -> Result<Option<CommandOutput>, AppError> {
    let action = tui::run(ctx)?;

    // The TUI has restored the terminal; render any follow-up as human output.
    let mut hctx = ctx.clone();
    hctx.mode = OutputMode::HumanWithColor;
    hctx.color = true;

    match action {
        TuiAction::Quit => Ok(None),
        TuiAction::Sync => {
            // The TUI's sync stays the plain flake update. Moving the pointer
            // is a deliberate, separately-invoked step — not what a keypress in
            // a catalogue browser should do.
            let output = sync::run(
                &hctx,
                &SyncArgs {
                    flake_dir: None,
                    build: false,
                    no_update: false,
                    pointer: None,
                },
            )?;
            render::emit(&hctx, &output);
            Ok(None)
        }
        TuiAction::Install(skills) => {
            let opts = AddOptions {
                source: install::plan::DEFAULT_SOURCE.to_owned(),
                skills,
                agents: Vec::new(),
                all_agents: false,
                scope: Scope::Project,
                copy: false,
                force: false,
                refresh: false,
            };
            let report = install::add(&hctx, &opts)?;
            render::emit(&hctx, &skill::report_output(&report));
            Ok(None)
        }
    }
}
