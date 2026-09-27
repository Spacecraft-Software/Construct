// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill vendor` — commit-ready skill trees for Claude Code cloud
//! sessions.
//!
//! Cloud sessions load skills only from a repository's committed
//! `.claude/skills/`, so this handler writes the `claude` target's tree for
//! each named skill into `<into>/.claude/skills/<name>/`. The tree comes from
//! [`bundle::vendor_plan`] — the same gates, palette vendoring, and projection
//! as `skill build --target claude` — and is written through
//! [`tree::write`] (stage, re-read, verify, swap).
//!
//! Safety rules: skill names must be valid Agent Skills ids before any path
//! is built from them; `.claude/`, `.claude/skills/`, and each skill
//! directory are inspected without following symlinks, and a symlink is
//! refused even under `--force`; a directory is replaced only when it carries
//! this command's ownership marker, or under `--force`. Every skill is
//! classified before the first byte is written (all-or-nothing). The command
//! never runs git in the consumer repository — it prints the `git add` /
//! `git commit` step instead. Its only git call is the read-only
//! `git ls-files -s -z` on the source catalogue that the shared bundle
//! pipeline uses for host-independent file modes.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::bundle::sink::{WriteError, Written};
use crate::bundle::tree::{self, DirState};
use crate::bundle::{self, frontmatter, palette, Members};
use crate::catalogue;
use crate::cli::VendorArgs;
use crate::commands::build::{build_error, note_diagnostic};
use crate::commands::shell_quote;
use crate::context::Context;
use crate::install::plan::{project_root, DEFAULT_SOURCE};
use crate::output::diagnostic::{Diagnostic, Severity};
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};
use crate::sources;

/// Where vendored skills land, relative to the consumer repository.
const SKILLS_REL: &str = ".claude/skills";

/// Why vendoring matters, carried in the output so agents see it too.
const CLOUD_NOTE: &str = "Claude Code cloud sessions load skills only from a repository's committed .claude/skills/; commit the vendored paths to make them available there";

/// What happens (or would happen) to one skill directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Created,
    Updated,
    Unchanged,
    Replaced,
}

impl Action {
    fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Unchanged => "unchanged",
            Self::Replaced => "replaced",
        }
    }

    /// The human label: a dry run reports what it would do.
    fn label(self, written: bool) -> &'static str {
        match (self, written) {
            (Self::Created, false) => "would create",
            (Self::Updated, false) => "would update",
            (Self::Replaced, false) => "would replace",
            (action, _) => action.as_str(),
        }
    }
}

/// One classified skill.
struct Entry<'a> {
    skill: &'a str,
    members: &'a Members,
    action: Action,
}

/// Run `construct skill vendor`.
///
/// # Errors
///
/// Exit 2 for a skill name that is not a valid Agent Skills id; exit 3 for a
/// missing source, `--into`, or skill; exit 5 for a §5.6 or pipeline gate
/// refusal, a symlinked or non-directory destination, or a directory this
/// command does not own (the last overridable with `--force`); exit 1 for an
/// I/O failure or a written file that fails its byte-identity re-read.
pub(crate) fn run(ctx: &Context, args: &VendorArgs) -> Result<CommandOutput, AppError> {
    let names = requested_names(ctx, &args.skills)?;
    let source_spec = args.source.as_deref().unwrap_or(DEFAULT_SOURCE);
    let source = sources::resolve_source(ctx, source_spec, false)?;
    let into = resolve_into(ctx, args.into.as_deref(), &names)?;
    let rerun = rerun_command(&names, &into, args.source.as_deref());

    let all = catalogue::root_skills(&source);
    let picked = catalogue::select(&all, &names).map_err(|unknown| {
        AppError::not_found(
            ctx,
            format!("unknown skill(s): {}", unknown.join(", ")),
            format!("construct skill find --source {}", quote(&source)),
        )
        .with_extension("unknown_skills", json!(unknown))
    })?;

    let plan = bundle::vendor_plan(&source, &picked).map_err(|e| build_error(ctx, &source, e))?;
    if let Some(bytes) = &plan.palette {
        let bad = palette_failures(&plan.trees, bytes);
        if !bad.is_empty() {
            return Err(integrity_error(ctx, &rerun, &bad));
        }
    }

    check_parents(ctx, &into)?;
    let skills_dir = into.join(SKILLS_REL);
    let entries = classify(ctx, &into, &skills_dir, &rerun, &plan.trees)?;

    for note in &plan.notes {
        note_diagnostic(ctx, note).emit(ctx);
    }

    let changed: Vec<&Entry<'_>> = entries
        .iter()
        .filter(|e| e.action != Action::Unchanged)
        .collect();
    let written = !ctx.dry_run;
    if written {
        write_all(
            ctx,
            &into,
            &skills_dir,
            &rerun,
            &changed,
            plan.palette.as_deref(),
        )?;
    }

    let commit_paths: Vec<String> = changed
        .iter()
        .map(|e| format!("{SKILLS_REL}/{}", e.skill))
        .collect();
    let next_step = (!commit_paths.is_empty()).then(|| next_step(&into, &commit_paths, &changed));
    if written {
        let message = if changed.is_empty() {
            format!(
                "{} skill(s) already current in {}; nothing to commit",
                entries.len(),
                into.join(SKILLS_REL).display()
            )
        } else {
            format!(
                "vendored {} skill(s) into {}; {CLOUD_NOTE}",
                changed.len(),
                into.join(SKILLS_REL).display()
            )
        };
        let mut d = Diagnostic::new(ctx, Severity::Ok, "VENDORED", message);
        if let Some(step) = &next_step {
            d = d.with_hint(step.clone());
        }
        d.emit(ctx);
    } else if !changed.is_empty() {
        let mut d = Diagnostic::new(
            ctx,
            Severity::Info,
            "VENDOR_PLANNED",
            format!(
                "--dry-run: would vendor {} skill(s) into {}; {CLOUD_NOTE}",
                changed.len(),
                into.join(SKILLS_REL).display()
            ),
        );
        if let Some(step) = &next_step {
            d = d.with_hint(step.clone());
        }
        d.emit(ctx);
    }

    Ok(output(
        &into,
        &source,
        &entries,
        &commit_paths,
        next_step,
        plan.palette.is_some(),
        written,
    ))
}

/// Validate and deduplicate the requested names before any path is built from
/// them. A name that is not a valid Agent Skills id (`../x`, `a/b`, `.hidden`)
/// can never name a catalogue skill, and must never reach a `join`.
fn requested_names(ctx: &Context, requested: &[String]) -> Result<Vec<String>, AppError> {
    let invalid: Vec<&String> = requested
        .iter()
        .filter(|n| !frontmatter::valid_name(n))
        .collect();
    if !invalid.is_empty() {
        return Err(AppError::invalid_argument(
            ctx,
            format!(
                "invalid skill name(s): {}; a skill id matches ^[a-z0-9]+(-[a-z0-9]+)*$ (max 64)",
                invalid
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!("construct skill find --source {DEFAULT_SOURCE}"),
        )
        .with_extension("invalid_skill_names", json!(invalid)));
    }
    let mut names: Vec<String> = Vec::with_capacity(requested.len());
    for n in requested {
        if !names.contains(n) {
            names.push(n.clone());
        }
    }
    Ok(names)
}

/// The canonical consumer repository. An explicit `--into` must be an existing
/// directory; the default is the enclosing git work tree of the current
/// directory, which must exist (a `.git` entry — git itself is never run).
fn resolve_into(ctx: &Context, into: Option<&Path>, names: &[String]) -> Result<PathBuf, AppError> {
    let dir = if let Some(dir) = into {
        if !dir.is_dir() {
            return Err(AppError::not_found(
                ctx,
                format!("--into '{}' is not an existing directory", dir.display()),
                format!("mkdir -p {}", quote(dir)),
            ));
        }
        dir.to_path_buf()
    } else {
        let root = project_root();
        if !root.join(".git").exists() {
            return Err(AppError::not_found(
                ctx,
                format!(
                    "'{}' is not inside a git work tree; name the repository to vendor into",
                    root.display()
                ),
                format!("construct skill vendor {} --into <repo>", names.join(" ")),
            ));
        }
        root
    };
    dir.canonicalize().map_err(|e| {
        AppError::not_found(
            ctx,
            format!("cannot resolve '{}': {e}", dir.display()),
            format!("construct skill vendor {} --into <repo>", names.join(" ")),
        )
    })
}

/// Refuse a symlinked or non-directory `.claude/` or `.claude/skills/`: a
/// write through either could land outside the repository. Never overridable.
fn check_parents(ctx: &Context, into: &Path) -> Result<(), AppError> {
    for rel in [".claude", SKILLS_REL] {
        let path = into.join(rel);
        let reason = match fs::symlink_metadata(&path) {
            Err(_) => return Ok(()),
            Ok(meta) if meta.file_type().is_symlink() => "symlink",
            Ok(meta) if !meta.is_dir() => "not a directory",
            Ok(_) => continue,
        };
        return Err(unsafe_path_error(ctx, &path, rel, reason));
    }
    Ok(())
}

/// The exit-5 refusal for a destination component `vendor` will not write
/// through.
fn unsafe_path_error(ctx: &Context, path: &Path, rel: &str, reason: &str) -> AppError {
    AppError::new(
        ctx,
        ErrorCode::Conflict,
        5,
        format!(
            "'{}' is a {reason}; `construct skill vendor` never writes through it",
            path.display()
        ),
        format!("ls -la {}", quote(path)),
    )
    .with_extension(
        "vendor_unsafe_paths",
        json!([{ "path": rel, "reason": reason }]),
    )
}

/// Classify every destination before anything is written, refusing every
/// unsafe path and every directory this command does not own at once.
fn classify<'a>(
    ctx: &Context,
    into: &Path,
    skills_dir: &Path,
    rerun: &str,
    trees: &'a [(String, Members)],
) -> Result<Vec<Entry<'a>>, AppError> {
    let mut entries = Vec::with_capacity(trees.len());
    let mut unsafe_paths: Vec<Value> = Vec::new();
    let mut foreign: Vec<Value> = Vec::new();
    let mut extras: Vec<Value> = Vec::new();
    for (skill, members) in trees {
        let dest = skills_dir.join(skill);
        let rel = format!("{SKILLS_REL}/{skill}");
        let state = tree::assess(&dest, skill, members).map_err(|e| {
            AppError::internal(
                ctx,
                format!("cannot inspect '{}': {e}", dest.display()),
                format!("ls -la {}", quote(&dest)),
            )
        })?;
        let action = match state {
            DirState::Missing => Action::Created,
            DirState::Owned { unchanged: true } => Action::Unchanged,
            DirState::Owned { unchanged: false } => Action::Updated,
            DirState::OwnedWithExtras(files) => {
                if !ctx.yes {
                    extras.push(json!({ "skill": skill, "path": rel, "files": files }));
                }
                Action::Replaced
            }
            DirState::Foreign => {
                if !ctx.yes {
                    foreign.push(json!({ "skill": skill, "path": rel }));
                }
                Action::Replaced
            }
            DirState::Symlink => {
                unsafe_paths.push(json!({ "path": rel, "reason": "symlink" }));
                continue;
            }
            DirState::NotADirectory => {
                unsafe_paths.push(json!({ "path": rel, "reason": "not a directory" }));
                continue;
            }
        };
        entries.push(Entry {
            skill,
            members,
            action,
        });
    }

    if let Some(first) = unsafe_paths.first() {
        let rel = first["path"].as_str().unwrap_or(SKILLS_REL);
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!(
                "{} destination(s) are symlinks or not directories; `construct skill vendor` never writes through them",
                unsafe_paths.len()
            ),
            format!("ls -la {}", quote(&into.join(rel))),
        )
        .with_extension("vendor_unsafe_paths", Value::Array(unsafe_paths)));
    }
    let force_hint = format!("{rerun} --force");
    if !foreign.is_empty() {
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!(
                "{} existing director(ies) under {SKILLS_REL} were not created by `construct skill vendor`",
                foreign.len()
            ),
            force_hint,
        )
        .with_extension("vendor_not_owned", Value::Array(foreign)));
    }
    if !extras.is_empty() {
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!(
                "{} vendored director(ies) hold files `construct skill vendor` did not write",
                extras.len()
            ),
            force_hint,
        )
        .with_extension("vendor_unowned_files", Value::Array(extras)));
    }
    Ok(entries)
}

/// Write every changed skill. Each vendored palette is verified in staging,
/// before its swap.
fn write_all(
    ctx: &Context,
    into: &Path,
    skills_dir: &Path,
    rerun: &str,
    changed: &[&Entry<'_>],
    palette_bytes: Option<&[u8]>,
) -> Result<(), AppError> {
    if changed.is_empty() {
        return Ok(());
    }
    let retry = format!("{rerun} --dry-run");
    fs::create_dir_all(skills_dir).map_err(|e| {
        AppError::internal(
            ctx,
            format!("cannot create '{}': {e}", skills_dir.display()),
            retry.clone(),
        )
    })?;
    // Defense in depth against a symlink swapped in after the first
    // `check_parents`: re-check each component with `symlink_metadata`,
    // which is platform-neutral (comparing a `canonicalize` result is not —
    // Windows returns a `\\?\` verbatim path that never equals the input).
    check_parents(ctx, into)?;

    for entry in changed {
        // Only a palette consumer's `assets/steelbore.toml` is the vendored copy.
        let consumer = palette::is_consumer(entry.skill);
        let verify = |rel: &str, on_disk: &[u8]| -> bool {
            !consumer
                || rel != palette::PALETTE_MEMBER
                || palette_bytes.is_none_or(|p| p == on_disk)
        };
        match tree::write(skills_dir, entry.skill, entry.members, &verify) {
            // The vendored palette was verified in staging, before the swap;
            // a rename does not change bytes, so there is no post-swap check
            // that could only destroy a verified tree.
            Ok(Written {
                dir: dest,
                leftover,
            }) => {
                if let Some((path, err)) = leftover {
                    Diagnostic::new(
                        ctx,
                        Severity::Warn,
                        "STALE_TREE_LEFT",
                        format!(
                            "{} is vendored, but the previous tree at '{}' could not be removed: {err}",
                            entry.skill,
                            path.display()
                        ),
                    )
                    .with_extension("skill", json!(entry.skill))
                    .with_extension("path", json!(path.display().to_string()))
                    .emit(ctx);
                }
                Diagnostic::new(
                    ctx,
                    Severity::Info,
                    "SKILL_VENDORED",
                    format!("{} {}", entry.action.as_str(), dest.display()),
                )
                .with_extension("skill", json!(entry.skill))
                .emit(ctx);
            }
            Err(WriteError::Io { path, err }) => {
                return Err(AppError::internal(
                    ctx,
                    format!("cannot write '{}': {err}", path.display()),
                    retry,
                ));
            }
            Err(WriteError::Verify(files)) => {
                if files.iter().any(|f| f == palette::PALETTE_MEMBER) {
                    return Err(integrity_error(ctx, rerun, &[entry.skill.to_owned()]));
                }
                return Err(AppError::internal(
                    ctx,
                    format!(
                        "{} file(s) written for {} are not byte-identical to what was built: {}",
                        files.len(),
                        entry.skill,
                        files.join(", ")
                    ),
                    retry,
                )
                .with_extension(
                    "write_integrity_failures",
                    json!([{ "skill": entry.skill, "files": files }]),
                ));
            }
        }
    }
    Ok(())
}

/// Consumers whose in-memory vendored palette differs from the source.
fn palette_failures(trees: &[(String, Members)], bytes: &[u8]) -> Vec<String> {
    trees
        .iter()
        .filter(|(skill, members)| {
            palette::is_consumer(skill)
                && members
                    .get(palette::PALETTE_MEMBER)
                    .is_none_or(|m| m.bytes != bytes)
        })
        .map(|(skill, _)| skill.clone())
        .collect()
}

/// The exit-1 error for a vendored palette that is not byte-identical to its
/// source.
fn integrity_error(ctx: &Context, rerun: &str, skills: &[String]) -> AppError {
    AppError::internal(
        ctx,
        format!(
            "vendored {} is not byte-identical to its source for: {}",
            palette::PALETTE_MEMBER,
            skills.join(", ")
        ),
        format!("{rerun} --verbose"),
    )
    .with_extension(
        "palette_integrity_failures",
        Value::Array(
            skills
                .iter()
                .map(|s| {
                    json!({
                        "skill": s,
                        "target": "vendor",
                        "path": format!("{SKILLS_REL}/{s}/{}", palette::PALETTE_MEMBER),
                    })
                })
                .collect(),
        ),
    )
}

/// The runnable commit step the user performs; never executed here.
fn next_step(into: &Path, paths: &[String], changed: &[&Entry<'_>]) -> String {
    let repo = quote(into);
    let skills: Vec<&str> = changed.iter().map(|e| e.skill).collect();
    format!(
        "git -C {repo} add {} && git -C {repo} commit -m \"chore: vendor Construct skills {}\"",
        paths.join(" "),
        skills.join(", ")
    )
}

/// `path` quoted for a POSIX shell when it holds anything beyond a safe set.
fn quote(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

/// The invocation that reproduces this run — names, `--into`, and
/// `--source` when one was given — for hints to extend with one flag.
fn rerun_command(names: &[String], into: &Path, source: Option<&str>) -> String {
    let mut cmd = format!(
        "construct skill vendor {} --into {}",
        names.join(" "),
        quote(into)
    );
    if let Some(src) = source {
        cmd.push_str(" --source ");
        cmd.push_str(&shell_quote(src));
    }
    cmd
}

/// The command output: the `data` payload and its human table.
fn output(
    into: &Path,
    source: &Path,
    entries: &[Entry<'_>],
    commit_paths: &[String],
    next_step: Option<String>,
    palette_loaded: bool,
    written: bool,
) -> CommandOutput {
    let skills: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "skill": e.skill,
                "path": format!("{SKILLS_REL}/{}", e.skill),
                "action": e.action.as_str(),
                "files": e.members.keys().collect::<Vec<_>>(),
            })
        })
        .collect();
    let palette_vendored: Vec<Value> = entries
        .iter()
        .filter(|e| {
            palette_loaded
                && palette::is_consumer(e.skill)
                && e.members.contains_key(palette::PALETTE_MEMBER)
        })
        .map(|e| {
            json!({
                "skill": e.skill,
                "path": format!("{SKILLS_REL}/{}/{}", e.skill, palette::PALETTE_MEMBER),
                "verified": true,
            })
        })
        .collect();
    let data = json!({
        "into": into.display().to_string(),
        "source": source.display().to_string(),
        "written": written,
        "planned": !written,
        "skills": skills,
        "commit_paths": commit_paths,
        "next_step": next_step,
        "palette_vendored": palette_vendored,
        "note": CLOUD_NOTE,
    });

    let mut rows: Vec<Vec<String>> = entries
        .iter()
        .map(|e| {
            vec![
                e.skill.to_owned(),
                e.action.label(written).to_owned(),
                format!("{SKILLS_REL}/{}", e.skill),
                e.members.len().to_string(),
            ]
        })
        .collect();
    rows.push(vec![
        if written { "total" } else { "planned" }.to_owned(),
        format!("{} to commit", commit_paths.len()),
        next_step.unwrap_or_else(|| "nothing to commit".to_owned()),
        String::new(),
    ]);
    let table = HumanRender::Table {
        headers: vec![
            "SKILL".to_owned(),
            "ACTION".to_owned(),
            "PATH".to_owned(),
            "FILES".to_owned(),
        ],
        rows,
    };
    let human = if written {
        table
    } else {
        HumanRender::Titled {
            title: format!(
                "[dry-run] would vendor {} skill(s) into {}",
                entries.len(),
                into.display()
            ),
            body: Box::new(table),
        }
    };
    CommandOutput::new(data, human)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{quote, Action};

    #[test]
    fn dry_run_labels_say_would() {
        assert_eq!(Action::Created.label(false), "would create");
        assert_eq!(Action::Updated.label(false), "would update");
        assert_eq!(Action::Replaced.label(false), "would replace");
        assert_eq!(Action::Unchanged.label(false), "unchanged");
        assert_eq!(Action::Created.label(true), "created");
    }

    #[test]
    fn quote_leaves_plain_paths_and_wraps_others() {
        assert_eq!(quote(Path::new("/srv/repo-1")), "/srv/repo-1");
        assert_eq!(quote(Path::new("/srv/my repo")), "'/srv/my repo'");
        assert_eq!(quote(Path::new("/srv/it's")), r"'/srv/it'\''s'");
    }
}
