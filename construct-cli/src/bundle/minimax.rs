// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `MiniMax` target: what `MiniMax` Agent's personal skill upload (Create →
//! Upload a skill → "Saved to My skills") accepts.
//!
//! The uploader takes the open Agent Skills layout as the ChatGPT target
//! builds it: one top-level `<name>/` folder whose `SKILL.md` carries `name`
//! and `description`. Verified on 2026-10-06 by uploading the nested
//! `chatgpt` bundle. The archive itself is built by the shared nested-archive
//! path; this module owns only the platform's one stated rule — "`.zip` or
//! `.skill` file must include exactly one `SKILL.md` file" — which a skill
//! breaks when a `references/` or `assets/` subtree carries a second one.
//!
//! `MiniMax` publishes no size or file-count limits for this uploader, so none
//! is enforced here; inventing one would refuse bundles the platform accepts.
//!
//! This target never produces a marketplace plugin package
//! (`.minimax-plugin/plugin.json`): that route is a public submission, which
//! Standard §6.4 gates on the maintainer's explicit authorization.

use serde_json::json;

use super::{Members, Problem, SKILL_MD};

/// Check that exactly one member of the projected skill is named `SKILL.md`,
/// at any depth. `members` are keyed by path relative to the skill root, so
/// the top-level file is the key `SKILL.md` itself.
///
/// # Errors
///
/// One `extra_skill_md` problem naming every nested `SKILL.md` path. A skill
/// without a top-level `SKILL.md` never reaches this point (collection
/// refuses it first).
pub(crate) fn check_members(skill: &str, members: &Members) -> Result<(), Vec<Problem>> {
    let nested: Vec<&str> = members
        .keys()
        .map(String::as_str)
        .filter(|path| *path != SKILL_MD && path.rsplit('/').next() == Some(SKILL_MD))
        .collect();
    if nested.is_empty() {
        return Ok(());
    }
    Err(vec![Problem::conflict(
        "minimax",
        format!(
            "{skill}: the minimax bundle must carry exactly one {SKILL_MD}, found {} more",
            nested.len()
        ),
        format!("construct skill build {skill} --target minimax --dry-run --verbose"),
        json!({ "skill": skill, "kind": "extra_skill_md", "detail": { "paths": nested } }),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Member;

    fn members(paths: &[&str]) -> Members {
        paths
            .iter()
            .map(|p| ((*p).to_owned(), Member::text("x")))
            .collect()
    }

    #[test]
    fn one_skill_md_is_accepted() {
        let m = members(&[SKILL_MD, "LICENSE", "references/guide.md"]);
        assert!(check_members("s", &m).is_ok());
    }

    #[test]
    fn a_file_merely_ending_in_skill_md_is_not_counted() {
        let m = members(&[SKILL_MD, "references/MY-SKILL.md", "references/notSKILL.md"]);
        assert!(check_members("s", &m).is_ok());
    }

    #[test]
    fn every_nested_skill_md_is_named() {
        let m = members(&[SKILL_MD, "assets/template/SKILL.md", "references/SKILL.md"]);
        let err = check_members("s", &m).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].key, "minimax");
        assert_eq!(err[0].detail["kind"], "extra_skill_md");
        assert_eq!(
            err[0].detail["detail"]["paths"],
            json!(["assets/template/SKILL.md", "references/SKILL.md"])
        );
    }
}
