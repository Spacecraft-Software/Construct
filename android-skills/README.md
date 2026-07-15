<!--
  README for the vendored Android-skills section of the Spacecraft Software Construct repository.
  Audience: humans browsing on GitHub, and agents loading these skills.
  Maintenance: this directory is VENDORED VERBATIM from github:android/skills — do not
  hand-edit skill content; keep the §2 catalogue aligned with the leaf subdirectories.
-->

# Spacecraft Software Construct — Android Skills

Google's official **[Android skills](https://github.com/android/skills)**,
vendored **verbatim and unmodified** into the Construct catalogue. These are
third-party Apache-2.0 skills authored by Google LLC — not Spacecraft Software
originals. See [`CREDITS.md`](CREDITS.md) for full provenance and the
[Standard §4.2](../spacecraft-standard-constitution/) upstream-preservation rationale.

Android skills follow the [open-standard agent-skills](https://agentskills.io/)
`SKILL.md` format — the same format Claude / Gemini / Codex already load — so
once installed they are picked up like any other skill. To learn more, read
Google's docs:

- [Android skills](https://developer.android.com/tools/agents/android-skills)
- [Android CLI](https://developer.android.com/tools/agents/android-cli)

<!-- §1 — What "vendored, flattened" means here -->
## Layout: flattened one level from upstream

Upstream groups skills by category (`<category>/<skill>/SKILL.md`, occasionally
`<category>/<group>/<skill>/SKILL.md`). This directory **drops the category
prefix** so each skill's leaf directory sits directly under `android-skills/` —
matching how Construct's own skill loader and the Nix flake discover skills (one
`SKILL.md` level down) and how the `android` CLI addresses a skill (by its leaf
name, e.g. `--skill=r8-analyzer`). Nothing *inside* a leaf is changed; the
`references/` subtrees are byte-identical to upstream. The upstream→vendored
path mapping is in [`CREDITS.md`](CREDITS.md).

<!-- §2 — Skill catalogue: keep alphabetical, one line per skill -->
## Skills in this section (19)

| Skill | Category | Purpose |
|-------|----------|---------|
| [`adaptive`](adaptive/) | Jetpack Compose | Adapt an app's UI across phones, tablets, foldables, laptops, desktop, TV, Auto, and XR — window sizes, pointer/keyboard input, multi-pane Nav3 Scenes, and Grid/FlexBox adaptive layouts. |
| [`agp-9-upgrade`](agp-9-upgrade/) | Build | Upgrade/migrate an Android project to Android Gradle Plugin (AGP) 9 (non-KMP projects only). |
| [`android-cli`](android-cli/) | DevTools | Install and use the `android` CLI — create projects, run apps, manage virtual devices, query SDK components and docs, and discover/install Android skills. |
| [`android-intent-security`](android-intent-security/) | Security | Audit `AndroidManifest.xml` components and Intent-handling code to prevent Intent redirection and unauthorized access. |
| [`appfunctions`](appfunctions/) | Device AI | Identify key user workflows and expose them to the system as AppFunctions (Kotlin) so agents can discover and run them on-device; refine KDoc for agent comprehension. |
| [`camerax`](camerax/) | Camera | Technical guidance for CameraX — camera features, async recording lifecycles, hardware interop, and ML Kit / Media3 effects. |
| [`display-glasses-with-jetpack-compose-glimmer`](display-glasses-with-jetpack-compose-glimmer/) | XR | Build projected Android XR apps for display glasses with the Jetpack Compose Glimmer UI toolkit and design system. |
| [`edge-to-edge`](edge-to-edge/) | System | Migrate a Jetpack Compose app to adaptive edge-to-edge; fix system-bar/IME inset overlaps and bar legibility. |
| [`engage-sdk-integration`](engage-sdk-integration/) | Play | Integrate, debug, and fix Play Engage SDK implementations — publishing code, data-class→entity mapping, error resolution. |
| [`migrate-xml-views-to-jetpack-compose`](migrate-xml-views-to-jetpack-compose/) | Jetpack Compose | Structured workflow to migrate an Android XML View to Jetpack Compose — planning, dependencies, theming, layout, validation, XML cleanup. |
| [`navigation-3`](navigation-3/) | Navigation | Install/migrate to Jetpack Navigation 3 — deep links, multiple backstacks, scenes, conditional navigation, results, Hilt/ViewModel/view interop. |
| [`perfetto-sql`](perfetto-sql/) | Profilers | Translate natural-language data intents into valid Perfetto SQL and run them against a local trace with `trace_processor`. |
| [`perfetto-trace-analysis`](perfetto-trace-analysis/) | Profilers | Analyze Perfetto traces to root-cause latency, memory, or jank issues in Android apps. |
| [`play-billing-library-version-upgrade`](play-billing-library-version-upgrade/) | Play | Upgrade/migrate a project from any legacy Google Play Billing Library version to the latest stable release. |
| [`r8-analyzer`](r8-analyzer/) | Performance | Analyze build files and R8 keep rules for redundancy/over-breadth to cut app size and troubleshoot ProGuard configs. |
| [`styles`](styles/) | Jetpack Compose | Integrate the Jetpack Compose Styles API — component themes, `Modifier.styleable`, and migrating hard-coded parameters to unified styles. |
| [`testing-setup`](testing-setup/) | Testing | Analyze and create a testing strategy for native Android apps — libraries, infrastructure, and unit/UI/screenshot/e2e harnesses. |
| [`verified-email`](verified-email/) | Identity | Implement verified email retrieval via the Credential Manager API — a secure, OTP-less email-verification sign-up flow. |
| [`wear-compose-m3`](wear-compose-m3/) | Wear | Wear OS Compose Material3 guidance — create/update/migrate Wear projects and core scaffold components; migrate from M2.5/Horologist. |

<!-- §3 — Installation -->
## Installation

These skills install two independent ways:

1. **Google's `android` CLI** (upstream-native) — installs from
   `github:android/skills`, not from this vendored copy:
   ```sh
   android skills add --skill=r8-analyzer --project=.
   android skills add --all
   ```
2. **Construct's Nix flake** — when `spacecraft.construct.enableAndroid = true`,
   these skills are merged into the canonical `~/.agents/skills/` tree alongside
   the cross-platform skills (same `SKILL.md` format), so every agent harness
   already symlinked there picks them up. See the repo-root
   [`README.md`](../README.md) and [`flake.nix`](../flake.nix).

<!-- §4 — No .zip/.skill bundles -->
## No `.zip` / `.skill` bundles

Unlike the root Construct skills, Android skills are **not** packaged as
`<name>.zip` / `<name>.skill` bundles. They are consumed either by Google's
`android` CLI (from upstream) or directly as `SKILL.md` directories via the
flake — neither path uses Construct's bundle distribution. The absence of
bundles here is intentional, not an oversight, and the `CONTRIBUTING.md` drift
sweep skips `android-skills/` accordingly.

<!-- §5 — Maintenance -->
## Maintenance — vendored, do not hand-edit

Skill content here is a verbatim copy of upstream. **Do not edit it in place** —
fixes belong upstream at `github:android/skills`. To refresh, re-vendor: fetch
upstream, re-flatten leaf directories, re-diff against upstream to confirm zero
drift, and bump the provenance commit recorded in [`CREDITS.md`](CREDITS.md).

<!-- §6 — License -->
## License

Apache License 2.0 © Google LLC — see [`LICENSE.txt`](LICENSE.txt) (upstream
verbatim) and [`../LICENSES/Apache-2.0.txt`](../LICENSES/Apache-2.0.txt) (REUSE).
Coverage is declared for `android-skills/**` in the repo-root
[`REUSE.toml`](../REUSE.toml). Per Standard §4.2 the upstream license is
preserved as-is and never relicensed.

---

*─── Vendored into Spacecraft Software ───*
