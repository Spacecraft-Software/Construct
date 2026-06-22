// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill add | list | remove | update` — the imperative installer's
//! command surface. These build options from the parsed args and delegate to
//! [`crate::install`], then wrap the structured report into a [`CommandOutput`].

use std::path::PathBuf;

use serde_json::Value;

use crate::cli::{AddArgs, RemoveArgs, ScopeQueryArgs};
use crate::context::Context;
use crate::install::{self, plan::AddOptions, plan::Scope, InstallReport, RemoveOptions};
use crate::output::error::AppError;
use crate::output::{CommandOutput, HumanRender};

/// Map the `--global` flag to a [`Scope`].
fn scope_of(global: bool) -> Scope {
    if global {
        Scope::Global
    } else {
        Scope::Project
    }
}

/// Build [`AddOptions`] from parsed args; `force` is set true for `update`.
fn add_options(args: &AddArgs, force: bool) -> AddOptions {
    let source = args
        .source
        .clone()
        .unwrap_or_else(|| install::plan::DEFAULT_SOURCE.to_owned());
    AddOptions {
        source: PathBuf::from(source),
        skills: args.skills.clone(),
        agents: args.agents.clone(),
        all_agents: args.all,
        scope: scope_of(args.global),
        copy: args.copy,
        force,
    }
}

/// `construct skill add` — install skills (idempotent; `--force`/`-y` overwrites).
pub(crate) fn add(ctx: &Context, args: &AddArgs) -> Result<CommandOutput, AppError> {
    let report = install::add(ctx, &add_options(args, ctx.yes))?;
    Ok(report_output(&report))
}

/// `construct skill update` — refresh installed skills from the source.
pub(crate) fn update(ctx: &Context, args: &AddArgs) -> Result<CommandOutput, AppError> {
    let report = install::add(ctx, &add_options(args, true))?;
    Ok(report_output(&report))
}

/// `construct skill list` — show installed skills per agent.
pub(crate) fn list(ctx: &Context, args: &ScopeQueryArgs) -> Result<CommandOutput, AppError> {
    let report = install::list_installed(ctx, scope_of(args.global), &args.agents)?;
    Ok(report_output(&report))
}

/// `construct skill remove` — remove installed skills.
pub(crate) fn remove(ctx: &Context, args: &RemoveArgs) -> Result<CommandOutput, AppError> {
    let opts = RemoveOptions {
        skills: args.skills.clone(),
        agents: args.agents.clone(),
        all_agents: args.all,
        scope: scope_of(args.global),
    };
    let report = install::remove(ctx, &opts)?;
    Ok(report_output(&report))
}

/// Wrap an [`InstallReport`] into machine `data` plus a human table.
fn report_output(report: &InstallReport) -> CommandOutput {
    let rows = report
        .items
        .iter()
        .map(|item| {
            vec![
                item.agent.clone(),
                item.skill.clone(),
                item.action.to_owned(),
                item.target.clone(),
            ]
        })
        .collect();
    let human = HumanRender::Table {
        headers: vec![
            "AGENT".to_owned(),
            "SKILL".to_owned(),
            "ACTION".to_owned(),
            "TARGET".to_owned(),
        ],
        rows,
    };
    let data = serde_json::to_value(report).unwrap_or(Value::Null);
    CommandOutput::new(data, human)
}
