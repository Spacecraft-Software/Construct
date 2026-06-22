// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box compliance tests for the `construct` binary, covering the
//! Spacecraft Software CLI Standard BLOCKER items: the JSON envelope, ISO 8601
//! UTC timestamps, UTF-8 without BOM, the agent-env cascade, canonical exit
//! codes, structured errors on stderr, and the self-describing `schema` /
//! `describe` surface.

use assert_cmd::Command;
use serde_json::Value;

/// A fresh invocation of the built binary.
fn bin() -> Command {
    Command::cargo_bin("construct").expect("the `construct` binary builds")
}

#[test]
fn describe_is_valid_json_and_succeeds() {
    let out = bin()
        .arg("describe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("describe emits valid JSON");
    assert_eq!(value["tool"], "construct");
    assert!(value["commands"].is_array());
    assert_eq!(value["context_files"][0], "CLAUDE.md");
    assert_eq!(value["schema_command"], "construct schema");
}

#[test]
fn describe_output_is_utf8_without_bom() {
    let out = bin()
        .arg("describe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        !out.starts_with(&[0xEF, 0xBB, 0xBF]),
        "output carries a UTF-8 BOM"
    );
    std::str::from_utf8(&out).expect("output is valid UTF-8");
}

#[test]
fn schema_is_valid_json_with_draft_2020_12() {
    let out = bin()
        .arg("schema")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("schema emits valid JSON");
    assert_eq!(
        value["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert!(value["commands"].is_array());
}

#[test]
fn schema_for_one_command_has_parameters_and_examples() {
    let out = bin()
        .args(["schema", "skill", "sync"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(value["command"], "construct skill sync");
    assert!(value["parameters"]["properties"]["flake_dir"].is_object());
    assert!(value["examples"].as_array().is_some_and(|a| !a.is_empty()));
}

#[test]
fn sync_dry_run_envelope_has_iso_utc_timestamp() {
    // Use the crate dir as an existing flake dir so the test never depends on
    // the host default (`/spacecraft-software/bravais`) existing — notably in
    // the isolated Nix build sandbox.
    let out = bin()
        .args([
            "skill",
            "sync",
            "--dry-run",
            "--json",
            "--flake-dir",
            env!("CARGO_MANIFEST_DIR"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("valid JSON envelope");
    let ts = value["metadata"]["timestamp"]
        .as_str()
        .expect("timestamp present");
    assert!(
        ts.ends_with('Z') && ts.contains('T'),
        "not ISO 8601 UTC: {ts}"
    );
    assert_eq!(value["metadata"]["dry_run"], true);
    assert_eq!(value["data"]["executed"], false);
    assert_eq!(value["data"]["input"], "construct");
}

#[test]
fn agent_env_forces_json_without_a_flag() {
    let out = bin()
        .env("AI_AGENT", "1")
        .args([
            "skill",
            "sync",
            "--dry-run",
            "--flake-dir",
            env!("CARGO_MANIFEST_DIR"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("AI_AGENT yields JSON");
    assert!(value["metadata"].is_object());
    assert!(value["data"].is_object());
}

#[test]
fn unknown_subcommand_is_usage_error() {
    bin().arg("frobnicate").assert().code(2);
}

#[test]
fn missing_flake_dir_is_structured_not_found() {
    let assertion = bin()
        .args([
            "skill",
            "sync",
            "--flake-dir",
            "/no/such/construct/dir",
            "--json",
        ])
        .assert()
        .code(3);
    let err = assertion.get_output().stderr.clone();
    let value: Value = serde_json::from_slice(&err).expect("structured error on stderr");
    assert_eq!(value["error"]["code"], "NOT_FOUND");
    assert_eq!(value["error"]["exit_code"], 3);
    assert!(value["error"]["hint"].is_string());
}

#[test]
fn explore_falls_back_to_json_when_not_a_tty() {
    // assert_cmd runs with a piped (non-TTY) stdout, so `--format explore` must
    // not launch the TUI — it falls back to JSON with a warning on stderr.
    let assertion = bin().args(["--format", "explore"]).assert().success();
    let out = assertion.get_output();
    serde_json::from_slice::<Value>(&out.stdout).expect("JSON fallback on stdout");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("TUI_FALLBACK"),
        "missing TUI fallback warning"
    );
}

#[test]
fn version_names_maintainer_and_site() {
    let out = bin()
        .arg("--version")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).expect("valid UTF-8");
    assert!(text.contains("Mohamed Hammad"), "version omits maintainer");
    assert!(
        text.contains("Construct.SpacecraftSoftware.org"),
        "version omits project URL"
    );
}
