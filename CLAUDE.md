# CLAUDE.md

@AGENTS.md

> Record project knowledge in `AGENTS.md`, not here. This file holds only
> Claude-Code-only context (Standard §5.7).

## Claude Code specifics

- `microsoft-rust-guidelines` is `user-invocable: false` on purpose — it is the
  mandatory auto-load Rust base and is hidden from the `/` menu by design. See
  the editing rules in `AGENTS.md`; do not "fix" its absence from the menu.
- Skills reach Claude at `~/.claude/skills/<skill>`, one of the per-harness paths
  Home Manager provisions (see "Local agent fan-out" in `AGENTS.md`).
