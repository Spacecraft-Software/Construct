// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for `construct skill ship` against throwaway git fixtures.
//! All use `--dry-run --no-sync`, so no commit, push, or flake update happens —
//! they exercise detection, the bundle-drift refusal, and remote validation.

use std::fs;
use std::path::Path;
use std::process::Command as Proc;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const REMOTE: &str = "git@github.com:Spacecraft-Software/Construct.git";

fn run_git(repo: &Path, args: &[&str]) {
    let status = Proc::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git runs")
        .status;
    assert!(status.success(), "git {args:?} failed");
}

/// A git work tree with the given origin URL and a committable identity.
fn fixture(remote: &str) -> TempDir {
    let dir = TempDir::new().expect("temp repo");
    let p = dir.path();
    run_git(p, &["init", "-q", "-b", "main"]);
    run_git(p, &["config", "user.email", "test@example.com"]);
    run_git(p, &["config", "user.name", "Test"]);
    run_git(p, &["config", "commit.gpgsign", "false"]);
    run_git(p, &["remote", "add", "origin", remote]);
    dir
}

fn write(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

fn bin() -> Command {
    Command::cargo_bin("construct").expect("binary builds")
}

#[test]
fn ship_dry_run_reports_plan_when_bundles_rebuilt() {
    let repo = fixture(REMOTE);
    let p = repo.path();
    write(p, "demo/SKILL.md", "---\nname: demo\n---\nv1\n");
    write(p, "demo.zip", "z1");
    write(p, "demo.skill", "s1");
    run_git(p, &["add", "demo/SKILL.md", "demo.zip", "demo.skill"]);
    run_git(p, &["commit", "-qm", "init"]);
    // Edit the source AND rebuild both bundles — the contract is satisfied.
    write(p, "demo/SKILL.md", "---\nname: demo\n---\nv2\n");
    write(p, "demo.zip", "z2");
    write(p, "demo.skill", "s2");

    let out = bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["data"]["status"], "planned");
    let shipped = v["data"]["shipped_skills"].as_array().unwrap();
    assert!(shipped.iter().any(|s| s == "demo"));
    let stage = v["data"]["would_stage"].as_array().unwrap();
    assert!(stage.iter().any(|s| s == "demo.zip"));
    assert!(stage.iter().any(|s| s == "demo.skill"));
    assert!(stage.iter().any(|s| s == "demo/SKILL.md"));
}

#[test]
fn ship_refuses_bundle_drift() {
    let repo = fixture(REMOTE);
    let p = repo.path();
    write(p, "demo/SKILL.md", "v1");
    write(p, "demo.zip", "z1");
    write(p, "demo.skill", "s1");
    run_git(p, &["add", "demo/SKILL.md", "demo.zip", "demo.skill"]);
    run_git(p, &["commit", "-qm", "init"]);
    // Edit the source only — bundles NOT rebuilt: drift.
    write(p, "demo/SKILL.md", "v2");

    let assertion = bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .code(5);
    let err: Value =
        serde_json::from_slice(&assertion.get_output().stderr).expect("structured error");
    assert_eq!(err["error"]["code"], "CONFLICT");
    assert!(err["error"]["hint"].as_str().unwrap().contains("zip"));
}

/// A `SKILL.md` whose folded `description` renders to `len` characters.
///
/// The folded scalar joins its wrapped lines with single spaces and keeps one
/// trailing newline, which the loader counts — so the body is `len - 1` filler
/// characters on a single indented line.
fn skill_md_with_description(len: usize) -> String {
    let filler = "d".repeat(len - 1);
    format!("---\nname: demo\ndescription: >\n  {filler}\n---\nbody\n")
}

#[test]
fn ship_refuses_oversized_description() {
    let repo = fixture(REMOTE);
    let p = repo.path();
    write(p, "demo/SKILL.md", &skill_md_with_description(900));
    write(p, "demo.zip", "z1");
    write(p, "demo.skill", "s1");
    run_git(p, &["add", "demo/SKILL.md", "demo.zip", "demo.skill"]);
    run_git(p, &["commit", "-qm", "init"]);
    // Push the description past the 1000-char cap; bundles rebuilt, so the only
    // thing standing between this and a push is the §5.6 gate.
    write(p, "demo/SKILL.md", &skill_md_with_description(1001));
    write(p, "demo.zip", "z2");
    write(p, "demo.skill", "s2");

    let assertion = bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .code(5);
    let err: Value =
        serde_json::from_slice(&assertion.get_output().stderr).expect("structured error");
    assert_eq!(err["error"]["code"], "CONFLICT");
    assert_eq!(err["error"]["oversized_skills"][0]["skill"], "demo");
    assert_eq!(err["error"]["oversized_skills"][0]["chars"], 1001);
    assert_eq!(err["error"]["oversized_skills"][0]["over_by"], 1);
}

#[test]
fn ship_allows_description_exactly_at_cap() {
    let repo = fixture(REMOTE);
    let p = repo.path();
    write(p, "demo/SKILL.md", "---\nname: demo\n---\nv1\n");
    write(p, "demo.zip", "z1");
    write(p, "demo.skill", "s1");
    run_git(p, &["add", "demo/SKILL.md", "demo.zip", "demo.skill"]);
    run_git(p, &["commit", "-qm", "init"]);
    // Exactly 1000 rendered characters is compliant — the cap is inclusive.
    write(p, "demo/SKILL.md", &skill_md_with_description(1000));
    write(p, "demo.zip", "z2");
    write(p, "demo.skill", "s2");

    let out = bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["data"]["status"], "planned");
}

#[test]
fn ship_rejects_wrong_remote() {
    let repo = fixture("https://example.com/foo/bar.git");
    let p = repo.path();
    write(p, "x.txt", "hi");
    bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .code(2);
}

#[test]
fn ship_nothing_to_ship_on_clean_repo() {
    let repo = fixture(REMOTE);
    let p = repo.path();
    write(p, "README.md", "hi");
    run_git(p, &["add", "README.md"]);
    run_git(p, &["commit", "-qm", "init"]);

    let out = bin()
        .args([
            "skill",
            "ship",
            "--repo",
            p.to_str().unwrap(),
            "--no-sync",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["data"]["status"], "nothing_to_ship");
}
