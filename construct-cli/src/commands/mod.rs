// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Command handlers and the dispatcher that routes a parsed [`Cli`] to them.
//!
//! Each handler returns `Result<CommandOutput, AppError>` (data commands) or
//! prints a self-describing document and returns `Ok(None)` (`schema`,
//! `describe`). Handlers never call `std::process::exit`; `main` owns the exit
//! code.

pub(crate) mod agent;
pub(crate) mod build;
pub(crate) mod describe;
pub(crate) mod explore;
pub(crate) mod mcp;
pub(crate) mod pointer;
pub(crate) mod schema;
pub(crate) mod ship;
pub(crate) mod skill;
pub(crate) mod sync;
pub(crate) mod vendor;

use crate::cli::{AgentCommand, Cli, Command, SkillCommand};
use crate::context::Context;
use crate::output::error::AppError;
use crate::output::mode::OutputMode;
use crate::output::CommandOutput;

/// `arg` quoted for a POSIX shell when it holds anything beyond a safe set,
/// so a printed hint can be pasted back verbatim.
pub(crate) fn shell_quote(arg: &str) -> String {
    let safe = !arg.is_empty()
        && arg
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-+,:@=".contains(&b));
    if safe {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

/// The current invocation, shell-quoted so it can be pasted back, with
/// `flag` appended — the runnable hint for an error a single flag resolves.
pub(crate) fn rerun_with(flag: &str) -> String {
    let mut parts = vec!["construct".to_owned()];
    parts.extend(
        std::env::args_os()
            .skip(1)
            .map(|a| shell_quote(&a.to_string_lossy())),
    );
    parts.push(flag.to_owned());
    parts.join(" ")
}

/// Route the parsed command to its handler.
///
/// `Ok(Some(output))` is rendered by the caller; `Ok(None)` means the handler
/// already emitted its own output (help, schema, describe).
pub(crate) fn dispatch(cli: &Cli, ctx: &Context) -> Result<Option<CommandOutput>, AppError> {
    // The provider shapes are `schema`'s alone; anywhere else they would
    // silently mean `--format json`.
    if let Some((_, name)) = ctx.schema_provider {
        if !matches!(cli.command, Some(Command::Schema { .. })) {
            return Err(AppError::invalid_argument(
                ctx,
                format!("--format {name} applies only to `construct schema`"),
                format!("construct schema --format {name}"),
            ));
        }
    }
    // `--format explore` launches the interactive TUI regardless of sub-command.
    if ctx.mode == OutputMode::Explore {
        return explore::run(ctx);
    }
    match &cli.command {
        Some(Command::Skill { verb }) => match verb {
            SkillCommand::Add(args) => skill::add(ctx, args).map(Some),
            SkillCommand::List(args) => skill::list(ctx, args).map(Some),
            SkillCommand::Delete(args) => skill::delete(ctx, args).map(Some),
            SkillCommand::Update(args) => skill::update(ctx, args).map(Some),
            SkillCommand::Find(args) => skill::find(ctx, args).map(Some),
            SkillCommand::Use(args) => skill::use_prompt(ctx, args).map(Some),
            SkillCommand::Init(args) => skill::init(ctx, args).map(Some),
            SkillCommand::Sync(args) => sync::run(ctx, args).map(Some),
            SkillCommand::Status(args) => pointer::status(ctx, args).map(Some),
            SkillCommand::Reset(args) => pointer::reset(ctx, args).map(Some),
            SkillCommand::Ship(args) => ship::run(ctx, args).map(Some),
            SkillCommand::Build(args) => build::run(ctx, args).map(Some),
            SkillCommand::Vendor(args) => vendor::run(ctx, args).map(Some),
        },
        Some(Command::Agent { verb }) => match verb {
            AgentCommand::List => Ok(Some(agent::list(ctx))),
        },
        Some(Command::Schema { noun, verb }) => {
            schema::run(ctx, noun.as_deref(), verb.as_deref())?;
            Ok(None)
        }
        Some(Command::Mcp(args)) => {
            mcp::run(ctx, args);
            Ok(None)
        }
        Some(Command::Describe { noun, verb }) => {
            describe::run(ctx, noun.as_deref(), verb.as_deref())?;
            Ok(None)
        }
        None => {
            // A bare invocation on a real terminal opens the explore TUI;
            // agents and pipelines get the structured capability manifest.
            if matches!(
                ctx.mode,
                OutputMode::HumanWithColor | OutputMode::HumanNoColor
            ) {
                explore::run(ctx)
            } else {
                let _ = describe::run(ctx, None, None);
                Ok(None)
            }
        }
    }
}
