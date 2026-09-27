// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The directory-tree sink behind `construct skill vendor`.
//!
//! A vendored skill lands at `<repo>/.claude/skills/<name>/` as plain files,
//! plus an ownership [`MARKER`] listing every file the command wrote. The
//! marker carries no timestamp and no source path: a timestamp would make every
//! re-vendor a diff, and an absolute path would leak a personal filesystem
//! path into the consumer's committed tree.
//!
//! [`assess`] classifies a destination without following symlinks;
//! [`write`] stages the whole tree in a sibling directory, re-reads and
//! verifies every file, and only then swaps it into place, so an I/O failure
//! never leaves a half-written skill. Nothing here runs git.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::sink::{clear, swap, sweep_stale, WriteError, Written};
use super::Members;

/// Ownership marker written into every vendored skill directory.
pub(crate) const MARKER: &str = ".construct-vendor.toml";

/// The only layout `vendor` writes.
const LAYOUT: &str = "claude";

/// The marker's `tool` value.
const TOOL: &str = "construct";

/// The marker's serialized shape.
#[derive(Debug, Serialize, Deserialize)]
struct Marker {
    tool: String,
    skill: String,
    layout: String,
    files: Vec<String>,
}

/// The marker text for `skill` owning exactly `members` (byte-sorted by
/// `Members`' own order, marker excluded).
pub(crate) fn marker_text(skill: &str, members: &Members) -> String {
    let marker = Marker {
        tool: TOOL.to_owned(),
        skill: skill.to_owned(),
        layout: LAYOUT.to_owned(),
        files: members.keys().cloned().collect(),
    };
    // Serializing a flat struct of strings cannot fail; fall back to an empty
    // body rather than panic, which the re-read would then reject.
    let body = toml::to_string(&marker).unwrap_or_default();
    format!(
        "# Written by `construct skill vendor`. Do not edit; re-run the command instead.\n{body}"
    )
}

/// What is at a skill's destination before anything is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DirState {
    /// Nothing there.
    Missing,
    /// A directory carrying this skill's marker, holding only files the
    /// marker lists. `unchanged` when every byte, mode, and the marker itself
    /// already match what would be written.
    Owned {
        /// Whether a write would be a no-op.
        unchanged: bool,
    },
    /// A directory carrying this skill's marker, but also holding files the
    /// marker does not list (or symlinks, which `vendor` never writes).
    OwnedWithExtras(Vec<String>),
    /// A directory without a valid marker for this skill.
    Foreign,
    /// A symlink, which is never followed, replaced, or removed.
    Symlink,
    /// Some other non-directory entry.
    NotADirectory,
}

/// Classify `dest` for `skill`, which would receive `members`. Never follows
/// a symlink.
///
/// # Errors
///
/// Any I/O error other than "not found" while inspecting `dest`.
pub(crate) fn assess(dest: &Path, skill: &str, members: &Members) -> io::Result<DirState> {
    let meta = match fs::symlink_metadata(dest) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(DirState::Missing),
        Err(err) => return Err(err),
    };
    if meta.file_type().is_symlink() {
        return Ok(DirState::Symlink);
    }
    if !meta.is_dir() {
        return Ok(DirState::NotADirectory);
    }
    let Some(listed) = read_marker(dest, skill)? else {
        return Ok(DirState::Foreign);
    };

    let mut on_disk: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut extras: Vec<String> = Vec::new();
    walk(dest, dest, &mut on_disk, &mut extras)?;
    on_disk.remove(MARKER);
    let listed: BTreeSet<String> = listed.into_iter().collect();
    extras.extend(on_disk.keys().filter(|p| !listed.contains(*p)).cloned());
    if !extras.is_empty() {
        extras.sort();
        return Ok(DirState::OwnedWithExtras(extras));
    }

    let unchanged = on_disk.len() == members.len()
        && fs::read(dest.join(MARKER))? == marker_text(skill, members).as_bytes()
        && members.iter().all(|(rel, member)| {
            on_disk
                .get(rel)
                .is_some_and(|path| file_matches(path, &member.bytes, member.mode))
        });
    Ok(DirState::Owned { unchanged })
}

/// The file list of a valid marker for `skill` at `dest`, or `None` when the
/// marker is missing, not a regular file, unparsable, or names another skill,
/// tool, or layout.
fn read_marker(dest: &Path, skill: &str) -> io::Result<Option<Vec<String>>> {
    let path = dest.join(MARKER);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() => {}
        Ok(_) => return Ok(None),
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    }
    let text = fs::read_to_string(&path)?;
    let Ok(marker) = toml::from_str::<Marker>(&text) else {
        return Ok(None);
    };
    let ours = marker.tool == TOOL && marker.skill == skill && marker.layout == LAYOUT;
    Ok(ours.then_some(marker.files))
}

/// Every non-directory entry under `dir`, keyed by its `/`-separated path
/// relative to `root`. Symlinks are never followed; each one is reported in
/// `symlinks` instead.
fn walk(
    root: &Path,
    dir: &Path,
    files: &mut BTreeMap<String, PathBuf>,
    symlinks: &mut Vec<String>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        let rel = relative(root, &path);
        if kind.is_symlink() {
            symlinks.push(rel);
        } else if kind.is_dir() {
            walk(root, &path, files, symlinks)?;
        } else {
            files.insert(rel, path);
        }
    }
    Ok(())
}

/// `path` relative to `root`, `/`-separated.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Whether the file at `path` holds exactly `bytes` with the exec bit `mode`
/// implies.
fn file_matches(path: &Path, bytes: &[u8], mode: u32) -> bool {
    fs::read(path).is_ok_and(|on_disk| on_disk == bytes) && mode_matches(path, mode)
}

#[cfg(unix)]
fn mode_matches(path: &Path, mode: u32) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    fs::metadata(path).is_ok_and(|m| (m.permissions().mode() & 0o111 != 0) == (mode & 0o111 != 0))
}

#[cfg(not(unix))]
fn mode_matches(_path: &Path, _mode: u32) -> bool {
    true
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Ok(())
}

/// Whether `rel` is a plain relative path that stays inside its root.
fn is_contained(rel: &str) -> bool {
    !rel.is_empty()
        && Path::new(rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Replace `<skills_dir>/<skill>/` with exactly `members` plus the marker.
///
/// Every file is staged in `<skills_dir>/.<skill>.tmp-<pid>/`, re-read, and
/// checked for byte-identity with its member and with `verify` (the vendored
/// palette check) before the staged directory is swapped into place: the
/// previous tree moves to `.<skill>.old-<pid>/` and back again if the second
/// rename fails. A failure at any step removes the staging directory and
/// leaves the previous tree intact. Once the new tree is in place, failing to
/// remove the old one is reported as [`Written::leftover`], not an error.
/// Stale staging and old directories an interrupted run left behind, from
/// any pid, are swept first when they carry this skill's marker. The caller
/// has already refused a symlink or non-directory at the destination, so the
/// swap only ever renames real directories.
///
/// # Errors
///
/// An I/O error naming the path, or the member paths that failed their
/// re-read.
pub(crate) fn write(
    skills_dir: &Path,
    skill: &str,
    members: &Members,
    verify: &dyn Fn(&str, &[u8]) -> bool,
) -> Result<Written, WriteError> {
    let dest = skills_dir.join(skill);
    let pid = std::process::id();
    let staging = skills_dir.join(format!(".{skill}.tmp-{pid}"));
    let old = skills_dir.join(format!(".{skill}.old-{pid}"));
    sweep_stale(skills_dir, skill, &|dir| {
        read_marker(dir, skill).is_ok_and(|m| m.is_some())
    });
    clear(&staging)?;
    clear(&old)?;

    let result = (|| {
        fs::create_dir_all(&staging).map_err(|e| WriteError::io(&staging, e))?;
        for (rel, member) in members {
            if !is_contained(rel) {
                return Err(WriteError::Verify(vec![rel.clone()]));
            }
            let path = staging.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| WriteError::io(parent, e))?;
            }
            fs::write(&path, &member.bytes).map_err(|e| WriteError::io(&path, e))?;
            set_mode(&path, member.mode).map_err(|e| WriteError::io(&path, e))?;
        }
        let marker = staging.join(MARKER);
        fs::write(&marker, marker_text(skill, members)).map_err(|e| WriteError::io(&marker, e))?;

        let mut failed = Vec::new();
        for (rel, member) in members {
            let path = staging.join(rel);
            let on_disk = fs::read(&path).map_err(|e| WriteError::io(&path, e))?;
            if on_disk != member.bytes || !verify(rel, &on_disk) {
                failed.push(rel.clone());
            }
        }
        if !failed.is_empty() {
            return Err(WriteError::Verify(failed));
        }
        swap(&staging, &dest, &old, MARKER)
    })();
    if result.is_err() {
        let _ = clear(&staging);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{assess, is_contained, marker_text, write, DirState, MARKER};
    use crate::bundle::{Member, Members};

    fn members() -> Members {
        let mut m = Members::new();
        m.insert("SKILL.md".to_owned(), Member::text("---\nname: s\n---\n"));
        m.insert("references/a.md".to_owned(), Member::text("# A\n"));
        m
    }

    #[test]
    fn marker_is_stable_and_lists_files_sorted() {
        let text = marker_text("s", &members());
        assert_eq!(text, marker_text("s", &members()));
        assert!(text.contains("files = [\"SKILL.md\", \"references/a.md\"]"));
        assert!(!text.contains("20"), "no timestamp: {text}");
    }

    #[test]
    fn write_then_assess_is_unchanged_and_prunes_on_rewrite() {
        let t = TempDir::new().unwrap();
        let skills = t.path();
        let dest = write(skills, "s", &members(), &|_, _| true).unwrap().dir;
        assert!(dest.join(MARKER).is_file());
        assert_eq!(
            assess(&dest, "s", &members()).unwrap(),
            DirState::Owned { unchanged: true }
        );
        let mut fewer = members();
        fewer.remove("references/a.md");
        assert_eq!(
            assess(&dest, "s", &fewer).unwrap(),
            DirState::Owned { unchanged: false }
        );
        write(skills, "s", &fewer, &|_, _| true).unwrap();
        assert!(!dest.join("references/a.md").exists());
        let leftovers: Vec<_> = fs::read_dir(skills).unwrap().flatten().collect();
        assert_eq!(leftovers.len(), 1, "no staging dirs left behind");
    }

    #[test]
    fn foreign_and_extras_are_detected() {
        let t = TempDir::new().unwrap();
        let dest = t.path().join("s");
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("mine.md"), "x").unwrap();
        assert_eq!(assess(&dest, "s", &members()).unwrap(), DirState::Foreign);
        write(t.path(), "s", &members(), &|_, _| true).unwrap();
        fs::write(dest.join("mine.md"), "x").unwrap();
        assert_eq!(
            assess(&dest, "s", &members()).unwrap(),
            DirState::OwnedWithExtras(vec!["mine.md".to_owned()])
        );
        // A marker for another skill does not confer ownership.
        assert_eq!(
            assess(&dest, "other", &members()).unwrap(),
            DirState::Foreign
        );
    }

    #[test]
    fn failed_verify_leaves_previous_tree() {
        let t = TempDir::new().unwrap();
        write(t.path(), "s", &members(), &|_, _| true).unwrap();
        let mut changed = members();
        changed.insert("references/a.md".to_owned(), Member::text("# Changed\n"));
        let err = write(t.path(), "s", &changed, &|rel, _| rel != "references/a.md");
        assert!(err.is_err());
        let leftovers: Vec<_> = fs::read_dir(t.path()).unwrap().flatten().collect();
        assert_eq!(leftovers.len(), 1, "staging removed after failure");
        assert_eq!(
            fs::read_to_string(t.path().join("s/references/a.md")).unwrap(),
            "# A\n",
            "the previous tree survives"
        );
    }

    #[test]
    fn containment_rejects_traversal() {
        assert!(is_contained("references/a.md"));
        assert!(!is_contained("../x"));
        assert!(!is_contained("/etc/passwd"));
        assert!(!is_contained("a/../../x"));
        assert!(!is_contained(""));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_destination_is_never_followed() {
        let t = TempDir::new().unwrap();
        let target = t.path().join("elsewhere");
        fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, t.path().join("s")).unwrap();
        assert_eq!(
            assess(&t.path().join("s"), "s", &members()).unwrap(),
            DirState::Symlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_failure_after_swap_is_a_leftover_and_is_swept_later() {
        use std::os::unix::fs::PermissionsExt as _;
        let t = TempDir::new().unwrap();
        let dest = write(t.path(), "s", &members(), &|_, _| true).unwrap().dir;
        // A read-only subdirectory makes the old tree unremovable after the
        // swap (the renames themselves only need the parent writable).
        let refs = dest.join("references");
        fs::set_permissions(&refs, fs::Permissions::from_mode(0o555)).unwrap();
        let mut changed = members();
        changed.insert("references/a.md".to_owned(), Member::text("# New\n"));
        let written = write(t.path(), "s", &changed, &|_, _| true).expect("swap succeeds");
        assert_eq!(
            fs::read_to_string(dest.join("references/a.md")).unwrap(),
            "# New\n"
        );
        let Some((old, _)) = written.leftover else {
            // Running as root: the removal cannot fail, nothing to sweep.
            return;
        };
        assert!(old.is_dir());
        fs::set_permissions(old.join("references"), fs::Permissions::from_mode(0o755)).unwrap();
        // A stale dir from another pid is swept only when it is ours.
        let stale = t.path().join(".s.old-1");
        fs::rename(&old, &stale).unwrap();
        let foreign = t.path().join(".s.tmp-2");
        fs::create_dir_all(&foreign).unwrap();
        fs::write(foreign.join("mine.md"), "x").unwrap();
        write(t.path(), "s", &changed, &|_, _| true).unwrap();
        assert!(!stale.exists(), "our stale tree is swept");
        assert!(foreign.join("mine.md").exists(), "a foreign dir is kept");
    }
}
