// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill ship` — the edit→ship→sync loop for the Construct catalogue.
//!
//! It detects local skill edits in the construct clone, **enforces** the
//! `.zip`/`.skill` bundling discipline (refusing to commit a skill-dir change
//! whose bundles weren't rebuilt — it does not rebuild them itself), stages the
//! shipped paths **explicitly by name** (never `git add -A`), creates a signed
//! UTC commit (via the repo's gitway signing config), pushes `origin main`, and
//! finally runs `skill sync` to repoint the consuming flake. All git operations
//! shell out to the system `git` so signing and credentials work transparently.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use serde_json::{json, Value};

use crate::cli::ShipArgs;
use crate::commands::sync;
use crate::context::Context;
use crate::install::plan::NON_SKILL_DIRS;
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};

/// Default catalogue clone to ship from.
const DEFAULT_REPO: &str = "/spacecraft-software/construct";
/// The remote a ship is allowed to push to (substring check).
const EXPECTED_REMOTE: &str = "Spacecraft-Software/Construct";
/// Assistant co-authorship trailer (CONTRIBUTING §4).
const COAUTHOR: &str = "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>";

/// Parsed working-tree changes, grouped for shipping decisions.
#[derive(Debug, Default)]
struct Changes {
    /// skill id → changed paths under that skill dir.
    skills: BTreeMap<String, Vec<String>>,
    /// skills whose `<name>.zip` changed.
    zip: BTreeSet<String>,
    /// skills whose `<name>.skill` changed.
    skill_bundle: BTreeSet<String>,
    /// Other changed root files (e.g. README.md, flake.lock, code).
    other: Vec<String>,
}

/// Run the ship loop (or, under `--dry-run`, report the exact plan).
#[allow(
    clippy::too_many_lines,
    reason = "a single linear detect -> enforce -> stage -> commit -> push -> sync flow; clearer unsplit"
)]
pub(crate) fn run(ctx: &Context, args: &ShipArgs) -> Result<CommandOutput, AppError> {
    let repo = args
        .repo
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_REPO));

    validate_repo(ctx, &repo)?;

    let skill_dirs = scan_skill_dirs(&repo);
    let changes = parse_status(ctx, &repo, &skill_dirs)?;
    let ahead = ahead_count(ctx, &repo);

    // Which skills are we shipping?
    let shipped: Vec<String> = if args.skills.is_empty() {
        changes.skills.keys().cloned().collect()
    } else {
        args.skills
            .iter()
            .filter(|s| changes.skills.contains_key(*s))
            .cloned()
            .collect()
    };

    // Enforce (do not perform) the bundle discipline: a shipped skill must have
    // both bundles rebuilt in the same changeset.
    let drifted: Vec<String> = shipped
        .iter()
        .filter(|s| !(changes.zip.contains(*s) && changes.skill_bundle.contains(*s)))
        .cloned()
        .collect();
    if !drifted.is_empty() {
        let first = &drifted[0];
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!(
                "skill source changed without rebuilt bundles: {}",
                drifted.join(", ")
            ),
            rebuild_cmd(&repo, first),
        )
        .with_extension("drifted_skills", json!(drifted)));
    }

    // Build the explicit stage list: shipped skills' files + their bundles +
    // catalogue-level root files (README.md, flake.lock). Never `git add -A`.
    let mut stage: Vec<String> = Vec::new();
    for skill in &shipped {
        if let Some(paths) = changes.skills.get(skill) {
            stage.extend(paths.iter().cloned());
        }
        for ext in ["zip", "skill"] {
            let bundle = format!("{skill}.{ext}");
            if repo.join(&bundle).exists() {
                stage.push(bundle);
            }
        }
    }
    for root in &changes.other {
        if matches!(root.as_str(), "README.md" | "flake.lock") {
            stage.push(root.clone());
        }
    }
    stage.sort();
    stage.dedup();

    let left_unstaged: Vec<String> = changes
        .other
        .iter()
        .filter(|p| !stage.contains(p))
        .cloned()
        .collect();

    let will_commit = !stage.is_empty();
    let message = args
        .message
        .clone()
        .unwrap_or_else(|| default_message(&shipped));

    // ── dry-run: report the plan, change nothing ────────────────────────────
    if ctx.dry_run {
        let data = json!({
            "repo": repo.display().to_string(),
            "status": if will_commit { "planned" } else if ahead > 0 { "planned_push" } else { "nothing_to_ship" },
            "shipped_skills": shipped,
            "would_stage": stage,
            "left_unstaged": left_unstaged,
            "would_commit": will_commit,
            "commit_message": message,
            "would_push": will_commit || ahead > 0,
            "would_sync": !args.no_sync,
            "unpushed_commits": ahead,
        });
        let human = HumanRender::Message(plan_summary(
            &repo,
            &shipped,
            &stage,
            will_commit,
            ahead,
            args.no_sync,
        ));
        return Ok(CommandOutput::new(data, human));
    }

    if !will_commit && ahead == 0 {
        // Nothing to commit or push — optionally still refresh the flake input.
        let synced_at = maybe_sync(ctx, args)?;
        let data = json!({
            "repo": repo.display().to_string(),
            "status": "nothing_to_ship",
            "shipped_skills": Value::Array(vec![]),
            "left_unstaged": left_unstaged,
            "committed": false,
            "pushed": false,
            "flake_updated": synced_at.is_some(),
            "synced_at": synced_at,
        });
        let human =
            HumanRender::Message("nothing to ship — working tree clean and up to date".to_owned());
        return Ok(CommandOutput::new(data, human));
    }

    // ── stage + commit ──────────────────────────────────────────────────────
    let mut committed = false;
    let mut commit_sha = head_sha(ctx, &repo)?;
    if will_commit {
        git_check(ctx, &repo, &stage_args(&stage))?;
        commit(ctx, &repo, &message)?;
        committed = true;
        commit_sha = head_sha(ctx, &repo)?;
    }

    // ── push ────────────────────────────────────────────────────────────────
    git_capture(ctx, &repo, &["push", "origin", "main"])?;
    let signed = head_signed(ctx, &repo);

    // ── sync ──────────────────────────────────────────────────────────────────
    let synced_at = maybe_sync(ctx, args)?;

    let data = json!({
        "repo": repo.display().to_string(),
        "status": "shipped",
        "shipped_skills": shipped,
        "staged": stage,
        "left_unstaged": left_unstaged,
        "committed": committed,
        "commit_sha": commit_sha,
        "signed": signed,
        "pushed": true,
        "flake_updated": synced_at.is_some(),
        "synced_at": synced_at,
        "timestamp": crate::time::now_iso8601(),
    });
    let human = HumanRender::Message(format!(
        "shipped {} — commit {} (signed: {}), pushed, {}",
        if shipped.is_empty() {
            "pending commits".to_owned()
        } else {
            shipped.join(", ")
        },
        short_sha(&commit_sha),
        signed,
        synced_at.map_or_else(|| "no sync".to_owned(), |t| format!("synced {t}")),
    ));
    Ok(CommandOutput::new(data, human))
}

// ── git helpers ───────────────────────────────────────────────────────────-

/// Validate that `repo` is a git work tree whose `origin` is the Construct remote.
fn validate_repo(ctx: &Context, repo: &Path) -> Result<(), AppError> {
    if !repo.is_dir() {
        return Err(AppError::not_found(
            ctx,
            format!("repo '{}' does not exist", repo.display()),
            "construct skill ship --repo /spacecraft-software/construct",
        ));
    }
    let inside = git_capture(ctx, repo, &["rev-parse", "--is-inside-work-tree"])?;
    if inside.trim() != "true" {
        return Err(AppError::new(
            ctx,
            ErrorCode::InvalidArgument,
            2,
            format!("'{}' is not a git work tree", repo.display()),
            "construct skill ship --repo <construct-clone>",
        ));
    }
    let remote = git_capture(ctx, repo, &["remote", "get-url", "origin"]).unwrap_or_default();
    if !remote.contains(EXPECTED_REMOTE) {
        return Err(AppError::new(
            ctx,
            ErrorCode::InvalidArgument,
            2,
            format!("origin '{}' is not the Construct remote", remote.trim()),
            "construct skill ship --repo <the-Construct-clone>",
        ));
    }
    Ok(())
}

/// Top-level directories that are skills (contain `SKILL.md`).
fn scan_skill_dirs(repo: &Path) -> BTreeSet<String> {
    let mut dirs = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(repo) {
        for entry in entries.flatten() {
            if !entry.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || NON_SKILL_DIRS.contains(&name.as_str()) {
                continue;
            }
            if repo.join(&name).join("SKILL.md").is_file() {
                dirs.insert(name);
            }
        }
    }
    dirs
}

/// Parse `git status --porcelain=v1` into grouped [`Changes`].
fn parse_status(
    ctx: &Context,
    repo: &Path,
    skill_dirs: &BTreeSet<String>,
) -> Result<Changes, AppError> {
    let status = git_capture(ctx, repo, &["status", "--porcelain=v1", "-uall"])?;
    let mut changes = Changes::default();
    for line in status.lines() {
        if line.len() < 4 {
            continue;
        }
        let mut path = line[3..].to_string();
        if let Some((_, new)) = path.split_once(" -> ") {
            path = new.to_string();
        }
        let path = path.trim().trim_matches('"').to_string();
        classify(&path, skill_dirs, &mut changes);
    }
    Ok(changes)
}

/// Classify one changed path into [`Changes`].
fn classify(path: &str, skill_dirs: &BTreeSet<String>, changes: &mut Changes) {
    if let Some(name) = path.strip_suffix(".zip") {
        if !name.contains('/') {
            changes.zip.insert(name.to_owned());
            return;
        }
    }
    if let Some(name) = path.strip_suffix(".skill") {
        if !name.contains('/') {
            changes.skill_bundle.insert(name.to_owned());
            return;
        }
    }
    if let Some((first, _)) = path.split_once('/') {
        if skill_dirs.contains(first) {
            changes
                .skills
                .entry(first.to_owned())
                .or_default()
                .push(path.to_owned());
            return;
        }
    }
    changes.other.push(path.to_owned());
}

/// Number of local commits ahead of the upstream (0 if no upstream).
fn ahead_count(ctx: &Context, repo: &Path) -> u32 {
    git_capture(ctx, repo, &["rev-list", "--count", "@{upstream}..HEAD"])
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Build the `git add -- <paths>` argument vector.
fn stage_args(stage: &[String]) -> Vec<String> {
    let mut args = vec!["add".to_owned(), "--".to_owned()];
    args.extend(stage.iter().cloned());
    args
}

/// Create a signed commit in UTC with the co-authorship trailer.
fn commit(ctx: &Context, repo: &Path, message: &str) -> Result<(), AppError> {
    let ts = crate::time::now_iso8601();
    let output = Proc::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "commit",
            "--date",
            ts.as_str(),
            "-m",
            message,
            "-m",
            COAUTHOR,
        ])
        .env("TZ", "UTC")
        .env("GIT_COMMITTER_DATE", &ts)
        .output()
        .map_err(|e| launch_error(ctx, &e))?;
    if !output.status.success() {
        return Err(AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("git commit failed: {}", tail(&output.stderr)),
            "cd <repo> && git status   # resolve, then re-run construct skill ship",
        ));
    }
    Ok(())
}

/// `HEAD` short signature status via `%G?` (`G` = good signature).
fn head_signed(ctx: &Context, repo: &Path) -> bool {
    git_capture(ctx, repo, &["show", "--no-patch", "--format=%G?", "HEAD"])
        .is_ok_and(|s| s.trim() == "G")
}

/// `HEAD` full SHA.
fn head_sha(ctx: &Context, repo: &Path) -> Result<String, AppError> {
    Ok(git_capture(ctx, repo, &["rev-parse", "HEAD"])?
        .trim()
        .to_owned())
}

/// Run a git command, returning stdout; error on non-zero exit.
fn git_capture(ctx: &Context, repo: &Path, args: &[&str]) -> Result<String, AppError> {
    let output = Proc::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| launch_error(ctx, &e))?;
    if !output.status.success() {
        return Err(AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("git {} failed: {}", args.join(" "), tail(&output.stderr)),
            "cd <repo> && git status",
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Run a git command (owned args), discarding stdout; error on non-zero exit.
fn git_check(ctx: &Context, repo: &Path, args: &[String]) -> Result<(), AppError> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git_capture(ctx, repo, &refs).map(|_| ())
}

/// Map a process-launch error to an `AppError` (git missing → dependency).
fn launch_error(ctx: &Context, err: &std::io::Error) -> AppError {
    if err.kind() == std::io::ErrorKind::NotFound {
        AppError::dependency_missing(
            ctx,
            "`git` was not found on PATH",
            "git --version   # install git, then re-run construct skill ship",
        )
    } else {
        AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("failed to launch git: {err}"),
            "verify git is installed and on PATH",
        )
    }
}

// ── misc helpers ─────────────────────────────────────────────────────────---

/// Run the flake sync unless `--no-sync`; returns the sync timestamp.
fn maybe_sync(ctx: &Context, args: &ShipArgs) -> Result<Option<String>, AppError> {
    if args.no_sync {
        return Ok(None);
    }
    let synced_at = sync::flake_update(ctx, &PathBuf::from(sync::DEFAULT_FLAKE_DIR))?;
    Ok(Some(synced_at))
}

/// The exact bundle-rebuild command for a drifted skill (a runnable hint).
fn rebuild_cmd(repo: &Path, skill: &str) -> String {
    let mut parts = vec![format!("{skill}/SKILL.md")];
    for candidate in ["LICENSE", "LICENSE.md", "CREDITS.md"] {
        if repo.join(skill).join(candidate).exists() {
            parts.push(format!("{skill}/{candidate}"));
        }
    }
    for dir in ["references", "assets"] {
        if repo.join(skill).join(dir).is_dir() {
            parts.push(format!("{skill}/{dir}"));
        }
    }
    let files = parts.join(" ");
    format!("rm -f {skill}.zip {skill}.skill && zip -qr {skill}.zip {files} && zip -qrD {skill}.skill {files}")
}

/// Default commit subject from the shipped skills.
fn default_message(shipped: &[String]) -> String {
    if shipped.is_empty() {
        "chore(skills): ship pending commits".to_owned()
    } else {
        format!("chore(skills): ship {}", shipped.join(", "))
    }
}

/// First 12 chars of a SHA for display.
fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

/// Human dry-run summary.
fn plan_summary(
    repo: &Path,
    shipped: &[String],
    stage: &[String],
    will_commit: bool,
    ahead: u32,
    no_sync: bool,
) -> String {
    let mut lines = vec![format!("[dry-run] ship from {}", repo.display())];
    if will_commit {
        lines.push(format!("  would stage: {}", stage.join(", ")));
        lines.push(format!(
            "  would commit + push skills: {}",
            shipped.join(", ")
        ));
    } else if ahead > 0 {
        lines.push(format!("  would push {ahead} existing commit(s)"));
    } else {
        lines.push("  nothing to commit or push".to_owned());
    }
    lines.push(format!(
        "  would sync: {}",
        if no_sync { "no" } else { "yes" }
    ));
    lines.join("\n")
}

/// Last few non-empty stderr lines, joined.
fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(3);
    lines.drain(..start);
    lines.join("; ")
}
