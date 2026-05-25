---
name: spacecraft-markdown-document
description: >
  Produces well-formed GitHub-Flavored Markdown (GFM) documents conforming to
  the GFM spec (https://github.github.com/gfm/) and the Spacecraft Software
  house style. SLASH COMMAND ONLY — this skill is NEVER auto-triggered; it runs
  exclusively when the agent is explicitly invoked via the
  `/spacecraft-markdown-document` slash command. Invoke it for: creating new .md
  files, converting prose or outlines to GFM, auditing existing Markdown for
  spec compliance, and generating GFM companion documents alongside ODF/MS Office
  deliverables. When the slash command is present, ALWAYS consult this skill
  before writing a single line of Markdown.
license: GPL-3.0-or-later
metadata:
  spdx: "SPDX-License-Identifier: GPL-3.0-or-later"
  author: "Mohamed Hammad & Spacecraft Software"
  maintainer: "Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>"
  website: "https://Construct.SpacecraftSoftware.org/"
---

<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->

# Spacecraft Markdown Document

**Author:** Mohamed Hammad & Spacecraft Software
**Maintainer:** Mohamed Hammad — <Mohamed.Hammad@SpacecraftSoftware.org>
**License:** GPL-3.0-or-later
**Spec:** [GitHub Flavored Markdown Spec v0.29-gfm](https://github.github.com/gfm/)
**Website:** <https://Construct.SpacecraftSoftware.org/>

---

## §0 — Slash-Command Protocol

This skill is **slash-command-only**. It activates exclusively when the agent
receives an explicit invocation in the form:

```
/spacecraft-markdown-document [SUBCOMMAND] [OPTIONS] [TARGET]
```

**Never** auto-trigger this skill from ambient context, even when markdown
content is present in the conversation. Wait for the explicit slash-command call.

### Subcommands

| Subcommand   | Description                                              |
|--------------|----------------------------------------------------------|
| `new`        | Create a new GFM document from a title / outline         |
| `convert`    | Convert prose, notes, or structured data to GFM          |
| `audit`      | Check an existing `.md` file for GFM spec compliance     |
| `companion`  | Generate a GFM companion `.md` for an ODF/MS Office file |
| `template`   | Emit a named blank template (see §6)                     |

When called without a subcommand, default to `new`.

---

## §1 — Preamble

Every `.md` file produced by this skill is a conforming GFM document.
GFM is a strict superset of CommonMark (spec v0.29-gfm, 2019-04-06).
All CommonMark rules apply; GFM extensions add four capabilities on top:
**Tables**, **Task list items**, **Strikethrough**, and **Autolinks (extended)**.

This skill is agent-agnostic — instructions apply equally to any language model,
autonomous agent, or automated pipeline that can read and follow a SKILL.md.

---

## §2 — Document Structure

Every document this skill produces follows this top-to-bottom layout:

```
[1] SPDX header comment (software files only)
[2] Front-matter comment block (always present)
[3] Document title — single H1
[4] Meta block (author, date, license, etc.) — optional but preferred
[5] Abstract / description paragraph — one short paragraph
[6] Table of contents (documents > 3 sections) — linked ATX headings
[7] Body sections — H2 and below; never H1 again after [3]
[8] Footer separator + attribution line
```

### §2.1 — SPDX Header

For `.md` files that are part of a Spacecraft Software source tree (i.e., not
pure content documents), include the SPDX header as an HTML comment on line 1:

```markdown
<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) YYYY Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->
```

Pure content documents (READMEs, design docs, release notes) **omit** the SPDX
comment but do include the front-matter comment block (§2.2).

### §2.2 — Front-Matter Comment Block

Immediately after the SPDX header (or at the very top for pure content docs),
include one HTML comment block with document metadata:

```markdown
<!-- GFM Document
     title:      <Document Title>
     author:     Mohamed Hammad & Spacecraft Software
     date:       YYYY-MM-DD (ISO 8601)
     version:    X.Y.Z  (semver; omit for single-revision docs)
     license:    GPL-3.0-or-later
     project:    <ProjectName>  (Spacecraft Software subproject name, if any)
     website:    https://Construct.SpacecraftSoftware.org/
-->
```

All keys are optional except `title`, `author`, `date`, and `license`.

### §2.3 — Heading Hierarchy

- **One H1 per document** — the document title only. Never use H1 mid-document.
- **H2** for top-level sections (`##`).
- **H3** for subsections (`###`). H4–H6 are reserved for deep nesting; avoid
  unless the document genuinely requires four or more nesting levels.
- ATX style (`#` prefix) is canonical. Setext style (underline) is allowed only
  when converting pre-existing documents that already use it — do not introduce
  setext headings in new documents.

---

## §3 — GFM Element Reference

Load `references/gfm-elements.md` for the full element-by-element reference
(syntax, edge cases, and spec citations). The quick-reference table below covers
the most common choices:

| Element             | Preferred syntax                     | Notes                              |
|---------------------|--------------------------------------|------------------------------------|
| Heading             | `## Section Name`                    | ATX only in new docs               |
| Emphasis            | `*italic*`                           | Asterisk form preferred            |
| Strong              | `**bold**`                           | Asterisk form preferred            |
| Strikethrough       | `~~text~~`                           | GFM extension                      |
| Inline code         | `` `code` ``                         | Single backtick                    |
| Fenced code block   | ` ```lang ` … ` ``` `                | Always supply a language tag       |
| Table               | Pipe syntax (§3.1)                   | GFM extension; colons for align    |
| Task list           | `- [ ] pending` / `- [x] done`       | GFM extension; inside unordered list |
| Ordered list        | `1.` with consistent numbers         | Use `1.` for all items if preferred |
| Unordered list      | `-` bullet                           | Prefer `-`; avoid `*` or `+` mix  |
| Blockquote          | `> text`                             | Nest with `>> text`                |
| Thematic break      | `---`                                | Three or more hyphens on own line  |
| Link                | `[label](url)`                       | Prefer inline over reference style |
| Reference link      | `[label][id]` + `[id]: url`          | Use when the same URL repeats      |
| Image               | `![alt](url)`                        | Always supply meaningful alt text  |
| Autolink (extended) | `https://example.com`                | GFM auto-linkifies bare URLs       |
| Hard line break     | `\` at end of line                   | Preferred over two trailing spaces |

### §3.1 — Tables

Tables are a GFM extension. Always include the delimiter row (second row):

```markdown
| Column A     | Column B     | Column C |
|--------------|:------------:|----------:|
| left-aligned | centered     | right    |
| row 2        | row 2        | row 2    |
```

Rules:
- Delimiter cells use `-` characters (at least one). The colon (`:`) positions
  the alignment: none = left, `:---:` = center, `---:` = right.
- Leading and trailing `|` are optional per spec but are **required** by this
  skill for readability.
- Header cells and body cells must align in column count — uneven tables cause
  undefined rendering.
- Leave one space inside each cell border: `| content |`, not `|content|`.

### §3.2 — Task Lists

Task list items are a GFM extension. They must appear inside an unordered list:

```markdown
- [x] Completed item
- [ ] Pending item
- [ ] Another pending item
```

The `x` must be lowercase. A space between `[` and `]` means unchecked.
Do not place task lists inside ordered lists — behaviour is undefined in some
renderers.

### §3.3 — Fenced Code Blocks

Always specify a language identifier after the opening fence:

```markdown
```rust
fn main() {
    println!("Hello, Spacecraft Software!");
}
```
```

Supported identifiers include `rust`, `nix`, `bash`, `sh`, `nushell`, `toml`,
`json`, `yaml`, `markdown`, `text`, `console`, etc. Use `text` when no
language applies. The opening and closing fences must use the same character
(`` ` `` or `~`) and have the same width (≥ 3 characters).

---

## §4 — Style Rules

### §4.1 — Line Length

Wrap prose lines at **100 characters**. Code blocks, tables, and URLs are exempt
from the 100-character limit. Never wrap headings.

### §4.2 — Blank Lines

- One blank line before and after every heading, code block, table, blockquote,
  and list.
- Two blank lines before an H2 heading to provide visual separation.
- No trailing blank lines at end-of-file (one final newline only).

### §4.3 — Lists

- Use `-` for all unordered-list bullets. Do not mix `-`, `*`, and `+` in one
  list — GFM starts a new list on marker change.
- For continuation paragraphs inside a list item, indent 4 spaces.
- Prefer *tight* lists (no blank lines between items) for short items; use
  *loose* lists (blank lines between items) only when items contain multiple
  paragraphs.

### §4.4 — Emphasis Conventions

- `*italic*` for light emphasis (definitions, titles, foreign words).
- `**bold**` for strong emphasis (key terms, warnings, required values).
- Do not use emphasis as a substitute for a heading.

### §4.5 — Links

- Prefer inline links `[label](url)` for one-time references.
- Use reference links `[label][id]` when the same URL appears three or more
  times in the document; collect all `[id]: url` definitions at the bottom of
  the file before the footer.
- Label text must be meaningful — never use bare URLs as labels
  (exception: the `<url>` autolink form where URL display is intentional).

### §4.6 — Language and Tone

- Use clear, concise, active-voice prose.
- Present tense for current-state descriptions; future tense for planned work.
- ISO 8601 dates only (`YYYY-MM-DD`). Never write dates as "May 25, 2026".
- Use metric units and UTC timestamps per The Steelbore Standard §12.

---

## §5 — Spacecraft Software Integration

Markdown is a plain-text format; it cannot carry visual theming. Nevertheless,
these integration rules apply:

- **Do not attempt to render Spacecraft Software palette colours in Markdown.**
  No inline HTML colour spans, no CSS blocks. The visual layer belongs entirely
  in the paired ODF/MS Office file (see `spacecraft-document-format` skill).
- **Reference the palette by token name in prose** when describing UI elements
  (e.g., "the background uses Void Navy (`#000027`)").
- **Companion documents** (paired with a tier-1 office file) must include the
  following HTML comment immediately after the front-matter block:

  ```markdown
  <!-- companion: <source-file-basename>.<ext> | palette: Spacecraft Software §9 -->
  ```

- Every Spacecraft Software `.md` file lives beside a `LICENSE` or `COPYING` file
  at the repository root that carries the GPL-3.0-or-later text. Do not duplicate
  the full licence inside the `.md`; reference it with:

  ```markdown
  **License:** GPL-3.0-or-later — see [`LICENSE`](../LICENSE)
  ```

---

## §6 — Templates

Load `references/document-templates.md` for ready-to-fill template bodies.
Available templates:

| Slug                | Use case                                    |
|---------------------|---------------------------------------------|
| `readme`            | Project README                              |
| `changelog`         | CHANGELOG.md (Keep a Changelog style)       |
| `contributing`      | CONTRIBUTING.md                             |
| `prd`               | Product Requirements Document               |
| `adr`               | Architecture Decision Record                |
| `release-notes`     | Release notes for a versioned artefact      |
| `companion`         | GFM companion for an ODF/MS Office document |
| `skill-readme`      | README for a Spacecraft Software skill      |

Invoke via: `/spacecraft-markdown-document template <slug>`

---

## §7 — Audit Mode

When invoked as `/spacecraft-markdown-document audit <file>`, perform the
following checks and emit a structured report:

```markdown
## GFM Audit: <filename>

### Errors (spec violations — must fix)
- …

### Warnings (style violations — should fix)
- …

### Info (suggestions — optional improvements)
- …

**Result:** PASS / FAIL
```

Audit checklist:
- [ ] Single H1 present; no H1 recurrence mid-document
- [ ] ATX heading style; no raw setext introductions in new content
- [ ] All fenced code blocks have a language tag
- [ ] Table delimiter rows present; column counts consistent
- [ ] Task list items only inside unordered lists
- [ ] No trailing whitespace (except intentional hard-line-break `\`)
- [ ] No tab characters outside code blocks (tabs in prose are spec-legal but
      visually inconsistent — flag as warning)
- [ ] Line length ≤ 100 chars in prose (warn on violations; code blocks exempt)
- [ ] SPDX comment present (for source-tree files)
- [ ] Front-matter comment block present
- [ ] Footer attribution line present
- [ ] All links resolvable (where the agent has filesystem or network access)
- [ ] Strikethrough uses `~~`, not `<del>` HTML
- [ ] No disallowed raw HTML elements (GFM §6.11: `<title>`, `<textarea>`,
      `<style>`, `<xmp>`, `<iframe>`, `<noembed>`, `<noframes>`, `<script>`,
      `<plaintext>`)

---

## §8 — Footer Template

End every document produced by this skill with:

```markdown
---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

For companion documents, append the source-file reference after the attribution:

```markdown
---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) — companion to `<source-file>` —*
```

---

## §9 — Acceptance Checklist

Before delivering any `.md` output, verify:

- [ ] Front-matter comment block present (title, author, date, license)
- [ ] SPDX header present (source-tree files only)
- [ ] Single H1 title; H2 for sections; no setext headings introduced
- [ ] All fenced code blocks carry a language tag
- [ ] Tables have delimiter rows; columns consistent; leading/trailing `|`
- [ ] Task lists inside unordered lists only; `x` lowercase
- [ ] Prose wrapped at 100 chars; headings not wrapped
- [ ] One blank line around blocks; two blank lines before H2
- [ ] Links have meaningful labels; repeated URLs use reference style
- [ ] Spacecraft Software palette referenced by token name, never rendered inline
- [ ] Companion comment added when paired with a tier-1 office document
- [ ] Footer attribution line present
- [ ] No disallowed raw HTML elements (§7 audit checklist)
- [ ] No trailing whitespace; single final newline at EOF

---

## §10 — Cross-References

| Need                                      | Skill to load                  |
|-------------------------------------------|--------------------------------|
| ODF / MS Office document with GFM pair    | `spacecraft-document-format`   |
| Brand colours, typography, palette tokens | `spacecraft-brand-guidelines`  |
| Full Spacecraft Software compliance       | `spacecraft-standard`          |
| CLI output in Markdown or plain text      | `spacecraft-cli-standard`      |

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
