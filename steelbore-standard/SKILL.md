---
name: steelbore-standard
description: >
  The authoritative compliance reference for ALL work on Steelbore projects and subprojects
  (Zamak, Lattice, Ferrocast, Craton, Ironway, Caliper, Mawaqit, and any future projects).
  ALWAYS load this skill before writing code, documentation, specifications, architecture
  decisions, UI designs, naming choices, or any other artifact for a Steelbore project —
  even if the user doesn't explicitly mention the Standard. If the user mentions "Steelbore",
  a Steelbore subproject name, or asks you to work on anything in the Steelbore ecosystem,
  consult this skill immediately. It encodes The Steelbore Standard v1.0 so you never need
  to ask for it or have it attached to a prompt again.
license: GPL-3.0-or-later
---
 
# The Steelbore Standard — Compliance Reference
 
**Version:** 1.0 | **Date:** 2026-03-08 | **Author:** Mohamed Hammad
**Copyright:** (c) 2026 Mohamed Hammad | **License:** GPL-3.0-or-later
 
This skill encodes The Steelbore Standard in full. Apply every applicable section
to any artifact you produce for a Steelbore project. The 12-point compliance checklist
in §13 is your audit gate — run through it mentally before finalising any output.
 
---
 
## §2 — Metallurgical Naming Convention
 
All project codenames, module identifiers, and public-facing component names **must**
come from the domain of metallurgy, materials science, or industrial forging.
 
| Category  | Examples                        | Domain                  |
|-----------|---------------------------------|-------------------------|
| Projects  | ZAMAK, Lattice, Steelbore       | Alloys, Crystal Structs |
| Modules   | Crucible, Anvil, Temper         | Forging Tools           |
| Utilities | Quench, Flux, Smelt             | Metallurgical Processes |
| Releases  | Ingot, Billet, Bloom            | Cast Forms              |
 
**Known project registry:**
- `Zamak` — Rust bootloader (Limine rewrite)
- `Lattice` — NixOS flake configuration
- `Ferrocast` — Rust PowerShell rewrite (16-crate workspace)
- `Craton` — Rust universal package manager
- `Ironway` — Rust OpenTTD rewrite
- `Caliper` — Rust raster-to-vector tracing engine (CLI+TUI)
- `Mawaqit` — Islamic prayer times app (Flutter + Rust CLI + libmawaqit)
 
When proposing new names (modules, utilities, releases), always draw from metallurgy.
Reject any proposed name that doesn't fit this convention.
 
---
 
## §3 — Priority Hierarchy (Non-Negotiable Order)
 
A higher-numbered priority **may never compromise** a lower-numbered one.
 
### §3.1 — Priority 1: Memory Safety
- **Preferred language: Rust** — governed by the Steelbore Rust Guidelines.
  → Always load the `rust-guidelines` skill before writing any Rust code.
- When Rust is not viable (Flutter/Dart, Zig, etc.), **mandatory mitigations**:
  - **ASLR** (Address Space Layout Randomization) on all compiled binaries
  - **CFI** (Control-Flow Integrity) wherever the toolchain supports it
- Memory-Safe Languages (MSLs) are always preferred. If an MSL alternative exists,
  it must be chosen unless a documented technical exemption is filed.
 
### §3.2 — Priority 2: Performance
- Concurrency must be **designed-in from the start**, never bolted on retroactively.
- Release builds must use CPU-optimized flags: `-march=native`, LTO, PGO where applicable.
- Benchmarking is **mandatory** before and after any optimization work; regressions must
  be documented and justified.
 
### §3.3 — Priority 3: Hardened Security
- Kernel hardening (XanMod, grsecurity profiles) where applicable.
- Sandboxing and privilege separation for all network-facing components.
- **Post-Quantum Cryptography (PQC) readiness**: all crypto subsystems must support
  PQC migration paths. Use hybrid schemes (classical + PQC candidate) where library
  support exists. Adopt NIST-finalized PQC standards within one major release cycle.
  - Current targets: ML-KEM-768, ML-DSA-65 (as used in Ferrocast)
- Dependency auditing: `cargo-audit` or equivalent before any third-party crate inclusion.
 
**Cardinal Rule:** Any optimization that weakens memory safety or security hardening
**must be rejected**, no exceptions.
 
---
 
## §4 — Licensing & Compliance
 
- **License:** GNU General Public License, version 3 or later (`GPL-3.0-or-later`)
- No proprietary, closed-source, or permissive-only exceptions for core project code.
 
### SPDX Headers (mandatory on software source code files only)
```
// SPDX-License-Identifier: GPL-3.0-or-later
```
**Software source code files** (`.rs`, `.ts`, `.js`, `.py`, `.sh`, `.ps1`, `.go`, etc.)
and project manifests (`Cargo.toml`, `package.json`, `flake.nix`, etc.) must include
the SPDX header/expression.

**Document files** (`.odf`, `.xlsx`, `.docx`, PDF, etc.) are **exempt** from
SPDX header requirements; the license is stated in the project root.
 
**When writing or reviewing any software source file**, check that the SPDX header is present.
When generating new source files, always include it.
 
---
 
## §5 — Platform & Systems Requirements
 
### §5.1 — POSIX Compliance
All CLI tools, daemons, and system utilities must be **POSIX-compliant**.
Platform-specific extensions go behind feature flags and must not be required
for core functionality.
 
### §5.2 — Post-Quantum Cryptography
Crypto subsystems must have migration paths to post-quantum algorithms.
Current implementations should use hybrid schemes where library support exists.
 
---
 
## §6 — Privacy-Friendly Application (PFA) Policy
 
Every Steelbore application must satisfy **all three** PFA requirements:
 
| Requirement        | Rule                                                                     |
|--------------------|--------------------------------------------------------------------------|
| No Tracking/No Ads | Zero advertising, tracking, analytics SDKs, or telemetry beacons        |
| Minimal Permissions| Only essential permissions; requested lazily at point of use, never eagerly |
| Local Storage      | User data stored locally by default; sync is strictly opt-in, E2E encrypted |
 
When reviewing or designing any feature that touches data handling, permissions,
or networking, verify all three PFA requirements are met.
 
---
 
## §7 — Key Bindings
 
All interactive applications must support **both**:
 
| Scheme    | Requirement                                                              |
|-----------|--------------------------------------------------------------------------|
| **CUA**   | Standard bindings (Ctrl+C/X/V/Z/S) must work in all text input contexts |
| **Vim**   | Modal editing layer (Normal / Insert / Visual mode) as opt-in feature.  |
|           | Minimum: hjkl navigation where full Vim layer is impractical            |
 
---
 
## §8 — Steelbore Color Palette (WCAG-Compliant)
 
The **only** permitted colors for Steelbore interfaces and documents:
 
| Token          | Hex       | RGB              | Role                        |
|----------------|-----------|------------------|-----------------------------|
| Void Navy      | `#000027` | RGB(0, 0, 39)    | **Background / Canvas**     |
| Molten Amber   | `#D98E32` | RGB(217, 142, 50)| Primary Text / Active Readout |
| Steel Blue     | `#4B7EB0` | RGB(75, 126, 176)| Primary Accent / Structural |
| Radium Green   | `#50FA7B` | RGB(80, 250, 123)| Success / Safe Status       |
| Red Oxide      | `#FF5C5C` | RGB(255, 92, 92) | Warning / Error Status      |
| Liquid Coolant | `#8BE9FD` | RGB(139, 233, 253)| Info / Links               |
 
**`#000027` (Void Navy) is the mandatory background for ALL Steelbore surfaces.**
No alternative background is permitted. This is non-negotiable.
 
For document/file generation → load the `steelbore-document-format` skill.
For IDE/terminal themes → load the `steelbore-theme-factory` skill.
 
> ⚠️ Note: The `steelbore-brand-guidelines` skill contains **outdated color and font
> data** that conflicts with the Standard. The values in §8 and §9 of this skill are
> authoritative. Do not use the brand-guidelines skill for color or font reference.
 
---
 
## §9 — Typography (FOSS-Licensed Fonts Only)
 
Acceptable font licenses: **OFL, Apache 2.0, Ubuntu Font License, CC0-1.0**
 
| Context        | Font              | License |
|----------------|-------------------|---------|
| Headings       | Share Tech Mono   | OFL     |
| Body / Code    | Inconsolata       | OFL     |
| Fallback       | monospace (system)| N/A     |
 
Never use proprietary fonts. When suggesting or using fonts in any Steelbore artifact,
verify they are available on Google Fonts or another FOSS-licensed repository.
 
---
 
## §10 — UI/UX Design System
 
- **Material Design** is the required component system for all graphical applications.
  Theme Material components with the §8 color palette.
- **WCAG 2.1 Level AA** contrast is the minimum for all color pairings.
  Any new color additions must be WCAG-verified before adoption.
- **Accessibility**: screen readers, keyboard-only navigation, and system accessibility
  preferences (reduced motion, high contrast) must all be respected.
 
---
 
## §11 — Date, Time & Units
 
| Concern     | Rule                                              | Example          |
|-------------|---------------------------------------------------|------------------|
| Date format | ISO 8601 only: `YYYY-MM-DD`                       | `2026-03-08`     |
| Time format | 24-hour only: `HH:MM:SS` — AM/PM never permitted  | `14:30:00`       |
| Timezone    | UTC for all timestamps, logs, commits, API responses | `2026-03-08T14:30:00Z` |
| Units       | Metric (SI) primary; imperial in parentheses only if locale requires | `100 km (62 mi)` |
 
Apply these conventions to all generated code, documentation, comments, and any
user-facing strings. Never output AM/PM time, non-ISO dates, or imperial-primary units.
 
---
 
## §13 — Compliance Checklist (Audit Gate)
 
Before finalising **any** Steelbore artifact, mentally verify:
 
- [ ] **§2** Metallurgical naming convention applied to all new identifiers
- [ ] **§3.1** Memory safety: Rust used, or ASLR+CFI mitigations documented
- [ ] **§3.2** Concurrency designed-in; benchmarking planned/documented
- [ ] **§3.3** Hardened security; PQC readiness addressed
- [ ] **§4** `GPL-3.0-or-later` license; SPDX headers on software source code files (excluding documents)
- [ ] **§5.1** POSIX-compliant CLI/system tools
- [ ] **§6** PFA: no tracking, minimal permissions, local storage default
- [ ] **§7** CUA + Vim-like key bindings planned/implemented
- [ ] **§8** Steelbore color palette used; Void Navy background mandatory
- [ ] **§9** FOSS-licensed fonts only (Share Tech Mono / Inconsolata)
- [ ] **§10** Material Design UI/UX; WCAG 2.1 AA verified
- [ ] **§11** ISO 8601 dates, 24h time, UTC timestamps, metric units
 
If any item is not applicable to the current artifact type (e.g., color palette
for a pure Rust library), note it as N/A rather than silently skipping it.
 
---
 
## Skill Cross-References
 
| Task                                 | Load this skill             |
|--------------------------------------|-----------------------------|
| Writing any Rust code                | `rust-guidelines`           |
| Generating DOCX / PDF documents      | `steelbore-document-format` |
| Creating IDE / terminal themes       | `steelbore-theme-factory`   |
| This skill (all other Steelbore work)| `steelbore-standard` ← you are here |
 
---
 
*─── Forged in Steelbore ───*
 