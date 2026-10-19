---
name: construct-cli
description: >
  Reference for the `construct` command-line tool — the Spacecraft Software
  Construct skills package manager. Invoke `construct` to install, list, find,
  update, and delete skills across ~70 AI agents; to sync the Construct
  catalogue into a consuming Nix flake; to build per-platform skill bundles;
  to vendor skills into a repository; and to ship local skill edits as a pull
  request. Every data command supports `--json` and a stable
  `{ metadata, data }` envelope; `construct describe` and `construct schema`
  expose the full surface, `construct schema --format openai|gemini|anthropic|mcp`
  emits LLM tool definitions, and `construct mcp` serves every command as an
  MCP tool over stdio. Consult this when an agent needs to drive or reason
  about the `construct` binary.
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
construct describe                         # compact capability manifest
construct schema                           # full JSON Schema (Draft 2020-12)
construct schema skill sync                # one command (LLM tool-call ready)
construct schema --format openai           # every command as OpenAI strict tools
construct mcp                              # MCP server over stdio, lazy schemas
```

`construct describe` is the authority for what is implemented; in brief:

- `construct skill add | list | delete | update | find | use | init` — the
  imperative installer across the agent registry (`construct agent list`).
  `skill delete` is destructive: off a terminal it needs `--yes` (an agent
  without it gets `MISSING_ARGUMENT` and a hint that is the same command plus
  `--yes`); `remove` and `rm` are aliases.
- `construct skill sync | status | reset` — the flake loop.
- `construct skill ship | build | vendor` — the ship loop and distribution.

## MCP

`construct mcp` speaks newline-delimited JSON-RPC 2.0 on stdio. `tools/list`
returns names, one-line descriptions, and capability hints only; call
`tools/get` with `{"name": "construct_skill_add"}` for a tool's full input
schema before `tools/call`. A call runs the command with `--json`; failures come
back as JSON-RPC error `-32000` whose `message` is the `error.code` and whose
`data` is the full structured error.

## Output contract

- Success: a single `{ "metadata": { tool, version, command, timestamp, … },
  "data": … }` document. Timestamps are `YYYY-MM-DDTHH:MM:SSZ` (ISO 8601 UTC,
  whole seconds). `--version` in machine mode is such an envelope too, with
  `maintainer` and `website` in its metadata.
- Failure: a non-zero exit code from the canonical map (2 usage, 3 not-found,
  127 dependency-missing, …) plus a single-line `{ "error": { code, exit_code,
  message, hint, … } }` object on stderr, where `hint` is a runnable command.
- Arguments carrying a control character (0x00–0x08, 0x0B–0x0C, 0x0E–0x1F) are
  refused before parsing, exit 2.
