// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Root-skill discovery in the Construct catalogue, and the single exclusion
//! set shared by `skill ship` and `skill build`.
//!
//! [`EXCLUDED_DIRS`] is the exact `NOT_ROOT_SKILLS` set of
//! `.github/check-skill-frontmatter.py`, which is itself a superset of
//! `flake.nix`'s `excludedDirs`. A unit test pins all three together whenever
//! the catalogue files are present. `install::plan::NON_SKILL_DIRS` is a
//! different list on purpose: it serves arbitrary third-party sources.

use std::path::{Path, PathBuf};

/// Top-level catalogue directories that never hold a root skill, even when
/// one of them carries its own `SKILL.md`. Vendored trees (`android-skills`,
/// `orca-skills`) are never read, let alone rewritten.
pub(crate) const EXCLUDED_DIRS: &[&str] = &[
    "grok-skills",
    "android-skills",
    "orca-skills",
    "perplexity-skills",
    "Excluded",
    "construct-cli",
    "LICENSES",
    ".git",
    ".github",
    ".githooks",
    ".claude",
];

/// The Grok-native skill tree, relative to the catalogue root.
pub(crate) const GROK_DIR: &str = "grok-skills";

/// Where a discovered skill lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    /// A top-level catalogue skill.
    Root,
    /// A Grok-native skill under `grok-skills/`.
    Grok,
}

/// One discovered skill directory.
#[derive(Debug, Clone)]
pub(crate) struct SkillRef {
    /// The directory name, which is also the skill id.
    pub(crate) name: String,
    /// The skill directory.
    pub(crate) dir: PathBuf,
    /// Root or Grok-native.
    pub(crate) origin: Origin,
}

/// The directories directly under `parent` that contain a `SKILL.md`, minus
/// `excluded` and dot-directories, byte-sorted by name.
fn scan(parent: &Path, excluded: &[&str], origin: Origin) -> Vec<SkillRef> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(parent) else {
        return out;
    };
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || excluded.contains(&name.as_str()) {
            continue;
        }
        let dir = parent.join(&name);
        if dir.join("SKILL.md").is_file() {
            out.push(SkillRef { name, dir, origin });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Every root skill in the catalogue at `repo`.
pub(crate) fn root_skills(repo: &Path) -> Vec<SkillRef> {
    scan(repo, EXCLUDED_DIRS, Origin::Root)
}

/// Every Grok-native skill under `repo/grok-skills/`.
pub(crate) fn grok_skills(repo: &Path) -> Vec<SkillRef> {
    scan(&repo.join(GROK_DIR), &[], Origin::Grok)
}

/// Restrict `all` to the `requested` names (every one of them, when
/// `requested` is empty). `Err` lists every requested name not in `all`.
pub(crate) fn select(all: &[SkillRef], requested: &[String]) -> Result<Vec<SkillRef>, Vec<String>> {
    if requested.is_empty() {
        return Ok(all.to_vec());
    }
    let unknown: Vec<String> = requested
        .iter()
        .filter(|r| !all.iter().any(|s| &s.name == *r))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(unknown);
    }
    Ok(all
        .iter()
        .filter(|s| requested.contains(&s.name))
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::{root_skills, select, EXCLUDED_DIRS};

    fn skill(root: &Path, dir: &str) {
        fs::create_dir_all(root.join(dir)).unwrap();
        fs::write(root.join(dir).join("SKILL.md"), "---\nname: x\n---\n").unwrap();
    }

    #[test]
    fn excluded_and_dot_dirs_are_skipped_and_result_is_sorted() {
        let t = tempfile::TempDir::new().unwrap();
        for d in ["zeta", "alpha", "Excluded", "perplexity-skills", ".hidden"] {
            skill(t.path(), d);
        }
        fs::create_dir_all(t.path().join("no-skill")).unwrap();
        let names: Vec<String> = root_skills(t.path()).into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["alpha".to_owned(), "zeta".to_owned()]);
    }

    #[test]
    fn select_lists_every_unknown_name() {
        let t = tempfile::TempDir::new().unwrap();
        skill(t.path(), "alpha");
        let all = root_skills(t.path());
        let err = select(
            &all,
            &["nope".to_owned(), "alpha".to_owned(), "gone".to_owned()],
        )
        .expect_err("unknown names");
        assert_eq!(err, vec!["nope".to_owned(), "gone".to_owned()]);
        assert_eq!(select(&all, &[]).unwrap().len(), 1);
    }

    /// Extract the quoted strings between `open` and the next `close`.
    fn quoted_between(text: &str, open: &str, close: char) -> Option<BTreeSet<String>> {
        let start = text.find(open)? + open.len();
        let end = start + text[start..].find(close)?;
        Some(
            text[start..end]
                .split('"')
                .skip(1)
                .step_by(2)
                .map(str::to_owned)
                .collect(),
        )
    }

    /// `EXCLUDED_DIRS` == `NOT_ROOT_SKILLS` ⊇ flake `excludedDirs`. Skips when
    /// the catalogue files are absent (the Nix build sandbox sees only
    /// `construct-cli/`).
    #[test]
    fn exclusion_set_matches_flake_and_frontmatter_checker() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let (Ok(flake), Ok(py)) = (
            fs::read_to_string(root.join("flake.nix")),
            fs::read_to_string(root.join(".github/check-skill-frontmatter.py")),
        ) else {
            return;
        };
        let flake_set = quoted_between(&flake, "excludedDirs = [", ']').expect("excludedDirs");
        let py_set = quoted_between(&py, "NOT_ROOT_SKILLS = {", '}').expect("NOT_ROOT_SKILLS");
        let ours: BTreeSet<String> = EXCLUDED_DIRS.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(ours, py_set, "EXCLUDED_DIRS drifted from NOT_ROOT_SKILLS");
        assert!(
            flake_set.is_subset(&ours),
            "flake excludedDirs has entries missing from EXCLUDED_DIRS: {:?}",
            flake_set.difference(&ours).collect::<Vec<_>>()
        );
    }
}
