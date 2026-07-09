# Credits

The `android-skills/` directory vendors Google's official **Android skills**
collection **verbatim and unmodified**. Every skill under this directory is
third-party work; none of it is a Spacecraft Software original or adaptation.
This file is the §15.3 human-readable counterpart to the machine-readable
`Apache-2.0` metadata declared for `android-skills/**` in the repo-root
[`REUSE.toml`](../REUSE.toml).

## Android skills

| Field      | Value |
|------------|-------|
| Name       | Android skills |
| Author(s)  | Google LLC |
| License    | Apache License 2.0 (see [`LICENSE.txt`](LICENSE.txt) and [`../LICENSES/Apache-2.0.txt`](../LICENSES/Apache-2.0.txt)) |
| Source URL | <https://github.com/android/skills> |
| Scope      | All 19 skills below, vendored verbatim — SKILL.md and `references/` content byte-for-byte identical to upstream, including Google's own frontmatter schema (`license` / `metadata`) and any embedded shell/Gradle examples. No content was edited, relicensed, or adapted. |

**Upstream provenance:** commit `79bee216c47432cbdb576a9bf1a62b65bee56f8f`
(2026-07-06), vendored 2026-07-10.

## Vendored skills (19)

The upstream repository groups skills by category; this directory flattens that
one level (the category prefix is dropped) so each skill's leaf directory sits
directly under `android-skills/`. The mapping from upstream path to vendored
directory:

| Vendored dir | Upstream path |
|--------------|---------------|
| `agp-9-upgrade` | `build/agp/agp-9-upgrade` |
| `camerax` | `camera/camerax` |
| `appfunctions` | `device-ai/appfunctions` |
| `android-cli` | `devtools/android-cli` |
| `verified-email` | `identity/verified-email` |
| `adaptive` | `jetpack-compose/adaptive` |
| `migrate-xml-views-to-jetpack-compose` | `jetpack-compose/migration/migrate-xml-views-to-jetpack-compose` |
| `styles` | `jetpack-compose/theming/styles` |
| `navigation-3` | `navigation/navigation-3` |
| `r8-analyzer` | `performance/r8-analyzer` |
| `engage-sdk-integration` | `play/engage-sdk-integration` |
| `play-billing-library-version-upgrade` | `play/play-billing-library-version-upgrade` |
| `perfetto-sql` | `profilers/perfetto-sql` |
| `perfetto-trace-analysis` | `profilers/perfetto-trace-analysis` |
| `android-intent-security` | `security/android-intent-security` |
| `edge-to-edge` | `system/edge-to-edge` |
| `testing-setup` | `testing/testing-setup` |
| `wear-compose-m3` | `wear/wear-compose-m3` |
| `display-glasses-with-jetpack-compose-glimmer` | `xr/display-glasses-with-jetpack-compose-glimmer` |

## Maintenance

These files are **vendored — do not hand-edit.** Fixes belong upstream; refresh
this directory by re-vendoring from `github:android/skills` (re-flatten the leaf
directories, then re-diff against upstream to confirm zero drift) and update the
provenance commit above. The flatten step is the only transformation applied.
