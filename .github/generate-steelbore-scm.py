#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Generate assets/steelbore.scm from assets/steelbore.toml.

Why this exists
---------------
`steelbore.toml` is the canonical machine-readable contract for Standard §11,
and §11.4 says its values are *read, never retyped*. Nix honours that directly
— `builtins.fromTOML` is built in, so Bravais parses the file itself. Guile has
no TOML reader in the standard distribution, and Guix evaluates system
configuration with plain Guile, so the Ginx side of Standard §11.6.5 had two
options: vendor a TOML parser into the system-configuration evaluation path, or
mirror the file into a sexp.

This is the mirror. It is *generated*, never hand-edited, so "read, never
retyped" still holds: there remains exactly one place a hex value or a slug is
authored, and drift is a CI failure rather than a slow divergence nobody sees.

Usage
-----
    python3 .github/generate-steelbore-scm.py            # write the file
    python3 .github/generate-steelbore-scm.py --check     # CI: fail on drift
"""

from __future__ import annotations

import argparse
import pathlib
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parent.parent
TOML = REPO / "steelbore-color-palette" / "assets" / "steelbore.toml"
SCM = REPO / "steelbore-color-palette" / "assets" / "steelbore.scm"


def scm_string(value: str) -> str:
    """A Scheme string literal. Palette values are hex, slugs and paths — no
    control characters — but escape defensively rather than assume."""
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def alist(pairs: list[tuple[str, str]], indent: int) -> list[str]:
    pad = " " * indent
    return [f"{pad}({scm_string(k)} . {scm_string(v)})" for k, v in pairs]


def render(data: dict) -> str:
    meta = data["meta"]
    out: list[str] = []
    w = out.append

    w("; SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>")
    w("; SPDX-License-Identifier: GPL-3.0-or-later")
    w(";")
    w("; GENERATED FROM steelbore.toml — DO NOT EDIT.")
    w(";")
    w("; Regenerate with:")
    w(";     python3 .github/generate-steelbore-scm.py")
    w("; CI fails if this file drifts from the TOML (--check).")
    w(";")
    w("; The Guile mirror of the canonical Steelbore palette contract. It exists")
    w("; because Guix evaluates system configuration in plain Guile, which has no")
    w("; TOML reader — so Standard §11.6.5's obligations on Steelbore OS (GNU Guix")
    w("; System / Ginx) would otherwise require retyping values that §11.4 says are")
    w("; read, never retyped. Editing this file instead of the TOML defeats the")
    w("; entire point: there would be two sources, and they would drift.")
    w(";")
    w(f"; Palette contract version {meta['version']} — {meta['standard']}.")
    w("")
    # Module name tracks the FILE name: Guile resolves (steelbore) to
    # steelbore.scm on the load path, and (steelbore palette) would demand
    # steelbore/palette.scm. Keeping the basename in step with steelbore.toml
    # matters more than a two-element module name.
    w("(define-module (steelbore)")
    w("  #:export (steelbore-meta")
    w("            steelbore-themes")
    w("            steelbore-resolution")
    w("            steelbore-theme-ref")
    w("            steelbore-polarity")
    w("            steelbore-counterpart))")
    w("")
    w(";;; Metadata (§11 [meta]).")
    w("(define steelbore-meta")
    w("  `((version . " + scm_string(meta["version"]) + ")")
    w("    (date . " + scm_string(meta["date"] if isinstance(meta["date"], str) else meta["date"].isoformat()) + ")")
    w("    (standard . " + scm_string(meta["standard"]) + ")")
    w("    (default-theme . " + scm_string(meta["default-theme"]) + ")")
    w("    (default-dark-theme . " + scm_string(meta["default-dark-theme"]) + ")")
    w("    (default-light-theme . " + scm_string(meta["default-light-theme"]) + ")")
    w("    (mono-theme . " + scm_string(meta["mono-theme"]) + ")")
    w("    (palette-family . " + f"({' '.join(scm_string(s) for s in meta['palette-family'])})" + ")")
    w("    (fidelity-palettes . " + f"({' '.join(scm_string(s) for s in meta['fidelity-palettes'])})" + ")")
    w("    ;; §11.6.1 — the themes every application MUST register.")
    w("    (registered-set . " + f"({' '.join(scm_string(s) for s in meta['registered-set'])})" + ")))")
    w("")
    w(";;; Every theme, as slug -> role alist. Role names are the §11.1 tokens")
    w(";;; verbatim; steelbore-mono binds 4-bit ANSI names rather than hex, and")
    w(";;; steelbore-classic binds only its legacy six roles (§11.2).")
    w("(define steelbore-themes")
    w("  `(")
    for slug in sorted(data["themes"]):
        theme = data["themes"][slug]
        roles = [(k, v) for k, v in theme.items() if isinstance(v, str)]
        w(f"    ({scm_string(slug)}")
        w("     .")
        w("     (")
        out.extend(alist(roles, 6))
        w("      ))")
    w("    ))")
    w("")
    w(";;; §11.6 — system theme resolution. Slugs and paths only; never a color.")
    res = data["resolution"]
    w("(define steelbore-resolution")
    w("  `((env-var . " + scm_string(res["env-var"]) + ")")
    w("    (system-file . " + scm_string(res["system-file"]) + ")")
    w("    (user-file . " + scm_string(res["user-file"]) + ")")
    w("    (registry . " + scm_string(res["registry"]) + ")")
    w("    (polarity")
    w("     .")
    w("     (")
    out.extend(alist(sorted(res["polarity"].items()), 6))
    w("      ))")
    w("    (pair")
    w("     .")
    w("     (")
    out.extend(alist(sorted(res["pair"].items()), 6))
    w("      ))))")
    w("")
    w(";;; Accessors. Each raises rather than returning #f for an unknown slug:")
    w(";;; §11.6.5 requires an OS to validate the slug at configuration-evaluation")
    w(";;; time, so an unknown theme fails the build rather than the boot.")
    w("(define (steelbore-theme-ref slug)")
    w('  "Return SLUG\'s role alist, or raise if it is not a registered theme."')
    w("  (or (assoc-ref steelbore-themes slug)")
    w("      (error \"steelbore: unknown theme slug (see steelbore.toml)\" slug)))")
    w("")
    w("(define (steelbore-polarity slug)")
    w('  "Return \\"light\\" or \\"dark\\" for SLUG\'s canvas (§11.6.2)."')
    w("  (or (assoc-ref (assoc-ref steelbore-resolution 'polarity) slug)")
    w("      (error \"steelbore: no polarity recorded for slug\" slug)))")
    w("")
    w("(define (steelbore-counterpart slug)")
    w('  "Return SLUG\'s counterpart in the other polarity (§11.6.2)."')
    w("  (or (assoc-ref (assoc-ref steelbore-resolution 'pair) slug)")
    w("      (error \"steelbore: no light/dark counterpart recorded for slug\" slug)))")
    w("")
    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit non-zero if the generated file is missing or stale",
    )
    args = ap.parse_args()

    with TOML.open("rb") as fh:
        data = tomllib.load(fh)

    for required in ("registered-set", "default-dark-theme", "default-light-theme", "mono-theme"):
        if required not in data["meta"]:
            print(
                f"{TOML.name}: [meta] is missing '{required}' — needs steelbore.toml >= 3.2.0 "
                "(Standard §11.6)",
                file=sys.stderr,
            )
            return 1
    if "resolution" not in data:
        print(
            f"{TOML.name}: no [resolution] table — needs steelbore.toml >= 3.2.0 (Standard §11.6)",
            file=sys.stderr,
        )
        return 1

    want = render(data)

    if args.check:
        if not SCM.exists():
            print(f"{SCM.name} does not exist — run: python3 {pathlib.Path(__file__).name}", file=sys.stderr)
            return 1
        if SCM.read_text() != want:
            print(
                f"{SCM.name} is stale — it no longer matches {TOML.name}.\n"
                f"Regenerate with: python3 .github/{pathlib.Path(__file__).name}",
                file=sys.stderr,
            )
            return 1
        print(f"{SCM.name} is in sync with {TOML.name}")
        return 0

    SCM.write_text(want)
    print(f"wrote {SCM.relative_to(REPO)} from {TOML.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
