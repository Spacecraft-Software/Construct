// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill add | list | delete | update | find | use | init` — the
//! skill command surface. These build options from the parsed args and delegate
//! to [`crate::install`] / [`crate::sources`], then wrap the result.

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::bundle::frontmatter::valid_name;
use crate::cli::{AddArgs, DeleteArgs, FindArgs, InitArgs, ScopeQueryArgs, UseArgs};
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

/// `construct skill delete` — delete installed skills.
pub(crate) fn delete(ctx: &Context, args: &DeleteArgs) -> Result<CommandOutput, AppError> {
    // Each name is joined onto an agent's skills directory and the result is
    // deleted, so `../x` or `/etc` must never get that far. Installed names
    // from third-party sources need not be strict ids, so the guard is "one
    // plain path component" rather than `valid_name`.
    if let Some(bad) = args.skills.iter().find(|n| !is_single_component(n)) {
        return Err(AppError::invalid_argument(
            ctx,
            format!("invalid skill name '{bad}'; a name is one directory entry, not a path"),
            "construct skill list --json",
        ));
    }
    let opts = RemoveOptions {
        skills: args.skills.clone(),
        agents: args.agents.clone(),
        all_agents: args.all,
        scope: scope_of(args.global),
    };
    let report = install::remove(ctx, &opts, |count| confirm_delete(ctx, count))?;
    Ok(report_output(&report))
}

/// The destructive-operation gate (validation-safety §6): `--yes` / `--force`
/// skips it; a human terminal is asked, defaulting to no; anything else —
/// an agent, a pipe, CI — is refused with the exact command to re-run.
fn confirm_delete(ctx: &Context, count: usize) -> Result<(), AppError> {
    use std::io::IsTerminal as _;
    if ctx.yes {
        return Ok(());
    }
    let what = if count == 1 {
        "1 installed skill".to_owned()
    } else {
        format!("{count} installed skills")
    };
    let interactive =
        !ctx.mode.is_machine() && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    if !interactive {
        return Err(AppError::new(
            ctx,
            ErrorCode::MissingArgument,
            2,
            format!("refusing to delete {what} without --yes in a non-interactive session"),
            crate::commands::rerun_with("--yes"),
        ));
    }
    if crate::output::diagnostic::confirm(&format!("Delete {what}?")) {
        Ok(())
    } else {
        Err(AppError::general(
            ctx,
            ErrorCode::Declined,
            "deletion declined at the confirmation prompt",
            crate::commands::rerun_with("--dry-run"),
        ))
    }
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
    // NAME becomes a directory under --dir: an id, never a path.
    if !valid_name(&args.name) {
        return Err(invalid_name(
            ctx,
            &args.name,
            "construct skill init my-skill",
        ));
    }
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

/// Whether `name` is exactly one ordinary path component: no separator, not
/// `.` or `..`, not absolute, not empty.
fn is_single_component(name: &str) -> bool {
    let mut parts = std::path::Path::new(name).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(std::path::Component::Normal(_)), None)
    ) && !name.contains('/')
}

/// A name that is not an Agent Skills id (`^[a-z0-9]+(-[a-z0-9]+)*$`, at
/// most 64 characters) — the agent threat model's path-traversal guard for
/// arguments that become path components.
fn invalid_name(ctx: &Context, name: &str, hint: &str) -> AppError {
    AppError::invalid_argument(
        ctx,
        format!(
            "invalid skill name '{name}'; a skill id matches ^[a-z0-9]+(-[a-z0-9]+)*$ (max 64)"
        ),
        hint,
    )
}

// ── helpers ───────────────────────────────────────────────────────────────-

/// Wrap an [`InstallReport`] into machine `data` plus a human table.
pub(crate) fn report_output(report: &InstallReport) -> CommandOutput {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_names_are_single_components() {
        for ok in ["foo", "find-skills", "Foo_Bar", "a.b"] {
            assert!(is_single_component(ok), "{ok}");
        }
        for bad in ["", ".", "..", "../x", "a/b", "/etc", "a/", "./a"] {
            assert!(!is_single_component(bad), "{bad}");
        }
    }
}
