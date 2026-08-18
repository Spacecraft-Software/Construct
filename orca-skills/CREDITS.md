# Credits

The `orca-skills/` directory vendors a three-skill subset of **Orca**'s
official skills collection **verbatim and unmodified**. Every skill under this
directory is third-party work; none of it is a Spacecraft Software original or
adaptation. This file is the §15.3 human-readable counterpart to the
machine-readable `MIT` metadata declared for `orca-skills/**` in the repo-root
[`REUSE.toml`](../REUSE.toml).

## Orca skills

| Field      | Value |
|------------|-------|
| Name       | Orca skills |
| Author(s)  | Lovecast Inc. |
| License    | MIT License (see [`LICENSE.txt`](LICENSE.txt) and [`../LICENSES/MIT.txt`](../LICENSES/MIT.txt)) |
| Source URL | <https://github.com/stablyai/orca> |
| Scope      | The three skills below, vendored verbatim — each `SKILL.md` byte-for-byte identical to upstream, including Orca's own frontmatter. No content was edited, relicensed, or adapted. |

**Upstream provenance:** commit `fe95698b95e7687857d2421549366b8771c71e36`
(2026-08-18), vendored 2026-08-18.

## Vendored skills (3)

Upstream groups these under a top-level `skills/` directory; this directory
drops that one level so each skill's leaf directory sits directly under
`orca-skills/`. The mapping from upstream path to vendored directory:

| Vendored dir | Upstream path |
|--------------|---------------|
| `orca-cli` | `skills/orca-cli` |
| `computer-use` | `skills/computer-use` |
| `orchestration` | `skills/orchestration` |

Upstream ships five further skills — `linear-tickets`, `orca-emulator`,
`orca-emulator-android`, `orca-linear`, `orca-per-workspace-env` — which are
deliberately **not** vendored: they target Linear, Android emulators, and
per-workspace environment plumbing that this tree has no use for. Adding one
later is a matter of copying its directory and extending the table above.

## Leaf-name note

`computer-use` and `orchestration` are generic names, and unlike every other
vendored tree here they carry no vendor prefix. That is deliberate and
load-bearing: Orca's own installer and its `orca skills` subcommand look these
directories up **by exact leaf name**, so renaming them to `orca-computer-use`
or similar would vendor the content while breaking the tool that consumes it.
The names are therefore reserved — a future Spacecraft skill must not claim
either one.
