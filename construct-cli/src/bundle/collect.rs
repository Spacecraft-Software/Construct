// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Skill directory → in-memory [`Members`].
//!
//! Keeps exactly the drift-sweep shippable set: `SKILL.md`, `CREDITS.md`,
//! `LICENSE`, `LICENSE.*` at the skill root, and everything under
//! `references/` and `assets/`. Anything else at the root (for example
//! `gnu-free-software/ATTRIBUTION.md`) is not shipped, matching today's
//! bundles. Dot-prefixed paths are skipped with a note, and symlinks are
//! refused: a symlink would make the bundle bytes depend on the host.
//!
//! # File modes
//!
//! A member's permission bits never come from the working tree, whose exec
//! bits depend on the host (`core.fileMode = false`, a Windows checkout, a
//! `chmod` nobody committed). When the catalogue is a git work tree, a file
//! is `0o755` exactly when the index records it as `100755`, read with one
//! `git ls-files -s -z` per build; every other file, and every file outside
//! a git work tree or without a usable `git`, is `0o644`. See [`Modes`].

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::json;

use super::{Member, Members, Note, NoteLevel, Problem, SKILL_MD};
use crate::catalogue::SkillRef;

/// Root-level file names that ship (besides `LICENSE.*`).
const ROOT_FILES: &[&str] = &[SKILL_MD, "CREDITS.md", "LICENSE"];

/// Subdirectories shipped recursively.
const TREES: &[&str] = &["references", "assets"];

/// Host-independent permission bits for collected files: the executable
/// set of the git index, or nothing (every file `0o644`).
#[derive(Debug, Default)]
pub(crate) struct Modes {
    /// The catalogue root the index paths are relative to.
    root: PathBuf,
    /// `/`-separated paths the index records as `100755`.
    executable: BTreeSet<String>,
}

impl Modes {
    /// Read the index of the git work tree at `repo` with one
    /// `git ls-files -s -z`. Outside a work tree, or when `git` cannot run,
    /// the set is empty and every file is `0o644`. This only reads the
    /// catalogue's index; it never writes to any repository.
    pub(crate) fn from_index(repo: &Path) -> Self {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["ls-files", "-s", "-z"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let executable = match output {
            Ok(out) if out.status.success() => parse_ls_files(&out.stdout),
            _ => BTreeSet::new(),
        };
        Self {
            root: repo.to_path_buf(),
            executable,
        }
    }

    /// The mode for the file at `path` (under the catalogue root).
    fn mode(&self, path: &Path) -> u32 {
        let rel = path.strip_prefix(&self.root).ok().map(|rel| {
            rel.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        });
        match rel {
            Some(rel) if self.executable.contains(&rel) => 0o755,
            _ => 0o644,
        }
    }
}

/// The paths `git ls-files -s -z` output records as `100755`. Each record is
/// `<mode> <object> <stage>\t<path>` terminated by NUL; a path is never
/// quoted under `-z`.
fn parse_ls_files(out: &[u8]) -> BTreeSet<String> {
    out.split(|b| *b == 0)
        .filter_map(|record| {
            let tab = record.iter().position(|b| *b == b'\t')?;
            let (meta, path) = (&record[..tab], &record[tab + 1..]);
            meta.starts_with(b"100755 ")
                .then(|| String::from_utf8_lossy(path).into_owned())
        })
        .collect()
}

/// Accumulated collection state for one skill.
struct Walk<'a> {
    skill: &'a str,
    modes: &'a Modes,
    members: Members,
    notes: Vec<Note>,
    problems: Vec<Problem>,
}

impl Walk<'_> {
    fn symlink(&mut self, rel: &str) {
        self.problems.push(Problem::conflict(
            "symlinks",
            format!(
                "{}/{rel} is a symlink; bundles ship regular files only",
                self.skill
            ),
            format!("$EDITOR {}/{rel}", self.skill),
            json!({ "skill": self.skill, "path": rel }),
        ));
    }

    /// The entry names of `dir`, or `None` when it cannot be listed. An entry
    /// that cannot be read is reported, never silently dropped.
    fn names(&mut self, dir: &Path, rel: &str) -> Option<Vec<String>> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => {
                self.io(rel, &err);
                return None;
            }
        };
        let mut names = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => names.push(entry.file_name().to_string_lossy().into_owned()),
                Err(err) => self.io(rel, &err),
            }
        }
        Some(names)
    }

    fn io(&mut self, rel: &str, err: &std::io::Error) {
        self.problems.push(Problem::internal(
            "io_failures",
            format!("cannot read {}/{rel}: {err}", self.skill),
            json!({ "skill": self.skill, "path": rel, "error": err.to_string() }),
        ));
    }

    fn file(&mut self, path: &Path, rel: String) {
        match fs::read(path) {
            Ok(bytes) => {
                self.members.insert(
                    rel,
                    Member {
                        bytes,
                        mode: self.modes.mode(path),
                    },
                );
            }
            Err(err) => self.io(&rel, &err),
        }
    }

    /// Recurse into a shipped subtree.
    fn tree(&mut self, dir: &Path, rel: &str) {
        let Some(mut names) = self.names(dir, rel) else {
            return;
        };
        names.sort();
        for name in names {
            let child_rel = format!("{rel}/{name}");
            if name.starts_with('.') {
                self.notes.push(Note {
                    level: NoteLevel::Info,
                    code: "DOTFILE_SKIPPED",
                    message: format!("{}: skipped dot-path {child_rel}", self.skill),
                    hint: None,
                    detail: json!({ "skill": self.skill, "path": child_rel }),
                });
                continue;
            }
            let path = dir.join(&name);
            let meta = match fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(err) => {
                    self.io(&child_rel, &err);
                    continue;
                }
            };
            if meta.file_type().is_symlink() {
                self.symlink(&child_rel);
            } else if meta.is_dir() {
                self.tree(&path, &child_rel);
            } else if meta.is_file() {
                self.file(&path, child_rel);
            }
        }
    }
}

/// Collect one skill: its `SKILL.md` text and its shippable members.
///
/// # Errors
///
/// Every symlink, unreadable file, non-UTF-8 `SKILL.md`, and a missing
/// license text (Standard §5.6 makes license carriage mandatory).
pub(crate) fn collect(
    skill: &SkillRef,
    modes: &Modes,
) -> Result<(String, Members, Vec<Note>), Vec<Problem>> {
    let mut walk = Walk {
        skill: &skill.name,
        modes,
        members: Members::new(),
        notes: Vec::new(),
        problems: Vec::new(),
    };
    let Some(mut names) = walk.names(&skill.dir, ".") else {
        return Err(walk.problems);
    };
    names.sort();
    for name in names {
        let ships_file = ROOT_FILES.contains(&name.as_str()) || name.starts_with("LICENSE.");
        let ships_tree = TREES.contains(&name.as_str());
        if !ships_file && !ships_tree {
            continue;
        }
        let path = skill.dir.join(&name);
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(err) => {
                walk.io(&name, &err);
                continue;
            }
        };
        if meta.file_type().is_symlink() {
            walk.symlink(&name);
        } else if ships_tree && meta.is_dir() {
            walk.tree(&path, &name);
        } else if ships_file && meta.is_file() {
            walk.file(&path, name);
        }
    }

    if !walk
        .members
        .keys()
        .any(|k| k == "LICENSE" || k.starts_with("LICENSE."))
    {
        walk.problems.push(Problem::conflict(
            "missing_license",
            format!(
                "{} has no LICENSE file; every bundle ships its license text",
                skill.name
            ),
            format!("cp LICENSE {}/LICENSE", skill.name),
            json!({ "skill": skill.name }),
        ));
    }

    let text = match walk
        .members
        .get(SKILL_MD)
        .map(|m| String::from_utf8(m.bytes.clone()))
    {
        Some(Ok(text)) => text,
        Some(Err(_)) => {
            walk.problems.push(Problem::conflict(
                "invalid_frontmatter",
                format!("{}/SKILL.md is not valid UTF-8", skill.name),
                format!("$EDITOR {}/SKILL.md", skill.name),
                json!({ "skill": skill.name, "error": "not valid UTF-8" }),
            ));
            String::new()
        }
        None => String::new(),
    };

    if walk.problems.is_empty() {
        Ok((text, walk.members, walk.notes))
    } else {
        Err(walk.problems)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use std::path::Path;

    use super::{collect, parse_ls_files, Modes};
    use crate::catalogue::{Origin, SkillRef};

    fn skill_ref(dir: &std::path::Path) -> SkillRef {
        SkillRef {
            name: "s".to_owned(),
            dir: dir.to_path_buf(),
            origin: Origin::Root,
        }
    }

    #[test]
    fn keeps_the_shippable_set_only() {
        let t = tempfile::TempDir::new().unwrap();
        let d = t.path();
        fs::write(d.join("SKILL.md"), "---\nname: s\n---\n").unwrap();
        fs::write(d.join("LICENSE.MIT"), "mit").unwrap();
        fs::write(d.join("ATTRIBUTION.md"), "not shipped").unwrap();
        fs::create_dir_all(d.join("references/deep")).unwrap();
        fs::write(d.join("references/deep/x.md"), "x").unwrap();
        fs::write(d.join("references/.hidden"), "h").unwrap();
        let (_, members, notes) = collect(&skill_ref(d), &Modes::default()).expect("collects");
        let keys: Vec<&str> = members.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["LICENSE.MIT", "SKILL.md", "references/deep/x.md"]
        );
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].code, "DOTFILE_SKIPPED");
    }

    #[test]
    fn missing_license_and_symlink_are_refused() {
        let t = tempfile::TempDir::new().unwrap();
        let d = t.path();
        fs::write(d.join("SKILL.md"), "---\nname: s\n---\n").unwrap();
        fs::create_dir_all(d.join("assets")).unwrap();
        std::os::unix::fs::symlink("/etc/hostname", d.join("assets/link")).unwrap();
        let problems = collect(&skill_ref(d), &Modes::default()).expect_err("refused");
        let keys: Vec<&str> = problems.iter().map(|p| p.key).collect();
        assert!(keys.contains(&"symlinks"));
        assert!(keys.contains(&"missing_license"));
    }

    #[test]
    fn index_modes_parse_only_100755_records() {
        let out = b"100755 0123456789abcdef0123456789abcdef01234567 0\ta/assets/run.sh\0\
100644 0123456789abcdef0123456789abcdef01234567 0\ta/assets/x.sh\0\
120000 0123456789abcdef0123456789abcdef01234567 0\ta/link\0\
100755 0123456789abcdef0123456789abcdef01234567 0\ta b/tab\there.sh\0";
        let exec = parse_ls_files(out);
        let got: Vec<&str> = exec.iter().map(String::as_str).collect();
        assert_eq!(got, vec!["a b/tab\there.sh", "a/assets/run.sh"]);
    }

    #[cfg(unix)]
    #[test]
    fn working_tree_exec_bit_is_ignored_outside_an_index() {
        use std::os::unix::fs::PermissionsExt as _;
        let t = tempfile::TempDir::new().unwrap();
        let d = t.path();
        fs::write(d.join("SKILL.md"), "---\nname: s\n---\n").unwrap();
        fs::write(d.join("LICENSE"), "gpl").unwrap();
        fs::create_dir_all(d.join("assets")).unwrap();
        fs::write(d.join("assets/run.sh"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(d.join("assets/run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let (_, members, _) = collect(&skill_ref(d), &Modes::default()).expect("collects");
        assert!(members.values().all(|m| m.mode == 0o644));

        let modes = Modes {
            root: d.parent().unwrap_or(Path::new("/")).to_path_buf(),
            executable: [format!(
                "{}/assets/run.sh",
                d.file_name().unwrap().to_string_lossy()
            )]
            .into_iter()
            .collect(),
        };
        let (_, members, _) = collect(&skill_ref(d), &modes).expect("collects");
        assert_eq!(members["assets/run.sh"].mode, 0o755);
        assert_eq!(members["SKILL.md"].mode, 0o644);
    }
}
