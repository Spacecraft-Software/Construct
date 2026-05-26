<!--
  README for the Spacecraft Software `Construct` repository.
  Audience: humans browsing on GitHub, and LLM agents loading these skills.
  Maintenance: keep the skill list in §2 aligned with the top-level directories.
-->

# Spacecraft Software Construct

A collection of **Claude / LLM agent skills** used across the Spacecraft Software
ecosystem. Each top-level directory is a self-contained skill — a `SKILL.md`
that the agent loads on demand, plus optional `references/` files consulted
only when a deeper lookup is warranted.

These skills encode conventions, tool preferences, brand rules, and compliance
requirements so agents produce Spacecraft Software-consistent output without needing
the rules re-attached to every prompt.

<!-- §1 — Who this is for -->
## Audience

- **LLM agents** (Claude Code, Gemini CLI, Copilot CLI, Codex, etc.) loading
  skills from `~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`.
- **Humans** reviewing, extending, or auditing Spacecraft Software's conventions.

<!-- §2 — Skill catalogue: keep alphabetical, one line per skill -->
## Skills in this repository

| Skill | Purpose |
|-------|---------|
| [`gnu-coding-standards`](gnu-coding-standards/) | Applies the GNU Coding Standards to C, Rust, GNU Guile, Go, and Python — error-message grammar, CLI contract, i18n, build conventions, and free-software philosophy. |
| [`spacecraft-guile-guidelines`](spacecraft-guile-guidelines/) | Write idiomatic, functional, concurrent GNU Guile (Guile Scheme 3.x) — fibers/CSP, SRFI-1, tail calls, hygienic macros. |
| [`microsoft-rust-guidelines`](microsoft-rust-guidelines/) | Enforces Microsoft Pragmatic Rust Guidelines before any `.rs` edit. |
| [`spacecraft-agentic-cli`](spacecraft-agentic-cli/) | Agent-facing UX layer for Spacecraft Software CLIs — pairs with `spacecraft-cli-standard`. |
| [`spacecraft-brand-guidelines`](spacecraft-brand-guidelines/) | Applies Spacecraft Software's official colors and typography to artifacts. |
| [`spacecraft-cli-preference`](spacecraft-cli-preference/) | Modern CLI substitutions: `eza` for `ls`, `rg` for `grep`, `gitway` for Git SSH, etc. |
| [`spacecraft-cli-standard`](spacecraft-cli-standard/) | Enforces the Spacecraft Software Dual-Mode Self-Documenting CLI Standard (SFRS v1.0.0) on every CLI. |
| [`spacecraft-cli-shell`](spacecraft-cli-shell/) | Syntax-compliance guard for Nushell / Ion / POSIX / Bash commands. |
| [`spacecraft-document-format`](spacecraft-document-format/) | ODF-primary office suite (`.odt` / `.ods` / `.odp`) with MS Office (`.docx` / `.xlsx` / `.pptx`) as secondary; GFM Markdown companion always paired; PDF as tertiary export; Void Navy + Standard §9 palette. |
| [`spacecraft-golang-guidelines`](spacecraft-golang-guidelines/) | High-performance concurrent Go guidance — goroutines, channels, errgroup, context cancellation, atomics, `sync.Pool`, pprof / race-detector workflow, and memory-safe parallelism patterns. |
| [`spacecraft-markdown-document`](spacecraft-markdown-document/) | Produces well-formed GFM documents conforming to the GitHub Flavored Markdown spec and Spacecraft Software house style. Slash-command only: `/spacecraft-markdown-document`. |
| [`spacecraft-missing-pkg`](spacecraft-missing-pkg/) | Handles missing-package situations in the Spacecraft Software workflow. |
| [`spacecraft-rust-guidelines`](spacecraft-rust-guidelines/) | High-performance concurrent Rust guidance — concurrency model selection, lock-free synchronisation, memory layout, tooling gates, and unsafe hygiene for Spacecraft Software systems. |
| [`spacecraft-standard`](spacecraft-standard/) | Authoritative compliance reference (The Spacecraft Software Standard v1.2). |
| [`spacecraft-theme-factory`](spacecraft-theme-factory/) | Generates Spacecraft Software-compliant themes for IDEs and terminals. |
| [`spacecraft-zig-guidelines`](spacecraft-zig-guidelines/) | Memory-safe high-performance concurrent Zig guidance — `std.Thread.Pool` / `std.Io.Threaded`, atomics, allocator discipline, comptime safety, and CPU-bound scaling patterns. |

<!-- §3 — Layout convention -->
## Directory layout

Every skill follows the same shape:

```
<skill-name>/
├── SKILL.md           # Frontmatter + the agent-facing instructions
├── LICENSE.md         # GPL-3.0-or-later (per Spacecraft Software Standard §4)
├── CREDITS.md         # Required when the skill builds on third-party work (Standard §13.3)
└── references/        # Optional; consulted only when depth is needed
    ├── <topic>.md
    └── ATTRIBUTION.md # Optional deeper credit file for adapted references
```

Skills are also distributed as `<skill-name>.skill` bundles (zipped) at the
repository root for drop-in installation.

<!-- §4 — Installation -->
## Installation

### Direct clone

Clone into any of the supported agent skill directories:

```sh
# Claude Code
git clone git@github.com:Spacecraft-Software/Construct.git ~/.claude/skills

# Gemini CLI
git clone git@github.com:Spacecraft-Software/Construct.git ~/.gemini/skills

# Codex
git clone git@github.com:Spacecraft-Software/Construct.git ~/.codex/skills

# Grok
git clone git@github.com:Spacecraft-Software/Construct.git ~/.grok/skills
```

The SSH remote is configured to work with
[Gitway](https://github.com/Spacecraft-Software/Gitway), Spacecraft Software's pinned-host-key
SSH transport for Git.

### Nix flake (Home Manager)

The repository is a Nix flake. Add it as an input and import the Home Manager
module:

```nix
{
  inputs.construct.url = "github:Spacecraft-Software/Construct";

  outputs = { self, nixpkgs, home-manager, construct, ... }: {
    homeConfigurations."you" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        construct.homeManagerModules.default
        {
          spacecraft.construct.enable = true;       # cross-platform skills
          spacecraft.construct.enableGrok = true;   # Grok skills
        }
      ];
    };
  };
}
```

With `enable`, all cross-platform skills are installed to `~/.agents/skills/`
and every known agent harness's skill path
(`~/.claude/skills`, `~/.gemini/skills`, `~/.codex/skills`, `~/.ai/skills`,
`~/.agent/skills`) becomes a directory symlink to that canonical location.
Add more paths via `spacecraft.construct.agentPaths`.

With `enableGrok`, Grok-specific skills (from [`grok-skills/`](grok-skills/))
install to `~/.grok/skills/` — kept separate because Grok's bundle format is
flat (no enclosing skill-name directory) and is not interchangeable with the
Claude/Gemini/Codex layout.

To pick up the latest commit, run `nix flake update construct` in the consumer
flake and rebuild.

Individual skills are also exposed as packages — e.g.
`nix build github:Spacecraft-Software/Construct#spacecraft-standard` produces
a `result/` directory with that skill's contents.

### Grok skills

Skills authored for the Grok agent platform live in
[`grok-skills/`](grok-skills/) with their own catalogue. The bundle layout
differs (flat zip root); see the section's README for details.

<!-- §5 — Contributing / standards -->
## Standards

All skills in this repository are expected to conform to
[The Spacecraft Software Standard](spacecraft-standard/) — including:

- **§4** GPL-3.0-or-later license declared in frontmatter.
- **§11** ISO 8601 dates throughout.
- Functional naming (no codenames for skill IDs).

<!-- §6 — License -->
## License

GPL-3.0-or-later. See `LICENSE.md` inside each skill directory for the full
text.

---

*─── Forged in Spacecraft Software ───*
