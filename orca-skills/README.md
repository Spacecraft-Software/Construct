# Orca skills (vendored)

Third-party skills from [Orca](https://github.com/stablyai/orca), vendored
verbatim under the MIT License. See [`CREDITS.md`](CREDITS.md) for provenance
and [`LICENSE.txt`](LICENSE.txt) for the upstream license text.

These are **not** Spacecraft Software skills. They are merged into the same
canonical install tree as the cross-platform skills because they use the same
open-standard `SKILL.md` format, so every agent on the machine reads them from
one place.

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
