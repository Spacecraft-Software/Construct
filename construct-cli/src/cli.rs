// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Command-line surface: the clap tree, the global flags shared by every
//! sub-command, and the value enums for `--format` and `--color`.
//!
//! The tree follows the Spacecraft Software CLI Standard noun-verb shape
//! (`construct <noun-singular> <verb>`) and every global flag in §3 is declared
//! once here as a `global = true` argument so it is accepted wherever it appears
//! on the line.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::output::error::AppError;
use crate::output::mode;

/// Long `--version` text — includes the maintainer line and project URL the
/// Standard (§3) requires.
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n",
    "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
    "\n",
    "https://Construct.SpacecraftSoftware.org/",
);

/// The Spacecraft Software Construct skills package manager.
#[derive(Debug, Parser)]
#[command(
    name = "construct",
    version,
    long_version = LONG_VERSION,
    about = "Install, discover, sync, and ship Spacecraft Software agent skills",
    after_help = "Project: https://Construct.SpacecraftSoftware.org/  \u{2014}  Maintainer: Mohamed Hammad",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub(crate) global: GlobalArgs,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Flags accepted identically across every Spacecraft Software CLI (Standard §3).
#[derive(Debug, Args)]
pub(crate) struct GlobalArgs {
    /// Emit machine-readable JSON (alias for `--format json`).
    #[arg(long, global = true)]
    pub(crate) json: bool,

    /// Output format.
    #[arg(long, value_enum, global = true, value_name = "FMT")]
    pub(crate) format: Option<FormatArg>,

    /// Restrict output to a comma-separated subset of fields.
    #[arg(long, global = true, value_delimiter = ',', value_name = "F1,F2,...")]
    pub(crate) fields: Option<Vec<String>>,

    /// Show the action plan without making any changes.
    #[arg(long, global = true)]
    pub(crate) dry_run: bool,

    /// Increase diagnostic output on stderr (repeatable).
    #[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Suppress non-error diagnostics on stderr.
    #[arg(long, short = 'q', global = true, conflicts_with = "verbose")]
    pub(crate) quiet: bool,

    /// Disable ANSI color (alias for `--color never`).
    #[arg(long, global = true)]
    pub(crate) no_color: bool,

    /// When to use ANSI color.
    #[arg(long, value_enum, global = true, value_name = "WHEN")]
    pub(crate) color: Option<ColorArg>,

    /// Render timestamps as absolute ISO 8601 UTC instead of relative time.
    #[arg(long, global = true)]
    pub(crate) absolute_time: bool,

    /// NUL-delimit list output for safe piping through `xargs -0`.
    #[arg(long, short = '0', global = true)]
    pub(crate) print0: bool,

    /// Assume "yes" for confirmations in non-interactive contexts.
    #[arg(long, short = 'y', visible_alias = "force", global = true)]
    pub(crate) yes: bool,
}

/// `--format` values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum FormatArg {
    Json,
    Jsonl,
    Yaml,
    Csv,
    Explore,
}

/// `--color` values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum ColorArg {
    Auto,
    Always,
    Never,
}

/// Top-level nouns.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Manage agent skills.
    Skill {
        #[command(subcommand)]
        verb: SkillCommand,
    },

    /// Inspect the registry of supported agents.
    Agent {
        #[command(subcommand)]
        verb: AgentCommand,
    },

    /// Emit JSON Schema (Draft 2020-12) for the tool or a specific command.
    #[command(after_help = "Examples:\n  construct schema\n  construct schema skill sync")]
    Schema {
        /// Restrict the schema to this noun (e.g. `skill`).
        noun: Option<String>,
        /// Restrict the schema to this verb under `noun` (e.g. `sync`).
        verb: Option<String>,
    },

    /// Emit the compact capability manifest for agent discovery.
    #[command(after_help = "Examples:\n  construct describe\n  construct describe --json")]
    Describe {
        /// Restrict the manifest to this noun.
        noun: Option<String>,
        /// Restrict the manifest to this verb under `noun`.
        verb: Option<String>,
    },
}

/// Verbs under the `agent` noun.
#[derive(Debug, Subcommand)]
pub(crate) enum AgentCommand {
    /// List every supported agent, its paths, and detection status.
    #[command(
        visible_alias = "ls",
        after_help = "Examples:\n  construct agent list\n  construct agent list --json --fields id,global_path"
    )]
    List,
}

/// Verbs under the `skill` noun.
#[derive(Debug, Subcommand)]
pub(crate) enum SkillCommand {
    /// Install skills from a catalogue source into one or more agents.
    #[command(
        after_help = "Examples:\n  construct skill add --agents claude-code,cursor\n  construct skill add --skills spacecraft-rust-guidelines --all --json\n  construct skill add --global --copy --agents gemini-cli"
    )]
    Add(AddArgs),

    /// List installed skills per agent.
    #[command(
        visible_alias = "ls",
        after_help = "Examples:\n  construct skill list\n  construct skill list --global --json"
    )]
    List(ScopeQueryArgs),

    /// Remove installed skills from one or more agents.
    #[command(
        visible_alias = "rm",
        after_help = "Examples:\n  construct skill remove --skills foo --agents cursor\n  construct skill remove --all --global --dry-run"
    )]
    Remove(RemoveArgs),

    /// Refresh installed skills from the source (overwrites in place).
    #[command(
        after_help = "Examples:\n  construct skill update --agents claude-code\n  construct skill update --all --json"
    )]
    Update(AddArgs),

    /// Browse a source's catalogue (name + description from SKILL.md).
    #[command(
        visible_alias = "search",
        after_help = "Examples:\n  construct skill find\n  construct skill find rust\n  construct skill find --source vercel-labs/skills --json"
    )]
    Find(FindArgs),

    /// Print selected skills' prompts to stdout without installing.
    #[command(
        after_help = "Examples:\n  construct skill use --skills spacecraft-rust-guidelines\n  construct skill use vercel-labs/skills --skills find-skills"
    )]
    Use(UseArgs),

    /// Scaffold a new skill directory with a SKILL.md template.
    #[command(
        after_help = "Examples:\n  construct skill init my-skill\n  construct skill init my-skill --dir ./skills"
    )]
    Init(InitArgs),

    /// Update the Construct flake input in a consuming flake (no rebuild).
    #[command(
        after_help = "Examples:\n  construct skill sync\n  construct skill sync --json\n  construct skill sync --flake-dir /etc/nixos --dry-run"
    )]
    Sync(SyncArgs),

    /// Ship local skill edits: commit (signed) + push, then sync.
    #[command(
        after_help = "Examples:\n  construct skill ship --dry-run\n  construct skill ship --skills spacecraft-rust-guidelines\n  construct skill ship --message \"docs: clarify X\" --json"
    )]
    Ship(ShipArgs),
}

/// Arguments for `construct skill add` / `construct skill update`.
#[derive(Debug, Args)]
pub(crate) struct AddArgs {
    /// Catalogue source path (default: the local Construct clone).
    pub(crate) source: Option<String>,

    /// Target agents (comma-separated ids); defaults to detected-installed.
    #[arg(
        short = 'a',
        long = "agents",
        value_delimiter = ',',
        value_name = "ID[,ID...]"
    )]
    pub(crate) agents: Vec<String>,

    /// Restrict to these skills (comma-separated); defaults to all in source.
    #[arg(
        short = 's',
        long = "skills",
        value_delimiter = ',',
        value_name = "NAME[,NAME...]"
    )]
    pub(crate) skills: Vec<String>,

    /// Install into each agent's global (home) skills dir instead of the project.
    #[arg(short = 'g', long = "global")]
    pub(crate) global: bool,

    /// Copy skills instead of symlinking them.
    #[arg(long)]
    pub(crate) copy: bool,

    /// Target every known agent (not just detected ones).
    #[arg(long)]
    pub(crate) all: bool,

    /// Re-clone a cached remote source before installing.
    #[arg(long)]
    pub(crate) refresh: bool,
}

/// Arguments for `construct skill list`.
#[derive(Debug, Args)]
pub(crate) struct ScopeQueryArgs {
    /// Restrict to these agents (comma-separated ids).
    #[arg(
        short = 'a',
        long = "agents",
        value_delimiter = ',',
        value_name = "ID[,ID...]"
    )]
    pub(crate) agents: Vec<String>,

    /// Inspect the global (home) skills dirs instead of the project.
    #[arg(short = 'g', long = "global")]
    pub(crate) global: bool,
}

/// Arguments for `construct skill remove`.
#[derive(Debug, Args)]
pub(crate) struct RemoveArgs {
    /// Skills to remove (comma-separated); defaults to all installed.
    #[arg(
        short = 's',
        long = "skills",
        value_delimiter = ',',
        value_name = "NAME[,NAME...]"
    )]
    pub(crate) skills: Vec<String>,

    /// Agents to remove from (comma-separated ids).
    #[arg(
        short = 'a',
        long = "agents",
        value_delimiter = ',',
        value_name = "ID[,ID...]"
    )]
    pub(crate) agents: Vec<String>,

    /// Operate on the global (home) skills dirs instead of the project.
    #[arg(short = 'g', long = "global")]
    pub(crate) global: bool,

    /// Required to remove across all agents when no `--skills`/`--agents` given.
    #[arg(long)]
    pub(crate) all: bool,
}

/// Arguments for `construct skill sync`.
#[derive(Debug, Args)]
pub(crate) struct SyncArgs {
    /// Directory of the consuming flake whose `construct` input is updated.
    #[arg(long, value_name = "DIR")]
    pub(crate) flake_dir: Option<PathBuf>,
}

/// Arguments for `construct skill find`.
#[derive(Debug, Args)]
pub(crate) struct FindArgs {
    /// Filter skills whose name or description contains this query.
    pub(crate) query: Option<String>,

    /// Source to browse (local path, git URL, or owner/repo; default Construct).
    #[arg(long, value_name = "SRC")]
    pub(crate) source: Option<String>,
}

/// Arguments for `construct skill use`.
#[derive(Debug, Args)]
pub(crate) struct UseArgs {
    /// Source (local path, git URL, or owner/repo); default the Construct clone.
    pub(crate) source: Option<String>,

    /// Skills whose prompts to print (comma-separated); default all in source.
    #[arg(
        short = 's',
        long = "skills",
        value_delimiter = ',',
        value_name = "NAME[,NAME...]"
    )]
    pub(crate) skills: Vec<String>,
}

/// Arguments for `construct skill init`.
#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// Name (directory) of the new skill.
    pub(crate) name: String,

    /// Parent directory to create the skill under (default: current dir).
    #[arg(long, value_name = "DIR")]
    pub(crate) dir: Option<PathBuf>,
}

/// Arguments for `construct skill ship`.
#[derive(Debug, Args)]
pub(crate) struct ShipArgs {
    /// The Construct clone to ship from (default: the local catalogue clone).
    #[arg(long, value_name = "DIR")]
    pub(crate) repo: Option<PathBuf>,

    /// Restrict to these skills (comma-separated); defaults to all changed.
    #[arg(
        short = 's',
        long = "skills",
        value_delimiter = ',',
        value_name = "NAME[,NAME...]"
    )]
    pub(crate) skills: Vec<String>,

    /// Commit message subject (default: derived from shipped skills).
    #[arg(short = 'm', long = "message", value_name = "MSG")]
    pub(crate) message: Option<String>,

    /// Commit and push but skip the final `skill sync` step.
    #[arg(long)]
    pub(crate) no_sync: bool,
}

/// Render a clap parse outcome (help, version, or error) and return the exit
/// code to use. Help and version are successes; everything else is a usage
/// error reported structurally in machine mode (Standard §1 item 8).
pub(crate) fn handle_parse_error(err: &clap::Error) -> i32 {
    use clap::error::ErrorKind;
    match err.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            // clap routes help/version to stdout; honor its formatting.
            let _ = err.print();
            0
        }
        _ => {
            if mode::is_machine_early() {
                AppError::usage_early(&clap_error_summary(err)).emit_to_stderr();
            } else {
                let _ = err.print();
            }
            2
        }
    }
}

/// Collapse a clap error into a single-line message suitable for the structured
/// error `message` field.
fn clap_error_summary(err: &clap::Error) -> String {
    err.to_string()
        .lines()
        .find(|line| {
            let t = line.trim();
            !t.is_empty() && !t.starts_with("Usage") && !t.starts_with("For more")
        })
        .unwrap_or("invalid command-line arguments")
        .trim()
        .trim_start_matches("error:")
        .trim()
        .to_string()
}
