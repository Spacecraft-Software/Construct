// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pure bundle pipeline behind `construct skill build`.
//!
//! Nothing here knows about [`crate::context::Context`], `AppError`, or
//! rendering: every function takes paths and bytes and returns data, or a list
//! of [`Problem`]s naming every offender. `commands/build.rs` is the only place
//! that maps a [`BuildError`] to an exit code, which keeps these modules
//! testable without a context and keeps exit-code policy in one place.
//!
//! For each `(skill, target)` the order is fixed:
//! collect → palette vendoring → frontmatter projection → round-trip verify →
//! target transform (Perplexity / Gemini consolidation, Gemini renaming,
//! single-file render) → post-emit
//! §5.6 gate → in-memory encoding. Every gate runs for every selected skill
//! before [`plan`] returns, so a refusal writes nothing (all-or-nothing).
//!
//! # Concurrency (Standard §3.2)
//!
//! The pipeline is deliberately serial. The workload is about 45 skills × 5
//! targets of small text files — tens of milliseconds, bound by I/O and
//! deflate — and serial order makes deterministic output ordering and
//! all-or-nothing gating trivial to guarantee. Each `(skill, target)` is a pure
//! function of its inputs, so a later `par_iter` would be a local change if
//! the catalogue ever grows enough to measure a benefit.

pub(crate) mod collect;
pub(crate) mod consolidate;
pub(crate) mod frontmatter;
pub(crate) mod gemini;
pub(crate) mod palette;
pub(crate) mod single;
pub(crate) mod sink;
pub(crate) mod tree;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::catalogue::{Origin, SkillRef};
use crate::gate::{self, Audit};

/// A build target: one distribution layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Target {
    /// Claude Code (local + web), claude.ai, Gemini CLI, Codex: nested
    /// `<name>/…` zips with spec-clean frontmatter.
    Claude,
    /// Grok: flat zips, `name` + `description` frontmatter only.
    Grok,
    /// Perplexity: the Claude layout, consolidated under 100 files.
    Perplexity,
    /// The Gemini app: flat `.zip` only, `name` + `description` frontmatter,
    /// only `.csv`/`.py`/`.txt`/`.md` members, consolidated like Perplexity.
    Gemini,
    /// One self-contained markdown file per skill, for loader-less platforms.
    SingleFile,
}

impl Target {
    /// Every target, in the fixed order `all` expands to.
    pub(crate) const ALL: [Self; 5] = [
        Self::Claude,
        Self::Grok,
        Self::Perplexity,
        Self::Gemini,
        Self::SingleFile,
    ];

    /// The stable lowercase name, also the `dist/<target>/` directory.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Perplexity => "perplexity",
            Self::Gemini => "gemini",
            Self::SingleFile => "single-file",
        }
    }
}

/// One file inside a bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Member {
    /// The file's bytes, exactly as read (or generated).
    pub(crate) bytes: Vec<u8>,
    /// `0o755` when the source had any exec bit, otherwise `0o644`.
    pub(crate) mode: u32,
}

impl Member {
    /// A generated, non-executable text member.
    pub(crate) fn text(text: impl Into<String>) -> Self {
        Self {
            bytes: text.into().into_bytes(),
            mode: 0o644,
        }
    }
}

/// A bundle's files keyed by `/`-separated path relative to the skill root.
/// The `BTreeMap` order is the archive order.
pub(crate) type Members = BTreeMap<String, Member>;

/// The file name every skill carries; its frontmatter is what gets projected.
pub(crate) const SKILL_MD: &str = "SKILL.md";

/// A refusal found by the pipeline, naming one offender.
#[derive(Debug, Clone)]
pub(crate) struct Problem {
    /// The `AppError` extension key the detail is grouped under.
    pub(crate) key: &'static str,
    /// `true` for a generator bug (exit 1), `false` for a conflict (exit 5).
    pub(crate) internal: bool,
    /// One-line, period-free description.
    pub(crate) message: String,
    /// A runnable recovery command.
    pub(crate) hint: String,
    /// The structured detail placed in the extension array.
    pub(crate) detail: Value,
}

impl Problem {
    /// A state conflict the caller resolves (exit 5).
    pub(crate) fn conflict(
        key: &'static str,
        message: impl Into<String>,
        hint: impl Into<String>,
        detail: Value,
    ) -> Self {
        Self {
            key,
            internal: false,
            message: message.into(),
            hint: hint.into(),
            detail,
        }
    }

    /// A generator integrity failure (exit 1).
    pub(crate) fn internal(key: &'static str, message: impl Into<String>, detail: Value) -> Self {
        Self {
            key,
            internal: true,
            message: message.into(),
            hint: "construct skill build --verbose --dry-run".to_owned(),
            detail,
        }
    }
}

/// Severity of a [`Note`]; errors are never notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoteLevel {
    /// Narration, visible under `--verbose`.
    Info,
    /// A degradation the caller should know about.
    Warn,
}

/// A non-fatal observation, emitted by the handler as a diagnostic.
#[derive(Debug, Clone)]
pub(crate) struct Note {
    /// Severity.
    pub(crate) level: NoteLevel,
    /// Stable upper-snake-case diagnostic code.
    pub(crate) code: &'static str,
    /// One-line description.
    pub(crate) message: String,
    /// Optional runnable follow-up.
    pub(crate) hint: Option<String>,
    /// Structured fields.
    pub(crate) detail: Value,
}

/// How an artifact's vendored palette is re-checked after it is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaletteCheck {
    /// Read the vendored palette back out of the zip.
    Zip {
        /// Its full in-archive path: `<prefix>assets/steelbore.toml`, with
        /// `.txt` appended for Gemini.
        path: String,
    },
    /// Extract the fenced `assets/steelbore.toml` section of a single file.
    Markdown,
}

/// One encoded output file.
#[derive(Debug, Clone)]
pub(crate) struct Artifact {
    /// The target it belongs to.
    pub(crate) target: Target,
    /// The skill it was built from.
    pub(crate) skill: String,
    /// The file name inside `dist/<target>/`.
    pub(crate) file_name: String,
    /// `"zip"`, `"skill"`, or `"md"`.
    pub(crate) format: &'static str,
    /// Archive entries (files + directory entries), or sections for `md`.
    pub(crate) entries: usize,
    /// The encoded bytes.
    pub(crate) bytes: Vec<u8>,
    /// Set when the artifact carries a vendored palette to verify.
    pub(crate) palette: Option<PaletteCheck>,
}

/// A consolidation that was applied (Perplexity or Gemini).
#[derive(Debug, Clone)]
pub(crate) struct Consolidation {
    /// The target whose bundle was consolidated.
    pub(crate) target: Target,
    /// The consolidated skill.
    pub(crate) skill: String,
    /// Files in the Claude-layout tree before consolidation.
    pub(crate) files_before: usize,
    /// Files in the emitted Perplexity bundle.
    pub(crate) files_after: usize,
    /// Category files emitted.
    pub(crate) categories: usize,
}

/// The fully gated, fully encoded result of a build, before any write.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    /// Every output file, in `(target, skill, file)` order.
    pub(crate) artifacts: Vec<Artifact>,
    /// Consolidations applied for Perplexity and Gemini.
    pub(crate) consolidated: Vec<Consolidation>,
    /// Non-fatal observations.
    pub(crate) notes: Vec<Note>,
    /// `(skill, path)` members left out of a single-file render (binary).
    pub(crate) skipped: Vec<(String, String)>,
    /// `(skill, count)` of link fragments dropped by single-file rendering.
    pub(crate) fragments_dropped: Vec<(String, usize)>,
    /// The palette source bytes, when any consumer was built.
    pub(crate) palette: Option<Vec<u8>>,
    /// Number of distinct skills built.
    pub(crate) skills: usize,
}

/// Why [`plan`] refused.
#[derive(Debug)]
pub(crate) enum BuildError {
    /// The §5.6 gate (source or emitted `SKILL.md`).
    Gate(Audit),
    /// A required input file is missing (exit 3).
    Missing {
        /// What is missing.
        what: &'static str,
        /// Where it was expected.
        path: PathBuf,
    },
    /// Every other refusal, all offenders listed.
    Problems(Vec<Problem>),
}

/// What to build.
#[derive(Debug)]
pub(crate) struct BuildInput<'a> {
    /// The catalogue root.
    pub(crate) repo: &'a Path,
    /// Selected root skills.
    pub(crate) root: Vec<SkillRef>,
    /// Selected Grok-native skills (built for the `grok` target only).
    pub(crate) grok: Vec<SkillRef>,
    /// Targets, deduplicated, in [`Target::ALL`] order.
    pub(crate) targets: Vec<Target>,
}

/// A collected skill with its vendored members.
#[derive(Debug)]
struct Prepared {
    skill: SkillRef,
    text: String,
    members: Members,
}

/// Gate, collect, project, transform, and encode every selected
/// `(skill, target)` in memory. Writes nothing.
///
/// # Errors
///
/// [`BuildError`] naming every offender when any gate refuses.
pub(crate) fn plan(input: &BuildInput<'_>) -> Result<Plan, BuildError> {
    let mut plan = Plan::default();
    let wants_grok = input.targets.contains(&Target::Grok);
    let grok: Vec<SkillRef> = if wants_grok {
        input.grok.clone()
    } else {
        Vec::new()
    };
    let everyone: Vec<&SkillRef> = input.root.iter().chain(grok.iter()).collect();
    plan.skills = everyone.len();

    // ── source §5.6 gate ────────────────────────────────────────────────────
    let audit = gate::audit(
        &everyone
            .iter()
            .map(|s| (s.name.clone(), s.dir.join(SKILL_MD)))
            .collect::<Vec<_>>(),
    );
    if !audit.is_clean() {
        return Err(BuildError::Gate(audit));
    }

    let prepared = prepare(input.repo, &everyone, &mut plan)?;
    // Every root skill, selected or not: single-file renders a link to any of
    // them as plain text naming it.
    let siblings: BTreeSet<String> = crate::catalogue::root_skills(input.repo)
        .into_iter()
        .map(|s| s.name)
        .collect();
    let categories = load_categories(input, &prepared)?;

    // ── per target ──────────────────────────────────────────────────────────
    let mut problems: Vec<Problem> = Vec::new();
    let mut emitted = Audit::default();
    for target in &input.targets {
        for p in &prepared {
            if p.skill.origin == Origin::Grok && *target != Target::Grok {
                continue;
            }
            match build_one(p, *target, categories.as_ref(), &siblings, &mut plan) {
                Ok(artifacts) => {
                    for a in &artifacts {
                        if let Some(text) = emitted_skill_md(a) {
                            gate::audit_text(
                                &p.skill.name,
                                &p.skill.dir.join(SKILL_MD),
                                &text,
                                &mut emitted,
                            );
                        }
                    }
                    plan.artifacts.extend(artifacts);
                }
                Err(found) => problems.extend(found),
            }
        }
    }

    if !problems.is_empty() {
        return Err(BuildError::Problems(problems));
    }
    if !emitted.is_clean() {
        return Err(BuildError::Gate(emitted));
    }

    // ── in-memory palette verification ──────────────────────────────────────
    if let Some(bytes) = &plan.palette {
        let failed = verify_artifacts(&plan.artifacts, bytes);
        if !failed.is_empty() {
            return Err(BuildError::Problems(failed));
        }
    }
    Ok(plan)
}

/// The gated Claude-layout trees behind `construct skill vendor`.
#[derive(Debug, Default)]
pub(crate) struct VendorPlan {
    /// `(skill, members)` in selection order; member keys are `/`-separated
    /// paths relative to the skill root.
    pub(crate) trees: Vec<(String, Members)>,
    /// Non-fatal observations from collection.
    pub(crate) notes: Vec<Note>,
    /// The palette source bytes, when a palette consumer was selected.
    pub(crate) palette: Option<Vec<u8>>,
}

/// Gate, collect, palette-vendor, and project the Claude-layout tree of every
/// selected root skill, in memory. The same prefix of the pipeline as
/// [`plan`] — source §5.6 gate, collection, validation, palette vendoring,
/// projection, and the post-emit gate on the projected `SKILL.md` — so a
/// vendored tree is exactly the `claude` target's archive content. Writes
/// nothing.
///
/// # Errors
///
/// [`BuildError`] naming every offender when any gate refuses.
pub(crate) fn vendor_plan(repo: &Path, root: &[SkillRef]) -> Result<VendorPlan, BuildError> {
    let everyone: Vec<&SkillRef> = root.iter().collect();
    let audit = gate::audit(
        &everyone
            .iter()
            .map(|s| (s.name.clone(), s.dir.join(SKILL_MD)))
            .collect::<Vec<_>>(),
    );
    if !audit.is_clean() {
        return Err(BuildError::Gate(audit));
    }

    let mut scratch = Plan::default();
    let prepared = prepare(repo, &everyone, &mut scratch)?;

    let mut problems: Vec<Problem> = Vec::new();
    let mut emitted = Audit::default();
    let mut trees = Vec::with_capacity(prepared.len());
    for p in &prepared {
        match claude_members(p) {
            Ok(members) => {
                if let Some(text) = members
                    .get(SKILL_MD)
                    .and_then(|m| std::str::from_utf8(&m.bytes).ok())
                {
                    gate::audit_text(
                        &p.skill.name,
                        &p.skill.dir.join(SKILL_MD),
                        text,
                        &mut emitted,
                    );
                }
                trees.push((p.skill.name.clone(), members));
            }
            Err(problem) => problems.push(problem),
        }
    }
    if !problems.is_empty() {
        return Err(BuildError::Problems(problems));
    }
    if !emitted.is_clean() {
        return Err(BuildError::Gate(emitted));
    }
    Ok(VendorPlan {
        trees,
        notes: scratch.notes,
        palette: scratch.palette,
    })
}

/// Collect, validate, and palette-vendor every skill, reporting every
/// offender at once.
fn prepare(
    repo: &Path,
    everyone: &[&SkillRef],
    plan: &mut Plan,
) -> Result<Vec<Prepared>, BuildError> {
    let mut problems: Vec<Problem> = Vec::new();

    // ── Grok-native names must not shadow root skills ───────────────────────
    let all_root = crate::catalogue::root_skills(repo);
    for g in everyone.iter().filter(|s| s.origin == Origin::Grok) {
        if all_root.iter().any(|r| r.name == g.name) {
            problems.push(Problem::conflict(
                "grok_name_collision",
                format!(
                    "grok-skills/{0} has the same name as root skill {0}",
                    g.name
                ),
                format!("$EDITOR grok-skills/{}/SKILL.md", g.name),
                json!({ "skill": g.name }),
            ));
        }
    }

    // ── collect + validate ──────────────────────────────────────────────────
    let mut prepared: Vec<Prepared> = Vec::new();
    let modes = collect::Modes::from_index(repo);
    for skill in everyone {
        match collect::collect(skill, &modes) {
            Ok((text, members, notes)) => {
                plan.notes.extend(notes);
                let found = match skill.origin {
                    Origin::Root => frontmatter::validate(&skill.name, &text),
                    Origin::Grok => frontmatter::validate_grok(&skill.name, &text),
                };
                problems.extend(found);
                prepared.push(Prepared {
                    skill: (*skill).clone(),
                    text,
                    members,
                });
            }
            Err(found) => problems.extend(found),
        }
    }

    // ── palette vendoring (root consumers only) ─────────────────────────────
    if prepared
        .iter()
        .any(|p| p.skill.origin == Origin::Root && palette::is_consumer(&p.skill.name))
    {
        let path = repo.join(palette::PALETTE_SOURCE);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            // Only an absent file is "missing"; any other read failure
            // (EACCES, EISDIR, …) is an I/O error, not a checkout hint.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(BuildError::Missing {
                    what: "palette source",
                    path,
                });
            }
            Err(err) => {
                problems.push(Problem::internal(
                    "io_failures",
                    format!("cannot read {}: {err}", palette::PALETTE_SOURCE),
                    json!({ "path": palette::PALETTE_SOURCE, "error": err.to_string() }),
                ));
                return Err(BuildError::Problems(problems));
            }
        };
        for p in &mut prepared {
            if p.skill.origin == Origin::Root && palette::is_consumer(&p.skill.name) {
                if let Err(problem) = palette::vendor(&p.skill.name, &mut p.members, &bytes) {
                    problems.push(problem);
                }
            }
        }
        plan.palette = Some(bytes);
    }

    if problems.is_empty() {
        Ok(prepared)
    } else {
        Err(BuildError::Problems(problems))
    }
}

/// The consolidation map, when a consolidating target (`perplexity`,
/// `gemini`) is on and the file exists. Its absence is an error only when
/// some skill needs it.
fn load_categories(
    input: &BuildInput<'_>,
    prepared: &[Prepared],
) -> Result<Option<consolidate::CategoriesFile>, BuildError> {
    let wanted = input
        .targets
        .iter()
        .any(|t| matches!(t, Target::Perplexity | Target::Gemini));
    let categories = if wanted {
        match consolidate::load(input.repo) {
            Ok(map) => map,
            Err(problem) => return Err(BuildError::Problems(vec![*problem])),
        }
    } else {
        None
    };
    if wanted
        && categories.is_none()
        && prepared.iter().any(|p| {
            p.skill.origin == Origin::Root && p.members.len() > consolidate::PERPLEXITY_MAX_FILES
        })
    {
        return Err(BuildError::Missing {
            what: "perplexity categories map",
            path: input.repo.join(consolidate::CATEGORIES_PATH),
        });
    }
    Ok(categories)
}

/// The `SKILL.md` text an artifact ships, for the post-emit gate. A single
/// file starts with its own frontmatter, so the whole document is audited.
fn emitted_skill_md(a: &Artifact) -> Option<String> {
    match a.format {
        "md" => String::from_utf8(a.bytes.clone()).ok(),
        // `.skill` carries the same SKILL.md as its `.zip` sibling.
        "zip" => sink::read_member(&a.bytes, &format!("{}{SKILL_MD}", a.zip_prefix()))
            .and_then(|b| String::from_utf8(b).ok()),
        _ => None,
    }
}

impl Artifact {
    /// The in-archive path prefix: `<skill>/`, except for the flat Grok and
    /// Gemini layouts.
    pub(crate) fn zip_prefix(&self) -> String {
        match self.target {
            Target::Grok | Target::Gemini => String::new(),
            _ => format!("{}/", self.skill),
        }
    }
}

/// Re-check the vendored palette of every artifact that carries one.
pub(crate) fn verify_artifacts(artifacts: &[Artifact], palette_bytes: &[u8]) -> Vec<Problem> {
    artifacts
        .iter()
        .filter_map(|a| {
            let check = a.palette.as_ref()?;
            if palette::verify(check, &a.bytes, palette_bytes) {
                None
            } else {
                Some(palette_integrity_problem(a))
            }
        })
        .collect()
}

/// The integrity failure for an artifact whose palette bytes changed.
pub(crate) fn palette_integrity_problem(a: &Artifact) -> Problem {
    Problem::internal(
        "palette_integrity_failures",
        format!(
            "vendored {} in {}/{} is not byte-identical to its source",
            palette::PALETTE_MEMBER,
            a.target.slug(),
            a.file_name
        ),
        json!({ "skill": a.skill, "target": a.target.slug(), "path": a.file_name }),
    )
}

/// Build every artifact for one `(skill, target)`.
fn build_one(
    p: &Prepared,
    target: Target,
    categories: Option<&consolidate::CategoriesFile>,
    siblings: &BTreeSet<String>,
    plan: &mut Plan,
) -> Result<Vec<Artifact>, Vec<Problem>> {
    match target {
        Target::Claude | Target::Grok => build_archive(p, target),
        Target::Perplexity => build_perplexity(p, categories, plan),
        Target::Gemini => build_gemini(p, categories, plan),
        Target::SingleFile => build_single(p, siblings, plan),
    }
}

/// `members` with its `SKILL.md` replaced by `text` (mode preserved).
fn with_skill_md(members: &Members, text: String) -> Members {
    let mut m = members.clone();
    let mode = m.get(SKILL_MD).map_or(0o644, |x| x.mode);
    m.insert(
        SKILL_MD.to_owned(),
        Member {
            bytes: text.into_bytes(),
            mode,
        },
    );
    m
}

/// Encode a zip, mapping an encoder failure to an internal problem.
fn encode(
    name: &str,
    target: Target,
    members: &Members,
    prefix: &str,
    dir_entries: bool,
) -> Result<(Vec<u8>, usize), Vec<Problem>> {
    sink::encode_zip(members, prefix, dir_entries).map_err(|err| {
        vec![Problem::internal(
            "encode_failures",
            format!("could not encode {name} for {}: {err}", target.slug()),
            json!({ "skill": name, "target": target.slug() }),
        )]
    })
}

/// An [`Artifact`] for `p`; `check` is kept only when the skill carries a
/// vendored palette.
fn artifact(
    p: &Prepared,
    target: Target,
    file_name: String,
    format: &'static str,
    (bytes, entries): (Vec<u8>, usize),
    check: PaletteCheck,
) -> Artifact {
    let vendored =
        p.members.contains_key(palette::PALETTE_MEMBER) && palette::is_consumer(&p.skill.name);
    Artifact {
        target,
        skill: p.skill.name.clone(),
        file_name,
        format,
        entries,
        bytes,
        palette: vendored.then_some(check),
    }
}

/// The Claude-layout members of a prepared skill: the spec-clean frontmatter
/// with `user-invocable` kept, over the collected (palette-vendored) files.
/// Shared by the `claude` target and `construct skill vendor`, so the two can
/// never ship different trees.
fn claude_members(p: &Prepared) -> Result<Members, Problem> {
    let text = frontmatter::project(
        &p.skill.name,
        &p.text,
        frontmatter::Profile::Claude {
            user_invocable: true,
        },
    )?;
    Ok(with_skill_md(&p.members, text))
}

/// Claude (nested, `user-invocable` kept) or Grok (flat, name + description):
/// a `.zip` with directory entries and a `.skill` without.
fn build_archive(p: &Prepared, target: Target) -> Result<Vec<Artifact>, Vec<Problem>> {
    let name = &p.skill.name;
    let (members, prefix) = if target == Target::Claude {
        (claude_members(p).map_err(|e| vec![e])?, format!("{name}/"))
    } else {
        let text =
            frontmatter::project(name, &p.text, frontmatter::Profile::Grok).map_err(|e| vec![e])?;
        (with_skill_md(&p.members, text), String::new())
    };
    let check = PaletteCheck::Zip {
        path: format!("{prefix}{}", palette::PALETTE_MEMBER),
    };
    Ok(vec![
        artifact(
            p,
            target,
            format!("{name}.zip"),
            "zip",
            encode(name, target, &members, &prefix, true)?,
            check.clone(),
        ),
        artifact(
            p,
            target,
            format!("{name}.skill"),
            "skill",
            encode(name, target, &members, &prefix, false)?,
            check,
        ),
    ])
}

/// Perplexity: the Claude layout minus `user-invocable`, consolidated when
/// over [`consolidate::PERPLEXITY_MAX_FILES`], files only (no directory
/// entries, `build.py` parity), `.zip` only.
fn build_perplexity(
    p: &Prepared,
    categories: Option<&consolidate::CategoriesFile>,
    plan: &mut Plan,
) -> Result<Vec<Artifact>, Vec<Problem>> {
    let name = &p.skill.name;
    let text = frontmatter::project(
        name,
        &p.text,
        frontmatter::Profile::Claude {
            user_invocable: false,
        },
    )
    .map_err(|e| vec![e])?;
    let members = consolidate_for(
        p,
        Target::Perplexity,
        with_skill_md(&p.members, text),
        categories,
        plan,
    )?;
    let prefix = format!("{name}/");
    Ok(vec![artifact(
        p,
        Target::Perplexity,
        format!("{name}.zip"),
        "zip",
        encode(name, Target::Perplexity, &members, &prefix, false)?,
        PaletteCheck::Zip {
            path: format!("{prefix}{}", palette::PALETTE_MEMBER),
        },
    )])
}

/// Consolidate `members` for a consolidating target when the skill is over
/// [`consolidate::PERPLEXITY_MAX_FILES`] — the same condition, map, and
/// output for Perplexity and Gemini, so the two cannot drift. An oversized
/// skill with no map is refused for Perplexity; Gemini's own file-count gate
/// refuses it there.
fn consolidate_for(
    p: &Prepared,
    target: Target,
    members: Members,
    categories: Option<&consolidate::CategoriesFile>,
    plan: &mut Plan,
) -> Result<Members, Vec<Problem>> {
    let name = &p.skill.name;
    let map = categories.and_then(|c| c.skill.get(name.as_str()));
    if members.len() > consolidate::PERPLEXITY_MAX_FILES {
        let Some(map) = map else {
            if target == Target::Gemini {
                return Ok(members);
            }
            return Err(vec![consolidate::problem(
                name,
                "oversized_without_map",
                &json!({ "files": members.len(), "max": consolidate::PERPLEXITY_MAX_FILES }),
            )]);
        };
        let before = members.len();
        let members = consolidate::consolidate(name, &members, map)?;
        plan.consolidated.push(Consolidation {
            target,
            skill: name.clone(),
            files_before: before,
            files_after: members.len(),
            categories: map.category.len(),
        });
        return Ok(members);
    }
    if map.is_some() {
        plan.notes.push(Note {
            level: NoteLevel::Warn,
            code: "STALE_CONSOLIDATION_MAP",
            message: format!(
                "{name} has a consolidation map but only {} files; not consolidated for {}",
                members.len(),
                target.slug()
            ),
            hint: Some(format!("$EDITOR {}", consolidate::CATEGORIES_PATH)),
            detail: json!({ "skill": name, "files": members.len(), "target": target.slug() }),
        });
    }
    Ok(members)
}

/// Gemini: the Grok projection (`name` + `description`), consolidated like
/// Perplexity, members renamed to the platform's allowed extensions with
/// their links rewritten, flat, files only (the uploads that were accepted
/// carried no directory entries), `.zip` only.
fn build_gemini(
    p: &Prepared,
    categories: Option<&consolidate::CategoriesFile>,
    plan: &mut Plan,
) -> Result<Vec<Artifact>, Vec<Problem>> {
    let name = &p.skill.name;
    let text =
        frontmatter::project(name, &p.text, frontmatter::Profile::Grok).map_err(|e| vec![e])?;
    let members = consolidate_for(
        p,
        Target::Gemini,
        with_skill_md(&p.members, text),
        categories,
        plan,
    )?;
    let members = gemini::transform(name, &members)?;
    Ok(vec![artifact(
        p,
        Target::Gemini,
        format!("{name}.zip"),
        "zip",
        encode(name, Target::Gemini, &members, "", false)?,
        PaletteCheck::Zip {
            path: gemini::platform_path(palette::PALETTE_MEMBER),
        },
    )])
}

/// Single-file: one self-contained markdown document.
fn build_single(
    p: &Prepared,
    siblings: &BTreeSet<String>,
    plan: &mut Plan,
) -> Result<Vec<Artifact>, Vec<Problem>> {
    let name = &p.skill.name;
    let text =
        frontmatter::project(name, &p.text, frontmatter::Profile::Header).map_err(|e| vec![e])?;
    let rendered = single::render(name, &text, &p.members, siblings)?;
    for path in &rendered.skipped {
        plan.skipped.push((name.clone(), path.clone()));
        plan.notes.push(Note {
            level: NoteLevel::Warn,
            code: "BINARY_ASSET_SKIPPED",
            message: format!("{name}: {path} is binary; left out of the single-file render"),
            hint: None,
            detail: json!({ "skill": name, "path": path }),
        });
    }
    if rendered.fragments_dropped > 0 {
        plan.notes.push(Note {
            level: NoteLevel::Info,
            code: "FRAGMENT_DROPPED",
            message: format!(
                "{name}: {} link fragment(s) now point at the inlined file's anchor",
                rendered.fragments_dropped
            ),
            hint: None,
            detail: json!({ "skill": name, "count": rendered.fragments_dropped }),
        });
    }
    if rendered.unlinked > 0 {
        plan.notes.push(Note {
            level: NoteLevel::Info,
            code: "LINK_UNLINKED",
            message: format!(
                "{name}: {} relative link(s) with no target in the single file kept as plain text",
                rendered.unlinked
            ),
            hint: None,
            detail: json!({ "skill": name, "count": rendered.unlinked }),
        });
    }
    plan.fragments_dropped
        .push((name.clone(), rendered.fragments_dropped));
    let entries = rendered.sections;
    Ok(vec![artifact(
        p,
        Target::SingleFile,
        format!("{name}.md"),
        "md",
        (rendered.text.into_bytes(), entries),
        PaletteCheck::Markdown,
    )])
}
