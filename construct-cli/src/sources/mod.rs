// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Skill *sources*: resolving a source spec (a local path, a git URL, or an
//! `owner/repo` shorthand) to a directory on disk — cloning remotes into an XDG
//! cache — and discovering the skills inside, scanning the common container
//! directories (`skills/`, `skills/.curated/`) the way `vercel-labs/skills`
//! does.

pub(crate) mod skillmd;

use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use crate::context::Context;
use crate::install::plan::{looks_remote, NON_SKILL_DIRS};
use crate::output::error::{AppError, ErrorCode};

/// Directories under a source root that may hold skills (`""` = the root).
const CONTAINER_DIRS: &[&str] = &["", "skills", "skills/.curated", ".curated"];

/// A skill found in a source, with its directory and (optional) description.
#[derive(Debug, Clone)]
pub(crate) struct DiscoveredSkill {
    pub(crate) name: String,
    pub(crate) dir: PathBuf,
    pub(crate) description: Option<String>,
}

/// Resolve a source spec to a directory on disk. Local paths are returned
/// canonicalized; git URLs / `owner/repo` shorthands are cloned (shallow) into
/// the XDG cache and reused on later calls unless `refresh` is set.
pub(crate) fn resolve_source(
    ctx: &Context,
    spec: &str,
    refresh: bool,
) -> Result<PathBuf, AppError> {
    let local = Path::new(spec);
    if local.exists() {
        return local.canonicalize().map_err(|e| {
            AppError::not_found(
                ctx,
                format!("cannot resolve source '{spec}': {e}"),
                "construct skill add /spacecraft-software/construct",
            )
        });
    }
    if looks_remote(spec) {
        let url = normalize_url(spec);
        let dest = cache_dir(ctx)?.join(sanitize(spec));
        clone_or_reuse(ctx, &url, &dest, refresh)?;
        return Ok(dest);
    }
    Err(AppError::not_found(
        ctx,
        format!("source '{spec}' does not exist"),
        "construct skill add /spacecraft-software/construct   # or owner/repo, or a git URL",
    ))
}

/// Discover the skills inside a source root (a dir is a skill if it contains
/// `SKILL.md`), scanning the container directories. First match per name wins.
pub(crate) fn discover(root: &Path) -> Vec<DiscoveredSkill> {
    let mut out: Vec<DiscoveredSkill> = Vec::new();
    for container in CONTAINER_DIRS {
        let base = if container.is_empty() {
            root.to_path_buf()
        } else {
            root.join(container)
        };
        if !base.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || NON_SKILL_DIRS.contains(&name.as_str()) {
                continue;
            }
            if out.iter().any(|s| s.name == name) {
                continue;
            }
            let dir = base.join(&name);
            if dir.join("SKILL.md").is_file() {
                let (_, description) = skillmd::frontmatter(&dir.join("SKILL.md"));
                out.push(DiscoveredSkill {
                    name,
                    dir,
                    description,
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Select skills from a source: every discovered skill, or just the requested
/// names (validated). Errors if the source has no skills or a name is unknown.
pub(crate) fn select_skills(
    ctx: &Context,
    root: &Path,
    requested: &[String],
) -> Result<Vec<DiscoveredSkill>, AppError> {
    let found = discover(root);
    if found.is_empty() {
        return Err(AppError::not_found(
            ctx,
            format!("no skills found in '{}'", root.display()),
            "construct skill find <source>",
        ));
    }
    if requested.is_empty() {
        return Ok(found);
    }
    let mut chosen = Vec::with_capacity(requested.len());
    for want in requested {
        match found.iter().find(|s| &s.name == want) {
            Some(skill) => chosen.push(skill.clone()),
            None => {
                return Err(AppError::not_found(
                    ctx,
                    format!("skill '{want}' not found in source"),
                    "construct skill find <source>",
                ))
            }
        }
    }
    Ok(chosen)
}

// ── source resolution helpers ─────────────────────────────────────────────-

/// Turn a remote spec into a clone URL (`owner/repo` → GitHub HTTPS).
fn normalize_url(spec: &str) -> String {
    if spec.contains("://") || spec.starts_with("git@") {
        spec.to_owned()
    } else {
        format!("https://github.com/{spec}")
    }
}

/// A filesystem-safe cache subdirectory name for a source spec.
fn sanitize(spec: &str) -> String {
    let mut s: String = spec
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s.trim_matches('-').to_owned()
}

/// The `construct` source cache directory (`$XDG_CACHE_HOME/construct/sources`).
fn cache_dir(ctx: &Context) -> Result<PathBuf, AppError> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .ok_or_else(|| {
            AppError::new(
                ctx,
                ErrorCode::InternalError,
                1,
                "cannot determine a cache directory ($XDG_CACHE_HOME / $HOME unset)",
                "set HOME or XDG_CACHE_HOME, then retry",
            )
        })?;
    let dir = base.join("construct/sources");
    std::fs::create_dir_all(&dir).map_err(|e| {
        AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("cannot create cache dir '{}': {e}", dir.display()),
            "check permissions on your cache directory",
        )
    })?;
    Ok(dir)
}

/// Clone a remote into `dest`, or reuse an existing checkout (refreshing it with
/// a fast-forward pull when `refresh` is set).
fn clone_or_reuse(ctx: &Context, url: &str, dest: &Path, refresh: bool) -> Result<(), AppError> {
    if dest.exists() {
        if refresh {
            let _ = Proc::new("git")
                .arg("-C")
                .arg(dest)
                .args(["pull", "--ff-only", "--depth", "1"])
                .output();
        }
        return Ok(());
    }
    let result = Proc::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(dest)
        .output();
    let output = match result {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::dependency_missing(
                ctx,
                "`git` was not found on PATH",
                "git --version   # install git, then retry",
            ));
        }
        Err(e) => {
            return Err(AppError::general(
                ctx,
                ErrorCode::NetworkError,
                format!("failed to launch git clone: {e}"),
                "git --version   # verify git, then retry",
            ));
        }
    };
    if !output.status.success() {
        let tail = String::from_utf8_lossy(&output.stderr);
        let tail = tail.lines().last().unwrap_or("clone failed");
        return Err(AppError::general(
            ctx,
            ErrorCode::NetworkError,
            format!("git clone {url} failed: {tail}"),
            format!("git clone --depth 1 {url}   # check the URL and your network"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_url, sanitize};

    #[test]
    fn normalize_owner_repo_to_github() {
        assert_eq!(
            normalize_url("vercel-labs/skills"),
            "https://github.com/vercel-labs/skills"
        );
        assert_eq!(
            normalize_url("https://example.com/x.git"),
            "https://example.com/x.git"
        );
        assert_eq!(
            normalize_url("git@github.com:o/r.git"),
            "git@github.com:o/r.git"
        );
    }

    #[test]
    fn sanitize_makes_safe_dir_names() {
        assert_eq!(sanitize("vercel-labs/skills"), "vercel-labs-skills");
        assert_eq!(
            sanitize("https://github.com/o/r.git"),
            "https-github.com-o-r.git"
        );
    }
}
