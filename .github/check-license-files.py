#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Enforce Steelbore Standard §4.3 license-file naming and §5.6 license carriage.

For every shipped skill this checks that:

  * the license file exists, is named per §4.3, and is a regular file
    (never a symlink -- a parent-relative link dangles once the directory
    is packaged alone, which is what bundling and the flake's per-skill
    ``cp -r`` do);
  * its bytes are identical to the canonical text in ``LICENSES/``, which
    is what keeps the copies one maintained text rather than the two
    independent copies §4.3 forbids;
  * the ``.zip`` and ``.skill`` bundles actually ship it -- a bundle is a
    distribution in its own right and the repo-root LICENSE never reaches
    the consumer.

Which license a skill is under comes from ``REUSE.toml``, so this gate has
no second source of truth to drift against.  Third-party vendored trees
(``android-skills/``, ``orca-skills/``) are exempt: §4.2 preserves upstream
layout verbatim, including upstream's own license filenames.
"""

from __future__ import annotations

import sys
import tomllib
import zipfile
from fnmatch import fnmatch
from pathlib import Path

VENDORED = ("android-skills", "orca-skills")
# §4.3: these names are non-compliant inside a skill directory.
FORBIDDEN = ("LICENSE.md", "LICENSE.txt", "LICENCE", "LICENCE.md", "COPYING")


def spdx_for(path: str, annotations: list[dict]) -> str:
    """Last matching REUSE.toml annotation wins, mirroring reuse's own precedence."""
    found = "?"
    for ann in annotations:
        paths = ann["path"]
        for pat in paths if isinstance(paths, list) else [paths]:
            if fnmatch(path, pat) or fnmatch(path, pat.rstrip("*") + "**"):
                found = ann["SPDX-License-Identifier"]
    return found


def expected_files(spdx: str) -> dict[str, str]:
    """Map an SPDX expression to {filename: spdx-id} per §4.3."""
    ids = [p.strip() for p in spdx.split(" OR ")]
    if len(ids) == 1:
        return {"LICENSE": ids[0]}
    # More than one license: one LICENSE.<TAG> per text, never concatenated.
    return {f"LICENSE.{i.split('-')[0]}": i for i in ids}


def main(argv: list[str]) -> int:
    repo = Path(argv[1] if len(argv) > 1 else ".").resolve()
    annotations = tomllib.loads((repo / "REUSE.toml").read_text())["annotations"]
    problems: list[str] = []

    skills = [(p.parent, p.parent.name) for p in sorted(repo.glob("*/SKILL.md"))
              if (repo / f"{p.parent.name}.zip").exists()]
    skills += [(p.parent, p.parent.name) for p in sorted(repo.glob("grok-skills/*/SKILL.md"))]

    for skill_dir, name in skills:
        rel = skill_dir.relative_to(repo).as_posix()
        if rel.split("/")[0] in VENDORED:
            continue
        spdx = spdx_for(f"{rel}/SKILL.md", annotations)
        wanted = expected_files(spdx)

        for fname, spdx_id in wanted.items():
            target = skill_dir / fname
            canonical = repo / "LICENSES" / f"{spdx_id}.txt"
            if not canonical.exists():
                problems.append(f"{rel}: no canonical text at LICENSES/{spdx_id}.txt")
                continue
            if not target.exists():
                problems.append(f"{rel}: missing {fname} (§5.6 license carriage; skill is {spdx})")
                continue
            if target.is_symlink():
                problems.append(f"{rel}/{fname}: is a symlink; §5.6 requires a regular file")
                continue
            if target.read_bytes() != canonical.read_bytes():
                problems.append(
                    f"{rel}/{fname}: not byte-identical to LICENSES/{spdx_id}.txt (§5.6)")

        for bad in FORBIDDEN:
            if (skill_dir / bad).exists():
                problems.append(f"{rel}/{bad}: non-compliant license filename (§4.3)")
        for stray in sorted(skill_dir.glob("LICENSE-*")):
            problems.append(
                f"{rel}/{stray.name}: §4.3 uses LICENSE.<TAG>, not LICENSE-<TAG>")

        # §5.6: the bundle is the distribution -- it must carry the text.
        flat = rel.startswith("grok-skills/")
        for ext in (".zip", ".skill"):
            bundle = (skill_dir.parent / f"{name}{ext}") if flat else (repo / f"{name}{ext}")
            if not bundle.exists():
                problems.append(f"{rel}: no {name}{ext} bundle")
                continue
            with zipfile.ZipFile(bundle) as zf:
                names = set(zf.namelist())
                for fname in wanted:
                    entry = fname if flat else f"{name}/{fname}"
                    if entry not in names:
                        problems.append(f"{name}{ext}: does not ship {entry} (§5.6)")

    for p in problems:
        print(p)
    print(f"\n{len(problems)} license-file problem(s). Checked {len(skills)} skills.")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
