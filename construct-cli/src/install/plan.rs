// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Install planning: option types, the project/global scope, and resolution of
//! the source skills, target agents, and base install directory. The actual
//! filesystem mutations live in the parent [`crate::install`] module.

use std::path::{Path, PathBuf};

use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::registry::{self, detect, Agent};

/// Default catalogue source — the local Construct clone.
pub(crate) const DEFAULT_SOURCE: &str = "/spacecraft-software/construct";

/// Top-level directories in a catalogue source that are never skills.
pub(crate) const NON_SKILL_DIRS: &[&str] = &[
    "grok-skills",
    "Excluded",
    ".git",
    ".github",
    ".claude",
    "construct-cli",
    "LICENSES",
];

/// Where skills are installed relative to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Under the current project root (the `.git` walk-up), using `project_path`.
    Project,
    /// Under `$HOME`, using the agent's `global_path`.
    Global,
}

impl Scope {
    /// Stable lowercase label for output.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Global => "global",
        }
    }
}

/// Resolved options for `skill add` / `skill update`.
#[derive(Debug)]
pub(crate) struct AddOptions {
    /// Raw source spec (local path, git URL, or `owner/repo`).
    pub(crate) source: String,
    pub(crate) skills: Vec<String>,
    pub(crate) agents: Vec<String>,
    pub(crate) all_agents: bool,
    pub(crate) scope: Scope,
    pub(crate) copy: bool,
    pub(crate) force: bool,
    /// Re-clone a cached remote source before installing.
    pub(crate) refresh: bool,
}

/// Heuristic: does this source string look like a remote git source (which is
/// Phase 3, not yet supported) rather than a local path?
pub(crate) fn looks_remote(source: &str) -> bool {
    source.contains("://")
        || source.starts_with("git@")
        || (source.matches('/').count() == 1
            && !source.starts_with('.')
            && !Path::new(source).exists())
}

/// Resolve which agents to target: explicit `--agents` (validated), else
/// `--all` (every agent), else the agents detected as installed.
pub(crate) fn resolve_agents(
    ctx: &Context,
    opts: &AddOptions,
    agents: &[Agent],
) -> Result<Vec<Agent>, AppError> {
    if !opts.agents.is_empty() {
        return select_by_id(ctx, &opts.agents, agents);
    }
    if opts.all_agents {
        return Ok(agents.to_vec());
    }
    let detected: Vec<Agent> = agents
        .iter()
        .filter(|a| detect::detect_installed(a))
        .cloned()
        .collect();
    if detected.is_empty() {
        return Err(AppError::new(
            ctx,
            ErrorCode::MissingArgument,
            2,
            "no agents specified and none detected as installed",
            "construct skill add --all   # or --agents <id> (see: construct agent list)",
        ));
    }
    Ok(detected)
}

/// Map a list of agent ids to registry entries, erroring on the first unknown id.
pub(crate) fn select_by_id(
    ctx: &Context,
    ids: &[String],
    agents: &[Agent],
) -> Result<Vec<Agent>, AppError> {
    let mut chosen = Vec::with_capacity(ids.len());
    for id in ids {
        match registry::find(agents, id) {
            Some(agent) => chosen.push(agent),
            None => {
                return Err(AppError::not_found(
                    ctx,
                    format!("unknown agent '{id}'"),
                    "construct agent list",
                ))
            }
        }
    }
    Ok(chosen)
}

/// The nearest ancestor of the current directory containing a `.git`, or the
/// current directory if none is found.
pub(crate) fn project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir: &Path = &cwd;
    loop {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return cwd,
        }
    }
}

/// The base install directory for an agent under the chosen scope (absolute).
/// `None` for a global install when the agent declares no global skills dir.
pub(crate) fn base_dir(agent: &Agent, scope: Scope, project_root: &Path) -> Option<PathBuf> {
    match scope {
        Scope::Project => Some(project_root.join(&agent.project_path)),
        Scope::Global => detect::global_base(agent),
    }
}
