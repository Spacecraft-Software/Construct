// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill build` — distributable bundles for every target platform.
//!
//! This handler is the only place the pure [`crate::bundle`] pipeline meets the
//! CLI contract: it resolves arguments, runs [`bundle::plan`] (every gate, all
//! in memory), maps a [`BuildError`] to an [`AppError`] with the canonical exit
//! code, and — unless `--dry-run` — writes each target directory through
//! [`sink`], re-reading every written file to verify it byte-for-byte. It
//! never runs git and never touches the committed root bundles.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde_json::{json, Value};

use crate::bundle::sink::{self, OutFile, Ownership, WriteError, Written};
use crate::bundle::{
    self, palette, Artifact, BuildError, BuildInput, NoteLevel, Plan, Problem, Target,
};
use crate::catalogue::{self, Origin, SkillRef};
use crate::cli::{BuildArgs, TargetArg};
use crate::commands::shell_quote;
use crate::commands::ship::DEFAULT_REPO;
use crate::context::Context;
use crate::gate;
use crate::output::diagnostic::{Diagnostic, Severity};
use crate::output::error::{AppError, ErrorCode};
use crate::output::progress::Progress;
use crate::output::{CommandOutput, HumanRender};

/// Run `construct skill build`.
///
/// # Errors
///
/// Exit 3 for a missing catalogue, unknown skill, or missing required input;
/// exit 5 for every gate refusal and for an output directory the build does
/// not own; exit 1 for an I/O failure or a written file that fails its
/// byte-identity re-read.
pub(crate) fn run(ctx: &Context, args: &BuildArgs) -> Result<CommandOutput, AppError> {
    let repo = args
        .repo
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_REPO));
    if !repo.is_dir() {
        return Err(AppError::not_found(
            ctx,
            format!("catalogue '{}' does not exist", repo.display()),
            format!("construct skill build --repo {DEFAULT_REPO}"),
        ));
    }
    let out = args.out.clone().unwrap_or_else(|| repo.join("dist"));
    let targets = resolve_targets(&args.targets);

    let (root, grok) = select(ctx, &repo, &args.skills, &targets)?;
    if root.is_empty() && grok.is_empty() {
        return Err(AppError::not_found(
            ctx,
            format!("no skills found in '{}'", repo.display()),
            format!("construct skill find --source {}", repo.display()),
        ));
    }

    let input = BuildInput {
        repo: &repo,
        root,
        grok,
        targets: targets.clone(),
    };
    // Progress on stderr (human TTY only): one step per `(skill, target)`
    // encoded in memory, then one per target directory written.
    let writes = if ctx.dry_run { 0 } else { targets.len() };
    let steps = bundle::step_count(&input) + writes;
    let progress = Progress::bar(
        ctx,
        "building bundles",
        u64::try_from(steps).unwrap_or(u64::MAX),
    );
    let mut started = false;
    let plan = bundle::plan(&input, &mut |skill, target| {
        if started {
            progress.inc(1);
        }
        started = true;
        progress.set_message(format!("{}/{skill}", target.slug()));
    });
    if started {
        progress.inc(1);
    }
    let plan = plan.map_err(|err| build_error(ctx, &repo, err))?;

    // Refuse before the first byte is written: a target directory that holds
    // files this command did not create is only replaced under --force.
    for target in &targets {
        let dir = out.join(target.slug());
        if sink::ownership(&dir) == Ownership::Foreign && !ctx.yes {
            return Err(AppError::new(
                ctx,
                ErrorCode::Conflict,
                5,
                format!(
                    "'{}' holds files `construct skill build` did not create",
                    dir.display()
                ),
                force_hint(args),
            )
            .with_extension(
                "out_dir_not_owned",
                json!([{ "target": target.slug(), "path": dir.display().to_string() }]),
            ));
        }
    }

    for note in &plan.notes {
        note_diagnostic(ctx, note).emit(ctx);
    }

    let written = !ctx.dry_run;
    if written {
        let full = args.skills.is_empty();
        for target in &targets {
            progress.set_message(format!("writing {}", target.slug()));
            write_target(ctx, &out, *target, &plan, full)?;
            progress.inc(1);
        }
        progress.finish();
        Diagnostic::new(
            ctx,
            Severity::Ok,
            "BUNDLES_BUILT",
            format!(
                "built {} file(s) for {} skill(s) into {}",
                plan.artifacts.len(),
                plan.skills,
                out.display()
            ),
        )
        .emit(ctx);
    } else {
        progress.finish();
    }

    Ok(output(&repo, &out, &targets, &plan, written))
}

/// The invocation the user ran — skills, `--target`, `--repo`, `--out` as
/// given — with `--force` appended, so following the hint replaces exactly
/// the directories this run refused and nothing else.
fn force_hint(args: &BuildArgs) -> String {
    let mut cmd = String::from("construct skill build");
    for skill in &args.skills {
        cmd.push(' ');
        cmd.push_str(&shell_quote(skill));
    }
    let targets: Vec<String> = args
        .targets
        .iter()
        .filter_map(ValueEnum::to_possible_value)
        .map(|v| v.get_name().to_owned())
        .collect();
    if !targets.is_empty() {
        cmd.push_str(" --target ");
        cmd.push_str(&targets.join(","));
    }
    for (flag, dir) in [("--repo", &args.repo), ("--out", &args.out)] {
        if let Some(dir) = dir {
            cmd.push(' ');
            cmd.push_str(flag);
            cmd.push(' ');
            cmd.push_str(&shell_quote(&dir.display().to_string()));
        }
    }
    cmd.push_str(" --force");
    cmd
}

/// Expand `all`, deduplicate, and keep [`Target::ALL`] order. No `--target`
/// means every target.
fn resolve_targets(args: &[TargetArg]) -> Vec<Target> {
    if args.is_empty() || args.contains(&TargetArg::All) {
        return Target::ALL.to_vec();
    }
    Target::ALL
        .into_iter()
        .filter(|t| {
            args.iter().any(|a| {
                matches!(
                    (a, t),
                    (TargetArg::Claude, Target::Claude)
                        | (TargetArg::ChatGpt, Target::ChatGpt)
                        | (TargetArg::Grok, Target::Grok)
                        | (TargetArg::Perplexity, Target::Perplexity)
                        | (TargetArg::Gemini, Target::Gemini)
                        | (TargetArg::SingleFile, Target::SingleFile)
                        | (TargetArg::MiniMax, Target::MiniMax)
                )
            })
        })
        .collect()
}

/// Resolve the requested skill names against the catalogue. A Grok-native
/// skill is known only when the `grok` target is selected, since no other
/// target builds it.
fn select(
    ctx: &Context,
    repo: &Path,
    requested: &[String],
    targets: &[Target],
) -> Result<(Vec<SkillRef>, Vec<SkillRef>), AppError> {
    let all_root = catalogue::root_skills(repo);
    let all_grok = if targets.contains(&Target::Grok) {
        catalogue::grok_skills(repo)
    } else {
        Vec::new()
    };
    let combined: Vec<SkillRef> = all_root.into_iter().chain(all_grok).collect();
    match catalogue::select(&combined, requested) {
        Ok(picked) => Ok(picked.into_iter().partition(|s| s.origin == Origin::Root)),
        Err(unknown) => Err(AppError::not_found(
            ctx,
            format!("unknown skill(s): {}", unknown.join(", ")),
            format!("construct skill find --source {}", repo.display()),
        )
        .with_extension("unknown_skills", json!(unknown))),
    }
}

/// Map a pipeline refusal to the canonical error.
pub(crate) fn build_error(ctx: &Context, repo: &Path, err: BuildError) -> AppError {
    match err {
        BuildError::Gate(audit) => gate::into_error(ctx, repo, &audit).unwrap_or_else(|| {
            AppError::new(
                ctx,
                ErrorCode::InternalError,
                1,
                "the frontmatter gate refused without naming an offender",
                "construct skill build --verbose --dry-run",
            )
        }),
        BuildError::Missing { what, path } => {
            let rel = path.strip_prefix(repo).unwrap_or(&path);
            AppError::not_found(
                ctx,
                format!("{what} not found at '{}'", path.display()),
                format!("git -C {} checkout -- {}", repo.display(), rel.display()),
            )
            .with_extension(
                "missing",
                json!({ "what": what, "path": path.display().to_string() }),
            )
        }
        BuildError::Problems(problems) => problems_error(ctx, &problems),
    }
}

/// One error for a list of problems: exit 1 when any is a generator
/// integrity failure, otherwise exit 5. Every offender is listed, grouped by
/// its extension key.
fn problems_error(ctx: &Context, problems: &[Problem]) -> AppError {
    let internal = problems.iter().any(|p| p.internal);
    let lead = problems
        .iter()
        .find(|p| p.internal == internal)
        .or_else(|| problems.first());
    let (message, hint) = lead.map_or_else(
        || {
            (
                "the build refused without naming an offender".to_owned(),
                "construct skill build --verbose --dry-run".to_owned(),
            )
        },
        |p| (p.message.clone(), p.hint.clone()),
    );
    let message = if problems.len() > 1 {
        format!("{message} (and {} more)", problems.len() - 1)
    } else {
        message
    };
    let (code, exit) = if internal {
        (ErrorCode::InternalError, 1)
    } else {
        (ErrorCode::Conflict, 5)
    };
    let mut grouped: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for p in problems {
        grouped.entry(p.key).or_default().push(p.detail.clone());
    }
    grouped.into_iter().fold(
        AppError::new(ctx, code, exit, message, hint),
        |err, (key, details)| err.with_extension(key, Value::Array(details)),
    )
}

/// A pipeline note as a diagnostic, with its detail fields as extensions.
pub(crate) fn note_diagnostic(ctx: &Context, note: &bundle::Note) -> Diagnostic {
    let severity = match note.level {
        NoteLevel::Info => Severity::Info,
        NoteLevel::Warn => Severity::Warn,
    };
    let mut d = Diagnostic::new(ctx, severity, note.code, note.message.clone());
    if let Some(hint) = &note.hint {
        d = d.with_hint(hint.clone());
    }
    if let Value::Object(fields) = &note.detail {
        for (k, v) in fields {
            d = d.with_extension(k, v.clone());
        }
    }
    d
}

/// Write one target directory and re-verify every written file.
fn write_target(
    ctx: &Context,
    out: &Path,
    target: Target,
    plan: &Plan,
    full: bool,
) -> Result<(), AppError> {
    let artifacts: Vec<&Artifact> = plan
        .artifacts
        .iter()
        .filter(|a| a.target == target)
        .collect();
    let files: Vec<OutFile<'_>> = artifacts
        .iter()
        .map(|a| OutFile {
            name: &a.file_name,
            bytes: &a.bytes,
        })
        .collect();
    let by_name: BTreeMap<&str, &Artifact> = artifacts
        .iter()
        .map(|a| (a.file_name.as_str(), *a))
        .collect();
    let verify = |name: &str, on_disk: &[u8]| -> bool {
        by_name
            .get(name)
            .is_some_and(|a| a.bytes == on_disk && palette_ok(a, on_disk, plan.palette.as_deref()))
    };
    let result = if full {
        sink::write_full(out, target, &files, &verify)
    } else {
        sink::write_partial(out, target, &files, &verify)
    };
    match result {
        Ok(Written { dir, leftover }) => {
            if let Some((path, err)) = leftover {
                Diagnostic::new(
                    ctx,
                    Severity::Warn,
                    "STALE_OUTPUT_LEFT",
                    format!(
                        "the new bundles are in place, but the previous output at '{}' could not be removed: {err}",
                        path.display()
                    ),
                )
                .with_extension("path", json!(path.display().to_string()))
                .emit(ctx);
            }
            for a in &artifacts {
                Diagnostic::new(
                    ctx,
                    Severity::Info,
                    "BUNDLE_WRITTEN",
                    format!("wrote {}", dir.join(&a.file_name).display()),
                )
                .with_extension("target", json!(target.slug()))
                .with_extension("skill", json!(a.skill))
                .emit(ctx);
            }
            Ok(())
        }
        Err(WriteError::Io { path, err }) => Err(AppError::new(
            ctx,
            ErrorCode::InternalError,
            1,
            format!("cannot write '{}': {err}", path.display()),
            format!("construct skill build --out {} --dry-run", out.display()),
        )),
        Err(WriteError::Verify(names)) => Err(verify_error(ctx, out, target, &by_name, &names)),
    }
}

/// Whether a written artifact's vendored palette (if any) is byte-identical
/// to the palette source.
fn palette_ok(a: &Artifact, on_disk: &[u8], palette_bytes: Option<&[u8]>) -> bool {
    match (&a.palette, palette_bytes) {
        (None, _) => true,
        (Some(check), Some(bytes)) => palette::verify(check, on_disk, bytes),
        (Some(_), None) => false,
    }
}

/// The exit-1 error for written files whose re-read failed verification.
fn verify_error(
    ctx: &Context,
    out: &Path,
    target: Target,
    by_name: &BTreeMap<&str, &Artifact>,
    names: &[String],
) -> AppError {
    let mut palette_failures = Vec::new();
    let mut write_failures = Vec::new();
    for name in names {
        let skill = by_name.get(name.as_str()).map(|a| a.skill.clone());
        let entry = json!({ "skill": skill, "target": target.slug(), "path": name });
        if by_name
            .get(name.as_str())
            .is_some_and(|a| a.palette.is_some())
        {
            palette_failures.push(entry);
        } else {
            write_failures.push(entry);
        }
    }
    let mut err = AppError::new(
        ctx,
        ErrorCode::InternalError,
        1,
        format!(
            "{} written file(s) in {}/{} are not byte-identical to what was built: {}",
            names.len(),
            out.display(),
            target.slug(),
            names.join(", ")
        ),
        format!("construct skill build --out {} --verbose", out.display()),
    );
    if !palette_failures.is_empty() {
        err = err.with_extension("palette_integrity_failures", Value::Array(palette_failures));
    }
    if !write_failures.is_empty() {
        err = err.with_extension("write_integrity_failures", Value::Array(write_failures));
    }
    err
}

/// `path` relative to `repo` when it lies inside it, else as given.
fn display_rel(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// The command output: the `data` payload and its human table.
fn output(
    repo: &Path,
    out: &Path,
    targets: &[Target],
    plan: &Plan,
    written: bool,
) -> CommandOutput {
    let bundles: Vec<Value> = plan
        .artifacts
        .iter()
        .map(|a| {
            json!({
                "target": a.target.slug(),
                "skill": a.skill,
                "path": display_rel(repo, &out.join(a.target.slug()).join(&a.file_name)),
                "format": a.format,
                "entries": a.entries,
                "bytes": a.bytes.len(),
            })
        })
        .collect();
    let consolidated: Vec<Value> = plan
        .consolidated
        .iter()
        .map(|c| {
            json!({
                "target": c.target.slug(),
                "skill": c.skill,
                "files_before": c.files_before,
                "files_after": c.files_after,
                "categories": c.categories,
            })
        })
        .collect();
    let palette_vendored: Vec<Value> = plan
        .artifacts
        .iter()
        .filter(|a| a.palette.is_some())
        .map(|a| {
            json!({
                "skill": a.skill,
                "target": a.target.slug(),
                "path": a.file_name,
                "verified": true,
            })
        })
        .collect();
    let skipped: Vec<Value> = plan
        .skipped
        .iter()
        .map(|(skill, path)| json!({ "skill": skill, "path": path, "reason": "binary" }))
        .collect();
    let fragments: Vec<Value> = plan
        .fragments_dropped
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(skill, count)| json!({ "skill": skill, "count": count }))
        .collect();
    let data = json!({
        "repo": repo.display().to_string(),
        "out_dir": out.display().to_string(),
        "targets": targets.iter().map(|t| t.slug()).collect::<Vec<_>>(),
        "skills": plan.skills,
        "written": written,
        "bundles": bundles,
        "consolidated": consolidated,
        "palette_vendored": palette_vendored,
        "skipped": skipped,
        "fragments_dropped": fragments,
        "built_at": crate::time::now_iso8601(),
    });

    CommandOutput::new(data, human(out, plan, written))
}

/// The human table; a dry run leads with a `[dry-run]` banner, as `ship` does.
fn human(out: &Path, plan: &Plan, written: bool) -> HumanRender {
    let mut rows: Vec<Vec<String>> = plan
        .artifacts
        .iter()
        .map(|a| {
            vec![
                a.target.slug().to_owned(),
                a.skill.clone(),
                a.file_name.clone(),
                a.entries.to_string(),
            ]
        })
        .collect();
    rows.push(vec![
        if written { "total" } else { "planned" }.to_owned(),
        format!("{} skill(s)", plan.skills),
        format!("{} file(s)", plan.artifacts.len()),
        String::new(),
    ]);
    let table = HumanRender::Table {
        headers: vec![
            "TARGET".to_owned(),
            "SKILL".to_owned(),
            "FILE".to_owned(),
            "ENTRIES".to_owned(),
        ],
        rows,
    };
    if written {
        table
    } else {
        HumanRender::Titled {
            title: format!(
                "[dry-run] would build {} file(s) for {} skill(s) into {}",
                plan.artifacts.len(),
                plan.skills,
                out.display()
            ),
            body: Box::new(table),
        }
    }
}
