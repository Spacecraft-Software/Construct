// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for the mutable skill pointer: `skill status` and
//! `skill reset`. The `nix build` half of `sync --build` is not covered here —
//! it needs a real store — but everything that decides *tracking vs moved* is.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("construct").expect("binary builds")
}

/// A state dir shaped like the Home-Manager module leaves it:
/// `pinned` -> a tree, `current` -> `pinned`.
fn make_state(tracking: bool) -> TempDir {
    let dir = TempDir::new().expect("temp state");
    let pinned_tree = dir.path().join("tree-pinned");
    fs::create_dir_all(&pinned_tree).expect("mkdir pinned tree");
    std::os::unix::fs::symlink(&pinned_tree, dir.path().join("pinned")).expect("symlink pinned");

    let current_target = if tracking {
        dir.path().join("pinned")
    } else {
        let moved = dir.path().join("tree-moved");
        fs::create_dir_all(&moved).expect("mkdir moved tree");
        moved
    };
    std::os::unix::fs::symlink(&current_target, dir.path().join("current"))
        .expect("symlink current");
    dir
}

fn status_json(pointer: &Path) -> Value {
    let out = bin()
        .args(["skill", "status", "--json", "--pointer"])
        .arg(pointer)
        .assert()
        .success();
    serde_json::from_slice(&out.get_output().stdout).expect("status envelope")
}

#[test]
fn status_reports_tracking_when_current_points_at_pinned() {
    let state = make_state(true);
    let data = status_json(&state.path().join("current"));
    assert_eq!(data["data"]["tracking_flake"], true);
    assert_eq!(data["data"]["dangling"], false);
}

#[test]
fn status_reports_moved_when_current_points_elsewhere() {
    let state = make_state(false);
    let data = status_json(&state.path().join("current"));
    assert_eq!(data["data"]["tracking_flake"], false);
    assert_ne!(
        data["data"]["pinned_store"], data["data"]["current_store"],
        "a moved pointer must resolve to a different tree"
    );
}

#[test]
fn status_reports_a_collected_target_as_dangling() {
    let state = make_state(false);
    // Simulate the tree being garbage-collected out from under the pointer.
    fs::remove_dir_all(state.path().join("tree-moved")).expect("rm moved tree");
    let data = status_json(&state.path().join("current"));
    assert_eq!(data["data"]["dangling"], true);
    assert_eq!(data["data"]["tracking_flake"], false);
}

#[test]
fn status_without_a_pointer_is_not_found() {
    let empty = TempDir::new().expect("temp dir");
    let assertion = bin()
        .args(["skill", "status", "--json", "--pointer"])
        .arg(empty.path().join("current"))
        .assert()
        .code(3);
    let err: Value =
        serde_json::from_slice(&assertion.get_output().stderr).expect("structured error");
    assert_eq!(err["error"]["code"], "NOT_FOUND");
}

#[test]
fn reset_points_current_back_at_pinned() {
    let state = make_state(false);
    let pointer = state.path().join("current");
    assert_eq!(status_json(&pointer)["data"]["tracking_flake"], false);

    bin()
        .args(["skill", "reset", "--json", "--pointer"])
        .arg(&pointer)
        .assert()
        .success();

    // The link text must be `pinned`, not pinned's target: going through
    // `pinned` is what keeps the tree rooted by the Home-Manager generation.
    assert_eq!(
        fs::read_link(&pointer).expect("current is a symlink"),
        state.path().join("pinned")
    );
    assert_eq!(status_json(&pointer)["data"]["tracking_flake"], true);
}

#[test]
fn reset_dry_run_changes_nothing() {
    let state = make_state(false);
    let pointer = state.path().join("current");
    let before = fs::read_link(&pointer).expect("current is a symlink");

    bin()
        .args(["skill", "reset", "--dry-run", "--json", "--pointer"])
        .arg(&pointer)
        .assert()
        .success();

    assert_eq!(fs::read_link(&pointer).expect("still a symlink"), before);
    assert_eq!(status_json(&pointer)["data"]["tracking_flake"], false);
}
