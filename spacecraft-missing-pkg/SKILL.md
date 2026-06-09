---
name: spacecraft-missing-pkg
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
description: >
  Provides any required software via the best available package manager,
  preferring ephemeral options over permanent installs. ALWAYS use this skill
  whenever a tool, binary, CLI utility, linter, formatter, language runtime,
  or any other software is missing or needs to be run — regardless of whether
  it's a system tool, Node package, Python package, Rust crate, or anything
  else.
  Trigger whenever a command fails with "command not found", "not installed",
  "missing dependency", or when you're about to reach for apt, dnf, pacman,
  yum, zypper, or any other system-distro package manager. Detect which
  managers are available and use the first applicable one in this priority
  order: Guix → Nix → language one-shot runners (npx / uvx, ephemeral) →
  Cargo (Rust crates only) → Homebrew → Flatpak (Flathub) → AppImage → Snap.
  Do NOT use system-distro managers (apt, dnf, pacman, zypper, yum) under any
  circumstances.
---

# Spacecraft Software Missing-Package Provisioner

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

Whenever a tool is missing from `PATH`, this skill walks a fixed priority
chain of package managers and uses the first one that is both **installed
on the system** and **has the requested tool packaged**.

Never fall back to system-distro managers (`apt`, `dnf`, `pacman`, `zypper`,
`yum`, etc.) — they require root, touch `/usr`, and leave durable, hard-to-
reverse changes.

---

## Priority chain

The chain is two bands. **Ephemeral** runners leave no durable state and are
**always preferred**; **permanent** installs are the fallback you reach for
only when nothing ephemeral has the tool. Within each band, walk top-down and
take the first manager that is both available and has the tool packaged.

### Band A — Ephemeral (no durable state, no cleanup — always prefer)

| # | Manager                | Syntax style                                  | Scope                              | Details                                       |
|---|------------------------|-----------------------------------------------|------------------------------------|-----------------------------------------------|
| 1 | **Guix**               | `guix shell <pkg> -- <cmd> <args>`            | Any tool                           | [references/guix.md](references/guix.md)      |
| 2 | **Nix**                | `nix-shell -p <pkg> --run "<cmd>"`            | Any tool                           | [references/nix.md](references/nix.md)        |
| 3 | **One-shot runners**   | `npx` · `uvx` · `pnpm dlx` · `bunx`           | Node / Python ecosystem packages   | [references/oneshot.md](references/oneshot.md) |

### Band B — Permanent (real install; tell the user what landed and how to remove it)

| # | Manager      | Syntax style                                                         | Sudo?   | Scope                | Details                                        |
|---|--------------|----------------------------------------------------------------------|---------|----------------------|------------------------------------------------|
| 4 | **Cargo**    | `cargo install <crate>` then `<binary>`                              | No      | **Rust crates only** | [references/cargo.md](references/cargo.md)     |
| 5 | **Homebrew** | `brew install <formula>` then `<binary>`                             | No      | Any tool             | [references/brew.md](references/brew.md)       |
| 6 | **Flatpak**  | `flatpak install --user -y flathub <app-id>` then `flatpak run <id>` | No      | Mostly GUI apps      | [references/flatpak.md](references/flatpak.md) |
| 7 | **AppImage** | download `*.AppImage`, `chmod +x`, run the file directly             | No      | Self-contained apps (mostly GUI/portable) | [references/appimage.md](references/appimage.md) |
| 8 | **Snap**     | `sudo snap install <pkg>` then `snap run <pkg>`                      | **Yes** | Any tool             | [references/snap.md](references/snap.md)       |

Band A leaves no durable state (Guix/Nix packages are garbage-collected later;
one-shot runners only populate a throwaway cache). Band B entries are real
artifacts — prefer the highest-ranked applicable option, and tell the user what
was installed and how to remove it.

**AppImage is the odd one out:** it is not a manager you install or detect with
`command -v`. It is "available" whenever upstream publishes an `*.AppImage` and
the host can run it (FUSE, or the `--appimage-extract` fallback) — see
[references/appimage.md](references/appimage.md). A single AppImage *can* be run
disposably (download to a temp dir, run, delete), but it sits in Band B because,
like Flatpak/Snap, it is a packaged-application format reached only when the
higher-ranked managers don't have the tool.

---

## Step 1 — Detect which managers are available

```bash
have() { command -v "$1" >/dev/null 2>&1; }

AVAILABLE=""
have guix       && AVAILABLE="$AVAILABLE guix"
have nix-shell  && AVAILABLE="$AVAILABLE nix"
have npx        && AVAILABLE="$AVAILABLE npx"
have uvx        && AVAILABLE="$AVAILABLE uvx"
have cargo      && AVAILABLE="$AVAILABLE cargo"
have brew       && AVAILABLE="$AVAILABLE brew"
have flatpak    && AVAILABLE="$AVAILABLE flatpak"
have snap       && AVAILABLE="$AVAILABLE snap"

echo "Available provisioners:${AVAILABLE:- none}"
```

**AppImage has no entry here** — there is no binary to `command -v`. It is
available whenever upstream ships an `*.AppImage` for the tool and the host can
execute it; treat it as a candidate in Band B regardless of this list.

If `AVAILABLE` is empty (and no AppImage is on offer), tell the user no
supported provisioner is installed. Do not fall back to
`apt`/`dnf`/`pacman`/etc.

---

## Step 2 — Pick the highest-ranked applicable manager

Walk the priority chain top-down and pick the first manager that is both
available AND has the tool packaged. Skip rules:

- **One-shot runners** (`npx` / `uvx`) apply only when the tool ships as a
  Node/npm package (`npx`) or a Python package with an entry point
  (`uvx`/`pipx run`). They are ephemeral, so prefer them over **any** Band B
  install when the tool is such an ecosystem package. If the runtime itself
  (`node` / `uv`) is missing, you can still run one-shot via Guix/Nix —
  `guix shell node -- npx <pkg>` — rather than dropping to a permanent install.
- **Cargo** applies only if the tool is a Rust crate on crates.io.
  Skip Cargo for non-Rust tools, even if `cargo` is installed.
- **Flatpak** rarely has CLI tools. Skip Flatpak if the tool is a CLI
  unless you've confirmed a Flathub app-id for it.
- **AppImage** applies when upstream publishes an `*.AppImage` release for the
  tool and no higher-ranked manager has it. It needs no manager installed —
  only the ability to download the file and run it.
- Any manager that doesn't have the package skips to the next one.

Look up each candidate package name in the manager's authoritative source
**before** invoking — do not guess, and do not fabricate names when a
lookup returns nothing. Full lookup sources and naming conventions:
**[references/lookup.md](references/lookup.md)**.

---

## Step 3 — Run the tool

Minimal syntax recap below; full examples and extras in each per-manager
reference file.

### 1. Guix (`guix shell`) — preferred

```bash
guix shell <pkg> -- <command> <args>
guix shell pkg1 pkg2 -- <command> <args>    # multiple tools
```

No `-p` flag; `--` is the separator; args after `--` are exec'd directly
(no shell), so pipelines need `sh -c '...'`. Do not use the deprecated
`guix environment --ad-hoc`. See **[references/guix.md](references/guix.md)**.

### 2. Nix (`nix-shell`)

```bash
nix-shell -p <pkg> --run "<command and args>"
nix-shell -p pkg1 -p pkg2 --run "<command>"    # multiple tools
```

`--run` executes through a shell, so pipes work naturally inside the quoted
string. See **[references/nix.md](references/nix.md)**.

### 3. One-shot runners (`npx` / `uvx`) — ephemeral, ecosystem-scoped

```bash
npx <pkg> [<args>]          # Node/npm package, run once, no global install
uvx <pkg> [<args>]          # Python package with an entry point (uv)
pipx run <pkg> [<args>]     # Python alternative when uv is absent
```

These download into a throwaway cache and run immediately — nothing lands on
`PATH`. When `node`/`uv` themselves are missing, bootstrap the runtime
ephemerally: `guix shell node -- npx <pkg>` or `nix-shell -p nodejs --run "npx
<pkg>"`. See **[references/oneshot.md](references/oneshot.md)**.

### 4. Cargo (`cargo install`) — Rust crates only

```bash
cargo install <crate>    # installs to ~/.cargo/bin/<binary>
<binary> <args>
```

Permanent, user-local. Binary name often differs from crate name
(`fd-find` → `fd`, `ripgrep` → `rg`). Prefer `cargo binstall` for pre-built
binaries when `cargo-binstall` is available. See
**[references/cargo.md](references/cargo.md)**.

### 5. Homebrew (`brew install`)

```bash
brew install <formula>
<binary> <args>
```

Permanent, user-local (macOS and Linux). See
**[references/brew.md](references/brew.md)**.

### 6. Flatpak (`flatpak`) — mostly GUI apps

```bash
flatpak install --user -y flathub <app-id>
flatpak run <app-id> [<args>]
```

App-ids are reverse-DNS. Most CLI tools are not on Flathub — only use
Flatpak when a confirmed app-id exists. See
**[references/flatpak.md](references/flatpak.md)**.

### 7. AppImage — download-and-run app bundle

```bash
# Download the upstream release asset, make it executable, run it
curl -L -o /tmp/<tool>.AppImage <release-asset-url>
chmod +x /tmp/<tool>.AppImage
/tmp/<tool>.AppImage [<args>]

# On hosts without FUSE, extract and run instead of mounting:
/tmp/<tool>.AppImage --appimage-extract && ./squashfs-root/AppRun [<args>]
```

A single self-contained executable — no install step and no manager. Reach for
it after Flatpak and before Snap, when upstream ships an `*.AppImage` and the
higher-ranked managers don't have the tool. Delete the file when done. See
**[references/appimage.md](references/appimage.md)**.

### 8. Snap (`snap install`) — last resort

```bash
sudo snap install <pkg>              # or --classic for CLIs that need host access
snap run <pkg> [<args>]
```

**Requires sudo.** If the AI is running without sudo, report the situation
rather than attempting the install. See
**[references/snap.md](references/snap.md)**.

---

## When to trigger

- A tool or binary is not found on `PATH` ("command not found", "No such file").
- About to reach for `apt install`, `dnf install`, `pacman -S`, `yay`,
  `zypper install`, `yum install`, etc.
- A script fails because of a missing interpreter, linter, or formatter.

---

## Minimal examples per manager

```bash
# Guix (ephemeral, preferred)
guix shell ripgrep -- rg 'TODO' src/

# Nix (ephemeral)
nix-shell -p ripgrep --run "rg 'TODO' src/"

# One-shot runners (ephemeral, ecosystem-scoped)
npx prettier --check .                 # Node package, run once
uvx ruff check .                       # Python package via uv
guix shell node -- npx prettier --check .   # when node itself is absent

# Cargo (Rust crate, permanent)
cargo install ripgrep && rg 'TODO' src/

# Homebrew (permanent)
brew install ripgrep && rg 'TODO' src/

# Flatpak (GUI app, example)
flatpak install --user -y flathub org.gimp.GIMP && flatpak run org.gimp.GIMP

# AppImage (download-and-run, after Flatpak / before Snap)
curl -L -o /tmp/app.AppImage <url> && chmod +x /tmp/app.AppImage && /tmp/app.AppImage

# Snap (last resort, requires sudo)
sudo snap install <pkg> && snap run <pkg>
```

For idiomatic, per-manager example workflows — multi-tool pipelines,
containerized invocations, language-ecosystem ad-hoc envs, and cleanup —
see the linked reference files above.

---

## When nothing in the chain works

If no manager in the chain has the tool, fall through to the guidance in
**[references/fallback.md](references/fallback.md)** (Git-source Cargo installs
and hard prohibitions on `apt`/`dnf`/`pacman`/`pip -g`/`npm -g`). Note that the
one-shot runners (`npx` / `uvx`) are now Band A tier 3, not a last-ditch
fallback — reach for them as a preferred ephemeral option, not only when
everything else fails.

*— Built by Spacecraft Software —*
