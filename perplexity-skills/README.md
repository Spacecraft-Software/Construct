<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Perplexity skills

Perplexity rejects an uploaded skill zip that contains **more than 100 files**.
Every skill in this repo is comfortably under that except
[`spacecraft-cli-preference`](../spacecraft-cli-preference/), which ships **110
per-tool `references/` files** (115 zip entries) so an agent can lazy-load
exactly the one tool it is about to run. Perplexity already accepts the *layout*
of the canonical bundle — it fails **only** on the file count.

This directory holds a **generated**, consolidated bundle for that one skill.
The canonical `spacecraft-cli-preference/` is untouched and remains the single
source of truth for Claude, Gemini, Codex, and Grok.

## Contents

| File | What it is |
|------|------------|
| `categories.toml` | The consolidation map: passthrough files, and each category's slug, title, and tools. Data, not code — read by `construct skill build --target perplexity`. |
| `spacecraft-cli-preference.zip` | The generated bundle to upload to Perplexity (20 entries, same nested `spacecraft-cli-preference/…` layout). A verbatim copy of `dist/perplexity/spacecraft-cli-preference.zip`. |

The generator is `construct skill build --target perplexity` (the Rust
`construct` CLI in [`../construct-cli/`](../construct-cli/)). It replaced the
former `build.py` script, whose output it reproduces except for two intended
changes: spec-clean frontmatter and a bundled `LICENSE`.

## How the consolidation works

The build merges the 110 per-tool `references/<tool>.md` files into **14
category files** and rewrites every `references/<tool>.md` link in `SKILL.md` to
`references/<category>.md#<tool>` (each tool becomes a `## <tool>` anchor). The
three non-tool reference files — `ATTRIBUTION.md`, `execution-context.md`, and
`local-execution.md` — pass through verbatim, as do `CREDITS.md` and
`LICENSE`. The frontmatter gets the Perplexity target's rewrite: `maintainer`
and `website` move under `metadata:` and `user-invocable` is dropped; the
`SKILL.md` body is otherwise the canonical text with only its links rewritten.

| `references/<category>.md` | Tools |
|---|---|
| `file-and-text` | eza, bat, fd, ripgrep, sd, delta, tokei, jaq, ouch, uutils, rustybox |
| `disk-and-files` | dust, dua, fclones, kondo, disktui, gptman, yazi, broot, superfile |
| `process-and-monitoring` | procs, bottom, kmon, macchina, bandwhich |
| `shells-and-terminal` | nushell, ion, brush, starship, atuin, zellij, zoxide, t-rec |
| `text-editors` | helix, rsvim, amp, msedit |
| `networking` | xh, curl, wget2, dog, gping, trippy, rustscan, sniffglue, monolith, lychee |
| `network-config` | impala, iwd, nmstate, adguardvpn-cli |
| `vcs-and-build` | gitui, jujutsu, gitway, cargo-update, rustup, lorri, cpx, podman, just |
| `package-managers` | omni, zap, am, topgrade, paru, linutil, dotter, nix, flatpak, brew, guix |
| `security-encryption` | rage, sequoia-chameleon, sequoia, rbw, sudo-rs |
| `multimedia` | rav1e, gifski, oxipng, viu, mpv, ffmpeg, yt-dlp, ncspot, termusic, radio-browser |
| `communication` | matrix-commander, iamb, rumatui, disrust, rivetui |
| `boot-login-session` | lanzaboote, greetd, tuigreet, lemurs, xdpc, cosmic-session, xremap |
| `ai-agents` | claude-code, aichat, gemini-cli, codex, gh-copilot, opencode, minimax-cli, grok-cli, kilo, kiro-cli, kimi-cli, gws-cli |

## Regenerating

**Never hand-edit the category files or the zipped `SKILL.md`.** Edit the
canonical `spacecraft-cli-preference/`, rebuild, and copy the output here:

```sh
construct skill build --target perplexity spacecraft-cli-preference
cp dist/perplexity/spacecraft-cli-preference.zip perplexity-skills/
```

The zip is deterministic (fixed mtimes, sorted entries), so an unchanged skill
rebuilds byte-identical and leaves no git diff. The build self-checks that every
rewritten `#anchor` resolves to a real `## <tool>` heading and fails otherwise.
When a tool is added to or removed from the canonical skill, update its entry in
[`categories.toml`](categories.toml) — the build asserts the map covers exactly
the canonical tool set and fails loudly if it drifts.

Per the repo's install-surface rule, regenerate and commit
`spacecraft-cli-preference.zip` in the **same commit** as any change to the
canonical `spacecraft-cli-preference/SKILL.md` or its `references/`. See the
"Perplexity bundle" section of [`../CONTRIBUTING.md`](../CONTRIBUTING.md).

## License

GPL-3.0-or-later. See `../LICENSE` at the repository root — the canonical
license text as a regular file, with `../LICENSES/GPL-3.0-or-later.txt` a
symlink back to it (Standard §4.3) — for the full text.
