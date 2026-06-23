<!--
  README for the Grok-specific skill section of the Spacecraft Software Construct repository.
  Audience: humans browsing on GitHub, and Grok agents loading these skills.
  Maintenance: keep the skill list in §2 aligned with the subdirectories of grok-skills/.
-->

# Spacecraft Software Construct — Grok Skills

Skills authored for the **Grok** agent platform. These live in a dedicated
subdirectory because Grok's skill bundle format differs from the
Claude / Gemini / Codex format used at the repository root:

- **Grok bundle layout** — `SKILL.md` and any `assets/` / `references/` live at
  the **root** of the `.zip` (no enclosing skill-name directory).
- **Claude/Gemini/Codex bundle layout** — `SKILL.md` etc. live inside a
  `<skill-name>/` directory at the root of the bundle.

The on-disk source layout inside this repo is the same as the rest of the
catalogue (`grok-skills/<skill-name>/SKILL.md` …); only the bundle packaging
differs.

<!-- §1 — Audience -->
## Audience

- **Grok agents** loading skills from `~/.grok/skills/`.
- **Humans** reviewing or extending Spacecraft Software's Grok-targeted skills.

<!-- §2 — Skill catalogue: keep alphabetical, one line per skill -->
## Skills in this section

| Skill | Purpose |
|-------|---------|
| [`gfm-markdown`](gfm-markdown/) | Creates, edits, and validates GitHub-Flavored Markdown — tables, task lists, alerts, footnotes, and rendering quirks. Ships READMEs, CHANGELOGs, PR descriptions, and technical-doc templates under `assets/`. |

<!-- §3 — Layout convention -->
## Directory layout

```
grok-skills/
├── README.md              (this file)
├── <skill-name>/
│   ├── SKILL.md           Frontmatter + agent-facing instructions
│   ├── assets/            Optional; templates the agent copies and fills in
│   └── references/        Optional; deeper reference files
├── <skill-name>.zip       Bundle (flat — files at zip root)
└── <skill-name>.skill     Bundle (flat, no directory entries; `-D` flag)
```

<!-- §4 — Bundling -->
## Bundling

Each skill ships as two flat bundles **inside this directory** (not at the
repo root). Build from inside the skill directory so paths land at the zip
root:

```sh
cd grok-skills/<name>
rm -f ../<name>.zip ../<name>.skill
zip -qr  ../<name>.zip   SKILL.md [assets] [references]
zip -qrD ../<name>.skill SKILL.md [assets] [references]
```

Omit any argument the skill does not have. Verify with `unzip -l` — `SKILL.md`
must appear at the top level (no `<name>/` prefix).

<!-- §5 — Installation -->
## Installation

```sh
git clone git@github.com:Spacecraft-Software/Construct.git ~/.grok/skills
```

Grok loads each subdirectory of `~/.grok/skills/grok-skills/` as a skill, or
you can symlink only the `grok-skills/` content depending on your Grok client.

The Nix flake at the repository root exposes a Home Manager module
(`spacecraft.construct.enableGrok = true`) that installs only the Grok skills
into `~/.grok/skills/`.

<!-- §6 — Standards -->
## Standards

These skills follow the Steelbore Standard's licensing (§4,
GPL-3.0-or-later) and ISO 8601 UTC date rules (§14), but use Grok's minimal
frontmatter schema (`name` + `description`) rather than the
Claude/Gemini/Codex schema with `license` / `maintainer` / `website` fields.
See [The Steelbore Standard](../spacecraft-standard/) for the full
compliance reference.

<!-- §7 — License -->
## License

GPL-3.0-or-later. See `../LICENSE.md` at the repository root for the full
text.

---

*─── Forged in Spacecraft Software ───*
