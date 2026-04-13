# nix-shell Examples & Reference

## Resolving the Correct nixpkgs Name

The package name in nixpkgs often differs from the CLI tool name (e.g. `markdownlint`
CLI is packaged as `markdownlint-cli2`, `node` is `nodejs`, `python` is `python3`).

### Method A — nix search (preferred)
```bash
nix search nixpkgs#<tool-name> 2>/dev/null | head -20
```
Extract the package name after `legacyPackages.x86_64-linux.` from the result.

### Method B — Educated guess + verify (last resort)
```bash
nix-shell -p <guessed-name> --run "<tool> --version" 2>&1 | head -5
```
If it fails with "error: undefined variable", go back to Method A.

---

## Bundling Multiple Tools

Bundle everything into **one** `nix-shell` invocation — faster (one Nix evaluation)
and keeps the environment consistent:

```bash
nix-shell -p tool1 -p tool2 -p tool3 --run "..."
```

Does the command pipeline require more than one tool? Does the tool depend on a
runtime that might not be present (e.g. a Node-based CLI needs `nodejs` alongside
it)? Think ahead and bundle all dependencies together.

---

## Common Tool → nixpkgs Name Mappings

| Tool / CLI          | nixpkgs attribute                          |
|---------------------|--------------------------------------------|
| `node` / `npm`      | `nodejs`                                   |
| `python` / `pip`    | `python3` + `python3Packages.<pkg>`        |
| `markdownlint`      | `markdownlint-cli2`                        |
| `prettier`          | `nodePackages.prettier`                    |
| `eslint`            | `nodePackages.eslint`                      |
| `just`              | `just`                                     |
| `git`               | `git`                                      |
| `sq` (Sequoia PGP)  | `sequoia-sq`                               |
| `gpg` / `gnupg`     | `sequoia-chameleon-gnupg`                  |
| `msedit`            | `msedit`                                   |
| `jq`                | `jq`                                       |
| `yq`                | `yq-go`                                    |
| `fd`                | `fd`                                       |
| `ripgrep` / `rg`    | `ripgrep`                                  |
| `shellcheck`        | `shellcheck`                               |
| `shfmt`             | `shfmt`                                    |
| `taplo`             | `taplo`                                    |
| `typos`             | `typos`                                    |
| `tokei`             | `tokei`                                    |
| `hyperfine`         | `hyperfine`                                |
| `pandoc`            | `pandoc`                                   |
| `ffmpeg`            | `ffmpeg`                                   |
| `imagemagick`       | `imagemagick`                              |
| `gh` (GitHub CLI)   | `gh`                                       |

When in doubt, always run `nix search` rather than guessing.

---

## Example Workflows

### Fix a Markdown file with markdownlint-cli2
```bash
# 1. Verify package name
nix search nixpkgs#markdownlint 2>/dev/null | head -10

# 2. Run the linter+fixer
nix-shell -p markdownlint-cli2 --run "markdownlint-cli2 --fix path/to/file.md 2>&1"
```

### Format a TOML file with taplo
```bash
nix-shell -p taplo --run "taplo fmt Cargo.toml 2>&1"
```

### Check shell scripts with shellcheck and reformat with shfmt
```bash
nix-shell -p shellcheck -p shfmt --run "
  shellcheck script.sh && shfmt -w script.sh
" 2>&1
```

### Run a Python script that needs a specific library
```bash
nix-shell -p "python3.withPackages(ps: [ps.requests ps.rich])" --run "python3 script.py"
```

### Quick benchmark with hyperfine
```bash
nix-shell -p hyperfine --run "hyperfine 'command_a' 'command_b'"
```

### Node-based CLI that needs a runtime alongside it
```bash
# Bundle nodejs with the tool to ensure the runtime is available
nix-shell -p nodejs -p nodePackages.eslint --run "eslint src/"
```
