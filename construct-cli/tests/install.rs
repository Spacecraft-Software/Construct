// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for the imperative installer: project-local symlinking,
//! `--dry-run` safety, the Home-Manager collision refusal, and an
//! add → list → remove roundtrip. All run against throwaway temp directories.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// A catalogue source dir containing the named skills (each with a `SKILL.md`).
fn make_source(skills: &[&str]) -> TempDir {
    let dir = TempDir::new().expect("temp source");
    for skill in skills {
        let path = dir.path().join(skill);
        fs::create_dir_all(&path).expect("mkdir skill");
        fs::write(path.join("SKILL.md"), "---\nname: x\n---\nbody\n").expect("write SKILL.md");
    }
    dir
}

/// A project dir with a `.git` marker so the project-root walk-up stops here.
fn make_project() -> TempDir {
    let dir = TempDir::new().expect("temp project");
    fs::create_dir_all(dir.path().join(".git")).expect("mkdir .git");
    dir
}

fn bin() -> Command {
    Command::cargo_bin("construct").expect("binary builds")
}

#[test]
fn add_project_local_symlinks_skill() {
    let src = make_source(&["demo"]);
    let proj = make_project();
    bin()
        .current_dir(proj.path())
        .args([
            "skill",
            "add",
            src.path().to_str().unwrap(),
            "--agents",
            "claude-code",
            "--json",
        ])
        .assert()
        .success();
    // claude-code's project_path is `.claude/skills`.
    let target = proj.path().join(".claude/skills/demo");
    let meta = fs::symlink_metadata(&target).expect("installed target exists");
    assert!(meta.file_type().is_symlink(), "expected a symlink install");
}

#[test]
fn add_dry_run_creates_nothing() {
    let src = make_source(&["demo"]);
    let proj = make_project();
    bin()
        .current_dir(proj.path())
        .args([
            "skill",
            "add",
            src.path().to_str().unwrap(),
            "--agents",
            "cursor",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success();
    // cursor's project_path is `.agents/skills`.
    assert!(!proj.path().join(".agents/skills/demo").exists());
}

#[test]
fn add_global_into_hm_managed_is_refused() {
    let src = make_source(&["demo"]);
    let home = TempDir::new().expect("temp home");
    // Simulate the Construct HM module: ~/.claude/skills -> ~/.agents/skills.
    let canonical = home.path().join(".agents/skills");
    fs::create_dir_all(&canonical).expect("mkdir canonical");
    fs::create_dir_all(home.path().join(".claude")).expect("mkdir .claude");
    std::os::unix::fs::symlink(&canonical, home.path().join(".claude/skills"))
        .expect("symlink hm dir");

    let assertion = bin()
        .env("HOME", home.path())
        .args([
            "skill",
            "add",
            src.path().to_str().unwrap(),
            "--agents",
            "claude-code",
            "--global",
            "--json",
        ])
        .assert()
        .code(5);
    let err: Value =
        serde_json::from_slice(&assertion.get_output().stderr).expect("structured error");
    assert_eq!(err["error"]["code"], "CONFLICT");
    assert_eq!(err["error"]["exit_code"], 5);
}

#[test]
fn add_list_remove_roundtrip() {
    let src = make_source(&["alpha", "beta"]);
    let proj = make_project();

    bin()
        .current_dir(proj.path())
        .args([
            "skill",
            "add",
            src.path().to_str().unwrap(),
            "--agents",
            "cursor",
            "--json",
        ])
        .assert()
        .success();

    let out = bin()
        .current_dir(proj.path())
        .args(["skill", "list", "--agents", "cursor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listed: Value = serde_json::from_slice(&out).expect("valid JSON");
    let items = listed["data"]["items"].as_array().expect("items array");
    assert!(items.iter().any(|i| i["skill"] == "alpha"));
    assert!(items.iter().any(|i| i["skill"] == "beta"));

    bin()
        .current_dir(proj.path())
        .args([
            "skill", "remove", "--skills", "alpha", "--agents", "cursor", "--json",
        ])
        .assert()
        .success();
    assert!(!proj.path().join(".agents/skills/alpha").exists());
    assert!(proj.path().join(".agents/skills/beta").exists());
}
