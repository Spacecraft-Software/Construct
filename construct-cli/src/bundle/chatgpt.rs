// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ChatGPT target: what ChatGPT's skill upload (Skills → Create → Upload
//! from your computer) accepts.
//!
//! ChatGPT follows the open Agent Skills layout: one top-level `<name>/`
//! folder holding exactly one `SKILL.md`, whose frontmatter is validated
//! against the Agent Skills specification. That is the Claude layout minus
//! Claude Code's own invocation controls, so the archive itself is built by
//! the shared nested-archive path; this module owns only the platform's
//! published size limits, which no other target shares.
//!
//! The limits are OpenAI's documented Skills limits. The web uploader's
//! per-file cap of 25 MB is the binding one in practice; the other two are
//! enforced so a bundle never fails at upload for a reason the build could
//! have reported.

use serde_json::json;

use super::{Members, Problem};

/// The largest uncompressed file a skill may carry, in bytes. OpenAI states
/// the limit as "25 MB"; the decimal reading is the smaller of the two, so it
/// is the one that can never be exceeded by accident.
pub(crate) const CHATGPT_MAX_MEMBER_BYTES: usize = 25_000_000;

/// The largest skill archive, in bytes (OpenAI: "Maximum zip upload size is
/// 50 MB"; decimal for the same reason as [`CHATGPT_MAX_MEMBER_BYTES`]).
pub(crate) const CHATGPT_MAX_ARCHIVE_BYTES: usize = 50_000_000;

/// The most files one skill version may carry (OpenAI: "Maximum file count
/// per skill version is 500").
pub(crate) const CHATGPT_MAX_FILES: usize = 500;

/// A ChatGPT refusal (`chatgpt` extension, `{skill, kind, detail}`).
fn problem(skill: &str, kind: &str, message: String, detail: &serde_json::Value) -> Problem {
    Problem::conflict(
        "chatgpt",
        message,
        format!("construct skill build {skill} --target chatgpt --dry-run --verbose"),
        json!({ "skill": skill, "kind": kind, "detail": detail }),
    )
}

/// Check the members of one skill against the file-count and per-file limits.
/// `members` carry the projected `SKILL.md`, exactly as they will be zipped.
///
/// # Errors
///
/// `too_many_files` when the skill is over [`CHATGPT_MAX_FILES`], and one
/// `file_too_large` per member over [`CHATGPT_MAX_MEMBER_BYTES`].
pub(crate) fn check_members(skill: &str, members: &Members) -> Result<(), Vec<Problem>> {
    let mut problems = Vec::new();
    if members.len() > CHATGPT_MAX_FILES {
        problems.push(problem(
            skill,
            "too_many_files",
            format!(
                "{skill}: the chatgpt bundle has {} files, over the limit of {CHATGPT_MAX_FILES}",
                members.len()
            ),
            &json!({ "files": members.len(), "max": CHATGPT_MAX_FILES }),
        ));
    }
    for (path, member) in members {
        let size = member.bytes.len();
        if size > CHATGPT_MAX_MEMBER_BYTES {
            problems.push(problem(
                skill,
                "file_too_large",
                format!(
                    "{skill}: {path} is {size} bytes, over the per-file limit of \
                     {CHATGPT_MAX_MEMBER_BYTES}"
                ),
                &json!({ "path": path, "bytes": size, "max": CHATGPT_MAX_MEMBER_BYTES }),
            ));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Check one encoded archive against the upload-size limit.
///
/// # Errors
///
/// `archive_too_large` when `bytes` exceeds [`CHATGPT_MAX_ARCHIVE_BYTES`].
pub(crate) fn check_archive(
    skill: &str,
    file_name: &str,
    bytes: usize,
) -> Result<(), Vec<Problem>> {
    if bytes > CHATGPT_MAX_ARCHIVE_BYTES {
        return Err(vec![problem(
            skill,
            "archive_too_large",
            format!(
                "{skill}: {file_name} is {bytes} bytes, over the upload limit of \
                 {CHATGPT_MAX_ARCHIVE_BYTES}"
            ),
            &json!({ "path": file_name, "bytes": bytes, "max": CHATGPT_MAX_ARCHIVE_BYTES }),
        )]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Member;

    fn members(n: usize, size: usize) -> Members {
        (0..n)
            .map(|i| {
                (
                    format!("references/f{i:03}.md"),
                    Member {
                        bytes: vec![b'x'; size],
                        mode: 0o644,
                    },
                )
            })
            .collect()
    }

    fn kinds(problems: &[Problem]) -> Vec<String> {
        problems
            .iter()
            .map(|p| p.detail["kind"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    #[test]
    fn at_the_limits_is_accepted() {
        assert!(check_members("s", &members(CHATGPT_MAX_FILES, 1)).is_ok());
        assert!(check_members("s", &members(1, CHATGPT_MAX_MEMBER_BYTES)).is_ok());
        assert!(check_archive("s", "s.zip", CHATGPT_MAX_ARCHIVE_BYTES).is_ok());
    }

    #[test]
    fn too_many_files_is_refused() {
        let err = check_members("s", &members(CHATGPT_MAX_FILES + 1, 1)).unwrap_err();
        assert_eq!(kinds(&err), vec!["too_many_files"]);
    }

    #[test]
    fn every_oversized_file_is_named() {
        let err = check_members("s", &members(2, CHATGPT_MAX_MEMBER_BYTES + 1)).unwrap_err();
        assert_eq!(kinds(&err), vec!["file_too_large", "file_too_large"]);
        assert_eq!(err[0].detail["detail"]["path"], "references/f000.md");
    }

    #[test]
    fn oversized_archive_is_refused() {
        let err = check_archive("s", "s.zip", CHATGPT_MAX_ARCHIVE_BYTES + 1).unwrap_err();
        assert_eq!(kinds(&err), vec!["archive_too_large"]);
        assert_eq!(err[0].key, "chatgpt");
    }
}
