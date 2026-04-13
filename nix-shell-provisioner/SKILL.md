---
name: nix-shell-provisioner
license: GPL-3.0-or-later
metadata:
  author: Mohamed Hammad
description: >
  Provides any required software temporarily via `nix-shell` instead of permanently
  installing it. ALWAYS use this skill whenever a tool, binary, CLI utility, linter,
  formatter, language runtime, or any other software is missing or needs to be run
  — regardless of whether it's a system tool, Node package, Python package, or
  anything else. This is the universal software provisioning method.
  Trigger whenever you see errors like "command not found", "not installed", "missing
  dependency", or when you're about to reach for pip, npm, apt, or any other package
  manager. Do NOT permanently install anything — always use nix-shell.
  Exception: use `cargo install` for Rust crates as normal.
---

# nix-shell Provisioner

`nix-shell` gives access to 100,000+ packages from nixpkgs without permanently
installing anything. It is the preferred way to run any tool that isn't already
on `PATH` — instead of `npm install -g`, `pip install`, `apt install`, `brew install`,
or similar permanent package managers.

**Exception:** `cargo install` remains the preferred approach for Rust crates.

---

## Usage

```bash
# Single tool
nix-shell -p <pkg> --run "<command and args>"

# Multiple tools
nix-shell -p pkg1 -p pkg2 --run "<command>"
```

Always use `--run` rather than dropping into an interactive shell.

---

## When to trigger

- A tool or binary is not found on `PATH`
- About to reach for `pip install`, `npm install -g`, `apt install`, `brew install`

---

## Examples

```bash
# jq not found
nix-shell -p jq --run "jq '.key' file.json"

# prettier not found
nix-shell -p nodePackages.prettier --run "prettier --write file.ts"

# Multiple tools
nix-shell -p ripgrep -p fd --run "fd -e rs | xargs rg 'pattern'"
```

If the CLI name differs from the nixpkgs attribute name, run
`nix search nixpkgs#<tool-name>` to find the correct attribute.

For a full name-mapping table, multi-tool bundling guidance, and worked examples
(markdownlint, taplo, shellcheck+shfmt, Python with libraries, hyperfine, etc.)
see [references/examples.md](references/examples.md).
