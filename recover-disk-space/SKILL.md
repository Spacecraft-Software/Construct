---
name: recover-disk-space
description: >
  Recovers disk space on the user's machine through a guided, approval-gated
  session: measures what is using space, then offers each cleanup as a
  multiple-choice question (recommended option first, always a Skip) and runs
  only what the user picked. Uses Vacuum (the Spacecraft disk-recovery CLI)
  whenever it is installed, and covers Nix and NixOS cleanup that Vacuum
  leaves out: old system, user and Home Manager generations, stray GC roots,
  store optimise and stale boot entries. Run it when the user calls
  /recover-disk-space or asks to free disk space. If a command fails with "No
  space left on device" during other work, only mention in one line that
  /recover-disk-space can help; never start it unasked. Nothing is deleted,
  collected, pruned or emptied without the user's explicit pick.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
metadata:
  spdx: "SPDX-License-Identifier: GPL-3.0-or-later"
  author: "Mohamed Hammad & Spacecraft Software"
---

# Recover disk space

This skill frees disk space without surprises. It measures, explains what can
go and what it costs, and then asks. **The user approves every action that
changes anything on disk, one decision at a time.** A cleaner session that
frees less is a success; one that deletes something the user wanted is a
failure no amount of reclaimed space makes up for.

Two references are read only when they apply:

| Reference | Read when |
|-----------|-----------|
| [`references/vacuum.md`](references/vacuum.md) | `vacuum` is on `PATH` |
| [`references/nix.md`](references/nix.md) | `nix` is on `PATH`, or the host is NixOS (`/etc/NIXOS` exists) |

## The approval rule (non-negotiable)

1. **Read-only is the only thing that runs unasked, and only after the user
   picked a survey in Step 1.** Read-only means: `df`, `findmnt`, `du` /
   `dust` / `dua` (non-interactive), `command -v`, `nix-store --gc
   --print-roots`, `nix-env --list-generations`, `nix config show`, and
   `vacuum scan|list|describe|schema` or `vacuum clean` **without** `--apply`
   (a dry run).
2. **Every other command needs a pick.** Before asking, show the exact command
   lines that the option will run. A pick approves those lines, as shown, and
   nothing else: a new candidate, a wider path, a stronger flag (`-d` instead
   of `--delete-older-than`, `--purge` instead of trash) is a new question.
3. **No confirmation-skipping flag is added on the user's behalf** — `--yes`,
   `-y`, `-f`, `--force`, `--purge`, `kondo -a` — unless the option the user
   picked shows that flag. Tools such as `vacuum clean --apply` need `--yes`
   when not on a terminal; that is fine *because* the picked option listed it.
   The same holds for prompts: a command that would stop to ask on stdin
   (where no one can answer) carries its own `-f` / `-y` in the option text.
4. **Paths are arguments, never shell text.** Pass every path or link the
   user picked as its own argument (the tool's argv), or quote it for the
   user's shell, and put `--` before paths where the command accepts it
   (`rm -r -- <path>`, `gio trash -- <path>`). A name with spaces, a
   leading `-` or shell metacharacters must reach the command unchanged.
5. **Never escalate.** Do not run `sudo`, `doas` or `run0`, and do not ask for
   a password. Commands that need root are handed to the user to run (see
   *Handing commands to the user*). This matches Vacuum's own rule.
6. **No pick, no action.** If the user dismisses a question, answers
   ambiguously, or the session cannot ask at all (headless, CI, a scheduled
   run), take **no** action for that decision and say so. Unlike other
   skills, a missing answer never defaults to the recommended option.
7. **Auto-loaded is not invoked.** If this skill is loaded because of a
   disk-full error in unrelated work, say in one line that
   `/recover-disk-space` can help, then return to the task. Do not survey.

## Asking the user

Ask each decision as one multiple-choice question, with the harness's
question tool (`AskUserQuestion` in Claude Code; a decision card on a thread
surface; else a numbered list answered with one number). Each question:

- offers two to four **concrete options**, each with the commands it runs and
  roughly what it frees (or "unknown until run" — package GC cannot be sized
  in advance);
- lists the **recommended option first**, marked, with a few words of reason;
- always includes **Skip** as an option, and leaves room for the user's own
  answer;
- states what cannot be undone (`--purge`, emptying the trash, collecting a
  generation removes the rollback to it).

Independent decisions (say, Nix generations and Vacuum's caches) go in one
round, one question each, when the tool allows it. Ask each decision once.

## Step 0 — Where this runs

Cleanup is only useful on the machine that is short of space. If this session
runs in a disposable cloud sandbox or a container that is not the user's
machine, say so before Step 1 and offer: free space **in this sandbox**, or
**hand the user the commands** to run on their own machine (Step 1 then
surveys nothing; each later step shows commands only). On the user's machine
through a remote-control or folder bridge, treat it as local.

## Step 1 — Survey (read-only, one question)

Ask which survey to run:

- **Quick survey (recommended)** — free space per filesystem (including the
  one holding `/nix/store` and the ESP if Nix is present), and `vacuum list` if Vacuum is installed. Seconds.
- **Full survey** — the quick survey plus the largest directories under
  `$HOME` and the Nix GC roots and generations. Can take a minute on a big
  home.
- **Skip** — go straight to a cleanup the user names.

The survey always starts with tool detection, which is cheap and read-only:

```sh
command -v vacuum nix nh home-manager dust dua kondo rg gio flatpak podman docker journalctl systemd-tmpfiles
df -h -x tmpfs -x devtmpfs -x efivarfs
```

Without Vacuum, size directories with `dust -d 1 ~` or `dua ~` (Rust; prefer
them over `du -sh`), falling back to `du -xsh ~/* ~/.[!.]* | sort -h`.

Report the survey as a short table: filesystem, used, free, percent; then
the top consumers. Name the filesystem that is short — cleaning `$HOME` does
nothing for a full `/boot`.

## Step 2 — Offer the cleanups

Offer only what the survey found and what applies to the filesystem that is
short. Order the questions by how much they free and how little they risk:

| Area | Source of the options | Typical risk |
|------|-----------------------|--------------|
| Nix generations and store | `references/nix.md` | Loses rollback targets |
| Stale boot entries (`/boot` full on NixOS) | `references/nix.md` | Low |
| Build artifacts (`target/`, `node_modules`, …) | Vacuum `build-artifacts`, `cargo-prune`; else `kondo` | Rebuild time |
| App and user caches | Vacuum `caches`; else named `~/.cache/<app>` dirs | Re-download |
| Stale temp files | Vacuum `temp-files`; root-owned ones via `systemd-tmpfiles --clean` | Low |
| Journal, Flatpak, containers | Vacuum `package-gc`; else the commands below | Low |
| Large files | Vacuum `large-files`; else `dust` / `dua` listing | The user's call, file by file |
| The trash | `vacuum empty-trash`; else `gio trash --empty` | **Irreversible** |

Without Vacuum, the system-level commands are:

```sh
flatpak --user uninstall --unused -y    # the user's installation; the session can run it
flatpak --system uninstall --unused     # shared runtimes: hand to the user (polkit asks)
sudo journalctl --vacuum-time=7d        # root: hand to the user
podman system prune -f                  # -f: the pick is the confirmation, no stdin prompt
docker system prune -f
sudo systemd-tmpfiles --clean           # root: hand to the user
```

Build artifacts without Vacuum: `kondo ~/projects` lists projects and asks per
project; **never `kondo -a`** unless the picked option said so. Skip any
project with uncommitted work the session itself is editing.

**Trash does not free space.** Moving a file to the XDG trash on the same
filesystem reclaims nothing until the trash is emptied. When the short
filesystem is the one holding `~/.local/share/Trash`, say so in the option
text, and offer emptying the trash as its own, separately approved question
(irreversible, never recommended first unless the user has already reviewed
its contents).

## Step 3 — Run what was picked

Run each approved command as shown, in the order the user approved them, and
stop at the first failure — report it and ask before going on. User-level
commands the session runs itself; root-level commands are handed over.

### Handing commands to the user

When the user must run commands themselves (root, or a machine this session
cannot reach), list **only the commands the user approved**, one per line
with a short comment each, then end with **one combined line of those same
commands** that stops at the first failure. Match the user's
shell: `;` in Nushell and mjsh (an external command's failure stops the
line), `&&` in POSIX shells (bash, zsh, dash). If unsure of the shell, check
`$env.NU_VERSION` / `$SHELL`, or give both lines. Example for Nushell:

```nu
sudo nix-collect-garbage --delete-older-than 14d          # system and root generations older than 14 days (your own: the session ran the user form)
sudo /run/current-system/bin/switch-to-configuration boot # drop boot entries for the collected generations
sudo journalctl --vacuum-time=7d                          # journal entries older than a week

sudo nix-collect-garbage --delete-older-than 14d; sudo /run/current-system/bin/switch-to-configuration boot; sudo journalctl --vacuum-time=7d
```

Use only commands that are on the user's system as surveyed. Ask the user to
say when they have run them, then re-measure.

## Step 4 — Report

Re-run `df` on the filesystems touched and report **before → after** per
filesystem, with what each approved action freed where it can be told apart.
List what was skipped and what was handed to the user but not yet confirmed
run. If space is still short, offer the next round of options (Step 2) —
asked again, never assumed.

## What this skill never does

- Deletes anything under `/nix/store` by hand. The store is changed only by
  Nix's own commands (`nix-collect-garbage`, `nix store gc`, `nix store
  optimise`). `rm` there corrupts the store.
- Deletes source code, version-control directories, or anything outside
  `$HOME`, the surveyed roots, and paths the user named.
- Follows symlinks when deleting, or removes a GC root without naming its
  target and asking.
- Changes declarative configuration (`nix.gc`, `boot.loader.*.
  configurationLimit`, journald limits) — it may *suggest* such a change as
  a follow-up, for the user to make in their flake.
- Runs a cleanup on a timer. Hosts that want periodic GC configure it
  declaratively (NixOS `nix.gc.automatic`); this skill is interactive by
  design.
