#!/bin/sh
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Spacecraft Software Texinfo build helper. POSIX sh (no bashisms) so it runs
# under dash/ash/Bash/Brush alike. Builds every output format from one source
# and applies the house HTML theme + A4 PDF geometry.
#
# Usage:   ./build.sh project.texi [outdir]
# Output:  <outdir>/{project.info, project.html, project.pdf, project.txt,
#                     project.docbook, project.epub}
#
# Toolchain: needs `texi2any` (a.k.a. makeinfo) and, for PDF, `texi2pdf` + TeX.
# Provision ephemerally per spacecraft-missing-pkg, e.g.:
#     guix shell texinfo texlive -- ./build.sh project.texi
#     nix shell nixpkgs#texinfo nixpkgs#texlive -- ./build.sh project.texi

set -eu

SRC="${1:?usage: build.sh project.texi [outdir]}"
OUT="${2:-build}"
BASE="$(basename "$SRC" .texi)"
CSS="$(dirname "$0")/spacecraft.css"

mkdir -p "$OUT"

echo "→ Info"
texi2any --info --no-split --output="$OUT/$BASE.info" "$SRC"

echo "→ HTML (house-themed)"
if [ -f "$CSS" ]; then
  texi2any --html --no-split --css-include="$CSS" --output="$OUT/$BASE.html" "$SRC"
else
  echo "  ! $CSS not found — building HTML unthemed" >&2
  texi2any --html --no-split --output="$OUT/$BASE.html" "$SRC"
fi

echo "→ plain text"
texi2any --plaintext --no-split --output="$OUT/$BASE.txt" "$SRC"

echo "→ DocBook"
texi2any --docbook --output="$OUT/$BASE.docbook" "$SRC"

echo "→ EPUB"
texi2any --epub3 --output="$OUT/$BASE.epub" "$SRC" || \
  echo "  ! EPUB build skipped (texi2any EPUB support unavailable)" >&2

echo "→ PDF (A4)"
if command -v texi2pdf >/dev/null 2>&1; then
  texi2pdf --texinfo=@afourpaper --quiet --clean -o "$OUT/$BASE.pdf" "$SRC" || \
    echo "  ! PDF build failed (is a TeX distribution installed?)" >&2
else
  echo "  ! texi2pdf not found — skipping PDF (install TeX, e.g. texlive)" >&2
fi

echo "✓ done → $OUT/"

# --- Nushell variant (the user's preferred shell) -------------------------
# The above is POSIX sh for portability. In Nushell:
#
#   def build [src: string, outdir: string = "build"] {
#     let base = ($src | path basename | str replace '.texi' '')
#     mkdir $outdir
#     texi2any --info     --no-split --output $"($outdir)/($base).info"     $src
#     texi2any --html     --no-split --css-include spacecraft.css --output $"($outdir)/($base).html" $src
#     texi2any --plaintext --no-split --output $"($outdir)/($base).txt"     $src
#     texi2pdf --texinfo=@afourpaper --clean -o $"($outdir)/($base).pdf"    $src
#   }
