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

/// The `data` schema shared by the installer commands (add/list/remove/update).
fn install_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "source": { "type": ["string", "null"] },
            "scope": { "type": "string" },
            "mode": { "type": ["string", "null"] },
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "agent": { "type": "string" },
                        "skill": { "type": "string" },
                        "scope": { "type": "string" },
                        "target": { "type": "string" },
                        "action": { "type": "string" },
                        "detail": { "type": ["string", "null"] }
                    }
                }
            }
        }
    })
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
            name: "construct skill ship".to_owned(),
            noun: "skill".to_owned(),
            verb: "ship".to_owned(),
            description: "Commit (signed) + push local skill edits, then sync".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "repo": { "type": "string", "description": "Construct clone to ship from" },
                    "skills": { "type": "array", "items": { "type": "string" }, "description": "Restrict to these skills (default: all changed)" },
                    "message": { "type": "string", "description": "Commit message subject" },
                    "no_sync": { "type": "boolean", "default": false, "description": "Skip the final flake sync" }
                }
            }),
            output_data: json!({
                "type": "object",
                "properties": {
                    "repo": { "type": "string" },
                    "status": { "type": "string" },
                    "shipped_skills": { "type": "array", "items": { "type": "string" } },
                    "committed": { "type": "boolean" },
                    "commit_sha": { "type": "string" },
                    "signed": { "type": "boolean" },
                    "pushed": { "type": "boolean" },
                    "flake_updated": { "type": "boolean" },
                    "synced_at": { "type": ["string", "null"] }
                }
            }),
            exit_codes: pairs(&[
                ("0", "SUCCESS — shipped (or planned under --dry-run)"),
                ("1", "GENERAL_FAILURE — a git or nix command failed"),
                ("2", "USAGE_ERROR — repo is not the Construct work tree"),
                ("3", "NOT_FOUND — repo path does not exist"),
                (
                    "5",
                    "CONFLICT — skill source changed without rebuilt .zip/.skill bundles",
                ),
                ("127", "DEPENDENCY_MISSING — git or nix not on PATH"),
            ]),
            examples: pairs(&[
                (
                    "construct skill ship --dry-run",
                    "Preview what would be committed and pushed",
                ),
                (
                    "construct skill ship --skills spacecraft-rust-guidelines",
                    "Ship one skill's edits",
                ),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: false,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill find".to_owned(),
            noun: "skill".to_owned(),
            verb: "find".to_owned(),
            description: "Browse a source's catalogue (name + description)".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "query": { "type": "string", "description": "Filter by name/description substring" },
                    "source": { "type": "string", "description": "Source to browse (default: Construct)" }
                }
            }),
            output_data: json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "description": { "type": ["string", "null"] }
                    }
                }
            }),
            exit_codes: pairs(&[
                ("0", "SUCCESS"),
                (
                    "3",
                    "NOT_FOUND — source has no skills or cannot be resolved",
                ),
                (
                    "127",
                    "DEPENDENCY_MISSING — git not on PATH (remote source)",
                ),
            ]),
            examples: pairs(&[
                ("construct skill find", "List the local Construct catalogue"),
                (
                    "construct skill find --source vercel-labs/skills --json",
                    "Browse a remote source",
                ),
            ]),
            supports_json: true,
            supports_dry_run: false,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill use".to_owned(),
            noun: "skill".to_owned(),
            verb: "use".to_owned(),
            description: "Print selected skills' prompts without installing".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "source": { "type": "string" },
                    "skills": { "type": "array", "items": { "type": "string" } }
                }
            }),
            output_data: json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "skill": { "type": "string" },
                        "content": { "type": "string" }
                    }
                }
            }),
            exit_codes: pairs(&[
                ("0", "SUCCESS"),
                ("3", "NOT_FOUND — source or skill not found"),
            ]),
            examples: pairs(&[
                (
                    "construct skill use --skills spacecraft-rust-guidelines",
                    "Print one skill's prompt",
                ),
                (
                    "construct skill use vercel-labs/skills --skills find-skills",
                    "Print from a remote source",
                ),
            ]),
            supports_json: true,
            supports_dry_run: false,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill init".to_owned(),
            noun: "skill".to_owned(),
            verb: "init".to_owned(),
            description: "Scaffold a new skill directory with a SKILL.md template".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["name"],
                "properties": {
                    "name": { "type": "string", "description": "Skill (directory) name" },
                    "dir": { "type": "string", "description": "Parent directory (default: current)" }
                }
            }),
            output_data: json!({
                "type": "object",
                "properties": {
                    "created": { "type": "boolean" },
                    "path": { "type": "string" }
                }
            }),
            exit_codes: pairs(&[
                ("0", "SUCCESS — created (or planned under --dry-run)"),
                ("1", "GENERAL_FAILURE — filesystem error"),
                ("5", "CONFLICT — a SKILL.md already exists at that path"),
            ]),
            examples: pairs(&[
                (
                    "construct skill init my-skill",
                    "Scaffold ./my-skill/SKILL.md",
                ),
                (
                    "construct skill init my-skill --dir ./skills",
                    "Scaffold under ./skills",
                ),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: false,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill add".to_owned(),
            noun: "skill".to_owned(),
            verb: "add".to_owned(),
            description: "Install skills from a catalogue source into one or more agents"
                .to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "source": { "type": "string", "description": "Catalogue source path (default: the local Construct clone)" },
                    "agents": { "type": "array", "items": { "type": "string" }, "description": "Target agent ids (default: detected-installed)" },
                    "skills": { "type": "array", "items": { "type": "string" }, "description": "Skills to install (default: all in source)" },
                    "global": { "type": "boolean", "default": false, "description": "Install into each agent's home skills dir" },
                    "copy": { "type": "boolean", "default": false, "description": "Copy instead of symlink" },
                    "all": { "type": "boolean", "default": false, "description": "Target every known agent" }
                }
            }),
            output_data: install_output_schema(),
            exit_codes: pairs(&[
                (
                    "0",
                    "SUCCESS — install plan applied (or planned under --dry-run)",
                ),
                (
                    "1",
                    "GENERAL_FAILURE / FEATURE_UNAVAILABLE — filesystem error or remote source",
                ),
                (
                    "2",
                    "MISSING_ARGUMENT — no agents specified and none detected",
                ),
                ("3", "NOT_FOUND — unknown agent, skill, or source path"),
                (
                    "5",
                    "CONFLICT — explicit --global into a Home-Manager-managed dir",
                ),
            ]),
            examples: pairs(&[
                (
                    "construct skill add --agents claude-code,cursor",
                    "Install all catalogue skills into two agents (project-local)",
                ),
                (
                    "construct skill add --skills spacecraft-rust-guidelines --all --json",
                    "Install one skill into every agent",
                ),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill list".to_owned(),
            noun: "skill".to_owned(),
            verb: "list".to_owned(),
            description: "List installed skills per agent".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "agents": { "type": "array", "items": { "type": "string" } },
                    "global": { "type": "boolean", "default": false }
                }
            }),
            output_data: install_output_schema(),
            exit_codes: pairs(&[("0", "SUCCESS"), ("3", "NOT_FOUND — unknown agent")]),
            examples: pairs(&[
                (
                    "construct skill list",
                    "List project-local installed skills",
                ),
                (
                    "construct skill list --global --json",
                    "List globally installed skills",
                ),
            ]),
            supports_json: true,
            supports_dry_run: false,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct skill remove".to_owned(),
            noun: "skill".to_owned(),
            verb: "remove".to_owned(),
            description: "Remove installed skills from one or more agents".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "skills": { "type": "array", "items": { "type": "string" } },
                    "agents": { "type": "array", "items": { "type": "string" } },
                    "global": { "type": "boolean", "default": false },
                    "all": { "type": "boolean", "default": false }
                }
            }),
            output_data: install_output_schema(),
            exit_codes: pairs(&[
                ("0", "SUCCESS — removed (or planned under --dry-run)"),
                (
                    "2",
                    "MISSING_ARGUMENT — refused to remove everything without a selection",
                ),
                ("3", "NOT_FOUND — unknown agent"),
            ]),
            examples: pairs(&[
                (
                    "construct skill remove --skills foo --agents cursor",
                    "Remove one skill from one agent",
                ),
                (
                    "construct skill remove --all --global --dry-run",
                    "Preview removing all globally installed skills",
                ),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: true,
            destructive: true,
        },
        CommandSpec {
            name: "construct skill update".to_owned(),
            noun: "skill".to_owned(),
            verb: "update".to_owned(),
            description: "Refresh installed skills from the source (overwrites in place)"
                .to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "source": { "type": "string" },
                    "agents": { "type": "array", "items": { "type": "string" } },
                    "skills": { "type": "array", "items": { "type": "string" } },
                    "global": { "type": "boolean", "default": false },
                    "copy": { "type": "boolean", "default": false },
                    "all": { "type": "boolean", "default": false }
                }
            }),
            output_data: install_output_schema(),
            exit_codes: pairs(&[
                ("0", "SUCCESS"),
                ("1", "GENERAL_FAILURE — filesystem error"),
                ("3", "NOT_FOUND — unknown agent, skill, or source"),
                (
                    "5",
                    "CONFLICT — explicit --global into a Home-Manager-managed dir",
                ),
            ]),
            examples: pairs(&[
                (
                    "construct skill update --agents claude-code",
                    "Refresh a single agent's skills",
                ),
                ("construct skill update --all --json", "Refresh every agent"),
            ]),
            supports_json: true,
            supports_dry_run: true,
            idempotent: true,
            destructive: false,
        },
        CommandSpec {
            name: "construct agent list".to_owned(),
            noun: "agent".to_owned(),
            verb: "list".to_owned(),
            description: "List every supported agent and its install paths".to_owned(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {}
            }),
            output_data: json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "display_name": { "type": "string" },
                        "project_path": { "type": "string" },
                        "global_path": { "type": ["string", "null"] },
                        "format": { "type": "string", "enum": ["directory", "flat"] },
                        "installed": { "type": "boolean" },
                        "hm_managed": { "type": "boolean" }
                    }
                }
            }),
            exit_codes: pairs(&[("0", "SUCCESS")]),
            examples: pairs(&[
                ("construct agent list", "List all supported agents"),
                (
                    "construct agent list --json --fields id,global_path",
                    "Just ids and global paths",
                ),
            ]),
            supports_json: true,
            supports_dry_run: false,
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
