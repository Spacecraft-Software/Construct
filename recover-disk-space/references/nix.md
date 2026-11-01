# Nix and NixOS cleanup

Read this when `nix` is on `PATH` or `/etc/NIXOS` exists. Everything here
follows the approval rule in [`../SKILL.md`](../SKILL.md): read-only probes
run after the survey pick; every collecting, deleting or optimising command
is a multiple-choice question, and anything under `sudo` is handed to the
user.

Vacuum does not touch `/nix` — the store is on its deny list. Its
`package-gc` category offers `nix-collect-garbage -d` (all old generations),
which is the bluntest option; this file is where the finer ones come from.

## Survey (read-only)

```sh
du -sh /nix/store                                          # slow on a big store; full survey only
nix config show auto-optimise-store                        # older Nix: nix show-config | grep auto-optimise
nix-env --list-generations --profile /nix/var/nix/profiles/system
nix-env --list-generations                                 # the user's default profile
nix profile history                                        # if the user uses `nix profile`
home-manager generations                                   # standalone Home Manager only
nix-store --gc --print-roots | rg -v '^(/proc|\{)'         # GC roots, without live process roots
nix-collect-garbage --delete-older-than 14d --dry-run      # what generations would go (user profiles)
df -h /boot                                                # NixOS: is the ESP the full one?
```

Also read, without changing them, what the host already does: `nix.gc`
options and `boot.loader.*.configurationLimit` in the flake if the session
can see it (`rg -n 'nix\.gc|configurationLimit|auto-optimise' <flake dir>`).
A host with weekly automatic GC and a small `configurationLimit` has little
to collect beyond very recent generations; say so instead of offering a
cleanup that will free nothing.

## The generations question

Collecting a generation removes the ability to roll back to it. Offer:

- **Older than 14 days, my profiles and the system's (recommended)** —
  keeps recent rollbacks. Two commands, because neither reaches the other's
  profiles: the session runs the user one, the root one is handed over.
  `nix-collect-garbage --delete-older-than 14d` (user)
  `sudo nix-collect-garbage --delete-older-than 14d` (system, root's and
  legacy `/nix/var/nix/profiles/per-user` profiles)
  Root's run does **not** see other users' profiles under their
  `~/.local/state/nix/profiles` (the default on current Nix); Nix's own
  manual says it "cannot know about all profiles". On a multi-user host,
  name the other users whose generations stay, and say each must run the
  user command themselves.
- **Older than 14 days, my user profiles only** — the session can run it:
  `nix-collect-garbage --delete-older-than 14d`
  (covers the user's `nix-env` / `nix profile` / Home Manager profiles under
  `~/.local/state/nix/profiles`, not the system profile)
- **Every old generation** — the most space, no rollback at all; again
  one user and one root command:
  `nix-collect-garbage -d` then `sudo nix-collect-garbage -d`
- **Skip**

Pick a period that fits the survey: if the oldest generation is ten days old,
14 days frees nothing, so offer 7 or 3 instead and say why.

If **`nh`** (Rust) is installed, offer its form instead — it keeps the
newest N generations per profile and prompts on its own:
`nh clean all --keep 3 --keep-since 14d --ask` (root parts prompt for sudo
in the user's terminal, so hand it over). `nh clean user …` is the
user-only variant. Check `nh clean --help` on the host before quoting other
flags.

Standalone Home Manager (not the NixOS module) keeps its own generations:
`home-manager expire-generations "-14 days"`, then a GC to free the store.

## Stale boot entries (`/boot` full)

On NixOS, collecting system generations does not by itself rewrite the boot
menu, so their kernels and initrds stay on the ESP. After the generations
question, if `/boot` is the short filesystem, offer:

- **Rewrite the boot entries (recommended)** — hand to the user:
  `sudo /run/current-system/bin/switch-to-configuration boot`
- **Skip**

The lasting fix is a lower `boot.loader.systemd-boot.configurationLimit`
(or `boot.loader.grub.configurationLimit`) in the flake — suggest it, do not
edit it.

## GC roots that pin the store

Stray `result` symlinks from `nix build`, old `.direnv/` flake profiles and
`nix-shell --add-root` links keep store paths alive however much is
collected. From the `--print-roots` survey, list roots under `$HOME` and the
project trees (link path → store path), with the store path's closure size
where cheap (`nix path-info -Sh <store path>`). Offer:

- **Remove the roots I tick** — a numbered list; the user names which.
  Removing the *symlink* (`rm <link>` — never the store path) is what frees
  it at the next GC.
- **Remove all `result*` links under the project roots**, listed in full
  first.
- **Skip**

Never remove roots under `/nix/var/nix/profiles`, `/run`, or `/proc` here —
those are generations or live processes, handled above or not at all.

## Collect garbage without touching generations

When the user wants space but no rollback lost, offer a plain GC: it deletes
only paths that no root references.

- `nix store gc` (user; frees only what nothing references)
- `nix store gc --max 10G` to stop after a set amount

## Store optimise

`nix store optimise` hardlinks identical files in the store. Skip it — and
say so — when `auto-optimise-store` is already `true`: the store is already
deduplicated as it is written, and a full optimise pass only costs time.
Otherwise offer it, with a note that it is CPU- and IO-heavy and takes
minutes on a large store: `sudo nix store optimise` (handed to the user).

## Order of a combined hand-over

Generations first, then boot entries, then a GC if the generations step did
not already include one (`nix-collect-garbage` does), then optimise. Root
removal (`rm` of user links) the session runs itself before the hand-over,
so the user's GC frees what those roots held.
