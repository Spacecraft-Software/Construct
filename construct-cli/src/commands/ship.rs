// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill ship` — the edit→branch→commit→PR loop for the Construct
//! catalogue.
//!
//! It detects local skill edits in the construct clone, **enforces** the
//! `.zip`/`.skill` bundling discipline (refusing to commit a skill-dir change
//! whose bundles weren't rebuilt — it does not rebuild them itself), stages the
//! shipped paths **explicitly by name** (never `git add -A`), creates a signed
//! UTC commit (via the repo's gitway signing config), pushes a **feature
//! branch**, and opens a **pull request**.
//!
//! It never pushes to the default branch. `CONTRIBUTING.md` requires every
//! change — including a one-line version bump — to go through a feature branch
//! → pull request → squash-merge → delete branch, in both the Standard and
//! Construct repositories. Merging is the maintainer's call: this command
//! opens the PR and stops.
//!
//! Because nothing lands on the default branch here, `ship` no longer runs
//! `skill sync` — repointing a consumer flake before the PR merges would pin an
//! unrelated revision. Run `construct skill sync` once the PR is merged.
//!
//! All git operations shell out to the system `git` so signing and credentials
//! work transparently; the pull request is opened with `gh`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use serde_json::{json, Value};

use crate::cli::ShipArgs;
use crate::context::Context;
use crate::install::plan::NON_SKILL_DIRS;
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};
use crate::sources::skillmd;

/// Default catalogue clone to ship from.
const DEFAULT_REPO: &str = "/spacecraft-software/construct";
/// The remote a ship is allowed to push to (substring check). Standard §6.4:
/// publication targets are limited to namespaces Spacecraft Software controls.
const EXPECTED_REMOTE: &str = "Spacecraft-Software/Construct";
/// Maximum rendered length of a skill's frontmatter `description` (Standard
/// §5.6).
///
/// The consuming skill loader rejects anything over **1024** characters at
/// install time — after the bundles are built and pushed — so the cap sits at
/// 1000 for a 24-character margin covering encoding and trailing-newline edge
/// cases. Raising it past the loader's limit would ship bundles that cannot be
/// installed. `.githooks/check-description-length.py` enforces the same number
/// in CI and in the pre-commit hook; changing one without the other lets a
/// bundle pass one gate and fail the next.
const DESCRIPTION_CAP: usize = 1000;
/// Assistant co-authorship trailer (CONTRIBUTING §4).
const COAUTHOR: &str = "Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>";
/// `owner/repo` slug passed to `gh --repo`, derived from [`EXPECTED_REMOTE`].
const REPO_SLUG: &str = EXPECTED_REMOTE;
/// Fallback default branch when `origin/HEAD` is not configured locally.
const FALLBACK_DEFAULT_BRANCH: &str = "main";
/// Prefix for generated feature branches.
const BRANCH_PREFIX: &str = "ship/";

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
    reason = "a single linear detect -> enforce -> branch -> stage -> commit -> push -> PR flow; clearer unsplit"
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

    // Enforce the Standard §5.6 description cap before anything is staged: the
    // loader rejects an over-long description at install time, by which point
    // the bundles are built, committed, and pushed. Cheaper to refuse here.
    let oversized: Vec<(String, usize)> = shipped
        .iter()
        .filter_map(|skill| {
            let len = skillmd::description_len(&repo.join(skill).join("SKILL.md"))?;
            (len > DESCRIPTION_CAP).then(|| (skill.clone(), len))
        })
        .collect();
    if let Some((first, _)) = oversized.first() {
        let detail = oversized
            .iter()
            .map(|(skill, len)| format!("{skill} ({len} chars, {} over)", len - DESCRIPTION_CAP))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(AppError::new(
            ctx,
            ErrorCode::Conflict,
            5,
            format!("SKILL.md description exceeds the {DESCRIPTION_CAP}-character cap: {detail}"),
            format!("$EDITOR {first}/SKILL.md  # trim the `description` frontmatter field"),
        )
        .with_extension(
            "oversized_skills",
            json!(oversized
                .iter()
                .map(|(skill, len)| json!({
                    "skill": skill,
                    "chars": len,
                    "over_by": len - DESCRIPTION_CAP,
                }))
                .collect::<Vec<_>>()),
        ));
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

    // Resolve the branch this work belongs on. Never the default branch —
    // CONTRIBUTING.md requires a feature branch + PR for every change.
    let default_branch = default_branch(ctx, &repo);
    let current = current_branch(ctx, &repo)?;
    let branch = resolve_branch(args, &shipped, &current, &default_branch);
    let needs_switch = current != branch;

    // ── dry-run: report the plan, change nothing ────────────────────────────
    if ctx.dry_run {
        let data = json!({
            "repo": repo.display().to_string(),
            "status": if will_commit { "planned" } else if ahead > 0 { "planned_pull_request" } else { "nothing_to_ship" },
            "shipped_skills": shipped,
            "would_stage": stage,
            "left_unstaged": left_unstaged,
            "would_commit": will_commit,
            "commit_message": message,
            "default_branch": default_branch,
            "current_branch": current,
            "branch": branch,
            "would_create_branch": needs_switch,
            "would_push_branch": will_commit || ahead > 0,
            "would_open_pull_request": will_commit || ahead > 0,
            "unpushed_commits": ahead,
        });
        let human = HumanRender::Message(plan_summary(
            &repo,
            &shipped,
            &stage,
            will_commit,
            ahead,
            &branch,
            needs_switch,
        ));
        return Ok(CommandOutput::new(data, human));
    }

    if !will_commit && ahead == 0 {
        let data = json!({
            "repo": repo.display().to_string(),
            "status": "nothing_to_ship",
            "shipped_skills": Value::Array(vec![]),
            "left_unstaged": left_unstaged,
            "committed": false,
            "branch": current,
            "pushed": false,
            "pull_request_url": Value::Null,
        });
        let human =
            HumanRender::Message("nothing to ship — working tree clean and up to date".to_owned());
        return Ok(CommandOutput::new(data, human));
    }

    // `gh` is required to open the PR. Fail before touching the work tree
    // rather than leaving a committed branch the caller must finish by hand.
    require_gh(ctx)?;

    // ── branch ──────────────────────────────────────────────────────────────
    if needs_switch {
        switch_branch(ctx, &repo, &branch)?;
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

    // ── push the branch (never the default branch) ──────────────────────────
    git_capture(ctx, &repo, &["push", "-u", "origin", branch.as_str()])?;
    let signed = head_signed(ctx, &repo);

    // ── open the pull request; merging is the maintainer's call ─────────────
    let pr_url = open_pull_request(ctx, &repo, &branch, &default_branch, &message, &shipped)?;

    let data = json!({
        "repo": repo.display().to_string(),
        "status": "pull_request_opened",
        "shipped_skills": shipped,
        "staged": stage,
        "left_unstaged": left_unstaged,
        "committed": committed,
        "commit_sha": commit_sha,
        "signed": signed,
        "branch": branch,
        "base_branch": default_branch,
        "pushed": true,
        "pull_request_url": pr_url,
        "timestamp": crate::time::now_iso8601(),
    });
    let human = HumanRender::Message(format!(
        "opened PR for {} — commit {} (signed: {}) on {}\n  {}\n  merge is the maintainer's call; run `construct skill sync` after it lands",
        if shipped.is_empty() {
            "pending commits".to_owned()
        } else {
            shipped.join(", ")
        },
        short_sha(&commit_sha),
        signed,
        branch,
        pr_url,
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

/// The repository's default branch, via `origin/HEAD` when it is configured.
///
/// A fresh clone that never ran `git remote set-head` has no `origin/HEAD`, and
/// a throwaway fixture repo has no remote refs at all — both fall back to
/// [`FALLBACK_DEFAULT_BRANCH`].
fn default_branch(ctx: &Context, repo: &Path) -> String {
    git_capture(
        ctx,
        repo,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )
    .ok()
    .and_then(|s| {
        s.trim()
            .rsplit_once('/')
            .map(|(_, name)| name.to_owned())
            .filter(|name| !name.is_empty())
    })
    .unwrap_or_else(|| FALLBACK_DEFAULT_BRANCH.to_owned())
}

/// The currently checked-out branch name.
fn current_branch(ctx: &Context, repo: &Path) -> Result<String, AppError> {
    Ok(
        git_capture(ctx, repo, &["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim()
            .to_owned(),
    )
}

/// Decide which branch the work belongs on.
///
/// An explicit `--branch` always wins. Otherwise, if the caller already put
/// themselves on a feature branch, respect it — re-running `ship` should add to
/// the open PR rather than fragment the work across branches. Only when sitting
/// on the default branch is a name generated.
fn resolve_branch(
    args: &ShipArgs,
    shipped: &[String],
    current: &str,
    default_branch: &str,
) -> String {
    if let Some(explicit) = args.branch.as_deref().map(str::trim) {
        if !explicit.is_empty() {
            return explicit.to_owned();
        }
    }
    if current != default_branch && current != "HEAD" {
        return current.to_owned();
    }
    generated_branch(shipped)
}

/// Generate a feature-branch name from the shipped skills.
///
/// Re-running with the same skills yields the same name on purpose: the second
/// run adds a commit to the branch the PR is already tracking.
fn generated_branch(shipped: &[String]) -> String {
    let slug = match shipped {
        [] => "catalogue".to_owned(),
        [only] => slugify(only),
        [first, rest @ ..] => format!("{}-plus-{}", slugify(first), rest.len()),
    };
    format!("{BRANCH_PREFIX}{slug}")
}

/// Reduce a skill id to characters that are safe in a git ref.
fn slugify(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_owned();
    if trimmed.is_empty() {
        "catalogue".to_owned()
    } else {
        trimmed
    }
}

/// Check out `branch`, creating it if it does not exist yet.
fn switch_branch(ctx: &Context, repo: &Path, branch: &str) -> Result<(), AppError> {
    let exists = git_capture(
        ctx,
        repo,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )
    .is_ok();
    let args: Vec<&str> = if exists {
        vec!["switch", branch]
    } else {
        vec!["switch", "-c", branch]
    };
    git_capture(ctx, repo, &args).map(|_| ())
}

/// Fail early if `gh` is not available, so we never strand a committed branch.
fn require_gh(ctx: &Context) -> Result<(), AppError> {
    match Proc::new("gh").arg("--version").output() {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::dependency_missing(
            ctx,
            "`gh` was not found on PATH — it is required to open the pull request",
            "gh --version   # install the GitHub CLI, then re-run construct skill ship",
        )),
        Err(e) => Err(AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("failed to launch gh: {e}"),
            "verify the GitHub CLI is installed and on PATH",
        )),
    }
}

/// Open the pull request and return its URL.
///
/// If a PR for this branch is already open, `gh` exits non-zero; that is not a
/// failure of the ship — the new commit is already pushed onto the branch the
/// existing PR tracks — so the existing URL is looked up and returned instead.
fn open_pull_request(
    ctx: &Context,
    repo: &Path,
    branch: &str,
    base: &str,
    title: &str,
    shipped: &[String],
) -> Result<String, AppError> {
    let body = pr_body(shipped);
    let output = Proc::new("gh")
        .arg("-C")
        .arg(repo)
        .args([
            "pr",
            "create",
            "--repo",
            REPO_SLUG,
            "--base",
            base,
            "--head",
            branch,
            "--title",
            title,
            "--body",
            body.as_str(),
        ])
        .output()
        .map_err(|e| gh_launch_error(ctx, &e))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    if let Some(url) = existing_pr_url(repo, branch) {
        return Ok(url);
    }
    Err(AppError::general(
        ctx,
        ErrorCode::InternalError,
        format!("gh pr create failed: {}", tail(&output.stderr)),
        format!(
            "cd {} && gh pr create --base {base} --head {branch}",
            repo.display()
        ),
    ))
}

/// URL of an already-open PR for `branch`, if there is one.
fn existing_pr_url(repo: &Path, branch: &str) -> Option<String> {
    let output = Proc::new("gh")
        .arg("-C")
        .arg(repo)
        .args([
            "pr", "view", branch, "--repo", REPO_SLUG, "--json", "url", "--jq", ".url",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!url.is_empty()).then_some(url)
}

/// Pull-request body describing what is being shipped.
fn pr_body(shipped: &[String]) -> String {
    let mut lines = vec![
        "Opened by `construct skill ship`.".to_owned(),
        String::new(),
    ];
    if shipped.is_empty() {
        lines.push("Ships pending catalogue commits.".to_owned());
    } else {
        lines.push("Skills shipped:".to_owned());
        lines.push(String::new());
        for skill in shipped {
            lines.push(format!(
                "- `{skill}` — source and both bundles (`.zip` + `.skill`) in the same commit"
            ));
        }
    }
    lines.push(String::new());
    lines.push(
        "Bundle-drift and the Standard §5.6 description cap were enforced before staging."
            .to_owned(),
    );
    lines.push(String::new());
    lines.push(
        "After this merges, run `construct skill sync` to repoint the consumer flake.".to_owned(),
    );
    lines.join("\n")
}

/// Map a `gh` launch error to an `AppError`.
fn gh_launch_error(ctx: &Context, err: &std::io::Error) -> AppError {
    if err.kind() == std::io::ErrorKind::NotFound {
        AppError::dependency_missing(
            ctx,
            "`gh` was not found on PATH — it is required to open the pull request",
            "gh --version   # install the GitHub CLI, then re-run construct skill ship",
        )
    } else {
        AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("failed to launch gh: {err}"),
            "verify the GitHub CLI is installed and on PATH",
        )
    }
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
    branch: &str,
    needs_switch: bool,
) -> String {
    let mut lines = vec![format!("[dry-run] ship from {}", repo.display())];
    if will_commit {
        lines.push(format!("  would stage: {}", stage.join(", ")));
        lines.push(format!("  would commit skills: {}", shipped.join(", ")));
    } else if ahead > 0 {
        lines.push(format!("  would push {ahead} existing commit(s)"));
    } else {
        lines.push("  nothing to commit or push".to_owned());
    }
    if will_commit || ahead > 0 {
        lines.push(format!(
            "  would {} branch: {branch}",
            if needs_switch { "create" } else { "reuse" }
        ));
        lines.push("  would open a pull request (never pushes to the default branch)".to_owned());
    }
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
