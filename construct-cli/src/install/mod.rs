// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The imperative installer: place skills into agent directories (symlink by
//! default, copy with `--copy`), list and remove them. This is the part of
//! `construct` that mirrors `vercel-labs/skills`.
//!
//! It **coexists** with the declarative Construct Home-Manager module rather
//! than replacing it. The coexistence rule (CLI Standard collision avoidance):
//!
//! 1. Project-local installs are the default and never touch HM's home paths.
//! 2. A *global* install into a directory HM owns (a symlink to
//!    `~/.agents/skills`) is refused — explicit `--agents` makes it a hard
//!    error (exit 5); broad selection (`--all` / auto-detected) skips it.

pub(crate) mod plan;

use std::io;
use std::os::unix::fs::symlink as unix_symlink;
use std::path::Path;

use serde::Serialize;

use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::registry::{self, detect, Agent, SkillFormat};
use plan::{AddOptions, Scope};

/// One (agent, skill) outcome.
#[derive(Debug, Serialize)]
pub(crate) struct ItemResult {
    pub(crate) agent: String,
    pub(crate) skill: String,
    pub(crate) scope: &'static str,
    pub(crate) target: String,
    pub(crate) action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) detail: Option<String>,
}

/// The structured result of an install/list/remove operation.
#[derive(Debug, Serialize)]
pub(crate) struct InstallReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    pub(crate) scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mode: Option<String>,
    pub(crate) items: Vec<ItemResult>,
}

/// Options for `skill remove`.
#[derive(Debug)]
pub(crate) struct RemoveOptions {
    pub(crate) skills: Vec<String>,
    pub(crate) agents: Vec<String>,
    pub(crate) all_agents: bool,
    pub(crate) scope: Scope,
}

// ── add ─────────────────────────────────────────────────────────────────────

/// Install skills from a catalogue source into the selected agents.
pub(crate) fn add(ctx: &Context, opts: &AddOptions) -> Result<InstallReport, AppError> {
    let source_str = opts.source.to_string_lossy().into_owned();
    if plan::looks_remote(&source_str) {
        return Err(AppError::new(
            ctx,
            ErrorCode::FeatureUnavailable,
            1,
            "remote git sources are not supported yet",
            "construct skill add /spacecraft-software/construct",
        ));
    }
    let source = opts.source.canonicalize().map_err(|_| {
        AppError::not_found(
            ctx,
            format!("source '{source_str}' does not exist"),
            "construct skill add /spacecraft-software/construct",
        )
    })?;

    let registry = registry::all();
    let agents = plan::resolve_agents(ctx, opts, &registry)?;
    let skills = plan::resolve_skills(ctx, &source, &opts.skills)?;
    let root = plan::project_root();

    // Pre-flight: an explicit `--agents` + `--global` into an HM-managed dir is
    // a hard refusal — the user named an agent the next rebuild would clobber.
    if opts.scope == Scope::Global && !opts.agents.is_empty() {
        let blocked: Vec<String> = agents
            .iter()
            .filter(|a| detect::global_is_hm_managed(a))
            .map(|a| a.id.clone())
            .collect();
        if !blocked.is_empty() {
            return Err(AppError::new(
                ctx,
                ErrorCode::Conflict,
                5,
                format!(
                    "global skills dir is Home-Manager-managed for: {}",
                    blocked.join(", ")
                ),
                "construct skill add --skills <name> /spacecraft-software/construct   # omit -g to install project-local",
            )
            .with_extension("hm_managed_agents", serde_json::json!(blocked)));
        }
    }

    let mut items = Vec::new();
    for agent in &agents {
        place_agent(ctx, agent, &source, &skills, opts, &root, &mut items)?;
    }

    Ok(InstallReport {
        source: Some(source.display().to_string()),
        scope: opts.scope.label(),
        mode: Some(if opts.copy { "copy" } else { "symlink" }.to_owned()),
        items,
    })
}

/// Install every selected skill into one agent, recording an item per skill.
fn place_agent(
    ctx: &Context,
    agent: &Agent,
    source: &Path,
    skills: &[String],
    opts: &AddOptions,
    root: &Path,
    items: &mut Vec<ItemResult>,
) -> Result<(), AppError> {
    let scope = opts.scope.label();

    if agent.format == SkillFormat::Flat {
        push_all(
            items,
            agent,
            skills,
            scope,
            "",
            "skipped_flat",
            "flat-format install not yet supported",
        );
        return Ok(());
    }

    let Some(base) = plan::base_dir(agent, opts.scope, root) else {
        push_all(
            items,
            agent,
            skills,
            scope,
            "",
            "skipped_no_global",
            "agent has no global skills dir",
        );
        return Ok(());
    };

    // Broad-selection global install into an HM-managed dir: skip gracefully.
    if opts.scope == Scope::Global && detect::classify(&base) == detect::TargetState::HmManaged {
        for skill in skills {
            items.push(ItemResult {
                agent: agent.id.clone(),
                skill: skill.clone(),
                scope,
                target: base.join(skill).display().to_string(),
                action: "refused_hm_managed",
                detail: Some("symlinked to ~/.agents/skills by Home Manager".to_owned()),
            });
        }
        return Ok(());
    }

    if !ctx.dry_run {
        std::fs::create_dir_all(&base).map_err(|e| io_error(ctx, &base, &e))?;
    }

    for skill in skills {
        let target = base.join(skill);
        let skill_source = source.join(skill);
        let (action, detail) = place(ctx, &skill_source, &target, opts.copy, opts.force)?;
        items.push(ItemResult {
            agent: agent.id.clone(),
            skill: skill.clone(),
            scope,
            target: target.display().to_string(),
            action,
            detail,
        });
    }
    Ok(())
}

/// Place one skill at `target` from `source`. Idempotent for symlinks; refuses
/// to clobber an existing entry without `--force`.
fn place(
    ctx: &Context,
    source: &Path,
    target: &Path,
    copy: bool,
    force: bool,
) -> Result<(&'static str, Option<String>), AppError> {
    let state = detect::classify(target);

    if !copy && state == detect::TargetState::Linked {
        if let Ok(dest) = std::fs::read_link(target) {
            if dest == source {
                return Ok(("unchanged", None));
            }
        }
    }

    let occupied = state != detect::TargetState::Free;
    if occupied && !force {
        return Ok((
            "skipped_occupied",
            Some("exists; pass --force / --yes to overwrite".to_owned()),
        ));
    }
    if ctx.dry_run {
        return Ok((
            if copy {
                "planned_copy"
            } else {
                "planned_symlink"
            },
            None,
        ));
    }
    if occupied {
        remove_path(target).map_err(|e| io_error(ctx, target, &e))?;
    }
    if copy {
        copy_dir(source, target).map_err(|e| io_error(ctx, target, &e))?;
        Ok(("copied", None))
    } else {
        unix_symlink(source, target).map_err(|e| io_error(ctx, target, &e))?;
        Ok(("symlinked", None))
    }
}

// ── list ────────────────────────────────────────────────────────────────────

/// List installed skills per agent under the chosen scope.
pub(crate) fn list_installed(
    ctx: &Context,
    scope: Scope,
    agent_filter: &[String],
) -> Result<InstallReport, AppError> {
    let registry = registry::all();
    let agents = if agent_filter.is_empty() {
        registry
    } else {
        plan::select_by_id(ctx, agent_filter, &registry)?
    };
    let root = plan::project_root();

    let mut items = Vec::new();
    for agent in &agents {
        let Some(base) = plan::base_dir(agent, scope, &root) else {
            continue;
        };
        if !base.is_dir() {
            continue;
        }
        for skill in installed_in(&base) {
            let target = base.join(&skill);
            let kind =
                if std::fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink()) {
                    "symlink"
                } else {
                    "dir"
                };
            items.push(ItemResult {
                agent: agent.id.clone(),
                skill,
                scope: scope.label(),
                target: target.display().to_string(),
                action: kind,
                detail: None,
            });
        }
    }

    Ok(InstallReport {
        source: None,
        scope: scope.label(),
        mode: Some("list".to_owned()),
        items,
    })
}

// ── remove ──────────────────────────────────────────────────────────────────

/// Remove installed skills from the selected agents.
pub(crate) fn remove(ctx: &Context, opts: &RemoveOptions) -> Result<InstallReport, AppError> {
    if opts.skills.is_empty() && opts.agents.is_empty() && !opts.all_agents {
        return Err(AppError::new(
            ctx,
            ErrorCode::MissingArgument,
            2,
            "refusing to remove everything without an explicit selection",
            "construct skill remove --skills <name>   # or --agents <id>, or --all",
        ));
    }

    let registry = registry::all();
    let agents = if opts.agents.is_empty() {
        registry
    } else {
        plan::select_by_id(ctx, &opts.agents, &registry)?
    };
    let root = plan::project_root();

    let mut items = Vec::new();
    for agent in &agents {
        let Some(base) = plan::base_dir(agent, opts.scope, &root) else {
            continue;
        };
        if !base.is_dir() {
            continue;
        }
        // Never delete from a directory Home Manager owns.
        if opts.scope == Scope::Global && detect::classify(&base) == detect::TargetState::HmManaged
        {
            continue;
        }
        let targets = if opts.skills.is_empty() {
            installed_in(&base)
        } else {
            opts.skills.clone()
        };
        for skill in targets {
            let target = base.join(&skill);
            if detect::classify(&target) == detect::TargetState::Free {
                if !opts.skills.is_empty() {
                    items.push(ItemResult {
                        agent: agent.id.clone(),
                        skill,
                        scope: opts.scope.label(),
                        target: target.display().to_string(),
                        action: "not_found",
                        detail: None,
                    });
                }
                continue;
            }
            let action = if ctx.dry_run {
                "planned_remove"
            } else {
                remove_path(&target).map_err(|e| io_error(ctx, &target, &e))?;
                "removed"
            };
            items.push(ItemResult {
                agent: agent.id.clone(),
                skill,
                scope: opts.scope.label(),
                target: target.display().to_string(),
                action,
                detail: None,
            });
        }
    }

    Ok(InstallReport {
        source: None,
        scope: opts.scope.label(),
        mode: Some("remove".to_owned()),
        items,
    })
}

// ── helpers ───────────────────────────────────────────────────────────────-

/// Push the same outcome for every skill (used for whole-agent skips).
fn push_all(
    items: &mut Vec<ItemResult>,
    agent: &Agent,
    skills: &[String],
    scope: &'static str,
    target: &str,
    action: &'static str,
    detail: &str,
) {
    for skill in skills {
        items.push(ItemResult {
            agent: agent.id.clone(),
            skill: skill.clone(),
            scope,
            target: target.to_owned(),
            action,
            detail: Some(detail.to_owned()),
        });
    }
}

/// Names of installed skills (dirs or symlinks) directly under a base dir.
fn installed_in(base: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            if entry
                .file_type()
                .is_ok_and(|t| t.is_dir() || t.is_symlink())
            {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
    }
    names.sort();
    names
}

/// Remove a path whether it is a symlink, file, or directory.
fn remove_path(path: &Path) -> io::Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

/// Recursively copy a directory tree.
fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Map a filesystem error to a structured `AppError`.
fn io_error(ctx: &Context, path: &Path, err: &io::Error) -> AppError {
    AppError::general(
        ctx,
        ErrorCode::InternalError,
        format!("filesystem error at '{}': {err}", path.display()),
        "check permissions and free space, then retry",
    )
}
