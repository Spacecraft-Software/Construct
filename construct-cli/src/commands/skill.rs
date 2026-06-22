// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill add | list | remove | update | find | use | init` — the
//! skill command surface. These build options from the parsed args and delegate
//! to [`crate::install`] / [`crate::sources`], then wrap the result.

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::cli::{AddArgs, FindArgs, InitArgs, RemoveArgs, ScopeQueryArgs, UseArgs};
use crate::context::Context;
use crate::install::{self, plan::AddOptions, plan::Scope, InstallReport, RemoveOptions};
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};
use crate::sources::{self, skillmd};

/// Map the `--global` flag to a [`Scope`].
fn scope_of(global: bool) -> Scope {
    if global {
        Scope::Global
    } else {
        Scope::Project
    }
}

/// Resolve the source spec, defaulting to the local Construct catalogue.
fn source_spec(explicit: Option<&String>) -> String {
    explicit
        .cloned()
        .unwrap_or_else(|| install::plan::DEFAULT_SOURCE.to_owned())
}

/// Build [`AddOptions`] from parsed args; `force` is set true for `update`.
fn add_options(args: &AddArgs, force: bool) -> AddOptions {
    AddOptions {
        source: source_spec(args.source.as_ref()),
        skills: args.skills.clone(),
        agents: args.agents.clone(),
        all_agents: args.all,
        scope: scope_of(args.global),
        copy: args.copy,
        force,
        refresh: args.refresh,
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

/// `construct skill find` — browse a source's catalogue (name + description).
pub(crate) fn find(ctx: &Context, args: &FindArgs) -> Result<CommandOutput, AppError> {
    let root = sources::resolve_source(ctx, &source_spec(args.source.as_ref()), false)?;
    let query = args.query.as_deref().map(str::to_lowercase);

    let mut records = Vec::new();
    let mut rows = Vec::new();
    for skill in sources::discover(&root) {
        let desc = skill.description.clone().unwrap_or_default();
        if let Some(q) = &query {
            if !format!("{} {desc}", skill.name).to_lowercase().contains(q) {
                continue;
            }
        }
        rows.push(vec![skill.name.clone(), truncate(&desc, 80)]);
        records.push(json!({ "name": skill.name, "description": skill.description }));
    }
    let human = HumanRender::Table {
        headers: vec!["SKILL".to_owned(), "DESCRIPTION".to_owned()],
        rows,
    };
    Ok(CommandOutput::new(Value::Array(records), human))
}

/// `construct skill use` — print selected skills' prompts without installing.
pub(crate) fn use_prompt(ctx: &Context, args: &UseArgs) -> Result<CommandOutput, AppError> {
    let root = sources::resolve_source(ctx, &source_spec(args.source.as_ref()), false)?;
    let selected = sources::select_skills(ctx, &root, &args.skills)?;

    let mut records = Vec::new();
    let mut sections = Vec::new();
    for skill in &selected {
        let content = skillmd::body(&skill.dir.join("SKILL.md"));
        sections.push(format!("# {}\n\n{content}", skill.name));
        records.push(json!({ "skill": skill.name, "content": content }));
    }
    let human = HumanRender::Message(sections.join("\n\n---\n\n"));
    Ok(CommandOutput::new(Value::Array(records), human))
}

/// `construct skill init` — scaffold a new skill directory with a `SKILL.md`.
pub(crate) fn init(ctx: &Context, args: &InitArgs) -> Result<CommandOutput, AppError> {
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    let skill_dir = dir.join(&args.name);
    let skill_md = skill_dir.join("SKILL.md");

    if skill_md.exists() {
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!("'{}' already exists", skill_md.display()),
            "construct skill init <other-name>",
        ));
    }

    let path = skill_md.display().to_string();
    if ctx.dry_run {
        let human = HumanRender::Message(format!("[dry-run] would create {path}"));
        return Ok(CommandOutput::new(
            json!({ "created": false, "path": path }),
            human,
        ));
    }

    std::fs::create_dir_all(&skill_dir).map_err(|e| io_error(ctx, &skill_md, &e))?;
    std::fs::write(&skill_md, template(&args.name)).map_err(|e| io_error(ctx, &skill_md, &e))?;
    let human = HumanRender::Message(format!("created {path}"));
    Ok(CommandOutput::new(
        json!({ "created": true, "path": path }),
        human,
    ))
}

// ── helpers ───────────────────────────────────────────────────────────────-

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

/// Truncate to `n` characters, appending an ellipsis when shortened.
fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let head: String = s.chars().take(n).collect();
        format!("{head}…")
    }
}

/// The house-style `SKILL.md` scaffold.
fn template(name: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: >\n  TODO: one paragraph (<= 1000 chars) describing what this skill does and\n  when an agent should use it.\nlicense: GPL-3.0-or-later\n---\n\n# {name}\n\nTODO: skill body.\n"
    )
}

/// Map a filesystem error to a structured `AppError`.
fn io_error(ctx: &Context, path: &std::path::Path, err: &std::io::Error) -> AppError {
    AppError::general(
        ctx,
        ErrorCode::InternalError,
        format!("filesystem error at '{}': {err}", path.display()),
        "check permissions and retry",
    )
}
