// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for `construct skill vendor`, all offline. The catalogue
//! and the consumer repository are generated in temp directories, and every
//! invocation passes `--into` except the one that checks the default — the
//! crate's own work tree must never receive a vendored skill.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// The palette source bytes the fixture carries.
const PALETTE: &str = "# palette\n[steelbore]\nvoid-navy = \"#000027\"\n";

fn write(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

fn typical(name: &str, body: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: >\n  {name} does a thing.\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\n---\n{body}"
    )
}

/// A small catalogue: a typical skill, a user-invocable one, a palette
/// consumer, and the palette source.
fn catalogue() -> TempDir {
    let t = TempDir::new().expect("temp catalogue");
    let p = t.path();
    write(
        p,
        "alpha/SKILL.md",
        &typical("alpha", "\n# Alpha\n\nSee `references/guide.md`.\n"),
    );
    write(p, "alpha/LICENSE", "gpl text\n");
    write(p, "alpha/references/guide.md", "# Guide\n");
    write(p, "alpha/references/other.md", "# Other\n");
    write(p, "alpha/assets/x.json", "{\"k\": 1}\n");
    write(
        p,
        "hidden-base/SKILL.md",
        "---\nname: hidden-base\ndescription: d\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\nuser-invocable: false\n---\nbody\n",
    );
    write(p, "hidden-base/LICENSE", "gpl text\n");
    write(
        p,
        "spacecraft-brand-guidelines/SKILL.md",
        &typical("spacecraft-brand-guidelines", "body\n"),
    );
    write(p, "spacecraft-brand-guidelines/LICENSE", "gpl text\n");
    write(
        p,
        "steelbore-color-palette/SKILL.md",
        &typical("steelbore-color-palette", "body\n"),
    );
    write(p, "steelbore-color-palette/LICENSE", "gpl text\n");
    write(p, "steelbore-color-palette/assets/steelbore.toml", PALETTE);
    t
}

/// A consumer repository: a bare `.git/` is all `vendor` looks for.
fn consumer() -> TempDir {
    let t = TempDir::new().expect("temp consumer");
    write(t.path(), ".git/HEAD", "ref: refs/heads/main\n");
    t
}

/// Every file under `dir` with its bytes, `/`-separated relative keys.
fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let path = e.path();
            if e.file_type().unwrap().is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy();
                out.insert(rel.replace('\\', "/"), fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn vendor(source: &Path, into: &Path, extra: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("construct")
        .expect("binary builds")
        .args(["skill", "vendor", "--json", "--source"])
        .arg(source)
        .arg("--into")
        .arg(into)
        .args(extra)
        .assert()
}

fn data(assert: &assert_cmd::assert::Assert) -> Value {
    let v: Value = serde_json::from_slice(&assert.get_output().stdout).expect("JSON envelope");
    v["data"].clone()
}

fn error(assert: &assert_cmd::assert::Assert) -> Value {
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    let line = stderr
        .lines()
        .find(|l| l.starts_with("{\"error\""))
        .unwrap_or_else(|| panic!("no structured error in: {stderr}"));
    let v: Value = serde_json::from_str(line).expect("single-line JSON error");
    v["error"].clone()
}

fn actions(d: &Value) -> Vec<String> {
    d["skills"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["action"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn vendor_creates_claude_tree_and_marker() {
    let cat = catalogue();
    let repo = consumer();
    let git_before = snapshot(&repo.path().join(".git"));

    let a = vendor(cat.path(), repo.path(), &["alpha", "hidden-base"]).success();
    let d = data(&a);
    assert_eq!(actions(&d), ["created", "created"]);
    assert_eq!(d["written"], true);
    assert_eq!(
        d["commit_paths"],
        serde_json::json!([".claude/skills/alpha", ".claude/skills/hidden-base"])
    );
    let step = d["next_step"].as_str().unwrap();
    assert!(step.starts_with("git -C "), "{step}");
    assert!(
        step.contains("add .claude/skills/alpha .claude/skills/hidden-base && git -C "),
        "{step}"
    );
    assert!(d["note"].as_str().unwrap().contains("cloud sessions"));

    let tree = snapshot(&repo.path().join(".claude/skills/alpha"));
    let names: Vec<&str> = tree.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        [
            ".construct-vendor.toml",
            "LICENSE",
            "SKILL.md",
            "assets/x.json",
            "references/guide.md",
            "references/other.md"
        ]
    );
    let skill_md = String::from_utf8(tree["SKILL.md"].clone()).unwrap();
    assert!(skill_md.contains("metadata:\n"), "{skill_md}");
    assert!(!skill_md.contains("\nmaintainer:"), "{skill_md}");

    let marker = String::from_utf8(tree[".construct-vendor.toml"].clone()).unwrap();
    assert!(marker.contains("skill = \"alpha\""));
    assert!(
        !marker.contains(&cat.path().display().to_string()),
        "no source path"
    );
    assert!(!marker.contains("2026-"), "no timestamp");

    // user-invocable survives for the claude layout.
    let hidden =
        fs::read_to_string(repo.path().join(".claude/skills/hidden-base/SKILL.md")).unwrap();
    assert!(hidden.contains("user-invocable: false"), "{hidden}");

    assert_eq!(
        snapshot(&repo.path().join(".git")),
        git_before,
        "git untouched"
    );
}

#[test]
fn vendor_rerun_is_unchanged() {
    let cat = catalogue();
    let repo = consumer();
    vendor(cat.path(), repo.path(), &["alpha"]).success();
    let before = snapshot(repo.path());
    let d = data(&vendor(cat.path(), repo.path(), &["alpha"]).success());
    assert_eq!(actions(&d), ["unchanged"]);
    assert_eq!(d["commit_paths"], serde_json::json!([]));
    assert!(d["next_step"].is_null());
    assert_eq!(snapshot(repo.path()), before);
}

#[test]
fn vendor_updates_and_prunes_owned_files() {
    let cat = catalogue();
    let repo = consumer();
    vendor(cat.path(), repo.path(), &["alpha"]).success();
    fs::remove_file(cat.path().join("alpha/references/other.md")).unwrap();
    let d = data(&vendor(cat.path(), repo.path(), &["alpha"]).success());
    assert_eq!(actions(&d), ["updated"]);
    let dest = repo.path().join(".claude/skills/alpha");
    assert!(!dest.join("references/other.md").exists());
    assert!(dest.join("references/guide.md").is_file());
    let leftovers: Vec<String> = fs::read_dir(repo.path().join(".claude/skills"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers, ["alpha"], "no staging directories left behind");
}

#[test]
fn vendor_refuses_unmarked_dir_unless_force() {
    let cat = catalogue();
    let repo = consumer();
    write(repo.path(), ".claude/skills/alpha/mine.md", "user file\n");

    let a = vendor(cat.path(), repo.path(), &["alpha"]).code(5);
    let e = error(&a);
    assert_eq!(e["code"], "CONFLICT");
    assert!(e["vendor_not_owned"].is_array(), "{e}");
    assert_eq!(
        e["hint"],
        format!(
            "construct skill vendor alpha --into {} --source {} --force",
            repo.path().display(),
            cat.path().display()
        )
    );
    assert!(repo.path().join(".claude/skills/alpha/mine.md").is_file());

    let d = data(&vendor(cat.path(), repo.path(), &["alpha", "--force"]).success());
    assert_eq!(actions(&d), ["replaced"]);
    assert!(!repo.path().join(".claude/skills/alpha/mine.md").exists());
    assert!(repo
        .path()
        .join(".claude/skills/alpha/.construct-vendor.toml")
        .is_file());
}

#[test]
fn vendor_refuses_user_added_files() {
    let cat = catalogue();
    let repo = consumer();
    vendor(cat.path(), repo.path(), &["alpha"]).success();
    write(repo.path(), ".claude/skills/alpha/notes.md", "mine\n");
    let e = error(&vendor(cat.path(), repo.path(), &["alpha"]).code(5));
    assert_eq!(
        e["vendor_unowned_files"][0]["files"],
        serde_json::json!(["notes.md"])
    );
    assert!(repo.path().join(".claude/skills/alpha/notes.md").is_file());
}

#[test]
fn vendor_palette_consumer_gets_verified_toml() {
    let cat = catalogue();
    let repo = consumer();
    let d = data(&vendor(cat.path(), repo.path(), &["spacecraft-brand-guidelines"]).success());
    assert_eq!(d["palette_vendored"][0]["verified"], true);
    let toml = fs::read(
        repo.path()
            .join(".claude/skills/spacecraft-brand-guidelines/assets/steelbore.toml"),
    )
    .unwrap();
    assert_eq!(toml, PALETTE.as_bytes());
}

#[test]
fn vendor_dry_run_writes_nothing() {
    let cat = catalogue();
    let repo = consumer();
    let before = snapshot(repo.path());
    let d = data(&vendor(cat.path(), repo.path(), &["alpha", "--dry-run"]).success());
    assert_eq!(d["written"], false);
    assert_eq!(d["planned"], true);
    assert_eq!(actions(&d), ["created"]);
    assert!(d["next_step"].as_str().is_some());
    assert!(!repo.path().join(".claude").exists());
    assert_eq!(snapshot(repo.path()), before);
}

#[test]
fn vendor_requires_git_root_when_into_defaulted() {
    let cat = catalogue();
    let bare = TempDir::new().unwrap();
    // --dry-run: should an ancestor of the temp dir ever be a git work tree,
    // the test fails on the exit code instead of writing into it.
    let a = Command::cargo_bin("construct")
        .unwrap()
        .current_dir(bare.path())
        .args([
            "skill",
            "vendor",
            "alpha",
            "--json",
            "--dry-run",
            "--source",
        ])
        .arg(cat.path())
        .assert()
        .code(3);
    assert!(error(&a)["hint"].as_str().unwrap().contains("--into"));
    assert!(!bare.path().join(".claude").exists());
}

#[test]
fn vendor_gate_refuses_oversized() {
    let cat = catalogue();
    let long = "x".repeat(1001);
    write(
        cat.path(),
        "alpha/SKILL.md",
        &format!("---\nname: alpha\ndescription: {long}\nlicense: GPL-3.0-or-later\n---\nbody\n"),
    );
    let repo = consumer();
    let e = error(&vendor(cat.path(), repo.path(), &["alpha", "hidden-base"]).code(5));
    assert!(e["oversized_skills"].is_array(), "{e}");
    assert!(!repo.path().join(".claude").exists(), "all-or-nothing");
}

#[test]
fn vendor_rejects_traversal_names() {
    let cat = catalogue();
    let repo = consumer();
    for bad in ["../alpha", "alpha/../../x", ".claude", "Alpha"] {
        let e = error(&vendor(cat.path(), repo.path(), &[bad]).code(2));
        assert!(e["invalid_skill_names"].is_array(), "{bad}: {e}");
    }
    assert!(!repo.path().join(".claude").exists());
}

#[test]
fn vendor_unknown_skill_is_not_found() {
    let cat = catalogue();
    let repo = consumer();
    let e = error(&vendor(cat.path(), repo.path(), &["alpha", "nope"]).code(3));
    assert_eq!(e["unknown_skills"], serde_json::json!(["nope"]));
    assert!(!repo.path().join(".claude").exists());
}

#[test]
fn vendor_requires_a_skill() {
    Command::cargo_bin("construct")
        .unwrap()
        .args(["skill", "vendor", "--json"])
        .assert()
        .code(2);
}

#[cfg(unix)]
#[test]
fn vendor_never_writes_through_symlinks() {
    use std::os::unix::fs::symlink;

    let cat = catalogue();
    let outside = TempDir::new().unwrap();

    // A symlinked skill directory: refused even under --force.
    let repo = consumer();
    fs::create_dir_all(repo.path().join(".claude/skills")).unwrap();
    symlink(outside.path(), repo.path().join(".claude/skills/alpha")).unwrap();
    let e = error(&vendor(cat.path(), repo.path(), &["alpha", "--force"]).code(5));
    assert_eq!(e["vendor_unsafe_paths"][0]["reason"], "symlink", "{e}");

    // A symlinked .claude/: refused before anything is classified.
    let repo2 = consumer();
    symlink(outside.path(), repo2.path().join(".claude")).unwrap();
    let e = error(&vendor(cat.path(), repo2.path(), &["alpha", "--force"]).code(5));
    assert_eq!(e["vendor_unsafe_paths"][0]["path"], ".claude", "{e}");

    assert!(
        snapshot(outside.path()).is_empty(),
        "nothing written outside"
    );
}

#[test]
fn schema_and_describe_list_skill_vendor() {
    let out = Command::cargo_bin("construct")
        .unwrap()
        .args(["schema", "skill", "vendor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["command"], "construct skill vendor");
    assert_eq!(v["parameters"]["required"], serde_json::json!(["skills"]));
    assert!(v["parameters"]["properties"]["into"].is_object());

    let out = Command::cargo_bin("construct")
        .unwrap()
        .arg("describe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(out)
        .unwrap()
        .contains("construct skill vendor"));
}
