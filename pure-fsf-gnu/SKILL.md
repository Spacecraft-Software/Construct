---
name: pure-fsf-gnu
description: >-
  Forces full GNU/FSF compliance: the strict posture for software meant to become an
  official GNU package, be upstreamed to GNU/FSF, or hosted on Savannah — not mere GNU
  style. Use ONLY when the artifact sheds Spacecraft/Steelbore identity for GNU: strictly
  off-GitHub, no nonfree JavaScript, no nonfree promotion, FSF copyright assignment, GFDL
  manuals, and no Steelbore branding (palette, theme, fonts) or aerospace codenames.
  Triggers: "pure FSF", "official GNU package", "FSF copyright assignment", "Savannah",
  "upstream to GNU", "drop branding for GNU". Composes gnu-coding-standards for the
  technical conventions and adds the binding identity/free-software posture. For GNU
  conventions WITHOUT shedding Steelbore identity (a Spacecraft project staying on
  GitHub), use gnu-coding-standards (compatible mode), not this.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# pure-fsf-gnu — the GNU-compliant (full-FSF) posture

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

This skill is the **GNU-compliant posture**: the artifact is (or is becoming) a real GNU
package — an official GNU program, something upstreamed to GNU/FSF, hosted on Savannah, or
audited as fully GNU/FSF-conformant. Here GNU compliance is a **binding external contract**:
a half-compliant GNU package is not GNU-acceptable, so on every dimension GNU governs, GNU
wins — including over the Steelbore Standard's identity rules. This is deliberately a
*narrow, strong* posture; for the common case, see "When this does not apply" below.

## When this applies — and when it does not

**Applies** only when the work must be GNU-compliant *as its actual goal* and is therefore
**shedding Spacecraft/Steelbore identity** to satisfy GNU: off GitHub, no vendor branding,
FSF copyright, GFDL manuals.

**Does NOT apply** to a Spacecraft/Steelbore project that merely interoperates with GNU,
adopts GNU conventions, or "wants to be GNU-friendly" while staying on GitHub under Steelbore
identity. That is **GNU-*compatible*** — `spacecraft-standard` stays primary and you load
`gnu-coding-standards` for the conventions that aid interop. **The differentiator is
identity-stripping:** invoke this skill only when the artifact gives up Steelbore identity
for GNU. If in doubt, you are almost certainly in compatible mode — use `gnu-coding-standards`.

## Load `gnu-coding-standards` first (compose, do not duplicate)

This skill is the **posture / identity layer only**. All GNU *technical* conventions —
the error-message grammar, the exact `--version` / `--help` contract, long options, i18n,
Texinfo mechanics, ChangeLog/NEWS, the `configure`/Makefile release process, the robustness
goals, and the per-language realizations (`c.md`, `rust.md`, `guile.md`, `go.md`,
`python.md`) — live in **`gnu-coding-standards`**. Load it and follow it; this skill does not
restate it. Language-craft skills (`microsoft-rust-guidelines`, `spacecraft-guile-guidelines`,
…) compose under `gnu-coding-standards` exactly as usual.

Under this posture, `gnu-coding-standards`' "Keeping free software free" section is **binding
in full** — not advisory.

## Binding FSF mandates (full fidelity)

- **License.** `GPL-3.0-or-later` for programs (`AGPL-3.0-or-later` if network-facing); ship
  the full text in `COPYING`; every source file carries the GPL header (SPDX identifiers are
  fine *alongside* the header, not instead of it).
- **Copyright.** Assign to the **FSF** where the package is or seeks to be FSF-held; record
  every contributor; nontrivial contributions need signed assignment papers *before* merge.
- **Strictly off-GitHub.** Host on GNU infrastructure (**Savannah**) or another free forge.
  Do **not** link to pages that require nonfree JavaScript to use — this includes
  `github.com` — quote excerpts instead of linking.
- **No nonfree.** Do not recommend, promote, depend on, or grant legitimacy to nonfree
  software, services, documentation, fonts, or codecs.
- **Manuals.** Texinfo, licensed **GFDL-1.3-or-later**. Plug-ins must assert a GPL-compatible
  license programmatically (`plugin_is_GPL_compatible` or the host-language equivalent).

## Overrides of the Steelbore Standard (identity yields to GNU)

For a GNU-compliant artifact, GNU wins on every identity dimension:

| Dimension | Steelbore default | Under this posture |
|---|---|---|
| Brand (§11/§12 + `spacecraft-brand-guidelines`) | Void Navy `#000027`, `Steelbore` theme, Share Tech Mono / Inconsolata | **None** — no vendor palette, theme, or fonts |
| Project naming (§2) | Aerospace / sci-fi codename | **GNU** package naming |
| Attribution (§15) | "Mohamed Hammad & Spacecraft Software", `@SpacecraftSoftware.org` | GNU `AUTHORS` + FSF/contributor copyright |
| Doc license (§4.1.1) | `CC-BY-SA-4.0` | **`GFDL-1.3-or-later`** (the Standard already permits this, §8.5) |
| Hosting | GitHub | **Off-GitHub** (Savannah / free forge) |

## Steelbore rules that still apply and STACK (GNU-silent / compatible)

These are invisible to GNU or fully compatible with it, so there is nothing to override —
keep them; they make the GNU artifact better:

- **§6.3 — signed & "Verified" commits.** Internal git hygiene GNU does not speak to. Keep.
- **§14 — ISO-8601 UTC by default.** GNU uses ISO dates; identical intent. Keep.
- **§3.3 — Security by Design.** A subset of GNU's robustness goals. Keep.
- **`--json` / schema / structured output.** Layer it **on top of** GNU's `--version` /
  `--help` contract — additive, never replacing the GNU-mandated text or exit behavior.

## Precedence (the relationship)

Resolve tensions by these rules, not by whichever skill spoke last:

- **One posture governs an artifact.** `gnu-coding-standards` supplies the GNU **technical
  base** under any posture; language skills compose under it.
- **This skill is supreme for a GNU-compliant artifact.** Where it and `spacecraft-standard`
  collide on identity, **this wins**; `spacecraft-standard` contributes only its
  GNU-silent/compatible clauses (above), which stack.
- **The lighter posture is GNU-*compatible*** — `gnu-coding-standards` + `spacecraft-standard`,
  Steelbore primary, GNU conventions adopted for interop. Use *that*, not this, unless the
  artifact is shedding Steelbore identity to be GNU.
- See `spacecraft-standard` (the GNU-compliance posture clause) and `gnu-coding-standards`
  ("Two GNU postures") for the reciprocal statements.
