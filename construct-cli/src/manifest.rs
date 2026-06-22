// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The command manifest: a single in-code source of truth describing every
//! implemented command (parameters, output shape, exit codes, examples). Both
//! `construct schema` and `construct describe` are derived from it, and a unit
//! test asserts it stays in sync with the clap tree so the two never drift
//! (schema-introspection §5).

use serde_json::{json, Value};

/// Tool-wide descriptive metadata.
#[derive(Debug)]
pub(crate) struct ToolInfo {
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) description: &'static str,
    pub(crate) license: &'static str,
    pub(crate) homepage: &'static str,
    pub(crate) global_flags: &'static [&'static str],
    pub(crate) output_formats: &'static [&'static str],
    pub(crate) context_files: &'static [&'static str],
    pub(crate) mcp_available: bool,
}

/// Static description of the tool as a whole.
pub(crate) fn tool_info() -> ToolInfo {
    ToolInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        description: env!("CARGO_PKG_DESCRIPTION"),
        license: "GPL-3.0-or-later",
        homepage: "https://Construct.SpacecraftSoftware.org/",
        global_flags: &[
            "--json",
            "--format",
            "--fields",
            "--dry-run",
            "--verbose",
            "--quiet",
            "--no-color",
            "--color",
            "--absolute-time",
            "--print0",
            "--yes",
            "--force",
        ],
        output_formats: &["json", "jsonl", "yaml", "csv", "explore"],
        context_files: &["CLAUDE.md", "AGENTS.md", "SKILL.md", "CONTRIBUTING.md"],
        mcp_available: false,
    }
}

/// One implemented command's full specification.
#[derive(Debug)]
pub(crate) struct CommandSpec {
    /// Full command path, e.g. `construct skill sync`.
    pub(crate) name: String,
    /// Noun segment for filtering (`skill`), empty for top-level meta-commands.
    pub(crate) noun: String,
    /// Verb segment for filtering (`sync`), empty for top-level meta-commands.
    pub(crate) verb: String,
    pub(crate) description: String,
    /// JSON Schema for the command's parameters (LLM-tool-call compatible).
    pub(crate) parameters: Value,
    /// JSON Schema for the `data` field of the `--json` response.
    pub(crate) output_data: Value,
    /// Exit code → description.
    pub(crate) exit_codes: Vec<(String, String)>,
    /// Example invocations (command, description).
    pub(crate) examples: Vec<(String, String)>,
    pub(crate) supports_json: bool,
    pub(crate) supports_dry_run: bool,
    pub(crate) idempotent: bool,
    pub(crate) destructive: bool,
}

impl CommandSpec {
    /// Whether this spec is selected by an optional `noun`/`verb` filter.
    pub(crate) fn matches(&self, noun: Option<&str>, verb: Option<&str>) -> bool {
        match (noun, verb) {
            (None, _) => true,
            (Some(n), None) => self.noun == n,
            (Some(n), Some(v)) => self.noun == n && self.verb == v,
        }
    }
}

/// Convert a slice of string pairs into owned tuples.
fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

/// Every command the binary currently implements. Extended as later phases add
/// commands; kept in lockstep with [`crate::cli`] by `tests::manifest_in_sync`.
#[allow(
    clippy::too_many_lines,
    reason = "a flat command-spec table reads more clearly as one list than split across helpers"
)]
pub(crate) fn commands() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            name: "construct skill sync".to_owned(),
            noun: "skill".to_owned(),
            verb: "sync".to_owned(),
            description: "Update the Construct flake input in a consuming flake (no rebuild)"
                .to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "required": [],
                "properties": {
                    "flake_dir": {
                        "type": "string",
                        "format": "uri-reference",
                        "description": "Directory of the consuming flake (default: /spacecraft-software/bravais)"
                    }
                }
            }),
            output_data: json!({
                "type": "object",
                "properties": {
                    "flake_dir": { "type": "string" },
                    "input": { "type": "string" },
                    "updated": { "type": "boolean" },
                    "executed": { "type": "boolean" },
                    "synced_at": { "type": "string", "format": "date-time" }
                }
            }),
            exit_codes: pairs(&[
                (
                    "0",
                    "SUCCESS — flake input updated (or planned under --dry-run)",
                ),
                (
                    "1",
                    "GENERAL_FAILURE — `nix flake update` reported an error",
                ),
                ("3", "NOT_FOUND — the flake directory does not exist"),
                ("127", "DEPENDENCY_MISSING — `nix` is not on PATH"),
            ]),
            examples: pairs(&[
                (
                    "construct skill sync",
                    "Update the input in the default bravais flake",
                ),
                (
                    "construct skill sync --json",
                    "Same, with machine-readable output",
                ),
                (
                    "construct skill sync --flake-dir /etc/nixos --dry-run",
                    "Show what would change for /etc/nixos without doing it",
                ),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct schema".to_owned(),
            noun: String::new(),
            verb: "schema".to_owned(),
            description: "Emit JSON Schema (Draft 2020-12) for the tool or a command".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "noun": { "type": "string", "description": "Restrict to this noun" },
                    "verb": { "type": "string", "description": "Restrict to this verb under noun" }
                }
            }),
            output_data: json!({ "type": "object", "description": "JSON Schema document" }),
            exit_codes: pairs(&[
                ("0", "SUCCESS"),
                (
                    "3",
                    "NOT_FOUND — no command matches the requested noun/verb",
                ),
            ]),
            examples: pairs(&[
                ("construct schema", "Full tool schema"),
                ("construct schema skill sync", "Schema for one command"),
            ]),
            supports_json: true,
            supports_dry_run: false,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct describe".to_owned(),
            noun: String::new(),
            verb: "describe".to_owned(),
            description: "Emit the compact capability manifest for agent discovery".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "noun": { "type": "string", "description": "Restrict to this noun" },
                    "verb": { "type": "string", "description": "Restrict to this verb under noun" }
                }
            }),
            output_data: json!({ "type": "object", "description": "Capability manifest" }),
            exit_codes: pairs(&[
                ("0", "SUCCESS"),
                (
                    "3",
                    "NOT_FOUND — no command matches the requested noun/verb",
                ),
            ]),
            examples: pairs(&[
                ("construct describe", "Capability manifest"),
                ("construct describe --json", "Same, machine-readable"),
            ]),
            supports_json: true,
            supports_dry_run: false,
            idempotent: true,
            destructive: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use clap::CommandFactory as _;

    use super::commands;
    use crate::cli::Cli;

    /// Collect every leaf command path (e.g. `construct skill sync`) from a
    /// clap command tree, skipping the auto-generated `help` sub-command.
    fn collect_leaves(cmd: &clap::Command, prefix: &str, out: &mut Vec<String>) {
        let subs: Vec<&clap::Command> = cmd
            .get_subcommands()
            .filter(|c| c.get_name() != "help")
            .collect();
        if subs.is_empty() {
            out.push(prefix.to_owned());
            return;
        }
        for sub in subs {
            let next = format!("{prefix} {}", sub.get_name());
            collect_leaves(sub, &next, out);
        }
    }

    #[test]
    fn manifest_in_sync_with_cli() {
        let root = Cli::command();
        let mut leaves = Vec::new();
        collect_leaves(&root, "construct", &mut leaves);

        let manifest: HashSet<String> = commands().into_iter().map(|c| c.name).collect();
        for leaf in &leaves {
            assert!(
                manifest.contains(leaf),
                "clap command '{leaf}' has no matching manifest entry"
            );
        }
    }

    #[test]
    fn manifest_names_unique() {
        let names: Vec<String> = commands().into_iter().map(|c| c.name).collect();
        let unique: HashSet<&String> = names.iter().collect();
        assert_eq!(
            names.len(),
            unique.len(),
            "duplicate command names in manifest"
        );
    }
}
