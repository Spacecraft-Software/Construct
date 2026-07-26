#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
"""Generate the Perplexity-compatible `spacecraft-cli-preference` bundle.

Perplexity caps uploaded zips at 100 files; the canonical skill ships 115
(110 per-tool `references/*.md` files). This regenerates a consolidated bundle
that merges the per-tool files into a handful of category files — same nested
`spacecraft-cli-preference/…` layout Perplexity already accepts — while leaving
the canonical skill untouched.

The canonical skill under `../spacecraft-cli-preference/` is the single source
of truth. NEVER hand-edit the emitted zip or its category files — edit a
canonical `references/<tool>.md` and re-run this script:

    python3 perplexity-skills/build.py

`CATEGORY_MAP` below is the only thing to maintain: when a tool is added to or
removed from the canonical skill, update its entry here. The script asserts the
map covers exactly the canonical tool set and fails loudly otherwise.
"""
from __future__ import annotations

import io
import re
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SRC = HERE.parent / "spacecraft-cli-preference"
REFS = SRC / "references"
OUT_ZIP = HERE / "spacecraft-cli-preference.zip"

# Non-tool reference files: copied through verbatim, never consolidated.
PASSTHROUGH_REFS = ("ATTRIBUTION.md", "local-execution.md")

# category slug -> human title for the category file's top H1.
CATEGORY_TITLES = {
    "file-and-text": "File & Text",
    "disk-and-files": "Disk & Files",
    "process-and-monitoring": "Process & Monitoring",
    "shells-and-terminal": "Shells & Terminal",
    "text-editors": "Text Editors",
    "networking": "Networking",
    "network-config": "Network Configuration",
    "vcs-and-build": "Version Control & Build",
    "package-managers": "Package Managers",
    "security-encryption": "Security & Encryption",
    "multimedia": "Multimedia",
    "communication": "Communication",
    "boot-login-session": "Boot, Login & Session",
    "ai-agents": "AI Agents & Coding CLIs",
}

# category slug -> tools (each tool == the canonical references/<tool>.md basename).
# Single source of truth for both consolidation and the SKILL.md link rewrite.
CATEGORY_MAP = {
    "file-and-text": ["eza", "bat", "fd", "ripgrep", "sd", "delta", "tokei",
                      "jaq", "ouch", "uutils", "rustybox"],
    "disk-and-files": ["dust", "dua", "fclones", "kondo", "disktui", "gptman",
                       "yazi", "broot", "superfile"],
    "process-and-monitoring": ["procs", "bottom", "kmon", "macchina", "bandwhich"],
    "shells-and-terminal": ["nushell", "ion", "brush", "starship", "atuin",
                            "zellij", "zoxide", "t-rec"],
    "text-editors": ["helix", "rsvim", "amp", "msedit"],
    "networking": ["xh", "curl", "wget2", "dog", "gping", "trippy", "rustscan",
                   "sniffglue", "monolith", "lychee"],
    "network-config": ["impala", "iwd", "nmstate", "adguardvpn-cli"],
    "vcs-and-build": ["gitui", "jujutsu", "gitway", "cargo-update", "rustup",
                      "lorri", "cpx", "podman", "just"],
    "package-managers": ["omni", "zap", "am", "topgrade", "paru", "linutil",
                         "dotter", "nix", "flatpak", "brew", "guix"],
    "security-encryption": ["rage", "sequoia-chameleon", "sequoia", "rbw", "sudo-rs"],
    "multimedia": ["rav1e", "gifski", "oxipng", "viu", "mpv", "ffmpeg", "yt-dlp",
                   "ncspot", "termusic", "radio-browser"],
    "communication": ["matrix-commander", "iamb", "rumatui", "disrust", "rivetui"],
    "boot-login-session": ["lanzaboote", "greetd", "tuigreet", "lemurs", "xdpc",
                           "cosmic-session", "xremap"],
    "ai-agents": ["claude-code", "aichat", "gemini-cli", "codex", "gh-copilot",
                  "opencode", "minimax-cli", "grok-cli", "kilo", "kiro-cli",
                  "kimi-cli", "gws-cli"],
}

# Reproducible zip: fixed member timestamp so identical content -> identical bytes
# (no spurious git diff on re-run). Bump only if the layout itself changes.
ZIP_DATE = (2026, 1, 1, 0, 0, 0)

_FENCE = re.compile(r"^\s*(```|~~~)")
_HEADING = re.compile(r"^(#{1,6}) ")
_LINK = re.compile(r"references/([A-Za-z0-9._-]+)\.md")


def die(msg: str) -> "None":
    sys.stderr.write(f"build.py: {msg}\n")
    raise SystemExit(1)


def canonical_tools() -> set[str]:
    return {p.stem for p in REFS.glob("*.md")
            if p.name not in PASSTHROUGH_REFS}


def tool_to_category(cmap: dict[str, list[str]]) -> dict[str, str]:
    inv: dict[str, str] = {}
    for cat, tools in cmap.items():
        for t in tools:
            if t in inv:
                die(f"tool {t!r} listed in two categories: {inv[t]} and {cat}")
            inv[t] = cat
    return inv


def consolidate(cat: str, tools: list[str]) -> str:
    """Merge the category's per-tool files into one markdown document.

    Each tool becomes a bare `## <basename>` anchor (GFM slug == basename),
    the original `# <title>` H1 kept as a bold subtitle so aliases like
    `(rg)` survive, and its body headings demoted one level — but only
    outside fenced code blocks, so `# ~/.gitconfig` comments stay intact.
    """
    out = [f"# {CATEGORY_TITLES[cat]}", ""]
    for tool in sorted(tools):
        raw = (REFS / f"{tool}.md").read_text(encoding="utf-8").splitlines()
        if not raw or not raw[0].startswith("# "):
            die(f"{tool}.md: expected a '# <title>' H1 on line 1")
        title = raw[0][2:].strip()
        out.append(f"## {tool}")
        out.append("")
        out.append(f"**{title}**")
        out.append("")
        in_fence = False
        for line in raw[1:]:
            if _FENCE.match(line):
                in_fence = not in_fence
                out.append(line)
                continue
            if not in_fence and _HEADING.match(line):
                out.append("#" + line)  # demote one level
            else:
                out.append(line)
        out.append("")
    return "\n".join(out).rstrip("\n") + "\n"


def rewrite_skill(text: str, inv: dict[str, str]) -> str:
    def repl(m: "re.Match[str]") -> str:
        token = m.group(1)
        cat = inv.get(token)
        if cat is None:
            return m.group(0)  # local-execution, ATTRIBUTION, <tool> placeholder
        return f"references/{cat}.md#{token}"
    return _LINK.sub(repl, text)


def self_check(skill_text: str, category_docs: dict[str, str]) -> "None":
    """Every rewritten references/<cat>.md#<anchor> must resolve to a
    `## <anchor>` heading in that category doc, else fail the build."""
    anchors = {cat: {ln[3:] for ln in doc.splitlines() if ln.startswith("## ")}
               for cat, doc in category_docs.items()}
    missing = []
    for cat, anchor in re.findall(r"references/([a-z0-9-]+)\.md#([A-Za-z0-9._-]+)",
                                  skill_text):
        if cat not in anchors:
            missing.append(f"{cat}.md (no such category file) #{anchor}")
        elif anchor not in anchors[cat]:
            missing.append(f"{cat}.md#{anchor} (no '## {anchor}' heading)")
    if missing:
        die("dangling anchors after rewrite:\n  " + "\n  ".join(sorted(set(missing))))


def main() -> "None":
    if not REFS.is_dir():
        die(f"canonical references dir not found: {REFS}")

    cmap_tools = {t for tools in CATEGORY_MAP.values() for t in tools}
    canon = canonical_tools()
    if cmap_tools != canon:
        extra = sorted(cmap_tools - canon)
        missing = sorted(canon - cmap_tools)
        die("CATEGORY_MAP is out of sync with the canonical skill.\n"
            f"  in map but not on disk: {extra}\n"
            f"  on disk but not mapped: {missing}\n"
            "  Edit CATEGORY_MAP to match, then re-run.")

    inv = tool_to_category(CATEGORY_MAP)
    category_docs = {cat: consolidate(cat, tools)
                     for cat, tools in CATEGORY_MAP.items()}

    skill_text = rewrite_skill((SRC / "SKILL.md").read_text(encoding="utf-8"), inv)
    self_check(skill_text, category_docs)

    # Assemble the members (arcname -> bytes) in the nested layout.
    members: dict[str, bytes] = {
        "spacecraft-cli-preference/SKILL.md": skill_text.encode("utf-8"),
        "spacecraft-cli-preference/CREDITS.md":
            (SRC / "CREDITS.md").read_bytes(),
    }
    for name in PASSTHROUGH_REFS:
        members[f"spacecraft-cli-preference/references/{name}"] = \
            (REFS / name).read_bytes()
    for cat, doc in category_docs.items():
        members[f"spacecraft-cli-preference/references/{cat}.md"] = \
            doc.encode("utf-8")

    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as zf:
        for arc in sorted(members):
            zi = zipfile.ZipInfo(arc, date_time=ZIP_DATE)
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = 0o644 << 16
            zf.writestr(zi, members[arc])
    OUT_ZIP.write_bytes(buf.getvalue())

    print(f"wrote {OUT_ZIP.relative_to(HERE.parent)} — "
          f"{len(members)} entries, {len(CATEGORY_MAP)} category files, "
          f"{len(canon)} tools consolidated.")


if __name__ == "__main__":
    main()
