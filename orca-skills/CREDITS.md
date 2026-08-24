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

**Upstream provenance:** `computer-use` (release revision 8) and
`orchestration` (revision 28) from commit
`fe95698b95e7687857d2421549366b8771c71e36` (2026-08-18); `orca-cli` at revision
37. Vendored 2026-08-18; `orca-cli` pinned back to revision 36 on 2026-08-19
and returned to revision 37 on 2026-08-24, when the installed Orca caught up
(see "Which revision to vendor" below). All three now match the installed
app's manifest exactly.

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

**Track the installed Orca app, not this repository's HEAD.** A skill taken
from HEAD can describe a command the installed binary does not have.

Orca ships a manifest inside its AppImage at
`resources/skills/current-manifest.json`, listing every bundled skill with a
`releaseRevision` and a per-file `exactSha256`, plus every historical revision
in `snapshot-registry.json`. That manifest is what tells you which revision the
installed app expects.

The app lagging HEAD is what bit on 2026-08-19: `orca-cli` at HEAD (3944 bytes)
advertised `share skills` and `skill sharing`, while the installed Orca's
`orca skills` subcommand offered only `list`, `get`, `install` and `update`. The
skill described a command the binary did not have. Pinning to revision 36 (3913
bytes) fixed the inaccuracy.

The pin is a **hold, not a destination** — it is released the moment the app
catches up. On 2026-08-24 the installed Orca had been rebuilt (AppImage dated
2026-08-22), its manifest listed `orca-cli` at revision 37, and `orca skills`
had grown the `share` subcommand the skill describes. The two lines that
revision 37 adds — `skill sharing` and `share skills` in the description — are
now accurate against the binary, so the tree returned to 37. Both halves of the
2026-08-19 reasoning had reversed: the warning and the inaccuracy would come
from *staying* at 36.

**A revision pin does not clear Orca's `Skipped — the copy here doesn't match
the official version` warning, and never did.** That claim stood here until
2026-08-25 and is wrong. Orca reads the installed directory with
`observeSkillPackage` and throws `skill-package-link` on any file whose
`nlink != 1`; the throw is caught and reported as status `unrecognized`, which
is exactly that row. Nix-store files are hardlinked by store optimisation
(these three sat at nlink 5–7) and are mode 444 besides, so
`classifyHomeSkillTopology` marks the path `read-only` for the updater too.
`computer-use` and `orchestration` were byte-for-byte the official revisions on
2026-08-25 and were flagged all the same.

So on a host running Orca this tree is not the install surface: Orca installs
and updates its own copies, and `spacecraft.construct.enableOrca` stays off.
Verifying against the manifest remains how the vendored bytes are kept honest
for hosts that do install from here.

To re-vendor after updating the Orca app, read the manifest for the revision it
now expects and verify each file against its `exactSha256`:

```sh
D=$(ls -dt ~/.cache/appimage-run/*/resources/skills | head -1)
jq '.skills[] | select(.name=="orca-cli") | {releaseRevision, files}' "$D/current-manifest.json"
sha256sum orca-skills/orca-cli/SKILL.md
```

`snapshot-registry.json`, beside that manifest, lists every historical revision
with the same per-file hashes. Hashing a vendored file against it identifies the
exact revision the copy holds, which separates "one release behind" from "someone
edited it" — a distinction the Orca dialog does not draw.

A mismatch here is the signal to re-vendor. **A match is not a promise of
silence** — see the mechanism above: on 2026-08-24 `computer-use` and
`orchestration` were byte-identical to the app's own manifest and Orca still
listed all three as `Skipped — the copy here doesn't match the official
version`. For those two the wording was simply untrue; the scanner never got as
far as comparing bytes.

The dialog's advice — "Remove it if you want Orca to update this skill" — is not
actionable against a store path: nothing can be removed from `/nix/store`, and a
Home-Manager switch would restore it regardless. The fix is upstream of the
dialog: leave `spacecraft.construct.enableOrca` off so these leaves are not
installed at all, and let `orca skills install` own real, writable copies
(`spacecraft.construct.perSkillLinks.enable = true` gives it the room). Where
the vendored copies ARE the install surface — no Orca app on the host — nothing
scans them and there is no dialog to satisfy.

## Leaf-name note

`computer-use` and `orchestration` are generic names, and unlike every other
vendored tree here they carry no vendor prefix. That is deliberate and
load-bearing: Orca's own installer and its `orca skills` subcommand look these
directories up **by exact leaf name**, so renaming them to `orca-computer-use`
or similar would vendor the content while breaking the tool that consumes it.
The names are therefore reserved — a future Spacecraft skill must not claim
either one.
