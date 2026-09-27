// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Deterministic zip encoding and atomic writes under `dist/<target>/`.
//!
//! # Reproducibility
//!
//! Identical members give identical bytes, given an identical `Cargo.lock`:
//! entries are written in byte order of their full archive path, every entry
//! carries the fixed [`BUNDLE_EPOCH`] timestamp and an explicit mode, deflate
//! runs at a fixed level, and no comment or extra field is written. A
//! compressor upgrade in the lock may change bytes; the lock is committed and
//! CI builds from it. No artifact ever carries a wall-clock timestamp.
//!
//! # Ownership
//!
//! Each target directory holds a [`MARKER`] file. A directory without one is
//! only replaced when it is empty or the caller passed `--force`, so a build
//! never deletes files it did not create. Only a directory this command
//! created, an empty one, or a full `--force` replacement ever receives the
//! marker; a partial build never adopts a foreign directory.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read as _, Write as _};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use super::{Members, Target};

/// The fixed timestamp of every entry: 2026-01-01T00:00:00, the same instant
/// `perplexity-skills/build.py` has always used (`ZIP_DATE`).
pub(crate) const BUNDLE_EPOCH: (u16, u8, u8, u8, u8, u8) = (2026, 1, 1, 0, 0, 0);

/// Deflate level: zlib's default, fixed so output does not follow a crate
/// default.
const DEFLATE_LEVEL: i64 = 6;

/// Ownership marker written into every target directory.
pub(crate) const MARKER: &str = ".construct-build";

/// The marker's content: no timestamp, so rebuilding is byte-stable.
fn marker_text(target: Target) -> String {
    format!(
        "# Written by `construct skill build`. Do not edit; the directory is regenerated.\ntool = \"construct\"\ntarget = \"{}\"\n",
        target.slug()
    )
}

/// One entry in archive order.
enum Entry<'a> {
    Dir,
    File(&'a [u8], u32),
}

/// Encode `members` into a zip under `prefix` (`<name>/`, or empty for a
/// flat layout). `dir_entries` adds an entry for every ancestor directory,
/// as `zip -r` does; without it the archive holds files only (`zip -D`).
///
/// Returns the bytes and the entry count.
///
/// # Errors
///
/// A [`zip::result::ZipError`] from the encoder (in-memory, so only on an
/// encoder bug or an out-of-range option).
pub(crate) fn encode_zip(
    members: &Members,
    prefix: &str,
    dir_entries: bool,
) -> Result<(Vec<u8>, usize), zip::result::ZipError> {
    let mut entries: BTreeMap<String, Entry<'_>> = BTreeMap::new();
    for (path, member) in members {
        let full = format!("{prefix}{path}");
        if dir_entries {
            let mut end = 0;
            while let Some(i) = full[end..].find('/') {
                end += i + 1;
                entries.insert(full[..end].to_owned(), Entry::Dir);
            }
        }
        entries.insert(full, Entry::File(&member.bytes, member.mode));
    }

    let (y, mo, d, h, mi, s) = BUNDLE_EPOCH;
    let epoch = DateTime::from_date_and_time(y, mo, d, h, mi, s)
        .map_err(|_| zip::result::ZipError::InvalidArchive("bundle epoch out of range".into()))?;
    let base = SimpleFileOptions::default()
        .last_modified_time(epoch)
        .large_file(false);

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, entry) in &entries {
        match entry {
            Entry::Dir => writer.add_directory(name.as_str(), base.unix_permissions(0o755))?,
            Entry::File(bytes, mode) => {
                writer.start_file(
                    name.as_str(),
                    base.compression_method(CompressionMethod::Deflated)
                        .compression_level(Some(DEFLATE_LEVEL))
                        .unix_permissions(*mode),
                )?;
                writer.write_all(bytes)?;
            }
        }
    }
    let bytes = writer.finish()?.into_inner();
    Ok((bytes, entries.len()))
}

/// Read one entry of an in-memory zip, or `None` when absent or unreadable.
pub(crate) fn read_member(zip_bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut out = Vec::new();
    file.read_to_end(&mut out).ok()?;
    Some(out)
}

/// Who owns a target directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ownership {
    /// Does not exist yet.
    Absent,
    /// Exists and is empty.
    Empty,
    /// Carries the [`MARKER`].
    Owned,
    /// Holds files and no marker.
    Foreign,
}

/// Classify `dir` for the ownership check.
pub(crate) fn ownership(dir: &Path) -> Ownership {
    if !dir.exists() {
        return Ownership::Absent;
    }
    if dir.join(MARKER).is_file() {
        return Ownership::Owned;
    }
    let empty = fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none());
    if empty {
        Ownership::Empty
    } else {
        Ownership::Foreign
    }
}

/// One file to write into a target directory.
#[derive(Debug)]
pub(crate) struct OutFile<'a> {
    /// File name inside the target directory.
    pub(crate) name: &'a str,
    /// The bytes.
    pub(crate) bytes: &'a [u8],
}

/// Write a file through a temp sibling + rename, so a reader never sees a
/// partial file.
fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dir.join(name))
}

/// A successful write.
#[derive(Debug)]
pub(crate) struct Written {
    /// The directory now holding the output.
    pub(crate) dir: PathBuf,
    /// The previous output, moved aside by the swap, that could not be
    /// removed afterwards. The new output is already in place, so this is a
    /// warning for the caller to report, never a failure.
    pub(crate) leftover: Option<(PathBuf, std::io::Error)>,
}

/// Replace `<out>/<target>/` wholesale: stage every file in a sibling
/// directory, run `verify` over the staged files, then swap the staged
/// directory into place. The swap renames the previous output aside first
/// and restores it if the second rename fails, so a failure at any step
/// leaves the previous output intact. `verify` returns the names that failed;
/// any failure aborts the swap.
///
/// A full replacement is the one write that adopts a directory without a
/// marker: the caller reaches it for a foreign directory only under
/// `--force`, which is the user asking for exactly that.
///
/// # Errors
///
/// An I/O error naming the path, or the names `verify` rejected.
pub(crate) fn write_full(
    out: &Path,
    target: Target,
    files: &[OutFile<'_>],
    verify: &dyn Fn(&str, &[u8]) -> bool,
) -> Result<Written, WriteError> {
    fs::create_dir_all(out).map_err(|e| WriteError::io(out, e))?;
    let slug = target.slug();
    let dest = out.join(slug);
    let pid = std::process::id();
    let staging = out.join(format!(".{slug}.tmp-{pid}"));
    let old = out.join(format!(".{slug}.old-{pid}"));
    sweep_stale(out, slug, &|dir| dir.join(MARKER).is_file());
    clear(&staging)?;
    clear(&old)?;
    fs::create_dir_all(&staging).map_err(|e| WriteError::io(&staging, e))?;
    let result = (|| {
        for f in files {
            fs::write(staging.join(f.name), f.bytes)
                .map_err(|e| WriteError::io(&staging.join(f.name), e))?;
        }
        fs::write(staging.join(MARKER), marker_text(target))
            .map_err(|e| WriteError::io(&staging.join(MARKER), e))?;
        let failed = reread_failures(&staging, files, verify)?;
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

/// Move `staging` to `dest`, moving any previous `dest` to `old` first and
/// back again if the second rename fails. Only a failure to remove `old`
/// after a successful swap is left for the caller, as [`Written::leftover`];
/// `old` is removed with its `marker` last, so a half-removed tree still
/// carries it and a later [`sweep_stale`] recognizes it.
pub(crate) fn swap(
    staging: &Path,
    dest: &Path,
    old: &Path,
    marker: &str,
) -> Result<Written, WriteError> {
    let had_dest = fs::symlink_metadata(dest).is_ok();
    if had_dest {
        fs::rename(dest, old).map_err(|e| WriteError::io(dest, e))?;
    }
    if let Err(err) = fs::rename(staging, dest) {
        if had_dest {
            let _ = fs::rename(old, dest);
        }
        return Err(WriteError::io(dest, err));
    }
    let leftover = if had_dest {
        remove_marker_last(old, marker)
            .err()
            .map(|e| (old.to_path_buf(), e))
    } else {
        None
    };
    Ok(Written {
        dir: dest.to_path_buf(),
        leftover,
    })
}

/// Remove `dir` and everything in it, its `marker` file last. Symlinks inside
/// are unlinked, never followed.
fn remove_marker_last(dir: &Path, marker: &str) -> std::io::Result<()> {
    // A symlinked output moved aside is unlinked, never emptied through.
    if fs::symlink_metadata(dir)?.file_type().is_symlink() {
        return fs::remove_file(dir);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name() == marker {
            continue;
        }
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    match fs::remove_file(dir.join(marker)) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    fs::remove_dir(dir)
}

/// Remove a leftover directory from an earlier crashed run. A symlink there
/// is unlinked, never followed.
///
/// # Errors
///
/// Any I/O error other than "not found", naming the path.
pub(crate) fn clear(path: &Path) -> Result<(), WriteError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
    .map_err(|e| WriteError::io(path, e))
}

/// Remove the `.<name>.tmp-<pid>` and `.<name>.old-<pid>` directories an
/// earlier interrupted run left in `parent`, whatever its pid, but only the
/// real directories `owned` recognizes by their marker. Best effort: an entry
/// that cannot be read or removed stays for the next run, and never fails
/// the current one.
pub(crate) fn sweep_stale(parent: &Path, name: &str, owned: &dyn Fn(&Path) -> bool) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries {
        // Best effort by design: an unreadable entry is simply not swept.
        let Ok(entry) = entry else { continue };
        let file_name = entry.file_name();
        let Some(pid) = file_name
            .to_str()
            .and_then(|n| n.strip_prefix('.'))
            .and_then(|n| n.strip_prefix(name))
            .and_then(|n| n.strip_prefix(".tmp-").or_else(|| n.strip_prefix(".old-")))
        else {
            continue;
        };
        if pid.is_empty() || !pid.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let path = entry.path();
        let real_dir = fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir());
        if real_dir && owned(&path) {
            let _ = fs::remove_dir_all(&path);
        }
    }
}

/// Write only `files` into `<out>/<target>/`, leaving other files alone.
/// Each file goes through temp + rename; a file whose re-read fails `verify`
/// is removed again.
///
/// The ownership [`MARKER`] is written only into a directory this call
/// creates, or one that was empty. A foreign directory (reached under
/// `--force`) receives the files but is never adopted: adopting it would let
/// a later full build delete files this command did not create.
///
/// # Errors
///
/// An I/O error naming the path, or the names `verify` rejected.
pub(crate) fn write_partial(
    out: &Path,
    target: Target,
    files: &[OutFile<'_>],
    verify: &dyn Fn(&str, &[u8]) -> bool,
) -> Result<Written, WriteError> {
    let dest = out.join(target.slug());
    let adopt = matches!(ownership(&dest), Ownership::Absent | Ownership::Empty);
    fs::create_dir_all(&dest).map_err(|e| WriteError::io(&dest, e))?;
    if adopt {
        write_atomic(&dest, MARKER, marker_text(target).as_bytes())
            .map_err(|e| WriteError::io(&dest.join(MARKER), e))?;
    }
    for f in files {
        write_atomic(&dest, f.name, f.bytes).map_err(|e| WriteError::io(&dest.join(f.name), e))?;
    }
    let failed = reread_failures(&dest, files, verify)?;
    if !failed.is_empty() {
        for name in &failed {
            let _ = fs::remove_file(dest.join(name));
        }
        return Err(WriteError::Verify(failed));
    }
    Ok(Written {
        dir: dest,
        leftover: None,
    })
}

/// Re-read every written file from disk and collect the names `verify`
/// rejects.
fn reread_failures(
    dir: &Path,
    files: &[OutFile<'_>],
    verify: &dyn Fn(&str, &[u8]) -> bool,
) -> Result<Vec<String>, WriteError> {
    let mut failed = Vec::new();
    for f in files {
        let path = dir.join(f.name);
        let on_disk = fs::read(&path).map_err(|e| WriteError::io(&path, e))?;
        if !verify(f.name, &on_disk) {
            failed.push(f.name.to_owned());
        }
    }
    Ok(failed)
}

/// A write-phase failure.
#[derive(Debug)]
pub(crate) enum WriteError {
    /// An I/O failure at `path`.
    Io {
        /// Where it failed.
        path: PathBuf,
        /// The OS error.
        err: std::io::Error,
    },
    /// Files whose re-read content failed verification.
    Verify(Vec<String>),
}

impl WriteError {
    pub(crate) fn io(path: &Path, err: std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            err,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use zip::{DateTime, ZipArchive};

    use super::{encode_zip, write_full, OutFile, BUNDLE_EPOCH, MARKER};
    use crate::bundle::{Member, Members, Target};

    fn members() -> Members {
        let mut m = Members::new();
        m.insert("SKILL.md".to_owned(), Member::text("---\nname: a\n---\n"));
        m.insert("references/x.md".to_owned(), Member::text("x\n"));
        m.insert(
            "assets/run.sh".to_owned(),
            Member {
                bytes: b"#!/bin/sh\n".to_vec(),
                mode: 0o755,
            },
        );
        m
    }

    fn names(bytes: &[u8]) -> Vec<String> {
        let mut a = ZipArchive::new(Cursor::new(bytes)).unwrap();
        (0..a.len())
            .map(|i| a.by_index(i).unwrap().name().to_owned())
            .collect()
    }

    #[test]
    fn encoding_is_byte_reproducible() {
        assert_eq!(
            encode_zip(&members(), "a/", true).unwrap(),
            encode_zip(&members(), "a/", true).unwrap()
        );
    }

    #[test]
    fn entries_carry_the_epoch_no_extra_data_and_their_mode() {
        let (bytes, count) = encode_zip(&members(), "a/", true).unwrap();
        let (year, month, day, hour, minute, second) = BUNDLE_EPOCH;
        let epoch = DateTime::from_date_and_time(year, month, day, hour, minute, second).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes.as_slice())).unwrap();
        assert_eq!(archive.len(), count);
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            assert_eq!(file.last_modified(), Some(epoch), "{}", file.name());
            assert!(
                file.extra_data().is_none_or(<[u8]>::is_empty),
                "{}",
                file.name()
            );
            if file.name() == "a/assets/run.sh" {
                assert_eq!(file.unix_mode().map(|m| m & 0o777), Some(0o755));
            }
        }
    }

    #[test]
    fn zip_has_sorted_dir_entries_and_skill_has_none() {
        let (with, _) = encode_zip(&members(), "a/", true).unwrap();
        assert_eq!(
            names(&with),
            vec![
                "a/",
                "a/SKILL.md",
                "a/assets/",
                "a/assets/run.sh",
                "a/references/",
                "a/references/x.md"
            ]
        );
        let (without, _) = encode_zip(&members(), "", false).unwrap();
        assert_eq!(
            names(&without),
            vec!["SKILL.md", "assets/run.sh", "references/x.md"]
        );
    }

    /// The swap renames the previous output aside before moving the new one
    /// in, so a failed swap leaves the previous output whole (it used to be
    /// deleted first, then lost when the rename failed).
    #[cfg(unix)]
    #[test]
    fn failed_swap_keeps_the_previous_output() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt as _;
        let t = tempfile::TempDir::new().unwrap();
        let out = t.path().join("dist");
        let old = [OutFile {
            name: "a.zip",
            bytes: b"old",
        }];
        write_full(&out, Target::Claude, &old, &|_, _| true).unwrap();
        let new = [OutFile {
            name: "a.zip",
            bytes: b"new",
        }];
        // `verify` runs after staging, right before the swap: a read-only
        // `out` makes every rename inside it fail.
        let lock = |_: &str, _: &[u8]| -> bool {
            fs::set_permissions(&out, fs::Permissions::from_mode(0o555)).unwrap();
            true
        };
        let result = write_full(&out, Target::Claude, &new, &lock);
        fs::set_permissions(&out, fs::Permissions::from_mode(0o755)).unwrap();
        if result.is_ok() {
            // Running as root: permissions do not bind, nothing to test.
            return;
        }
        assert_eq!(fs::read(out.join("claude/a.zip")).unwrap(), b"old");
        assert!(out.join("claude").join(MARKER).is_file());
    }

    /// A symlinked target directory is replaced by a real one; the swap
    /// unlinks the link and never empties the directory it points to.
    #[cfg(unix)]
    #[test]
    fn symlinked_output_is_unlinked_not_emptied() {
        use std::fs;
        let t = tempfile::TempDir::new().unwrap();
        let elsewhere = t.path().join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join(MARKER), "x").unwrap();
        fs::write(elsewhere.join("keep.txt"), "keep").unwrap();
        let out = t.path().join("dist");
        fs::create_dir_all(&out).unwrap();
        std::os::unix::fs::symlink(&elsewhere, out.join("claude")).unwrap();
        let files = [OutFile {
            name: "a.zip",
            bytes: b"new",
        }];
        let written = write_full(&out, Target::Claude, &files, &|_, _| true).unwrap();
        assert!(written.leftover.is_none());
        assert!(elsewhere.join(MARKER).is_file());
        assert!(elsewhere.join("keep.txt").is_file());
        let meta = fs::symlink_metadata(out.join("claude")).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
    }
}
