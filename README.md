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
| [`rust-guidelines`](rust-guidelines/) | Enforces Microsoft Pragmatic Rust Guidelines before any `.rs` edit. |
| [`spacecraft-agentic-cli`](spacecraft-agentic-cli/) | Agent-facing UX layer for Spacecraft Software CLIs — pairs with `spacecraft-cli-standard`. |
| [`spacecraft-brand-guidelines`](spacecraft-brand-guidelines/) | Applies Spacecraft Software's official colors and typography to artifacts. |
| [`spacecraft-cli-preference`](spacecraft-cli-preference/) | Modern CLI substitutions: `eza` for `ls`, `rg` for `grep`, `gitway` for Git SSH, etc. |
| [`spacecraft-cli-standard`](spacecraft-cli-standard/) | Enforces the Spacecraft Software Dual-Mode Self-Documenting CLI Standard (SFRS v1.0.0) on every CLI. |
| [`spacecraft-cli-shell`](spacecraft-cli-shell/) | Syntax-compliance guard for Nushell / Ion / POSIX / Bash commands. |
| [`spacecraft-document-format`](spacecraft-document-format/) | ODF-primary office suite (`.odt` / `.ods` / `.odp`) with MS Office (`.docx` / `.xlsx` / `.pptx`) as secondary; GFM Markdown companion always paired; PDF as tertiary export; Void Navy + Standard §9 palette. |
| [`spacecraft-missing-pkg`](spacecraft-missing-pkg/) | Handles missing-package situations in the Spacecraft Software workflow. |
| [`spacecraft-standard`](spacecraft-standard/) | Authoritative compliance reference (The Spacecraft Software Standard v1.2). |
| [`spacecraft-theme-factory`](spacecraft-theme-factory/) | Generates Spacecraft Software-compliant themes for IDEs and terminals. |

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

Clone into any of the supported agent skill directories:

```sh
# Claude Code
git clone git@github.com:Spacecraft-Software/Construct.git ~/.claude/skills

# Gemini CLI
git clone git@github.com:Spacecraft-Software/Construct.git ~/.gemini/skills

# Codex
git clone git@github.com:Spacecraft-Software/Construct.git ~/.codex/skills
```

The SSH remote is configured to work with
[Gitway](https://github.com/Spacecraft-Software/Gitway), Spacecraft Software's pinned-host-key
SSH transport for Git.

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
