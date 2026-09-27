// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Standard §5.6 frontmatter gate, shared by `skill ship` and `skill build`.
//!
//! The counting algorithm is [`skillmd::field_len`] — this module owns only the
//! **policy** around it: which fields are capped, what the caps are, and the
//! exact shape of the refusal. Three refusals, all `CONFLICT` / exit 5, reported
//! in this order:
//!
//! 1. a frontmatter no strict YAML parser accepts (`invalid_frontmatter`) — a
//!    failure of the gate, never an exemption from it: a strict loader sees no
//!    `description` at all for such a skill;
//! 2. a `description` over [`skillmd::DESCRIPTION_CAP`] (`oversized_skills`);
//! 3. a `compatibility` over [`skillmd::COMPATIBILITY_CAP`]
//!    (`oversized_compatibility`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::commands::shell_quote;
use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::sources::skillmd::{self, COMPATIBILITY_CAP, DESCRIPTION_CAP};

/// Every gate offender found across a set of skills.
#[derive(Debug, Default)]
pub(crate) struct Audit {
    /// `(skill, parser reason)` for a frontmatter that does not parse.
    pub(crate) invalid: Vec<(String, String)>,
    /// `(skill, chars)` for a description over [`DESCRIPTION_CAP`].
    pub(crate) oversized: Vec<(String, usize)>,
    /// `(skill, chars)` for a compatibility over [`COMPATIBILITY_CAP`].
    pub(crate) oversized_compat: Vec<(String, usize)>,
    /// The source `SKILL.md` of every skill audited, for the refusal's hint:
    /// a Grok-native skill lives under `grok-skills/`, not `<skill>/`.
    pub(crate) sources: BTreeMap<String, PathBuf>,
}

impl Audit {
    /// Whether no offender was recorded.
    pub(crate) fn is_clean(&self) -> bool {
        self.invalid.is_empty() && self.oversized.is_empty() && self.oversized_compat.is_empty()
    }
}

/// Audit every `(skill, SKILL.md path)` pair. The description is measured by
/// [`skillmd::description_len`] — the check `skill ship` has always run — and
/// an unreadable file is not measurable and is skipped, exactly as that
/// function treats it.
pub(crate) fn audit(skills: &[(String, PathBuf)]) -> Audit {
    let mut a = Audit::default();
    for (skill, path) in skills {
        let description = skillmd::description_len(path);
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        record(
            skill,
            path,
            description,
            skillmd::field_len(&text, "compatibility"),
            &mut a,
        );
    }
    a
}

/// Audit one `SKILL.md` text (a generated one, after projection) into `a`.
/// `source` is the skill's own `SKILL.md`, the file a refusal tells the user
/// to edit.
///
/// A skill already recorded as an offender of the same kind is not recorded
/// twice, so auditing a source and then each of its emitted variants lists it
/// once.
pub(crate) fn audit_text(skill: &str, source: &Path, text: &str, a: &mut Audit) {
    record(
        skill,
        source,
        skillmd::field_len(text, "description"),
        skillmd::field_len(text, "compatibility"),
        a,
    );
}

/// Record one skill's measurements. A frontmatter that does not parse is
/// recorded as invalid and nothing else about it is measured.
fn record(
    skill: &str,
    source: &Path,
    description: Result<Option<usize>, skillmd::InvalidFrontmatter>,
    compatibility: Result<Option<usize>, skillmd::InvalidFrontmatter>,
    a: &mut Audit,
) {
    a.sources
        .entry(skill.to_owned())
        .or_insert_with(|| source.to_owned());
    let invalid = |a: &mut Audit, err: &skillmd::InvalidFrontmatter| {
        if !a.invalid.iter().any(|(s, _)| s == skill) {
            a.invalid.push((skill.to_owned(), err.reason().to_owned()));
        }
    };
    match description {
        Err(err) => return invalid(a, &err),
        Ok(Some(len)) if len > DESCRIPTION_CAP => {
            if !a.oversized.iter().any(|(s, _)| s == skill) {
                a.oversized.push((skill.to_owned(), len));
            }
        }
        Ok(_) => {}
    }
    match compatibility {
        Err(err) => invalid(a, &err),
        Ok(Some(len)) if len > COMPATIBILITY_CAP => {
            if !a.oversized_compat.iter().any(|(s, _)| s == skill) {
                a.oversized_compat.push((skill.to_owned(), len));
            }
        }
        Ok(_) => {}
    }
}

/// The refusal for an audit with offenders, or `None` for a clean one.
///
/// `repo` is the catalogue root named in the invalid-frontmatter hint. The
/// messages, hints, and extension keys are the ones `skill ship` has always
/// produced; `tests/ship.rs` pins them.
pub(crate) fn into_error(ctx: &Context, repo: &Path, a: &Audit) -> Option<AppError> {
    if !a.invalid.is_empty() {
        let detail = a
            .invalid
            .iter()
            .map(|(skill, reason)| format!("{skill} ({reason})"))
            .collect::<Vec<_>>()
            .join(", ");
        return Some(
            AppError::new(
                ctx,
                ErrorCode::Conflict,
                5,
                format!("SKILL.md frontmatter is not valid YAML: {detail}"),
                format!(
                    "cd {} && python3 .github/validate-configs.py",
                    repo.display()
                ),
            )
            .with_extension(
                "invalid_frontmatter",
                json!(a
                    .invalid
                    .iter()
                    .map(|(skill, reason)| json!({ "skill": skill, "error": reason }))
                    .collect::<Vec<_>>()),
            ),
        );
    }

    if let Some((first, _)) = a.oversized.first() {
        let detail = a
            .oversized
            .iter()
            .map(|(skill, len)| format!("{skill} ({len} chars, {} over)", len - DESCRIPTION_CAP))
            .collect::<Vec<_>>()
            .join(", ");
        return Some(
            AppError::new(
                ctx,
                ErrorCode::Conflict,
                5,
                format!(
                    "SKILL.md description exceeds the {DESCRIPTION_CAP}-character cap: {detail}"
                ),
                format!(
                    "cd {} && python3 .githooks/check-description-length.py {}",
                    shell_quote(&repo.display().to_string()),
                    shell_quote(&source_of(a, repo, first, true))
                ),
            )
            .with_extension(
                "oversized_skills",
                json!(a
                    .oversized
                    .iter()
                    .map(|(skill, len)| json!({
                        "skill": skill,
                        "chars": len,
                        "over_by": len - DESCRIPTION_CAP,
                    }))
                    .collect::<Vec<_>>()),
            ),
        );
    }

    if let Some((first, _)) = a.oversized_compat.first() {
        let detail = a
            .oversized_compat
            .iter()
            .map(|(skill, len)| format!("{skill} ({len} chars, {} over)", len - COMPATIBILITY_CAP))
            .collect::<Vec<_>>()
            .join(", ");
        return Some(
            AppError::new(
                ctx,
                ErrorCode::Conflict,
                5,
                format!(
                    "SKILL.md compatibility exceeds the {COMPATIBILITY_CAP}-character cap: {detail}"
                ),
                format!("$EDITOR {}", shell_quote(&source_of(a, repo, first, false))),
            )
            .with_extension(
                "oversized_compatibility",
                json!(a
                    .oversized_compat
                    .iter()
                    .map(|(skill, len)| json!({
                        "skill": skill,
                        "chars": len,
                        "over_by": len - COMPATIBILITY_CAP,
                    }))
                    .collect::<Vec<_>>()),
            ),
        );
    }
    None
}

/// The `SKILL.md` a refusal points at: relative to `repo` when the hint runs
/// from there (`from_repo`), otherwise as audited — the path the user's own
/// `--repo` produced. Falls back to `<repo>/<skill>/SKILL.md`.
fn source_of(a: &Audit, repo: &Path, skill: &str, from_repo: bool) -> String {
    let path = a
        .sources
        .get(skill)
        .cloned()
        .unwrap_or_else(|| repo.join(skill).join("SKILL.md"));
    let shown = if from_repo {
        path.strip_prefix(repo).unwrap_or(&path)
    } else {
        &path
    };
    shown.display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{audit_text, source_of, Audit};

    fn with_compat(len: usize) -> String {
        format!(
            "---\nname: d\ndescription: x\ncompatibility: {}\n---\nbody\n",
            "c".repeat(len)
        )
    }

    #[test]
    fn compatibility_at_the_cap_passes_and_one_over_fails() {
        let mut a = Audit::default();
        audit_text("ok", Path::new("d/SKILL.md"), &with_compat(500), &mut a);
        assert!(a.is_clean());
        audit_text("over", Path::new("d/SKILL.md"), &with_compat(501), &mut a);
        assert_eq!(a.oversized_compat, vec![("over".to_owned(), 501)]);
    }

    #[test]
    fn invalid_frontmatter_is_recorded_once_and_not_measured() {
        let mut a = Audit::default();
        let bad = "---\nname: d\ndescription: a: b\n---\nbody\n";
        audit_text("bad", Path::new("d/SKILL.md"), bad, &mut a);
        audit_text("bad", Path::new("d/SKILL.md"), bad, &mut a);
        assert_eq!(a.invalid.len(), 1);
        assert!(a.oversized.is_empty());
    }

    #[test]
    fn oversized_description_is_recorded() {
        let mut a = Audit::default();
        let text = format!("---\nname: d\ndescription: {}\n---\nb\n", "x".repeat(1001));
        audit_text("big", Path::new("d/SKILL.md"), &text, &mut a);
        assert_eq!(a.oversized, vec![("big".to_owned(), 1001)]);
    }

    #[test]
    fn hint_names_the_real_skill_md() {
        let mut a = Audit::default();
        let repo = Path::new("/r");
        let grok = Path::new("/r/grok-skills/g/SKILL.md");
        audit_text("g", grok, &with_compat(501), &mut a);
        assert_eq!(source_of(&a, repo, "g", true), "grok-skills/g/SKILL.md");
        assert_eq!(source_of(&a, repo, "g", false), "/r/grok-skills/g/SKILL.md");
        assert_eq!(source_of(&a, repo, "x", true), "x/SKILL.md");
    }

    #[test]
    fn non_string_compatibility_is_invalid() {
        let mut a = Audit::default();
        audit_text(
            "n",
            Path::new("d/SKILL.md"),
            "---\nname: d\ndescription: x\ncompatibility: [a]\n---\nb\n",
            &mut a,
        );
        assert_eq!(a.invalid.len(), 1);
    }
}
