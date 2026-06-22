// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for general sources: catalogue browsing (`find`), container
//! directory discovery, `use`, `init`, and installing from a container-layout
//! source. All use local temp directories — no network.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// Write a `<rel>/SKILL.md` with the given frontmatter and body under `root`.
fn skill(root: &Path, rel: &str, name: &str, desc: &str, body: &str) {
    let dir = root.join(rel);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {desc}\n---\n\n{body}\n"),
    )
    .unwrap();
}

fn bin() -> Command {
    Command::cargo_bin("construct").expect("binary builds")
}

#[test]
fn find_lists_skills_including_container_dirs() {
    let src = TempDir::new().unwrap();
    skill(src.path(), "alpha", "alpha", "first skill", "A body");
    skill(src.path(), "skills/beta", "beta", "second skill", "B body");

    let out = bin()
        .args([
            "skill",
            "find",
            "--source",
            src.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let names: Vec<&str> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"alpha"), "top-level skill missing");
    assert!(names.contains(&"beta"), "container-dir skill missing");
}

#[test]
fn find_filters_by_query() {
    let src = TempDir::new().unwrap();
    skill(src.path(), "alpha", "alpha", "all about rust", "x");
    skill(src.path(), "beta", "beta", "all about python", "y");

    let out = bin()
        .args([
            "skill",
            "find",
            "rust",
            "--source",
            src.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let names: Vec<&str> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["alpha"]);
}

#[test]
fn use_prints_skill_body() {
    let src = TempDir::new().unwrap();
    skill(src.path(), "alpha", "alpha", "desc", "UNIQUE_BODY_TOKEN");

    let out = bin()
        .args([
            "skill",
            "use",
            "--skills",
            "alpha",
            src.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let content = v["data"][0]["content"].as_str().unwrap();
    assert!(content.contains("UNIQUE_BODY_TOKEN"));
}

#[test]
fn init_scaffolds_and_refuses_existing() {
    let dir = TempDir::new().unwrap();
    bin()
        .args([
            "skill",
            "init",
            "my-skill",
            "--dir",
            dir.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let md = dir.path().join("my-skill/SKILL.md");
    assert!(md.is_file(), "SKILL.md not scaffolded");
    let content = fs::read_to_string(&md).unwrap();
    assert!(content.contains("name: my-skill"));

    // A second init at the same path is a conflict.
    bin()
        .args([
            "skill",
            "init",
            "my-skill",
            "--dir",
            dir.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(5);
}

#[test]
fn add_from_container_dir_installs() {
    let src = TempDir::new().unwrap();
    skill(src.path(), "skills/gamma", "gamma", "desc", "body");
    let proj = TempDir::new().unwrap();
    fs::create_dir_all(proj.path().join(".git")).unwrap();

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
    // cursor installs project-local under `.agents/skills`; gamma was under skills/.
    let target = proj.path().join(".agents/skills/gamma");
    let meta = fs::symlink_metadata(&target).expect("installed");
    assert!(meta.file_type().is_symlink());
}
