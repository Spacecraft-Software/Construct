# Building Texinfo Output

`texi2any` (a.k.a. `makeinfo`) is the single translator for all non-TeX formats;
`texi2pdf`/`texi2dvi` drive TeX for print. They are the same project. This guide
covers provisioning, every output format, house theming, and a Makefile pattern.

## Provisioning the toolchain (spacecraft-missing-pkg)

The toolchain is often not installed. Provision it **ephemerally**, never via a
system package manager (`apt`/`dnf`/`pacman` are forbidden by house policy). In
priority order:

```sh
# Guix (preferred) — Info/HTML/text/DocBook/EPUB:
guix shell texinfo -- texi2any --html doc/gitway.texi
# …add TeX for PDF:
guix shell texinfo texlive -- texi2pdf doc/gitway.texi

# Nix:
nix shell nixpkgs#texinfo -- texi2any --html doc/gitway.texi
nix shell nixpkgs#texinfo nixpkgs#texlive -- texi2pdf doc/gitway.texi
```

`texi2any` is Perl; `texi2pdf` additionally needs a TeX distribution (TeX Live).
For PDF the relevant TeX Live components are the base engine plus `texinfo.tex`.

## Output formats at a glance

| Format | Command | Notes |
|--------|---------|-------|
| **Info** | `makeinfo --no-split FILE.texi` | Native format; also the strictest validator. `--no-split` = one file. |
| **HTML** (one page) | `texi2any --html --no-split --css-include=spacecraft.css FILE.texi` | House-themed; embed the CSS. |
| **HTML** (split) | `texi2any --html --css-ref=spacecraft.css FILE.texi` | One page per node; reference an external stylesheet. |
| **plain text** | `texi2any --plaintext --no-split FILE.texi` | Unstyled (expected). |
| **DocBook** | `texi2any --docbook FILE.texi` | Structural XML for downstream pipelines. |
| **EPUB** | `texi2any --epub3 FILE.texi` | E-book; availability depends on build. |
| **PDF** | `texi2pdf --texinfo=@afourpaper FILE.texi` | Via TeX; A4 per house geometry. |
| **DVI** | `texi2dvi FILE.texi` | Legacy print intermediate. |

Useful flags: `--output=DIR/FILE` (destination), `--error-limit=N`, `-D 'var val'`
(define a `@value` at build time), `--no-validate` (avoid — masks errors),
`--verbose`.

## HTML theming (house brand)

HTML is the format that carries the full Spacecraft theme. Two ways to attach
`assets/spacecraft.css`:

- `--css-include=spacecraft.css` — **embeds** the CSS in the page's `<style>`
  block. Best for single-file (`--no-split`) distribution. House default.
- `--css-ref=URL` — adds a `<link>` to an external stylesheet. Best for split
  output where pages share one stylesheet (e.g. host `spacecraft.css` beside the
  HTML and pass `--css-ref=spacecraft.css`).

The theme paints Void Navy `#000027` background, Share Tech Mono headings,
Inconsolata body, and palette colours for links, code blocks, tables, and
definition headers (Standard §11/§12). Fonts load from Google Fonts with a
system-monospace fallback for offline viewing.

## PDF / print (TeX)

`texi2pdf` wraps TeX + `texinfo.tex`. Apply house A4 geometry **without editing
the source** via `--texinfo=@afourpaper` (injects the command into a temp copy):

```sh
texi2pdf --texinfo=@afourpaper --quiet --clean -o doc/gitway.pdf doc/gitway.texi
```

`--clean` removes the `.aux`/`.toc`/`.log` scratch files; `--build=tidy` keeps
them in a `*.t2d` directory if you need to debug. Run twice (or let `texi2pdf`
loop) so the table of contents resolves.

**Brand caveat for PDF.** `texinfo.tex` is a TeX format with a fixed Computer
Modern–style design; it does **not** consume `spacecraft.css` and cannot be themed
to full palette parity the way HTML can. What house style *can* control in PDF:
paper size (`@afourpaper` → A4), body font size (`@fonttextsize 10|11`), and
microtypography (`@microtype on`). Treat PDF as a faithful print rendering, not a
brand surface; point users to the HTML build for the themed experience. Do not
claim Void Navy backgrounds in PDF.

## A Makefile pattern

A minimal, portable `doc/Makefile` (SPDX header per §4.3; software-class GPL):

```make
# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
NAME    = gitway
SRC     = $(NAME).texi
CSS     = spacecraft.css
BUILD   = build

all: info html pdf text
info:  ; @mkdir -p $(BUILD); makeinfo --no-split $(SRC) -o $(BUILD)/$(NAME).info
html:  ; @mkdir -p $(BUILD); texi2any --html --no-split --css-include=$(CSS) $(SRC) -o $(BUILD)/$(NAME).html
text:  ; @mkdir -p $(BUILD); texi2any --plaintext --no-split $(SRC) -o $(BUILD)/$(NAME).txt
pdf:   ; @mkdir -p $(BUILD); texi2pdf --texinfo=@afourpaper --quiet --clean -o $(BUILD)/$(NAME).pdf $(SRC)
check: ; makeinfo --no-split $(SRC) -o /dev/null   # zero warnings = pass
clean: ; rm -rf $(BUILD) *.t2d *.aux *.toc *.log
.PHONY: all info html text pdf check clean
```

For an all-formats one-shot without a Makefile, use `assets/build.sh` (POSIX `sh`,
with a Nushell variant inline).

## Verify, don't assume

After any build, treat a nonzero exit *or any stderr warning* as failure. The
fastest gate is `makeinfo --no-split FILE.texi -o /dev/null` — if that is silent,
the structure is sound. Then spot-check the HTML renders the theme and the PDF is
A4 (`pdfinfo FILE.pdf` → `Page size: … (A4)`).
