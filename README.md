<!--
  README for the Steelbore `Construct` repository.
  Audience: humans browsing on GitHub, and LLM agents loading these skills.
  Maintenance: keep the skill list in §2 aligned with the top-level directories.
-->

# Steelbore Construct

A collection of **Claude / LLM agent skills** used across the Steelbore
ecosystem. Each top-level directory is a self-contained skill — a `SKILL.md`
that the agent loads on demand, plus optional `references/` files consulted
only when a deeper lookup is warranted.

These skills encode conventions, tool preferences, brand rules, and compliance
requirements so agents produce Steelbore-consistent output without needing
the rules re-attached to every prompt.

<!-- §1 — Who this is for -->
## Audience

- **LLM agents** (Claude Code, Gemini CLI, Copilot CLI, Codex, etc.) loading
  skills from `~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`.
- **Humans** reviewing, extending, or auditing Steelbore's conventions.

<!-- §2 — Skill catalogue: keep alphabetical, one line per skill -->
## Skills in this repository

| Skill | Purpose |
|-------|---------|
| [`nix-shell-provisioner`](nix-shell-provisioner/) | Use `nix-shell` for transient tool provisioning (preferred over `pip`, `npm -g`, `apt`, `brew`). |
| [`rust-guidelines`](rust-guidelines/) | Enforces Microsoft Pragmatic Rust Guidelines before any `.rs` edit. |
| [`steelbore-agentic-cli`](steelbore-agentic-cli/) | Agent-facing UX layer for Steelbore CLIs — pairs with `steelbore-cli-standard`. |
| [`steelbore-brand-guidelines`](steelbore-brand-guidelines/) | Applies Steelbore's official colors and typography to artifacts. |
| [`steelbore-cli-preference`](steelbore-cli-preference/) | Modern CLI substitutions: `eza` for `ls`, `rg` for `grep`, `gitway` for Git SSH, etc. |
| [`steelbore-cli-standard`](steelbore-cli-standard/) | Enforces the Steelbore Dual-Mode Self-Documenting CLI Standard (SFRS v1.0.0) on every CLI. |
| [`steelbore-cli-shell`](steelbore-cli-shell/) | Syntax-compliance guard for Nushell / Ion / POSIX / Bash commands. |
| [`steelbore-document-format`](steelbore-document-format/) | ODF-primary office suite (`.odt` / `.ods` / `.odp`) with MS Office (`.docx` / `.xlsx` / `.pptx`) as secondary; GFM Markdown companion always paired; PDF as tertiary export; Void Navy + Standard §9 palette. |
| [`steelbore-missing-pkg`](steelbore-missing-pkg/) | Handles missing-package situations in the Steelbore workflow. |
| [`steelbore-standard`](steelbore-standard/) | Authoritative compliance reference (The Steelbore Standard v1.1). |
| [`steelbore-theme-factory`](steelbore-theme-factory/) | Generates Steelbore-compliant themes for IDEs and terminals. |

<!-- §3 — Layout convention -->
## Directory layout

Every skill follows the same shape:

```
<skill-name>/
├── SKILL.md           # Frontmatter + the agent-facing instructions
├── LICENSE.md         # GPL-3.0-or-later (per Steelbore Standard §4)
└── references/        # Optional; consulted only when depth is needed
    ├── <topic>.md
    └── ATTRIBUTION.md # Required when references are adapted from external sources
```

Skills are also distributed as `<skill-name>.skill` bundles (zipped) at the
repository root for drop-in installation.

<!-- §4 — Installation -->
## Installation

Clone into any of the supported agent skill directories:

```sh
# Claude Code
git clone git@github.com:Steelbore/Construct.git ~/.claude/skills

# Gemini CLI
git clone git@github.com:Steelbore/Construct.git ~/.gemini/skills

# Codex
git clone git@github.com:Steelbore/Construct.git ~/.codex/skills
```

The SSH remote is configured to work with
[Gitway](https://github.com/Steelbore/Gitway), Steelbore's pinned-host-key
SSH transport for Git.

<!-- §5 — Contributing / standards -->
## Standards

All skills in this repository are expected to conform to
[The Steelbore Standard](steelbore-standard/) — including:

- **§4** GPL-3.0-or-later license declared in frontmatter.
- **§11** ISO 8601 dates throughout.
- Functional naming (no codenames for skill IDs).

<!-- §6 — License -->
## License

GPL-3.0-or-later. See `LICENSE.md` inside each skill directory for the full
text.

---

*─── Forged in Steelbore ───*
