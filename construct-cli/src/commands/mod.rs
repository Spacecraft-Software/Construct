// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Command handlers and the dispatcher that routes a parsed [`Cli`] to them.
//!
//! Each handler returns `Result<CommandOutput, AppError>` (data commands) or
//! prints a self-describing document and returns `Ok(None)` (`schema`,
//! `describe`). Handlers never call `std::process::exit`; `main` owns the exit
//! code.

pub(crate) mod agent;
pub(crate) mod describe;
pub(crate) mod schema;
pub(crate) mod ship;
pub(crate) mod skill;
pub(crate) mod sync;

use clap::CommandFactory as _;

use crate::cli::{AgentCommand, Cli, Command, SkillCommand};
use crate::context::Context;
use crate::output::error::AppError;
use crate::output::CommandOutput;

/// Route the parsed command to its handler.
///
/// `Ok(Some(output))` is rendered by the caller; `Ok(None)` means the handler
/// already emitted its own output (help, schema, describe).
pub(crate) fn dispatch(cli: &Cli, ctx: &Context) -> Result<Option<CommandOutput>, AppError> {
    match &cli.command {
        Some(Command::Skill { verb }) => match verb {
            SkillCommand::Add(args) => skill::add(ctx, args).map(Some),
            SkillCommand::List(args) => skill::list(ctx, args).map(Some),
            SkillCommand::Remove(args) => skill::remove(ctx, args).map(Some),
            SkillCommand::Update(args) => skill::update(ctx, args).map(Some),
            SkillCommand::Sync(args) => sync::run(ctx, args).map(Some),
            SkillCommand::Ship(args) => ship::run(ctx, args).map(Some),
        },
        Some(Command::Agent { verb }) => match verb {
            AgentCommand::List => Ok(Some(agent::list(ctx))),
        },
        Some(Command::Schema { noun, verb }) => {
            schema::run(ctx, noun.as_deref(), verb.as_deref())?;
            Ok(None)
        }
        Some(Command::Describe { noun, verb }) => {
            describe::run(ctx, noun.as_deref(), verb.as_deref())?;
            Ok(None)
        }
        None => {
            no_command(ctx);
            Ok(None)
        }
    }
}

/// Behavior when no sub-command is given. Agents and pipelines get the
/// structured capability manifest; an interactive user gets help text. (A later
/// phase replaces the interactive branch with the explore TUI.)
fn no_command(ctx: &Context) {
    if ctx.is_machine() {
        let _ = describe::run(ctx, None, None);
    } else {
        let _ = Cli::command().print_long_help();
    }
}
