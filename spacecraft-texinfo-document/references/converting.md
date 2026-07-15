# Converting To and From Texinfo

The common job is **Markdown/README → Texinfo manual**. `pandoc` does the heavy
lifting on the body; you then wrap it in the house skeleton and rebuild the node
tree. Conversion is never push-button — pandoc knows nothing of house style,
definition commands, or the node/menu structure that makes a real manual.

## Provisioning

`pandoc` per `spacecraft-missing-pkg` (ephemeral, never a system manager):

```sh
guix shell pandoc -- pandoc --version
nix shell nixpkgs#pandoc -- pandoc --version
```

For the round-trip you also need the Texinfo toolchain (`building.md`).

## Markdown → Texinfo (the main flow)

`pandoc` emits a usable body but a thin shell. The reliable recipe:

1. **Convert the body** (GFM in, Texinfo out), to a fragment:

   ```sh
   pandoc -f gfm -t texinfo README.md -o body.texi
   ```

   Pandoc maps headings to `@chapter`/`@section`/…, lists to
   `@itemize`/`@enumerate`, code fences to `@verbatim`/`@example`, inline code to
   `@code{}`, links to `@uref{}`. It also emits a `@node` before each section.

2. **Wrap in the house skeleton.** Start from `assets/template.texi`, keep its
   header (inline SPDX, `@documentencoding UTF-8`, `@copying` CC-BY-SA-4.0,
   `@titlepage`, `Top` node), and paste the converted body between the master
   `@menu` and `@bye`. Do **not** ship pandoc's bare output — it lacks the SPDX
   header, `@copying`, encoding declaration, and title page that house style
   requires.

3. **Rebuild the node tree** (`authoring.md`). Verify each `@node` is followed by
   one sectioning command, give nodes clean names (strip `, : . ( )`), and write
   the master `@menu` (and any sub-menus) to mirror the sections. Pandoc's nodes
   are a starting point, not a finished tree.

4. **House-polish the markup.** Convert API surfaces to definition commands
   (`definition-commands.md`) — pandoc renders a function as a plain code span; a
   reference manual wants `@deffn`/`@deftypefn`. Replace `@today{}`-style or
   non-ISO dates with ISO 8601. Add `@cindex`/`@findex` entries.

5. **Lint to zero** (`linting.md`) and build all formats (`building.md`).

### Quick fragment-merge helper

```sh
pandoc -f gfm -t texinfo README.md -o /tmp/body.texi
# then hand-merge /tmp/body.texi into a copy of assets/template.texi
```

A direct `pandoc --standalone -t texinfo` produces a complete file, but its
preamble is generic (no SPDX, no `@copying`, no house attribution), so the
skeleton-merge above is preferred.

## Texinfo → Markdown / other

- **Plain text** (faithful, no tooling beyond Texinfo):
  `texi2any --plaintext --no-split FILE.texi`.
- **GFM Markdown** (e.g. to produce a README from a manual):
  `pandoc -f texinfo -t gfm FILE.texi -o FILE.md`. Pandoc's Texinfo *reader* is
  weaker than its writer — definition commands and some block commands degrade;
  review the output. Routing through DocBook can preserve more structure:
  `texi2any --docbook FILE.texi -o FILE.xml && pandoc -f docbook -t gfm FILE.xml`.
- **DocBook / EPUB / HTML** are native `texi2any` outputs (`building.md`) — no
  pandoc needed.

## What conversion cannot do

- It will not produce a correct node/menu tree on its own — you own that.
- It will not apply house style (SPDX, `@copying`, encoding, attribution, ISO
  dates, theme) — start from the template and merge.
- It will not turn code spans into definition commands — that is editorial work.

Treat conversion as a 70%-of-the-way tool: it saves typing the prose; the manual
structure and house compliance are authored on top.
