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
fn schema_for_skill_build_has_parameters_and_exit_codes() {
    let out = bin()
        .args(["schema", "skill", "build"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(value["command"], "construct skill build");
    assert!(value["parameters"]["properties"]["target"].is_object());
    assert!(value["parameters"]["properties"]["skills"].is_object());
    assert!(value["examples"].as_array().is_some_and(|a| !a.is_empty()));
}

#[test]
fn describe_lists_skill_build() {
    let out = bin()
        .arg("describe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).expect("UTF-8");
    assert!(
        text.contains("construct skill build"),
        "describe omits skill build"
    );
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
fn explore_fallback_under_agent_env_is_a_diagnostic_envelope() {
    // Under a real agent env value, the fallback warning must be the
    // single-line `{"diagnostic":{...}}` envelope (diagnostics.md §3) —
    // the agent floor is `warn`, so a warn-severity diagnostic still emits.
    let assertion = bin()
        .env("AI_AGENT", "claude-code_2-1-218_agent")
        .args(["--format", "explore"])
        .assert()
        .success();
    let out = assertion.get_output();
    serde_json::from_slice::<Value>(&out.stdout).expect("JSON fallback on stdout");
    let stderr = String::from_utf8(out.stderr.clone()).expect("valid UTF-8");
    let line = stderr.lines().next().expect("one stderr line");
    let value: Value = serde_json::from_str(line).expect("single-line diagnostic JSON");
    assert_eq!(value["diagnostic"]["severity"], "warn");
    assert_eq!(value["diagnostic"]["code"], "TUI_FALLBACK");
    assert!(
        value["diagnostic"]["hint"]
            .as_str()
            .is_some_and(|h| h.starts_with("construct ")),
        "hint must be a runnable construct command"
    );
    assert!(value["diagnostic"]["reason"].is_string());
    assert!(!stderr.contains('\u{1b}'), "no ANSI in machine mode");
}

#[test]
fn quiet_suppresses_the_tui_fallback_warning() {
    // `--quiet` raises the severity floor to errors-only: the warn-severity
    // TUI fallback is not written, and stdout still carries the JSON payload.
    let assertion = bin()
        .args(["--format", "explore", "--quiet"])
        .assert()
        .success();
    let out = assertion.get_output();
    serde_json::from_slice::<Value>(&out.stdout).expect("JSON fallback on stdout");
    assert!(
        out.stderr.is_empty(),
        "quiet must suppress the fallback warning, got: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn quiet_never_suppresses_errors() {
    // Errors bypass the severity floor: the structured error object still
    // lands on stderr under --quiet.
    let assertion = bin()
        .args([
            "skill",
            "sync",
            "--flake-dir",
            "/no/such/construct/dir",
            "--json",
            "--quiet",
        ])
        .assert()
        .code(3);
    let err = assertion.get_output().stderr.clone();
    let value: Value = serde_json::from_slice(&err).expect("structured error on stderr");
    assert_eq!(value["error"]["code"], "NOT_FOUND");
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

#[test]
fn help_subcommand_succeeds() {
    let out = bin()
        .arg("help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).expect("valid UTF-8");
    assert!(text.contains("Usage"), "help omits usage");
    assert!(text.contains("skill"), "help omits the skill noun");
}

#[test]
fn help_for_nested_command_succeeds() {
    // `construct help skill sync` resolves the nested path and exits 0.
    bin().args(["help", "skill", "sync"]).assert().success();
}

#[test]
fn describe_lists_the_accessible_global_flags() {
    let out = bin()
        .arg("describe")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("describe emits valid JSON");
    let text = value.to_string();
    assert!(
        text.contains("\"--accessible\""),
        "describe omits --accessible"
    );
    assert!(
        text.contains("\"--no-accessible\""),
        "describe omits --no-accessible"
    );
}

#[test]
fn accessible_and_no_accessible_conflict() {
    bin()
        .args(["describe", "--accessible", "--no-accessible"])
        .assert()
        .code(2);
}

#[test]
fn verbose_reports_the_accessible_mode_source() {
    let assert = bin()
        .args(["describe", "--json", "--verbose", "--accessible"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    let line = stderr
        .lines()
        .find(|l| l.contains("ACCESSIBLE_MODE"))
        .unwrap_or_else(|| panic!("no ACCESSIBLE_MODE diagnostic in: {stderr}"));
    let v: Value = serde_json::from_str(line).expect("single-line diagnostic");
    assert_eq!(v["diagnostic"]["severity"], "info");
    assert_eq!(v["diagnostic"]["accessible"], true);
    assert_eq!(v["diagnostic"]["source"], "flag");
}

#[test]
fn timestamps_are_whole_seconds_utc() {
    let out = bin()
        .args(["agent", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("valid JSON envelope");
    let ts = value["metadata"]["timestamp"].as_str().expect("timestamp");
    // YYYY-MM-DDTHH:MM:SSZ exactly (Standard §14, CLI Standard §1 item 1).
    assert_eq!(ts.len(), 20, "not YYYY-MM-DDTHH:MM:SSZ: {ts}");
    assert!(ts.ends_with('Z') && ts.as_bytes()[10] == b'T', "{ts}");
}

#[test]
fn version_json_is_an_envelope_with_maintainer_and_website() {
    let out = bin()
        .args(["--version", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&out).expect("--version --json is JSON");
    assert_eq!(value["metadata"]["tool"], "construct");
    assert_eq!(
        value["metadata"]["maintainer"],
        "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>"
    );
    assert_eq!(
        value["metadata"]["website"],
        "https://Construct.SpacecraftSoftware.org/"
    );
    assert_eq!(value["data"]["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn every_help_page_carries_the_project_footer() {
    for args in [
        vec!["--help"],
        vec!["skill", "--help"],
        vec!["skill", "delete", "--help"],
        vec!["agent", "list", "--help"],
        vec!["mcp", "--help"],
    ] {
        let out = bin()
            .args(&args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8(out).expect("valid UTF-8");
        assert!(
            text.contains("https://Construct.SpacecraftSoftware.org/")
                && text.contains("Mohamed Hammad"),
            "{args:?}: help lacks the project footer"
        );
    }
}

#[test]
fn usage_error_hint_points_at_the_nearest_help() {
    let err = bin()
        .args(["skill", "vendor", "--json"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&err).expect("structured error");
    assert_eq!(value["error"]["hint"], "construct skill vendor --help");
}

#[test]
fn control_characters_in_arguments_are_refused() {
    let err = bin()
        .args(["skill", "find", "a\u{1b}[31mb", "--json"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&err).expect("structured error");
    assert_eq!(value["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(value["error"]["exit_code"], 2);
}

#[test]
fn init_refuses_a_name_that_is_a_path() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    bin()
        .current_dir(dir.path())
        .args(["skill", "init", "../escape", "--json"])
        .assert()
        .code(2);
    assert!(!dir.path().join("../escape").exists());
}

#[test]
fn explore_fallback_hint_is_the_same_command_as_json() {
    let err = bin()
        .env("AI_AGENT", "test-harness")
        .args(["--format", "explore", "agent", "list"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let line = String::from_utf8(err).expect("utf-8");
    let value: Value = serde_json::from_str(line.trim()).expect("diagnostic envelope");
    assert_eq!(value["diagnostic"]["hint"], "construct agent list --json");
}

#[test]
fn schema_provider_formats() {
    let openai: Value = serde_json::from_slice(
        &bin()
            .args(["schema", "skill", "list", "--format", "openai"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("openai JSON");
    assert_eq!(openai["type"], "function");
    assert_eq!(openai["function"]["name"], "construct_skill_list");
    assert_eq!(openai["function"]["strict"], true);
    assert_eq!(
        openai["function"]["parameters"]["additionalProperties"],
        false
    );

    let gemini: Value = serde_json::from_slice(
        &bin()
            .args(["schema", "--format", "gemini"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("gemini JSON");
    assert_eq!(
        gemini["function_declarations"][0]["parameters"]["type"],
        "OBJECT"
    );

    let mcp: Value = serde_json::from_slice(
        &bin()
            .args(["schema", "skill", "delete", "--format", "mcp"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .expect("mcp JSON");
    assert_eq!(mcp["tools"][0]["name"], "construct_skill_delete");
    assert!(mcp["tools"][0]["inputSchema"]["properties"].is_object());
}

#[test]
fn provider_format_is_refused_outside_schema() {
    let err = bin()
        .args(["agent", "list", "--format", "openai"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&err).expect("structured error");
    assert_eq!(value["error"]["hint"], "construct schema --format openai");
}

#[test]
fn mcp_stdio_lists_lazily_and_calls_tools() {
    let input = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/get","params":{"name":"construct_agent_list"}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"construct_agent_list","arguments":{"fields":["id"]}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"construct_skill_list","arguments":{"agents":["no-such-agent"]}}}"#,
    ]
    .join("\n");
    let out = bin()
        .arg("mcp")
        .write_stdin(input + "\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let frames: Vec<Value> = String::from_utf8(out)
        .expect("utf-8")
        .lines()
        .map(|l| serde_json::from_str(l).expect("one JSON-RPC frame per line"))
        .collect();
    assert_eq!(frames.len(), 5, "the notification gets no answer");
    assert_eq!(frames[0]["result"]["serverInfo"]["name"], "construct");
    let tools = frames[1]["result"]["tools"].as_array().expect("tools");
    assert!(tools
        .iter()
        .all(|t| t["inputSchema"] == serde_json::json!({ "type": "object" })));
    assert!(!tools.iter().any(|t| t["name"] == "construct_mcp"));
    assert!(frames[2]["result"]["outputSchema"].is_object());
    let data = &frames[3]["result"]["structuredContent"]["data"];
    assert!(data[0]["id"].is_string() && data[0].get("global_path").is_none());
    assert_eq!(frames[4]["error"]["code"], -32000);
    assert_eq!(frames[4]["error"]["message"], "NOT_FOUND");
    assert_eq!(frames[4]["error"]["data"]["exit_code"], 3);
}
