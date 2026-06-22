---
name: construct-cli
description: >
  Reference for the `construct` command-line tool — the Spacecraft Software
  Construct skills package manager. Invoke `construct` to sync the Construct
  skill catalogue into a consuming Nix flake (`construct skill sync`, which runs
  `nix flake update construct` without rebuilding), and — in later phases — to
  install skills into any of ~70 AI agents, browse the catalogue, and ship local
  skill edits. Every data command supports `--json` and a stable
  `{ metadata, data }` envelope; `construct schema` and `construct describe`
  expose the full surface for agents. Consult this when an agent needs to drive
  or reason about the `construct` binary.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# construct

`construct` is a dual-mode (human + agent-native) CLI for managing Spacecraft
Software agent skills. It auto-detects its audience: an explicit
`--format`/`--json` flag wins, then the agent environment (`AI_AGENT`, `AGENT`,
`CI`) forces JSON, then a TTY gets colored human output, and a pipe gets JSON.

## Discover the surface first

```sh
construct describe          # compact capability manifest
construct schema            # full JSON Schema (Draft 2020-12)
construct schema skill sync # schema for one command (LLM tool-call ready)
```

## Implemented commands

- `construct skill sync [--flake-dir DIR] [--dry-run] [--json]` — update the
  `construct` flake input in a consuming flake (default
  `/spacecraft-software/bravais`). Does **not** run `nixos-rebuild`.

## Output contract

- Success: a single `{ "metadata": { tool, version, command, timestamp, … },
  "data": … }` document. Timestamps are ISO 8601 UTC with a `Z` suffix.
- Failure: a non-zero exit code from the canonical map (2 usage, 3 not-found,
  127 dependency-missing, …) plus a single-line `{ "error": { code, exit_code,
  message, hint, … } }` object on stderr, where `hint` is a runnable command.

## Planned (later phases)

`construct skill add|list|find|use|update|remove|init`, `construct agent list`,
the full edit→ship→sync loop (`construct skill ship`), and the `--format
explore` TUI.
