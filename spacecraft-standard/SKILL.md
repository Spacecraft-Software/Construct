---
name: spacecraft-standard
description: >
  The authoritative compliance reference for ALL work on Spacecraft Software-umbrella projects and
  subprojects (Zamak, Bravais, Ferrocast, Craton, Ironway, Caliper, Mawaqit, and any future
  projects). ALWAYS load this skill before writing code, documentation, specifications,
  architecture decisions, UI designs, naming choices, or any other artifact for a
  Spacecraft Software-umbrella project — even if the user doesn't explicitly mention the Standard.
  If the user mentions "Spacecraft Software", a Spacecraft Software subproject name, or asks you to work on
  anything in the Spacecraft Software ecosystem, consult this skill immediately. It encodes
  The Steelbore Standard v1.25 (§3.3 renamed to "Secure / Security By Design"; §8 Documentation (Texinfo); §7 Shell Environment; §15.2 email obfuscation note) so
  you never need to ask for it or have it attached to a prompt again.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# The Steelbore Standard — Compliance Reference

**Version:** 1.25 | **Date:** 2026-06-20 | **Author:** Mohamed Hammad
**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** Copyright (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

This skill encodes The Steelbore Standard in full. Apply every applicable section
to any artifact you produce under the standard. The 15-point compliance checklist
in §16 is your audit gate — run through it mentally before finalising any output.

> **License note:** This skill is `GPL-3.0-or-later` (skills are software-class, §4.1.1).
> The *published* Standard document it encodes is licensed `CC-BY-SA-4.0`.

**Changelog:** see [`references/CHANGELOG.md`](references/CHANGELOG.md) for the full version history.

---

## §1 — Preamble

The Steelbore Standard defines the engineering principles, compliance requirements, and design conventions that govern all software produced under Spacecraft Software. The umbrella encompasses two categories of work: **Steelbore OS** — the operating system and all OS-specific artifacts (configurations, themes, OS tooling) — and **independent Spacecraft Software projects** such as Zamak, Ironway, Ferrocast, and Caliper, which are designed to work with Steelbore OS but are not OS-specific and may run on any compliant platform. Both categories are full citizens of Spacecraft Software and subject to this standard in full. Where a project-specific specification conflicts with this standard, the stricter of the two requirements shall prevail.

**Standard name vs. project naming.** "The Steelbore Standard" is the canonical, stable name of *this standard*. It is independent of the projects it governs and of the umbrella organization name — the standard retains this name regardless of any future renames. The v1.7 umbrella rename (Steelbore → Spacecraft Software) and the v1.8 reinstatement of this standard's name are recorded in the changelog. Versioning of project codenames (see §2) and versioning of the standard are separate concerns.

---

## §2 — Aerospace, Sci-Fi & AI Naming Convention

All **new** project codenames, module identifiers, and public-facing component names
**must** draw from one of the following domains:

- **Real aerospace and astronomy** — orbital mechanics terms, propulsion concepts,
  named missions/programs, stellar objects and phenomena, observatories.
- **Science-fiction franchises with space / AI / cybernetic themes** — naming is
  meant to be enjoyable as well as fitting. The following are explicitly endorsed
  canonical sources:
  - *2001: A Space Odyssey*, *The Matrix*, *Terminator* — the original canonical trio
  - *The Hitchhiker's Guide to the Galaxy* — also a rich vein for in-jokes (Vogon,
    Marvin, 42, Babel fish, Heart of Gold)
  - *Hackers* (1995)
  - Spielberg films (*Close Encounters of the Third Kind*, *E.T. the Extra-Terrestrial*,
    *A.I. Artificial Intelligence*, *Minority Report*, *Ready Player One*, etc.)
  - *Ghost in the Shell*
  - *Equilibrium*
  - *Dune*
  - *Æon Flux*
  - *Super 8*
  - *LOST* (TV series)
  - *Cloverfield* films
  - Robot / android names from any sci-fi film or franchise (e.g., HAL, Data, Bishop,
    T-800, GERTY, TARS, Marvin)

  Other franchises (e.g., *Alien*, *Blade Runner*, *Ex Machina*) remain acceptable
  if they fit the space-machine-AI register.
- **Generic sci-fi / AI vocabulary** — hyperspace, neural, cybernetic, synthetic,
  sentinel, oracle, daemon, vector, lattice (the lowercase common noun), etc.

| Category  | Examples                            | Domain                          |
|-----------|-------------------------------------|---------------------------------|
| Projects  | Apollo, Discovery, Skynet, Trinity  | Missions / Ships / AI Machines  |
| Modules   | Apogee, HAL, Cortex, Sentinel       | Subsystems / AI Cores           |
| Utilities | Boost, Throttle, Trace, Telemetry   | Operational Verbs / Telemetry   |
| Releases  | Vega, Pulsar, Quasar, Nebula        | Stellar Phenomena               |

Names must be **fitting for space-related and futuristic AI machines** — the
test is whether the name would feel at home on the hull of a spacecraft or in
the boot banner of an AI machine. Reject proposed names that don't pass this test.

### §2.1 — Legacy Metallurgical Registry (pre-v1.2)

Projects named before the v1.2 convention drew from metallurgy, materials science,
and industrial forging. These names are **preserved as-is** unless explicitly
renamed by the maintainer. The v1.2 convention applies prospectively — no forced
back-rename.

| Codename    | Status                | Description                                                    |
|-------------|-----------------------|----------------------------------------------------------------|
| `Steelbore` | Renamed to Spacecraft Software (umbrella, v1.7) | Former umbrella organization name. Renamed 2026-05-15 under the v1.7 brand consolidation. The OS line (`Steelbore OS`, `Steelbore OS Bravais`, `Steelbore OS Lattice`) retains the Steelbore name. |
| `Aetheric`  | Active                | Next-generation extensible text editor (Pulsar + Quasar + Nebula IPC). |
| `Zamak`     | Active                | Rust bootloader (Limine rewrite)                               |
| `Bravais`   | Completed (renamed)   | NixOS flake configuration. Renamed from `Lattice` due to collision with Lattice OS. `Bravais` is still a metallurgical-era name (Bravais lattice) and predates the v1.2 convention. |
| `Ferrocast` | Planning              | Rust PowerShell rewrite (16-crate workspace)                   |
| `Craton`    | Reserved              | Rust universal package manager — codename registered; no work started yet. |
| `Ironway`   | Active                | Rust OpenTTD rewrite                                           |
| `Caliper`   | Active                | Rust raster-to-vector tracing engine (CLI+TUI)                 |
| `Mawaqit`   | Planning (**Pending rename**) | Islamic prayer times app (Flutter + Rust CLI + libmawaqit). To be renamed under the v1.2 aerospace/sci-fi/AI convention. |
| `Anvil`     | Completed             | Rust workspace; benches and CHANGELOG; legacy forging-tool name.                |
| `Flux`      | Completed             | Rust workspace; CHANGELOG and deny.toml; legacy metallurgical-flux name.        |
| `Pearlite`  | Active                | Rust workspace; audit.toml, clippy.toml, CHANGELOG; steel microstructure name.  |
| `Ferrite_OS`| Active                | Custom OS / DOS-emulation experiments; ferrite (iron-based material) name.      |
| `Forge`     | Active                | Production flavor tooling (forge-cli, forge-build, forge-activate); forging-tool name. |

Existing legacy-named projects MAY be renamed under the v1.2 convention at the
maintainer's discretion — renames are optional. When a rename happens, update
this table and §15.1's subdomain table in the same commit.

### §2.2 — Skill IDs are functional, not codenamed

Skill directory names and SKILL.md `name` fields are **functional identifiers**
(e.g., `spacecraft-standard`, `spacecraft-document-format`) and are not subject to
the §2 codename convention. §2 reserves codenames for projects/modules/utilities/releases,
not for skill identifiers.

---

## §3 — Priority Hierarchy (Non-Negotiable Order)

A higher-numbered priority **may never compromise** a lower-numbered one.

### §3.1 — Priority 1: Stability
Software must behave predictably and remain correct under sustained and adverse
conditions. Stability is the foremost priority. **Memory safety is the single most
important contributor to stability and the primary means of achieving it — but it is
not the whole of Priority 1.**

**Memory safety (primary lever):**
- **Preferred language: Rust** — governed by the Spacecraft Software Rust Guidelines.
  → Always load the `microsoft-rust-guidelines` skill before writing any Rust code.
- When Rust is not viable (Flutter/Dart, Zig, etc.), **mandatory mitigations**:
  - **ASLR** (Address Space Layout Randomization) on all compiled binaries
  - **CFI** (Control-Flow Integrity) wherever the toolchain supports it
- Memory-Safe Languages (MSLs) are always preferred. If an MSL alternative exists,
  it must be chosen unless a documented technical exemption is filed.

**Beyond memory safety, stability also requires:**
- **Robust error handling** — failures must be surfaced and handled, never silently
  swallowed; no panics / `unwrap` / `expect` on untrusted or fallible input in production paths.
- **Fault tolerance and graceful degradation** — components must survive partial failure,
  degrade gracefully under load or dependency loss, and recover rather than crash.
- **Verified by testing** — stability properties must be backed by tests (unit, integration,
  and fuzz/property where applicable) gating CI, not asserted by inspection alone.

### §3.2 — Priority 2: Performance
Performance is the foremost priority after stability. The default means of achieving it is
**multi-core, multi-thread concurrency** — parallelism is the expected baseline, not an
afterthought — *unless* concurrency would materially degrade performance (synchronization
overhead, lock contention, or inherently serial / small workloads outweighing the gains),
in which case a simpler or serial approach must be chosen and the trade-off documented.
- Concurrency must be **designed-in from the start**, never bolted on retroactively.
- Release builds should use CPU-optimized flags — `-march=native`, LTO, PGO —
  **where the toolchain and target support them reliably.** Any such flag known to
  break or destabilize builds on a given platform/toolchain/linker (e.g., LTO under
  certain NixOS, cross-compilation, or static-linking setups) MUST be disabled and
  the reason documented. Stability (P1) outranks Performance (P2) — never ship a
  broken or unstable build for the sake of a flag.
- Benchmarking is **mandatory** before and after any optimization work; regressions must
  be documented and justified — and it is the evidence by which the concurrency-vs-serial
  trade-off above is decided.

### §3.2.1 — Platform-Specific Compiler & Linker Flag Caveats

Compiler and linker optimization flags are **not universally portable** across operating
systems and distributions. Just as systemd-specific settings do not apply to non-systemd
distros (e.g., GNU Guix System, Void Linux, Gentoo with OpenRC), linker and LTO flags
must be adapted to the target platform's toolchain layout.

**NixOS / Steelbore OS Bravais:** Because NixOS isolates packages in the `/nix/store`,
GCC's LTO plugin is not on the standard linker search path. When using `-flto` (Link Time
Optimization) on NixOS, you **must** explicitly point GCC's linker to the GCC LTO plugin
via `-fuse-ld=mold` (preferred) or `-fuse-ld=bfd` (fallback). Without this, LTO-enabled
builds will fail to link.

> **Rule:** Whenever recommending or applying compiler/linker flags — especially `-flto`,
> `-march=native`, or PGO — verify whether the target OS requires supplementary flags or
> alternative linker selection. Document the OS-specific requirements alongside the flags.

### §3.3 — Priority 3: Secure / Security By Design
- Kernel hardening (XanMod, grsecurity profiles) where applicable.
- Sandboxing and privilege separation for all network-facing components.
- **Post-Quantum Cryptography (PQC) readiness**: all crypto subsystems must support
  PQC migration paths. Use hybrid schemes (classical + PQC candidate) where library
  support exists. Adopt NIST-finalized PQC standards within one major release cycle.
  - Current targets: ML-KEM-768, ML-DSA-65 (as used in Ferrocast)
- Dependency auditing: `cargo-audit` or equivalent before any third-party crate inclusion.

**Cardinal Rule:** Any optimization that weakens **stability (including memory safety)** or
security hardening **must be rejected**, no exceptions.

---

## §4 — Licensing & Compliance

### §4.1 — Project License (GPL-3.0-or-later or AGPL-3.0-or-later)

- **License:** strong copyleft — each project chooses **`GPL-3.0-or-later`** or
  **`AGPL-3.0-or-later`**, whichever fits the project better:
  - Use **`AGPL-3.0-or-later`** when the software is **network-facing** — anything
    users interact with primarily over a network (servers, web services, SaaS,
    hosted APIs, multiplayer/network daemons). AGPL closes the "SaaS loophole."
  - Use **`GPL-3.0-or-later`** for everything else (local CLIs, libraries,
    desktop/TUI apps, OS components, bootloaders).
  - GPLv3 and AGPLv3 are mutually compatible by design — an umbrella mixing both is fine.
- No proprietary, closed-source, or permissive-only license for core project code.
- **Review & migrate (existing projects).** Not merely prospective: existing projects are
  to be **reviewed and relicensed** to whichever of `GPL-3.0-or-later` / `AGPL-3.0-or-later`
  best fits them (AGPL for network-facing). Migration is the maintainer's per-project
  decision on that project's own signed commit; a deliberately retained non-best-fit
  license must be documented.

#### §4.1.1 — Artifact license classes

The GPL/AGPL choice above governs **software**. License by artifact class:

| Artifact class | Default license |
|----------------|-----------------|
| **Software** — code, manifests, build tooling, and **skills** | `GPL-3.0-or-later` (or `AGPL-3.0-or-later` if network-facing, §4.1) |
| **Documents** — specifications, prose guides, books, and document deliverables (per `spacecraft-document-format`), incl. the published Standard | `CC-BY-SA-4.0` by default (`CC-BY-4.0` when intended for maximal reuse) |
| **Third-party-derived artifacts** | Preserve the **upstream** license per §4.2 (e.g., `MIT`, `GFDL-1.3-or-later`) — never relicensed to the project default |

Skills are **software-class** → `GPL-3.0-or-later` (no skill is network-facing → no AGPL).
Deliberate split for the Standard: the **published Standard document** is `CC-BY-SA-4.0`,
while this `spacecraft-standard` **skill** encoding is `GPL-3.0-or-later`.

### §4.2 — Upstream License Compliance (preserve what you build on)

When a project incorporates, adapts, or links third-party code, it MUST satisfy that
upstream's license in full — independent of the project's own GPL/AGPL choice:

- **Preserve verbatim** all upstream copyright notices, license texts,
  `NOTICE`/`AUTHORS` files, and in-file license headers — never strip, rewrite, or
  relicense them.
- **Ship** each distinct upstream license text in the project's `LICENSES/` directory (§4.3).
- **Verify compatibility** of the upstream license with the project's GPL/AGPL license
  before inclusion.
- This is the legal/mechanical obligation; §15.3's `CREDITS.md` is the human-readable
  narrative counterpart. When both are triggered, both apply.

### §4.3 — SPDX & REUSE Compliance

Spacecraft Software follows the **[REUSE specification](https://reuse.software)** for
unambiguous, machine-readable license and copyright metadata. Every project MUST be
`reuse lint`-clean.

**Every file carries two SPDX tags** — copyright *and* license:
```
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later
```
(Substitute the project's actual license — `GPL-3.0-or-later` or `AGPL-3.0-or-later` —
and the correct comment syntax for the file type.)

- **Software source files** (`.rs`, `.ts`, `.js`, `.py`, `.sh`, `.ps1`, `.go`, etc.)
  and project manifests (`Cargo.toml`, `package.json`, `flake.nix`, etc.) carry both
  tags as an inline header.
- **Files that cannot carry an inline header** — documents (`.odt`, `.docx`, `.pdf`, …),
  images, binary assets, generated files — are covered by a `.license` sidecar file
  **or** an entry in the repo-root `REUSE.toml`. No file is left uncovered (this
  replaces the former blanket "documents are exempt" rule).
- **`LICENSES/` directory:** verbatim text of every license used lives in
  `LICENSES/<SPDX-id>.txt` (e.g., `LICENSES/GPL-3.0-or-later.txt`,
  `LICENSES/AGPL-3.0-or-later.txt`, plus any upstream licenses per §4.2). A root
  `LICENSE` file MAY remain as a pointer for GitHub's license detection.
- **CI gate:** `reuse lint` MUST pass before shipping.

When writing or reviewing any file, confirm REUSE coverage; when generating a new file,
add the two-tag header (or the `.license` sidecar / `REUSE.toml` entry for files that
can't carry one).

---

## §5 — Project Posture

Spacecraft Software is a personal hobby project. This posture is the **default** for every
project under the umbrella and is non-negotiable. Individual projects may
adopt a more open posture (see §5.3) but never a more closed one.

§4 defines the formal license; this section defines the **stated stance** that
sits alongside it. License says what the user *may* do; posture says what they
should *expect* from the maintainer.

### §5.1 — Default Posture (Personal / Hobby)

| Aspect         | Default                                                        |
|----------------|----------------------------------------------------------------|
| Audience       | Maintainer's own use case                                      |
| Pace           | Hobby pace; no service-level commitments                       |
| Warranty       | None — provided AS IS                                          |
| Liability      | None — see project `NOTICE.md`                                 |
| Contributions  | Welcome but not guaranteed to be accepted                      |
| Forking        | Encouraged                                                     |
| License        | GPL-3.0-or-later or AGPL-3.0-or-later, per §4.1 (formal terms govern in any conflict) |

### §5.2 — Required Posture Files (per project)

Every Spacecraft Software project repository **must** ship the following files at its
root, derived from the canonical Spacecraft Software templates:

| File              | Purpose                                                     |
|-------------------|-------------------------------------------------------------|
| `README.md`       | Includes a "Project Posture" section linking to the two below |
| `NOTICE.md`       | Full no-warranty / no-liability statement; defers to the project's GPL/AGPL license (§4.1) for binding terms |
| `CONTRIBUTING.md` | Contribution scope, PR-acceptance discretion, sign-off, security reporting, license-of-contributions |
| `LICENSES/`       | REUSE license directory (§4.3): verbatim text of every license used (`GPL-3.0-or-later` or `AGPL-3.0-or-later`, plus any upstream licenses per §4.2). A root `LICENSE` MAY remain as a GitHub-detection pointer. |

Customize only the project name, scope, and any project-specific carve-outs.

### §5.3 — General-Use Carve-Out

A project may declare itself **intended for general use**. When it does:

- The declaration MUST appear in that project's `README.md` posture section.
- The no-warranty / no-liability stance from §5.1 still applies in full —
  general-use status changes audience and intent, **not** legal terms.
- General-use projects must hold a higher release-quality bar:
  semantic versioning, maintained `CHANGELOG.md`, deprecation policy, and a
  documented support window for the current major version.

**General-use registry** (keep in sync with §15.1 subdomain table):

| Project      | Posture       |
|--------------|---------------|
| Anvil-SSH    | General-use   |
| (all others) | Personal      |

### §5.4 — Maintainer Discretion

PR acceptance, feature scope, naming, architecture, and roadmap are at the
maintainer's sole discretion. This is stated openly so contributors can
calibrate effort accordingly. Rejection reflects fit, not quality.

### §5.5 — Package Distribution Requirements

Every released package **must** ship first-party package definitions for the following package managers, committed alongside the release:

| File                    | Package manager / format            |
|-------------------------|-------------------------------------|
| `packaging/guix.scm`    | GNU Guix — Scheme package definition |
| `packaging/default.nix` | Nix — Nix flake / derivation        |
| `packaging/PKGBUILD`    | Arch Linux — `makepkg`-compatible   |

**Rules:**

- All three files MUST be present and buildable before a release tag is pushed.
- Each file must reference the exact release version and source archive SHA-256 checksum so that the package can be built reproducibly from the tagged release. Use the format native to each package manager:
  - **Guix (`guix.scm`):** `(sha256 (base32 "<nix-base32-hash>"))` inside the `origin` stanza.
  - **Nix (`default.nix`):** `sha256 = "<sri-or-hex-hash>";` inside the `fetchurl` or `fetchFromGitHub` call.
  - **Arch (`PKGBUILD`):** `sha256sums=('<hex-hash>')` array variable alongside `source=()`.
- The `packaging/` directory is tracked in the project's version-control repository alongside the source code.
- These files are software-class artifacts and inherit the project's GPL/AGPL license (§4.1); each file must carry the standard SPDX two-tag header (§4.3).
- If a package manager's ecosystem imposes a stricter naming scheme or directory layout, comply with that scheme while still meeting the above requirements.

---

## §6 — Platform & Systems Requirements

### §6.1 — POSIX Compliance
All CLI tools, daemons, and system utilities must be **POSIX-compliant**.
Platform-specific extensions go behind feature flags and must not be required
for core functionality.

### §6.2 — Post-Quantum Cryptography
Crypto subsystems must have migration paths to post-quantum algorithms.
Current implementations should use hybrid schemes where library support exists.

### §6.3 — Signed & Verified Commits (Non-Negotiable)

Every commit pushed to a Spacecraft Software-controlled Git remote **must** be
cryptographically signed and show "Verified" on the hosting platform's
commit/PR view (GitHub today; Gitway or any future Spacecraft Software host inherits
the same rule).

**Mandatory rules — violation blocks shipping:**

| Rule | Detail |
|------|--------|
| All commits signed | `commit.gpgsign=true` configured globally. SSH signing (`gpg.format=ssh`) is the current default; GPG is acceptable. The signing key MUST be registered as a **Signing** key on the hosting platform — Authentication-only keys do not validate signatures. |
| Authorized signing identity | All commits from v1.12 onwards must be signed with the `Mohamed.Hammad@SpacecraftSoftware.org` key. The committer email and the signing key identity must both resolve to `Mohamed.Hammad@SpacecraftSoftware.org`. Commits predating v1.12 are exempt from this requirement. |
| Hosting-platform "Verified" required | Every commit on a Spacecraft Software remote must show "Verified" on the platform's commit/PR view. Unsigned or "Unverified" commits MUST be remediated (re-signed via rebase or amend by the original author) before merge to a default branch. |
| Programmatic commits signed too | Bots, CI pipelines, scripted commits, and assistant-driven commits inherit the same rule — no `--no-gpg-sign`, no signing-disabled subshells. The signing pipeline runs unattended. |
| Rewrites preserve signatures | Rebase, amend, cherry-pick, and squash MUST re-sign each resulting commit. Don't push history that lost signatures through rewriting. |
| Local verification is best-effort | `git log --show-signature` may report "No signature" on a given host when `~/.ssh/allowed_signers` is not populated — this is a local-verifier gap, not a signing failure. The hosting platform's "Verified" badge is authoritative. |

**Algorithm note:** Ed25519 SSH signing is the current default. §6.2 calls
for PQC readiness across the cryptographic surface; commit-signing
algorithm migration is gated on hosting-platform support for post-quantum
key formats. When GitHub (or Spacecraft Software's own Gitway) accepts PQC signing
keys, Spacecraft Software commits migrate accordingly.

---

## §7 — Shell Environment

Spacecraft Software tooling, documentation, and CI pipelines target **four first-class shell environments**: **Nushell**, **Ion**, **Brush**, and **Bash**. All four are equally supported; none is deprecated or downgraded.

### §7.1 — Script Portability Policy

| Rule | Detail |
|------|--------|
| Default: POSIX-compatible | Shell scripts in source trees, CI pipelines, `Makefile` targets, and documentation examples **must** be written to the POSIX sh subset unless a shell-native feature is required. POSIX scripts run correctly in Bash and Brush without modification. |
| Nushell / Ion native variants when needed | When a task cannot be expressed cleanly in POSIX sh (structured data pipelines, typed parameters, Nushell modules), provide a Nushell (`.nu`) and/or Ion-native variant alongside the POSIX version. Do not force POSIX-only idioms that degrade the Nushell or Ion experience. |
| No Bashisms in shared scripts | Bash-only extensions (`[[ ]]`, `(( ))`, process substitution `<(...)`, `${var^^}`, indexed arrays) are prohibited in files intended for all four shells. Bash-specific scripts are permitted only when explicitly scoped (e.g., `#!/usr/bin/env bash` shebang, clearly labeled). |
| Graceful shell detection | Tools that need runtime shell detection must inform the user or degrade gracefully rather than silently failing in non-Bash environments. |

---

## §8 — Documentation (Texinfo)

Spacecraft Software user-facing projects should ship a **Texinfo manual** as the canonical technical reference. Texinfo is the preferred format for reference documentation that accompanies distributed software, following GNU project conventions.

### §8.1 — When a Texinfo Manual Is Required

| Project type | Requirement |
|---|---|
| CLI / TUI / GUI application with substantive user-facing functionality | **MUST** ship a Texinfo manual covering invocation, options, concepts, and examples |
| Library with a public API | **SHOULD** ship a Texinfo reference manual covering all public interfaces |
| Simple script / internal tooling | **MAY** skip; a well-structured `README.md` suffices |

### §8.2 — Source Format and File Layout

- Source files use the `.texi` extension (Texinfo 7.x+).
- Manuals live at `doc/<project>.texi` in the project root.
- The project's top-level `Makefile` must expose three targets: `make info`, `make html`, `make pdf`.

### §8.3 — Required Structural Elements

Every Texinfo manual must include the following elements:

| Element | Purpose |
|---|---|
| `@dircategory` / `@direntry` | Registers the manual in the Info directory |
| `@copying` block | License statement and copyright notice |
| `@titlepage` | Title, version, author, and copyright |
| `@node Top` + `@top` | Required top-level node for Info readers |
| `@menu` per chapter | Navigation structure |

### §8.4 — Output Formats

Build and ship all three output formats:

| Format | Tool | Purpose |
|---|---|---|
| `.info` | `makeinfo` / `texi2any` | Info readers (Emacs, standalone `info`) |
| `.html` | `makeinfo --html` | Project documentation website |
| `.pdf` | `texi2pdf` | Printable reference |

Install `.info` files using `install-info` at package install time so they appear in the system Info directory.

### §8.5 — Licensing

Texinfo manuals are **document-class** artifacts (§4.1.1) and default to **CC-BY-SA-4.0**. **GFDL-1.3-or-later** is a permitted alternative when the manual is distributed alongside GPL-licensed software and compatibility with GNU documentation collections is desired. Include the chosen license in `LICENSES/` per §4.3.

### §8.6 — Packaging Integration

Package manifests must install the `.info` file and register it with `install-info`:

| Package manager | Requirements |
|---|---|
| **Guix** (`packaging/guix.scm`) | Add `texinfo` as a native input; run `install-info` in the install phase |
| **Nix** (`packaging/default.nix`) | Add `texinfo` to `nativeBuildInputs`; standard Autoconf/Make `installPhase` handles `install-info` automatically |
| **PKGBUILD** (`packaging/PKGBUILD`) | Add `texinfo` to `makedepends`; `install -Dm644` for `.info` files; call `install-info` in `post_install` |

---

## §9 — Privacy-Friendly Application (PFA) Policy

Every Spacecraft Software application must satisfy **all three** PFA requirements:

| Requirement        | Rule                                                                     |
|--------------------|--------------------------------------------------------------------------|
| No Tracking/No Ads | Zero advertising, tracking, analytics SDKs, or telemetry beacons        |
| Minimal Permissions| Only essential permissions; requested lazily at point of use, never eagerly |
| Local Storage      | User data stored locally by default; sync is strictly opt-in, E2E encrypted |

When reviewing or designing any feature that touches data handling, permissions,
or networking, verify all three PFA requirements are met.

---

## §10 — Key Bindings

All interactive applications must support **both**:

| Scheme    | Requirement                                                              |
|-----------|--------------------------------------------------------------------------|
| **CUA**   | Standard bindings (Ctrl+C/X/V/Z/S) must work in all text input contexts  |
| **Vim**   | Modal editing layer (Normal / Insert / Visual mode) as opt-in feature. Minimum: hjkl navigation where full Vim layer is impractical |

---

## §11 — Spacecraft Software Color Palette (WCAG-Compliant)

The **only** permitted colors for Spacecraft Software interfaces and documents:

| Token          | Hex       | RGB              | Role                        |
|----------------|-----------|------------------|-----------------------------|
| Void Navy      | `#000027` | RGB(0, 0, 39)    | **Background / Canvas**     |
| Molten Amber   | `#D98E32` | RGB(217, 142, 50)| Primary Text / Active Readout |
| Steel Blue     | `#4B7EB0` | RGB(75, 126, 176)| Primary Accent / Structural |
| Radium Green   | `#50FA7B` | RGB(80, 250, 123)| Success / Safe Status       |
| Red Oxide      | `#FF5C5C` | RGB(255, 92, 92) | Warning / Error Status      |
| Liquid Coolant | `#8BE9FD` | RGB(139, 233, 253)| Info / Links               |

**`#000027` (Void Navy) is the mandatory background for ALL Spacecraft Software surfaces.**
No alternative background is permitted. This is non-negotiable.

For document/file generation → load the `spacecraft-document-format` skill.
For IDE/terminal themes → load the `spacecraft-theme-factory` skill.

### §11.1 — Steelbore Theme (Application Theming Standard)

When building a new Spacecraft Software application (GUI, TUI, or web), all palette
references **must** be accessed through a named theme called **`Steelbore`** rather than
referenced as bare hex literals. The `Steelbore` theme is the canonical color contract:

| Theme token  | Maps to palette token | Hex       |
|--------------|-----------------------|-----------|
| `background` | Void Navy             | `#000027` |
| `foreground` | Molten Amber          | `#D98E32` |
| `accent`     | Steel Blue            | `#4B7EB0` |
| `success`    | Radium Green          | `#50FA7B` |
| `error`      | Red Oxide             | `#FF5C5C` |
| `info`       | Liquid Coolant        | `#8BE9FD` |

**Rationale:** isolating palette references behind the `Steelbore` theme name makes it
trivial for end users to substitute a custom theme without touching application logic —
swap the theme, not every hex literal.

- The theme file/module **must** be named `steelbore` (snake_case) in the project's
  theme registry, configuration layer, or equivalent (e.g., `themes/steelbore.json`,
  `steelbore.toml`, a Rust `Theme::Steelbore` variant).
- Hard-coding palette hex values directly in UI logic is **forbidden** for new apps.
  Use theme tokens exclusively.
- Existing apps are encouraged but not required to migrate; new apps are required.

---

## §12 — Typography (FOSS-Licensed Fonts Only)

Acceptable font licenses: **OFL, Apache 2.0, Ubuntu Font License, CC0-1.0**

| Context        | Font              | License |
|----------------|-------------------|---------|
| Headings       | Share Tech Mono   | OFL     |
| Body / Code    | Inconsolata       | OFL     |
| Fallback       | monospace (system)| N/A     |

Never use proprietary fonts. When suggesting or using fonts in any Spacecraft Software artifact,
verify they are available on Google Fonts or another FOSS-licensed repository.

---

## §13 — UI/UX Design System

- **Material Design** is the required component system for all graphical applications.
  Theme Material components with the §11 color palette.
- **WCAG 2.1 Level AA** contrast is the minimum for all color pairings.
  Any new color additions must be WCAG-verified before adoption.
- **Accessibility:** screen readers, keyboard-only navigation, and system accessibility
  preferences (reduced motion, high contrast) must all be respected.

---

## §14 — Date, Time & Units

### §14.1 — Date & Time Format Rules

| Concern      | Rule                                                             | Example                      |
|--------------|------------------------------------------------------------------|------------------------------|
| Date format  | ISO 8601 only: `YYYY-MM-DD`                                      | `2026-03-08`                 |
| Time format  | 24-hour only: `HH:MM:SS` — AM/PM is **never** permitted          | `14:30:00`                   |
| Timestamp    | Combined ISO 8601 UTC: `YYYY-MM-DDTHH:MM:SSZ`                    | `2026-03-08T14:30:00Z`       |
| Timezone     | **UTC Z is the default and preferred primary** for general-purpose, cross-system, and machine-readable timestamps. A project whose core domain is inherently local-time-bound (e.g., solar/prayer-time calculations) may declare local time as its primary record instead — a documented exception, not a free choice. See §14.2 and §14.2.1 | `Z` not `+00:00`             |
| Duration     | ISO 8601 duration format only                                    | `PT1H30M` not "1h 30m"       |
| Units        | Metric (SI) primary; imperial in parentheses only if locale requires | `100 km (62 mi)`         |

Apply these conventions to all generated code, documentation, comments, and any
user-facing strings. Never output AM/PM time, non-ISO dates, or imperial-primary units.

### §14.2 — UTC Z Timezone Policy

**UTC Z is the default and preferred timezone for stored, transmitted, logged,
and committed timestamps across Spacecraft Software projects.** It is the
convention every project should reach for first — it keeps cross-project tooling,
sorting, and interchange simple and unambiguous. Under this default, the `Z`
suffix is required on primary timestamps, and local time expressed as a UTC
offset (e.g., `2026-05-24T13:34:55+03:00`) may optionally accompany a UTC Z value
as a secondary, human-convenience field — but UTC Z remains the authoritative
record.

This is a strong default, not a universal mandate forced onto every domain
regardless of fit — §14.2.1 documents the exception that lets a project whose
domain is genuinely local-time-bound use local time as its primary record instead.

**Rules for projects under the UTC Z default — apply unless a project has filed
the §14.2.1 exception:**

| Rule | Detail |
|------|--------|
| `Z` suffix required | Every **primary** stored/transmitted timestamp MUST end with `Z`. `2026-03-08T14:30:00Z` ✓. A companion local-time field with UTC offset is permitted alongside it. |
| No offset notation as replacement | Offset notation (`+03:00`, `-05:00`, etc.) is **forbidden as a replacement** for UTC Z. It is permitted only as an optional companion field alongside a `Z`-suffixed primary. |
| No bare local time in data | Local-time timestamps **without** timezone info are **forbidden** in files, databases, logs, API responses, and commits. |
| Log entries use UTC + `Z` | Every log line timestamp must be `YYYY-MM-DDTHH:MM:SS.sssZ` (millisecond precision encouraged). |
| Commit timestamps use UTC | `GIT_COMMITTER_DATE` and `GIT_AUTHOR_DATE` must be UTC when set programmatically. |
| File metadata written by Spacecraft Software tools | mtime/ctime written by Spacecraft Software tools must be UTC-sourced. |

### §14.2.1 — Domain Exception: Inherently Local-Time-Bound Projects

A project whose core domain is fundamentally defined by **local civil or solar
time** — not by a moment in absolute (UTC) time — may declare local time as the
**primary** representation for that domain's data. Examples: prayer-time
calculations (`Mawaqit`), sunrise/sunset tables, local event or business-hours
scheduling. For data like this, the meaningful value *is* "06:14 local, at this
place" — collapsing it to a UTC instant first and treating that as authoritative
would misrepresent what the data actually is.

**Conditions for the exception:**

1. **Document it.** The project's README or spec must state explicitly which
   data uses local time as primary, and the *domain* reason why — not developer
   or user convenience.
2. **Keep the default everywhere else.** General-purpose machinery within the
   same project — logs, commit timestamps, internal cross-system APIs,
   telemetry — still follows the §14.2 UTC Z default. The exception covers the
   domain data itself, not the whole project.
3. **Preserve UTC derivability.** Store or compute the IANA timezone (e.g.,
   `Africa/Cairo`) alongside the local value, so a UTC instant remains derivable
   for interchange, comparison, and storage portability.
4. **This is an exception, not an escape hatch.** "Local time is more
   convenient" or "our users are mostly in one timezone" do not qualify — the
   domain itself must be inherently local-time-bound.

### §14.3 — Local Time as Optional Companion

**For projects under the UTC Z default** (§14.2), local time expressed as a UTC
offset is permitted as an **optional companion** to the UTC Z primary value — in
human-facing display, in API responses (as an additional field, never replacing
the UTC Z field), and in stored records where timezone context aids human
readers. The UTC Z value is always present and always authoritative; the
local-time companion is supplemental only. (A project operating under the
§14.2.1 domain exception inverts these roles for its domain data — local time is
primary there, with UTC kept derivable rather than displayed as authoritative.)

- The `--absolute-time` flag (defined in `spacecraft-cli-standard` §3) disables
  relative-time rendering but always renders as UTC, not local time.
- If a future CLI wants to show local time in human mode, it MUST:
  1. Accept a `--tz <IANA-zone>` flag (e.g., `--tz Africa/Cairo`).
  2. Render local time only to stdout in human mode — never in `--json` output.
  3. Always include the UTC value alongside the local rendering.
  4. Never persist or transmit the local-time rendering.
- JSON/machine output (`--format json/jsonl/yaml/csv`) MUST always use UTC + `Z`.

### §14.4 — Duration Format

Durations follow ISO 8601 duration notation:

| Format   | Example   | Meaning             |
|----------|-----------|---------------------|
| `PTnHnMnS` | `PT1H30M` | 1 hour 30 minutes |
| `PnD`    | `P7D`     | 7 days              |
| `PnYnM`  | `P1Y6M`   | 1 year 6 months     |

Prose forms like "1h 30m", "90 minutes", "1.5 hours" are **forbidden** in
machine-readable output. They are acceptable in `--help` text only.

### §14.5 — Rust Implementation Guidance

When writing Rust code that handles time:

| Concern | Rule |
|---------|------|
| Crate choice | Use `jiff` (preferred) or `chrono` — never `time` 0.1.x |
| UTC type | `jiff::Timestamp` or `chrono::DateTime<chrono::Utc>` for all stored values |
| Local type | `chrono::Local` and `jiff::Zoned` (with non-UTC zone) are **forbidden** in serialized output |
| Serialization | Always serialize as `"2026-03-08T14:30:00Z"` (string, ISO 8601, `Z` suffix) |
| `serde` | Use `#[serde(with = "...")]` or a newtype that enforces UTC on deserialization |
| `SystemTime` | Acceptable for internal durations; convert to UTC ISO 8601 string before any output |
| No `NaiveDateTime` in output | `chrono::NaiveDateTime` has no timezone — forbidden in any serialized or logged value |

---

## §15 — Attribution, Maintainer & Contact

**Maintainer:** Mohamed Hammad
**Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** Copyright (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://SpacecraftSoftware.org/](https://SpacecraftSoftware.org/)

### §15.1 — Project Pages

Each Spacecraft Software project has a dedicated subdomain following the pattern
`https://<ProjectName>.SpacecraftSoftware.org/`. Use the project-specific URL in all
project-level outputs; use `https://SpacecraftSoftware.org/` only for umbrella references.

| Project                    | URL                                              |
|----------------------------|--------------------------------------------------|
| Spacecraft Software (main) | https://SpacecraftSoftware.org/                  |
| The Steelbore Standard     | https://Standard.SpacecraftSoftware.org/         |
| Aetheric                   | https://Aetheric.SpacecraftSoftware.org/         |
| Gitway                     | https://Gitway.SpacecraftSoftware.org/           |
| Ferrocast                  | https://Ferrocast.SpacecraftSoftware.org/        |
| Caliper                    | https://Caliper.SpacecraftSoftware.org/          |
| Craton                     | https://Craton.SpacecraftSoftware.org/           |
| Ironway                    | https://Ironway.SpacecraftSoftware.org/          |
| Zamak                      | https://Zamak.SpacecraftSoftware.org/            |
| Bravais                    | https://Bravais.SpacecraftSoftware.org/          |
| Mawaqit                    | https://Mawaqit.SpacecraftSoftware.org/          |
| Flux                       | https://Flux.SpacecraftSoftware.org/             |
| Anvil                      | https://Anvil.SpacecraftSoftware.org/            |
| Construct                  | https://Construct.SpacecraftSoftware.org/        |
| Ferrite_OS                 | https://Ferrite.SpacecraftSoftware.org/          |
| Forge                      | https://Forge.SpacecraftSoftware.org/            |
| Ginx                       | https://Ginx.SpacecraftSoftware.org/             |
| Loran                      | https://Loran.SpacecraftSoftware.org/            |
| Pearlite                   | https://Pearlite.SpacecraftSoftware.org/         |
| MCP Servers                | https://MCP-Servers.SpacecraftSoftware.org/      |
| Lode                       | https://Lode.SpacecraftSoftware.org/             |
| Sonde                      | https://Sonde.SpacecraftSoftware.org/            |
| Vault                      | https://Vault.SpacecraftSoftware.org/            |
| Docs                       | https://Docs.SpacecraftSoftware.org/             |

When a new project is created, add its subdomain to this table immediately.

### §15.2 — Mandatory Attribution in Project Outputs

Every Spacecraft Software product **must** surface the following attribution in at least one
of: `--help` output, `--version` output, README, or About/Info screen.

**Required attribution block:**
```
Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software  |  License: GPL-3.0-or-later
https://<ProjectName>.SpacecraftSoftware.org/
```
(The License line shows the project's own license — GPL-3.0-or-later or AGPL-3.0-or-later per §4.1.)

**Per-surface rules:**

| Surface           | Required content                                                        |
|-------------------|-------------------------------------------------------------------------|
| `--version`       | Maintainer name, project URL, copyright year                            |
| `--help`          | Project URL and maintainer name (at footer)                             |
| README            | "Maintainer" section: name, `Mohamed.Hammad@SpacecraftSoftware.org`, project URL |
| About / Info (GUI/TUI) | Maintainer name, project URL, copyright year                       |
| SPDX header       | REUSE two-tag header (§4.3): `SPDX-FileCopyrightText` + `SPDX-License-Identifier` (`GPL-3.0-or-later` or `AGPL-3.0-or-later`) |

**Specific rules:**
- The contact email is always `Mohamed.Hammad@SpacecraftSoftware.org` — never a personal
  domain, GitHub handle, or other address.
- The copyright year reflects the year of first release or current year, or a range
  (e.g., `2025-2026`) when a project spans multiple years.
- Link text for project pages must use the full URL as the display text or a clear
  label (e.g., `[Gitway](https://Gitway.SpacecraftSoftware.org/)`), never an opaque label.
- For CLI `--version` output in human mode, the footer line format is:
  ```
  Maintained by Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
  https://<ProjectName>.SpacecraftSoftware.org/
  ```
- For CLI `--version` output in JSON/machine mode, include in `metadata`:
  ```json
  "maintainer": "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>",
  "website": "https://<ProjectName>.SpacecraftSoftware.org/"
  ```
- **Email obfuscation in plain-text prose.** In plain-text prose contexts (README body,
  CONTRIBUTING.md, human-readable documentation) where the address is not a clickable
  link, `Mohamed.Hammad [at] SpacecraftSoftware.org` is permitted as a scraper-resistant
  form. `# Maintainer:` lines in PKGBUILDs and `SPDX-FileCopyrightText` headers **must**
  always use the full address — those formats are parsed by `makepkg`/`pkgcheck` and
  `reuse lint` respectively, and obfuscation breaks them.

### §15.3 — Third-Party Attribution

Spacecraft Software artifacts must give credit where credit is due. When a project
or skill **substantially builds on third-party work**, that credit appears
in a `CREDITS.md` at the artifact's root — `<project-root>/CREDITS.md` for
projects, `<skill-name>/CREDITS.md` for skills.

`CREDITS.md` is the inbound counterpart to §15.2's outbound attribution:
§15.2 tells consumers who maintains Spacecraft Software; §15.3 tells consumers whose
work Spacecraft Software stands on.

**Triggers** (any one obligates a `CREDITS.md`):

- Content adapted, derived, or copied verbatim from an external source
  under any license (permissive or copyleft).
- A library, framework, or specification whose ideas or implementation
  form a substantial conceptual basis for the artifact, beyond routine
  dependency use.
- Named prior art, research, or design work whose insights were borrowed.

**Not triggered by** (license metadata alone suffices):

- Routine package-manager dependencies whose `LICENSE` files are surfaced
  mechanically via Cargo, npm, pip, Nix, etc.
- Well-known standards and specifications (POSIX, RFC, ISO, GFM, ODF,
  OOXML) that the artifact conforms to but does not redistribute.
- Public-domain conventions and common idioms.

**Required content per credited work:**

| Field      | Required | Example                                          |
|------------|----------|--------------------------------------------------|
| Name       | Yes      | `Microsoft Pragmatic Rust Guidelines`            |
| Author(s)  | Yes      | `Microsoft Corporation`                          |
| License    | Yes      | `MIT License`                                    |
| Source URL | Yes      | `https://github.com/microsoft/rust-guidelines`   |
| Scope      | Yes      | One-line description of what was adapted/used    |

A skill MAY keep a deeper, scope-limited attribution file inside its
`references/` directory (typically `references/ATTRIBUTION.md`) when the
credit applies specifically to adapted reference content. The root
`CREDITS.md` remains canonical and should link down to any such deeper
file.

SPDX headers (§4) cover license compliance mechanically; `CREDITS.md` is
the human-readable narrative — who, what, and how the upstream work
shaped the Spacecraft Software artifact.

---

## §16 — Compliance Checklist (Audit Gate)

Before finalising **any** Spacecraft Software artifact, mentally verify:

- [ ] **§2** Aerospace/Sci-Fi/AI naming convention applied to all **new** identifiers; legacy (pre-v1.2) names preserved unless explicitly renamed
- [ ] **§3.1** Stability: memory safety (Rust, or ASLR+CFI documented); robust error handling, fault tolerance, and test-verified
- [ ] **§3.2** Performance: multi-core/multi-thread concurrency by default (or serial trade-off documented); concurrency designed-in; benchmarking before/after
- [ ] **§3.3** Hardened security; PQC readiness addressed
- [ ] **§4.1** License is `GPL-3.0-or-later` or `AGPL-3.0-or-later` (AGPL for network-facing; per §4.1)
- [ ] **§4.2** Upstream copyright notices, license texts, and `NOTICE`/`AUTHORS` preserved verbatim; upstream licenses shipped in `LICENSES/`
- [ ] **§4.3** REUSE-compliant: two-tag SPDX header (`SPDX-FileCopyrightText` + `SPDX-License-Identifier`) on every file (or `.license` sidecar / `REUSE.toml` entry); `LICENSES/` directory present; `reuse lint` passes
- [ ] **§5** Project Posture: README/NOTICE/CONTRIBUTING present; default personal-hobby stance applied; general-use carve-outs declared in project README
- [ ] **§5.5** Package distribution: `packaging/guix.scm`, `packaging/default.nix`, and `packaging/PKGBUILD` present, buildable, and carrying correct version + SHA-256 checksum (in each package manager's native format) before any release tag is pushed
- [ ] **§6.1** POSIX-compliant CLI/system tools
- [ ] **§7** Shell scripts are POSIX-compatible; Nushell/Ion native variants provided where shell-native idioms are required; no Bashisms in shared scripts
- [ ] **§8** Texinfo manual present for user-facing programs (`doc/<project>.texi`); builds to `.info`, `.html`, and `.pdf`; `install-info` hook present in all three package manifests (§5.5) — N/A for scripts and internal tooling
- [ ] **§9** PFA: no tracking, minimal permissions, local storage default
- [ ] **§10** CUA + Vim-like key bindings planned/implemented
- [ ] **§11** Spacecraft Software color palette used; Void Navy background mandatory; new apps expose colors via a named `Steelbore` theme (§11.1) — no bare hex literals in UI logic
- [ ] **§12** FOSS-licensed fonts only (Share Tech Mono / Inconsolata)
- [ ] **§13** Material Design UI/UX; WCAG 2.1 AA verified
- [ ] **§14** ISO 8601 dates; 24h time; UTC Z is the default primary timestamp (companion local time with UTC offset permitted, never a replacement) — unless the project filed the §14.2.1 domain exception for inherently local-time-bound data; ISO 8601 durations; metric units
- [ ] **§15** Attribution present: maintainer name (`Mohamed Hammad`), contact (`Mohamed.Hammad@SpacecraftSoftware.org`), and project URL in `--version` / README / About
- [ ] **§15.3** Third-party work credited in `CREDITS.md` at project/skill root when triggers apply; deeper `references/ATTRIBUTION.md` present where reference content is adapted from external sources
- [ ] **§6.3** All commits to Spacecraft Software Git remotes cryptographically signed with the `Mohamed.Hammad@SpacecraftSoftware.org` key and showing "Verified" on the hosting platform; rewrites preserve signatures; programmatic and assistant-driven commits signed too

If any item is not applicable to the current artifact type (e.g., color palette
for a pure Rust library), note it as N/A rather than silently skipping it.

---

## Skill Cross-References

| Task                                  | Load this skill                                    |
|---------------------------------------|----------------------------------------------------|
| Writing any Rust code                 | `microsoft-rust-guidelines`                        |
| Writing or reviewing shell scripts    | `spacecraft-cli-shell` + `spacecraft-cli-preference` |
| Generating DOCX / ODT / PDF documents | `spacecraft-document-format`                       |
| Authoring a Texinfo manual            | `spacecraft-standard` (§8)                         |
| Creating IDE / terminal themes        | `spacecraft-theme-factory`                         |
| All other Spacecraft Software work    | `spacecraft-standard`                              |

---

*— Built by Spacecraft Software —*
