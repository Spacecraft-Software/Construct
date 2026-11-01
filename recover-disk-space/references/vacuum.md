# Using Vacuum

Read this when `vacuum` is on `PATH`. Vacuum is the Spacecraft Software
disk-space recovery CLI and TUI (Rust, <https://Vacuum.SpacecraftSoftware.org/>).
It is safe by construction — dry-run unless `--apply`, XDG trash unless
`--purge`, bounded to `$HOME` plus its configured roots, never follows
symlinks, never escalates — and this skill leans on that rather than
re-implementing it.

The host's own copy is the authority: run `vacuum describe --json` once at
the start of a session and use the commands, categories and cleaner ids it
reports. What follows matches Vacuum as of 2026-11; where the two disagree,
the host wins.

## Survey (read-only)

```sh
vacuum describe --json                 # capability manifest: commands, categories, safety
vacuum list --json                     # reclaimable candidates, every category
vacuum scan --json --limit 20          # largest consumers under the configured roots
```

Roots come from `--root`, then `VACUUM_ROOTS`, then `roots` in
`$XDG_CONFIG_HOME/vacuum/config.toml`, then `$HOME`. Report which roots were
scanned, so the user knows what was not looked at.

Categories: `build-artifacts`, `package-gc`, `caches`, `temp-files`,
`large-files`. Cleaner ids (finer than categories): `build-artifacts`,
`cargo-prune`, `caches`, `temp-files`, `package-gc`, `large-files`.
`package-gc` candidates report `0` bytes: the amount is unknown until the
tool runs; say "unknown", not "0".

## One question per category that has candidates

Show the candidates (path, size, risk) from the dry run, then offer, for
example for build artifacts:

- **Prune dead units only (recommended for projects in use)** — keeps warm
  build caches:
  `vacuum clean --cleaner cargo-prune --apply --yes`
- **Remove whole `target/` and `node_modules` dirs to the trash** — full
  rebuild next time; frees space only once the trash is emptied if it sits
  on the same filesystem:
  `vacuum clean --category build-artifacts --apply --yes`
- **Remove them permanently** — frees space now, cannot be undone:
  `vacuum clean --category build-artifacts --apply --purge --yes`
- **Skip**

`cargo-prune` and `build-artifacts` overlap on the same `target/`: offer one
or the other, never both in one pick.

**Bind the pick to what the user saw.** `vacuum clean --apply` re-scans and
acts on whatever that scan finds, not on the earlier listing. So, right
before applying, re-run the same command without `--apply` (a dry run) and
compare its candidates with the ones the user approved. Identical: apply.
Anything new or larger: show the difference and ask again. Narrow the scope
to what was approved with `--cleaner` and `--root <path>` where that
separates it. When the approved set cannot be isolated that way, offer the
TUI hand-over below instead of applying.

The same shape works for `caches`. For `temp-files`, Vacuum always purges
(trashing under `/tmp` would reclaim nothing) and says so; put that in the
option text. Root-owned temp files are left to `sudo systemd-tmpfiles
--clean`, which Vacuum prints — hand it to the user.

**`package-gc`** wraps commands such as `nix-collect-garbage -d`,
`flatpak uninstall --unused -y`, `journalctl --vacuum-time=7d`,
`podman system prune -f`. Never apply this category through Vacuum, which
would run every command in it: list each command it would run, offer them
individually, run the user-level ones the user picks directly (not via
`vacuum clean`), and route the Nix one
through [`nix.md`](nix.md), which has gentler options than `-d`. Commands
marked as needing root are handed to the user, not run through Vacuum.

**`large-files`** is the user's call, file by file: show the list, ask the
user to name the entries, then offer trash or permanent removal for exactly
those paths (`gio trash -- <path>` or `rm -r -- <path>`, each path its own
argument, as the option shows), or
the TUI hand-over below. Never offer "all large files".

`--yes` appears in these commands only because a non-terminal `--apply`
requires it; the user's pick of that option is the confirmation. Never add
`--purge` to an option that did not show it.

## Emptying the trash

`vacuum empty-trash` (dry run) shows what is there; `vacuum empty-trash
--apply --yes` empties it permanently. Ask it as its own question, after the
categories, with the dry-run listing in front of the user, and re-run the
dry run right before applying: if anything was added to the trash since,
show it and ask again. Never recommend
it first unless the user has just reviewed the listing.

## The TUI

`vacuum` (or `vacuum tui`) opens the interactive TUI. In an agent session,
offer it only as a hand-over: "run `vacuum` in your terminal to browse and
pick yourself". The TUI's own prompts are the user's approvals there.

## Not installed

When `vacuum` is missing, do not install it unasked. Mention once that it
covers most of this skill's ground safely, with the install line for the
host — on NixOS / Nix: `nix run github:Spacecraft-Software/Vacuum` (try
once) or the flake input (`inputs.vacuum.url =
"github:Spacecraft-Software/Vacuum"`). **Never `cargo install vacuum`** — an
unrelated crate holds that name. Then carry on with the fallbacks in
`SKILL.md`.
