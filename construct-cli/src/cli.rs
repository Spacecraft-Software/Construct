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

/// The maintainer line `--version` carries (CLI Standard §3).
pub(crate) const MAINTAINER: &str = "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>";

/// The project URL `--version` and every help footer carry (CLI Standard §3).
pub(crate) const WEBSITE: &str = "https://Construct.SpacecraftSoftware.org/";

/// Long `--version` text — includes the maintainer line and project URL the
/// Standard (§3) requires.
const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n",
    "Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
    "\n",
    "https://Construct.SpacecraftSoftware.org/",
);

/// Footer appended to the help of every command, not only the root: CLI
/// Standard §3 requires the project URL and maintainer on each `--help`.
pub(crate) const HELP_FOOTER: &str =
    "Project: https://Construct.SpacecraftSoftware.org/  \u{2014}  Maintainer: Mohamed Hammad";

/// The Spacecraft Software Construct skills package manager.
#[derive(Debug, Parser)]
// `help` stays enabled (clap default) so `construct help [CMD]` works; the
// `manifest_in_sync_with_cli` test skips the auto-generated `help` leaf. Do
// NOT add `disable_help_subcommand` — it regresses `construct help` to exit 2.
#[command(
    name = "construct",
    version,
    long_version = LONG_VERSION,
    about = "Install, discover, sync, and ship Spacecraft Software agent skills",
    after_help = "Examples:\n  construct skill list\n  construct describe --json",
    propagate_version = true
)]
pub(crate) struct Cli {
    #[command(flatten)]
    pub(crate) global: GlobalArgs,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// The clap command tree with [`HELP_FOOTER`] appended to every command's
/// help. All parsing and introspection go through this, so no command's help
/// can lose the footer.
pub(crate) fn command() -> clap::Command {
    use clap::CommandFactory as _;
    with_footer(Cli::command())
}

/// Append [`HELP_FOOTER`] to `cmd`'s `after_help` and recurse into its
/// sub-commands.
fn with_footer(cmd: clap::Command) -> clap::Command {
    let after = match cmd.get_after_help() {
        Some(text) => format!("{text}\n\n{HELP_FOOTER}"),
        None => HELP_FOOTER.to_owned(),
    };
    cmd.after_help(after).mut_subcommands(with_footer)
}

/// Parse the process arguments against [`command`].
pub(crate) fn parse() -> Result<Cli, clap::Error> {
    use clap::FromArgMatches as _;
    let mut matches = command().try_get_matches()?;
    Cli::from_arg_matches_mut(&mut matches)
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

    /// Accessible mode (Standard §18): no animation; progress as static,
    /// append-only lines at most once per second. Beats `SPACECRAFT_A11Y`.
    #[arg(long, global = true, conflicts_with = "no_accessible")]
    pub(crate) accessible: bool,

    /// Force accessible mode off, overriding `SPACECRAFT_A11Y=1`.
    #[arg(long, global = true)]
    pub(crate) no_accessible: bool,

    /// Assume "yes" for confirmations in non-interactive contexts.
    #[arg(long, short = 'y', visible_alias = "force", global = true)]
    pub(crate) yes: bool,
}

/// `--format` values.
///
/// The four provider values are accepted only by `construct schema`
/// (agentic-cli `llm-tool-calling.md` §6) and are hidden from the shared
/// global help; any other command refuses them with a usage error.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum FormatArg {
    Json,
    Jsonl,
    Yaml,
    Csv,
    Explore,
    /// `schema` only: Anthropic `tools[]` entries (`input_schema`).
    #[value(hide = true)]
    Anthropic,
    /// `schema` only: OpenAI strict function tools.
    #[value(name = "openai", hide = true)]
    OpenAi,
    /// `schema` only: Gemini `function_declarations`.
    #[value(hide = true)]
    Gemini,
    /// `schema` only: the MCP `tools/list` shape.
    #[value(hide = true)]
    Mcp,
}

/// The LLM function-calling shape `construct schema --format <provider>`
/// emits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SchemaProvider {
    Anthropic,
    OpenAi,
    Gemini,
    Mcp,
}

impl FormatArg {
    /// The provider this value names, when it is one of the `schema`-only
    /// provider formats.
    pub(crate) fn provider(self) -> Option<SchemaProvider> {
        match self {
            Self::Anthropic => Some(SchemaProvider::Anthropic),
            Self::OpenAi => Some(SchemaProvider::OpenAi),
            Self::Gemini => Some(SchemaProvider::Gemini),
            Self::Mcp => Some(SchemaProvider::Mcp),
            Self::Json | Self::Jsonl | Self::Yaml | Self::Csv | Self::Explore => None,
        }
    }

    /// The `--format` spelling of this value.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Yaml => "yaml",
            Self::Csv => "csv",
            Self::Explore => "explore",
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::Gemini => "gemini",
            Self::Mcp => "mcp",
        }
    }
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
    #[command(
        after_help = "Examples:\n  construct skill list\n  construct skill find rust --json"
    )]
    Skill {
        #[command(subcommand)]
        verb: SkillCommand,
    },

    /// Inspect the registry of supported agents.
    #[command(
        after_help = "Examples:\n  construct agent list\n  construct agent list --json --fields id"
    )]
    Agent {
        #[command(subcommand)]
        verb: AgentCommand,
    },

    /// Emit JSON Schema (Draft 2020-12) for the tool or a specific command.
    #[command(
        after_help = "Provider shapes (this command only): --format anthropic | openai | gemini | mcp\n\nExamples:\n  construct schema\n  construct schema skill sync --json\n  construct schema skill list --format openai"
    )]
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

    /// Serve every command as an MCP tool over stdio (JSON-RPC, lazy schemas).
    #[command(
        after_help = "Advertises tool names and one-line descriptions only; `tools/get` returns\na tool's full schema. Each `tools/call` runs the matching command with\n--json in a child process.\n\nExamples:\n  construct mcp\n  construct mcp --transport stdio --verbose\n  construct schema --format mcp --json   # the tool set it serves"
    )]
    Mcp(McpArgs),
}

/// `--transport` values for `construct mcp`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum TransportArg {
    /// Newline-delimited JSON-RPC on stdin/stdout.
    Stdio,
}

/// Arguments for `construct mcp`.
#[derive(Debug, Args)]
pub(crate) struct McpArgs {
    /// MCP transport (stdio is the only one, and the default).
    #[arg(long, value_enum, default_value = "stdio", value_name = "T")]
    pub(crate) transport: TransportArg,
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

    /// Delete installed skills from one or more agents.
    ///
    /// `delete` is the Standard verb (CLI Standard §2 rule 7); `remove` is
    /// kept as a hidden alias so existing scripts keep working.
    #[command(
        visible_alias = "rm",
        alias = "remove",
        after_help = "Destructive: off a terminal it needs --yes (or --force); on a terminal it asks first.\n\nExamples:\n  construct skill delete --skills foo --agents cursor --yes\n  construct skill delete --all --global --dry-run --json"
    )]
    Delete(DeleteArgs),

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
        after_help = "Examples:\n  construct skill use --skills spacecraft-rust-guidelines\n  construct skill use vercel-labs/skills --skills find-skills --json"
    )]
    Use(UseArgs),

    /// Scaffold a new skill directory with a SKILL.md template.
    #[command(
        after_help = "NAME must be a skill id: lowercase letters, digits and single hyphens.\n\nExamples:\n  construct skill init my-skill\n  construct skill init my-skill --dir ./skills --dry-run --json"
    )]
    Init(InitArgs),

    /// Update the Construct flake input in a consuming flake (no rebuild).
    #[command(
        after_help = "With --build, also builds the flake's `skills` output and moves the\nmutable pointer to it, so the new skills are live immediately — no\nrebuild, no sudo. Requires the Home-Manager module's mutablePointer\nmode; without it there is no pointer to move.\n\nExamples:\n  construct skill sync\n  construct skill sync --build\n  construct skill sync --build --no-update   # re-point from the current lock\n  construct skill sync --flake-dir /etc/nixos --dry-run --json"
    )]
    Sync(SyncArgs),

    /// Show whether the live skill tree tracks the flake-pinned one.
    #[command(after_help = "Examples:\n  construct skill status\n  construct skill status --json")]
    Status(PointerArgs),

    /// Point the live skill tree back at what flake.lock pins.
    #[command(
        after_help = "Examples:\n  construct skill reset\n  construct skill reset --dry-run --json"
    )]
    Reset(PointerArgs),

    /// Ship local skill edits: branch + commit (signed) + open a pull request.
    #[command(
        after_help = "Never pushes to the default branch — every change goes through a\nfeature branch and a pull request, and merging is the maintainer's call.\nRun `construct skill sync` after the PR merges.\n\nExamples:\n  construct skill ship --dry-run\n  construct skill ship --skills spacecraft-rust-guidelines\n  construct skill ship --branch ship/palette-fix --message \"docs: clarify X\" --json"
    )]
    Ship(ShipArgs),

    /// Build distributable bundles for every target platform under dist/.
    #[command(
        after_help = "Targets: claude (Claude Code, claude.ai, Gemini CLI, Codex), chatgpt (ChatGPT web),
grok (flat zips),\nperplexity (consolidated under 100 files), gemini (Gemini app: flat zips,
.csv/.py/.txt/.md only), single-file (one .md per skill),\nminimax (MiniMax Agent web, one SKILL.md per bundle).\nZips are deterministic; nothing is committed. A refused skill stops the\nwhole build before anything is written.\n\nExamples:\n  construct skill build\n  construct skill build --target claude,grok --json\n  construct skill build spacecraft-cli-preference --target perplexity --dry-run"
    )]
    Build(BuildArgs),

    /// Vendor skills into a repository's .claude/skills/ for Claude Code cloud sessions.
    #[command(
        after_help = "Writes the Claude-layout tree for each skill into <repo>/.claude/skills/<name>/,\nthe only place Claude Code cloud sessions load skills from. Never runs git:\ncommit the printed paths yourself. Replaces only directories it created\n(marked by .construct-vendor.toml) unless --force.\n\nExamples:\n  construct skill vendor spacecraft-rust-guidelines microsoft-rust-guidelines\n  construct skill vendor spacecraft-brand-guidelines --into ../site --dry-run --json"
    )]
    Vendor(VendorArgs),
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

/// Arguments for `construct skill delete`.
#[derive(Debug, Args)]
pub(crate) struct DeleteArgs {
    /// Skills to delete (comma-separated); defaults to all installed.
    #[arg(
        short = 's',
        long = "skills",
        value_delimiter = ',',
        value_name = "NAME[,NAME...]"
    )]
    pub(crate) skills: Vec<String>,

    /// Agents to delete from (comma-separated ids).
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

    /// Required to delete across all agents when no `--skills`/`--agents` given.
    #[arg(long)]
    pub(crate) all: bool,
}

/// Arguments for `construct skill sync`.
#[derive(Debug, Args)]
pub(crate) struct SyncArgs {
    /// Directory of the consuming flake whose `construct` input is updated.
    #[arg(long, value_name = "DIR")]
    pub(crate) flake_dir: Option<PathBuf>,

    /// After updating, build the flake's `skills` output and move the pointer
    /// to it — applies the new skills with no rebuild and no sudo.
    #[arg(long)]
    pub(crate) build: bool,

    /// Skip the flake update; build the pointer from the lock as it stands.
    #[arg(long, requires = "build")]
    pub(crate) no_update: bool,

    /// Pointer to move (default: ~/.local/state/construct/current).
    #[arg(long, value_name = "PATH", requires = "build")]
    pub(crate) pointer: Option<PathBuf>,
}

/// Arguments for the pointer verbs (`skill status`, `skill reset`).
#[derive(Debug, Args)]
pub(crate) struct PointerArgs {
    /// Pointer to inspect or reset (default: ~/.local/state/construct/current).
    #[arg(long, value_name = "PATH")]
    pub(crate) pointer: Option<PathBuf>,
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

    /// Feature branch to commit onto (default: derived from the shipped
    /// skills, or the current branch when it is not the default branch).
    #[arg(short = 'b', long = "branch", value_name = "NAME")]
    pub(crate) branch: Option<String>,

    /// Deprecated no-op. `ship` opens a pull request instead of pushing to the
    /// default branch, so nothing lands to sync; run `construct skill sync`
    /// after the PR merges.
    #[arg(long, hide = true)]
    pub(crate) no_sync: bool,
}

/// `--target` values for `construct skill build`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum TargetArg {
    /// Claude Code (local + web), claude.ai, Gemini CLI, Codex.
    Claude,
    /// ChatGPT (web): Claude layout without the invocation controls.
    #[value(name = "chatgpt")]
    ChatGpt,
    /// Grok: flat bundles, name + description frontmatter.
    Grok,
    /// Perplexity: Claude layout, consolidated under 100 files.
    Perplexity,
    /// The Gemini app: flat zips of .csv/.py/.txt/.md files only.
    Gemini,
    /// One self-contained markdown file per skill.
    SingleFile,
    /// `MiniMax` Agent (web): ChatGPT layout, one `SKILL.md` per bundle.
    #[value(name = "minimax")]
    MiniMax,
    /// Every target above.
    All,
}

/// Arguments for `construct skill build`.
#[derive(Debug, Args)]
pub(crate) struct BuildArgs {
    /// Skills to build (default: every root skill, plus Grok-native skills
    /// for the grok target).
    #[arg(value_name = "SKILL")]
    pub(crate) skills: Vec<String>,

    /// Targets to build (comma-separated or repeated; default: all).
    #[arg(
        short = 't',
        long = "target",
        value_enum,
        value_delimiter = ',',
        value_name = "T[,T...]"
    )]
    pub(crate) targets: Vec<TargetArg>,

    /// The Construct catalogue clone to build from.
    #[arg(long, value_name = "DIR")]
    pub(crate) repo: Option<PathBuf>,

    /// Output root; each target writes <out>/<target>/ (default: <repo>/dist).
    #[arg(long, value_name = "DIR")]
    pub(crate) out: Option<PathBuf>,
}

/// Arguments for `construct skill vendor`.
#[derive(Debug, Args)]
pub(crate) struct VendorArgs {
    /// Skills to vendor (at least one).
    #[arg(value_name = "SKILL", required = true, num_args = 1..)]
    pub(crate) skills: Vec<String>,

    /// Consumer repository to vendor into; writes <into>/.claude/skills/<name>/
    /// (default: the enclosing git work tree of the current directory).
    #[arg(long, value_name = "DIR")]
    pub(crate) into: Option<PathBuf>,

    /// Catalogue source: a local path, git URL, or owner/repo.
    #[arg(long, value_name = "SRC")]
    pub(crate) source: Option<String>,
}

/// Render a clap parse outcome (help, version, or error) and return the exit
/// code to use. Help and version are successes; everything else is a usage
/// error reported structurally in machine mode (Standard §1 item 8).
pub(crate) fn handle_parse_error(err: &clap::Error) -> i32 {
    use clap::error::ErrorKind;
    // `args_os`, not `args`: clap reports a non-UTF-8 argument as a usage
    // error, and `std::env::args` would panic on the same input.
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let machine = mode::is_machine_requested(&args);
    match err.kind() {
        // CLI Standard §3: in machine mode `--version` is an envelope whose
        // metadata carries the maintainer and website keys.
        ErrorKind::DisplayVersion if machine => {
            crate::output::render::emit_version_json();
            0
        }
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            // clap routes help/version to stdout; honor its formatting.
            let _ = err.print();
            0
        }
        _ => {
            if machine {
                AppError::usage_early(&clap_error_summary(err), &help_hint(&args)).emit_to_stderr();
            } else {
                let _ = err.print();
            }
            2
        }
    }
}

/// Global flags that take a separate value, so the value is not mistaken for
/// a sub-command name when walking the tree.
const VALUE_FLAGS: [&str; 3] = ["--format", "--fields", "--color"];

/// The `--help` invocation for the deepest command `args` names, e.g.
/// `construct skill vendor --help` for a bad `skill vendor` line — the
/// runnable recovery step for a usage error (tips-thinking), rather than the
/// root help.
pub(crate) fn help_hint(args: &[String]) -> String {
    let root = command();
    let mut cmd = &root;
    let mut path = vec!["construct".to_owned()];
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if VALUE_FLAGS.contains(&arg.as_str()) {
            iter.next();
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        let Some(sub) = cmd
            .get_subcommands()
            .find(|c| c.get_name() == arg || c.get_all_aliases().any(|a| a == arg))
        else {
            break;
        };
        path.push(sub.get_name().to_owned());
        cmd = sub;
    }
    path.push("--help".to_owned());
    path.join(" ")
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

/// The first argument carrying a C0 control character the agent threat model
/// rejects at parse time (CLI Standard `validation-safety.md` §2): 0x00–0x08,
/// 0x0B–0x0C, 0x0E–0x1F. Tab, line feed, and carriage return stay legal —
/// a commit message may span lines. Returns the 1-based argument position
/// and the offending byte.
pub(crate) fn find_control_char(args: &[std::ffi::OsString]) -> Option<(usize, u8)> {
    args.iter().enumerate().find_map(|(i, arg)| {
        arg.as_encoded_bytes()
            .iter()
            .copied()
            .find(|&b| matches!(b, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F))
            .map(|b| (i + 1, b))
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn help_hint_names_the_deepest_valid_command() {
        assert_eq!(
            help_hint(&strings(&["skill", "vendor", "--bogus"])),
            "construct skill vendor --help"
        );
        assert_eq!(
            help_hint(&strings(&["--format", "json", "skill", "bogus"])),
            "construct skill --help"
        );
        assert_eq!(help_hint(&strings(&["--bogus"])), "construct --help");
        assert_eq!(
            help_hint(&strings(&["skill", "rm"])),
            "construct skill delete --help"
        );
    }

    #[test]
    fn control_characters_are_found_but_whitespace_is_not() {
        let ok: Vec<OsString> = ["a\tb", "line\nnext", "cr\r"]
            .iter()
            .map(OsString::from)
            .collect();
        assert_eq!(find_control_char(&ok), None);
        let bad: Vec<OsString> = ["skill", "x\u{1b}[31m"]
            .iter()
            .map(OsString::from)
            .collect();
        assert_eq!(find_control_char(&bad), Some((2, 0x1b)));
        let nul: Vec<OsString> = ["a\0"].iter().map(OsString::from).collect();
        assert_eq!(find_control_char(&nul), Some((1, 0)));
    }

    #[test]
    fn every_command_help_carries_the_footer() {
        fn walk(cmd: &clap::Command, path: &str) {
            let after = cmd
                .get_after_help()
                .map(ToString::to_string)
                .unwrap_or_default();
            assert!(after.contains(HELP_FOOTER), "{path}: help lacks the footer");
            for sub in cmd.get_subcommands() {
                walk(sub, &format!("{path} {}", sub.get_name()));
            }
        }
        walk(&command(), "construct");
    }
}
