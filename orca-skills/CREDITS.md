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

**Upstream provenance:** `computer-use` and `orchestration` from commit
`fe95698b95e7687857d2421549366b8771c71e36` (2026-08-18); `orca-cli` pinned one
revision behind that (see "Which revision to vendor" below). Vendored
2026-08-18, `orca-cli` re-pinned 2026-08-19.

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

## Which revision to vendor

**Track the installed Orca app, not this repository's HEAD.** The two are not
the same thing, and following HEAD is what breaks Orca's own skill check.

Orca ships a manifest inside its AppImage at
`resources/skills/current-manifest.json`, listing every bundled skill with a
`releaseRevision` and a per-file `exactSha256`. Orca compares what is installed
on disk against *that* manifest. A copy taken from the repository's HEAD can be
newer than the app, and Orca reports it as `Skipped — the copy here doesn't
match the official version`, which reads like corruption but only means the
tree is ahead of the binary.

That is exactly what happened on 2026-08-19: `orca-cli` at HEAD (3944 bytes)
advertised `share skills` and `skill sharing`, while the installed Orca's
`orca skills` subcommand offered only `list`, `get`, `install` and `update`. The
skill described a command the binary did not have. Pinning to revision 36 (3913
bytes) fixed both the warning and the inaccuracy.

To re-vendor after updating the Orca app, read the manifest for the revision it
now expects and verify each file against its `exactSha256`:

```sh
D=$(ls -dt ~/.cache/appimage-run/*/resources/skills | head -1)
jq '.skills[] | select(.name=="orca-cli") | {releaseRevision, files}' "$D/current-manifest.json"
sha256sum orca-skills/orca-cli/SKILL.md
```

A mismatch here is the signal to re-vendor; a match means Orca will leave the
tree alone.

## Leaf-name note

`computer-use` and `orchestration` are generic names, and unlike every other
vendored tree here they carry no vendor prefix. That is deliberate and
load-bearing: Orca's own installer and its `orca skills` subcommand look these
directories up **by exact leaf name**, so renaming them to `orca-computer-use`
or similar would vendor the content while breaking the tool that consumes it.
The names are therefore reserved — a future Spacecraft skill must not claim
either one.
