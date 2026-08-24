# Orca skills (vendored)

Third-party skills from [Orca](https://github.com/stablyai/orca), vendored
verbatim under the MIT License. See [`CREDITS.md`](CREDITS.md) for provenance
and [`LICENSE.txt`](LICENSE.txt) for the upstream license text.

These are **not** Spacecraft Software skills, and they are **not installed by
default**. They use the same open-standard `SKILL.md` format as the
cross-platform skills, so `spacecraft.construct.enableOrca = true` merges them
into the canonical install tree — but only turn that on where the Orca app is
absent.

On a host running Orca, Orca installs and updates these skills itself
(`orca skills install`, which is `npx skills add` underneath) and its updater
cannot accept a copy served from the Nix store: it throws `skill-package-link`
on any file with `nlink != 1` (store optimisation hardlinks them) and skips
unwritable paths (store files are mode 444), reporting the skill as
`Unrecognized` in Settings → Update skills. Byte-identity with the official
revision does not change that.

## Contents

| Skill | Purpose |
|-------|---------|
| `orca-cli` | Drive the `orca` CLI — worktrees, terminals, repos, automations, artifacts, and Orca's embedded browser |
| `computer-use` | Inspect and operate local desktop app windows via accessibility trees, screenshots, and UI actions |
| `orchestration` | Structured multi-agent coordination — threaded messages, task DAGs, decision gates, coordinator loops |

## Updating

Re-copy each `SKILL.md` from the upstream `skills/` directory at a new commit,
then update the provenance commit and date in [`CREDITS.md`](CREDITS.md). Do
not edit the vendored files in place — any local change makes this tree an
*adaptation* rather than a verbatim vendoring, which changes its licensing
posture under Standard §4.2.

Which revision to take, and how to verify it, is in
[`CREDITS.md`](CREDITS.md) § *Which revision to vendor*. Freshness here is a
manual check against the installed app's manifest; Orca's own updater never
audits this tree, and on an Orca host it is not installing from it at all.
