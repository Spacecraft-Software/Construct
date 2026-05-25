<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->

<!-- GFM Document
     title:      GFM Elements Reference
     author:     Mohamed Hammad & Spacecraft Software
     date:       2026-05-25
     license:    GPL-3.0-or-later
     website:    https://Construct.SpacecraftSoftware.org/
-->

# GFM Elements Reference

Companion reference for the `spacecraft-markdown-document` skill.
Source spec: <https://github.github.com/gfm/> (v0.29-gfm, 2019-04-06).

## Table of Contents

- [Block Elements](#block-elements)
  - [Thematic Breaks](#thematic-breaks)
  - [Headings](#headings)
  - [Code Blocks](#code-blocks)
  - [HTML Blocks](#html-blocks)
  - [Block Quotes](#block-quotes)
  - [Lists](#lists)
  - [Tables (GFM)](#tables-gfm)
- [Inline Elements](#inline-elements)
  - [Code Spans](#code-spans)
  - [Emphasis and Strong](#emphasis-and-strong)
  - [Strikethrough (GFM)](#strikethrough-gfm)
  - [Links](#links)
  - [Images](#images)
  - [Autolinks](#autolinks)
  - [Hard and Soft Line Breaks](#hard-and-soft-line-breaks)
- [Task List Items (GFM)](#task-list-items-gfm)
- [Disallowed Raw HTML (GFM)](#disallowed-raw-html-gfm)
- [Character Escapes](#character-escapes)
- [Edge Cases and Gotchas](#edge-cases-and-gotchas)

---

## Block Elements

### Thematic Breaks

Three or more hyphens, asterisks, or underscores on a line by themselves.

```markdown
---
***
___
```

**Spacecraft Software canonical form:** `---` only. The blank lines before and
after are not required by spec but are required by this skill for readability.

A thematic break is not a heading separator — it is a section divider with
semantic meaning. Use it sparingly.

---

### Headings

#### ATX Headings (canonical)

```markdown
# H1 — Document title only
## H2 — Top-level section
### H3 — Subsection
#### H4 — Deep subsection (use sparingly)
##### H5
###### H6
```

The `#` characters must be followed by at least one space. An optional closing
sequence of `#` characters is allowed but must be preceded by a space — this
skill does not use closing `#` sequences.

Blank lines before and after each heading are not required by spec but are
required by this skill.

#### Setext Headings (legacy, avoid)

```markdown
H1 Heading
==========

H2 Heading
----------
```

Allowed only in documents converting pre-existing content. Do not introduce
setext headings in new documents — they only support H1 and H2, so the full
heading hierarchy cannot be expressed.

---

### Code Blocks

#### Fenced (canonical)

Opening and closing fences of `` ` `` or `~`, at least 3 characters wide, with
the same character and width. The closing fence must be on its own line.

````markdown
```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```
````

The info string (language tag) follows the opening fence. It is used for syntax
highlighting. Always supply one. Use `text` when no language applies.

Common tags: `rust`, `nix`, `nushell`, `sh`, `bash`, `toml`, `json`, `yaml`,
`markdown`, `console`, `text`, `c`, `cpp`, `python`, `typescript`, `javascript`.

If the content itself contains triple backticks, use a tilde fence or a
quadruple-backtick fence:

`````markdown
````markdown
```code inside markdown block```
````
`````

#### Indented Code Blocks (avoid)

Four-space-indented blocks are valid CommonMark but ambiguous in context. This
skill does not produce indented code blocks — use fenced blocks exclusively.

---

### HTML Blocks

Raw HTML is passed through by most GFM renderers except for the disallowed
elements (see §Disallowed Raw HTML). Use sparingly and only when GFM syntax
cannot express the required structure (e.g., `<details>`/`<summary>` toggles).

```markdown
<details>
<summary>Click to expand</summary>

Expanded content here.

</details>
```

---

### Block Quotes

```markdown
> This is a blockquote.
>
> It can span multiple paragraphs.

>> Nested blockquote.
```

The `>` marker may optionally be followed by a space. Blank lines between
paragraphs of a blockquote must also be prefixed with `>`.

---

### Lists

#### Unordered Lists

```markdown
- First item
- Second item
  - Nested item (2-space or 4-space indent)
  - Another nested item
- Third item
```

Canonical bullet: `-`. Avoid mixing `-`, `*`, and `+` — GFM treats marker
changes as separate lists.

Continuation paragraph inside an item: indent 4 spaces to align with the text:

```markdown
- First item

    This is a continuation paragraph for the first item.

- Second item
```

#### Ordered Lists

```markdown
1. First item
2. Second item
3. Third item
```

Using `1.` for every item is spec-legal and preferred in auto-generated lists
(prevents renumbering pain), but explicit sequential numbers are preferred for
human-authored lists for clarity.

#### Tight vs. Loose Lists

**Tight** (no blank lines between items — preferred for short items):

```markdown
- Alpha
- Beta
- Gamma
```

**Loose** (blank lines between items — required when items have multiple
paragraphs):

```markdown
- Alpha

  With extra context.

- Beta

  With its own extra context.
```

---

### Tables (GFM)

Tables are a GFM extension. They consist of a header row, a delimiter row, and
zero or more body rows.

```markdown
| Left aligned | Centered | Right aligned |
|:-------------|:--------:|--------------:|
| cell         | cell     | cell          |
| cell         | cell     | cell          |
```

**Alignment via colons in delimiter row:**
- `:---` or `---` → left (default)
- `:---:` → center
- `---:` → right

**Rules enforced by this skill:**
- Leading and trailing `|` required.
- At least one `-` per delimiter cell.
- Header row and body rows must have the same number of cells.
- Minimum one space padding inside cells: `| content |`.
- Do not embed newlines inside table cells.
- Pipe characters inside cell content must be escaped: `\|`.

---

## Inline Elements

### Code Spans

Single backticks for inline code. Use double backticks when the content itself
contains a backtick:

```markdown
Use `cargo build --release` to compile.
A literal backtick: `` ` ``.
```

---

### Emphasis and Strong

```markdown
*italic* or _italic_
**bold** or __bold__
***bold italic*** or ___bold italic___
```

**Canonical forms for this skill:** `*italic*` and `**bold**`.
Underscore forms are avoided to prevent accidental triggering inside
identifiers like `MY_CONSTANT` (GFM does not emphasise underscore-delimited
spans inside words, but asterisk forms are unambiguous everywhere).

---

### Strikethrough (GFM)

GFM extension using double tildes:

```markdown
~~deprecated text~~
```

---

### Links

#### Inline Links

```markdown
[label](https://example.com)
[label](https://example.com "Optional title")
```

#### Reference Links

```markdown
[label][ref-id]
[ref-id]: https://example.com
[ref-id]: https://example.com "Optional title"
```

Collect all reference definitions at the bottom of the file, before the footer
separator. Reference IDs are case-insensitive.

#### Autolinks (CommonMark)

Angle-bracket form — renders the URL as a clickable link with the URL as label:

```markdown
<https://SpacecraftSoftware.org/>
<mailto:Mohamed.Hammad@SpacecraftSoftware.org>
```

#### Autolinks (GFM extension)

Bare URLs are auto-linked without angle brackets:

```markdown
https://SpacecraftSoftware.org/
```

---

### Images

```markdown
![Alt text](https://example.com/image.png)
![Alt text](https://example.com/image.png "Optional title")
```

Alt text is **mandatory** for accessibility. Never use an empty `![]()`. Write
a description that conveys the image's content to a reader who cannot see it.

Reference form:

```markdown
![Alt text][img-id]
[img-id]: https://example.com/image.png
```

---

### Hard and Soft Line Breaks

**Soft line break** — a newline in source that becomes a space in output.
GFM wraps long paragraphs in the source; they render as a single paragraph.

**Hard line break** — a `\` at end of line forces a `<br>` in output:

```markdown
Line one.\
Line two on a new line.
```

Two trailing spaces also produce a hard line break per CommonMark spec, but
they are invisible and easily stripped by editors. This skill uses `\` only.

---

## Task List Items (GFM)

Must appear inside an unordered list only. The checkbox marker is `[ ]` (space
= unchecked) or `[x]` (lowercase x = checked):

```markdown
- [x] Write the SKILL.md
- [x] Write the references
- [ ] Package the skill
- [ ] Write test cases
```

On GitHub, checked boxes are rendered as checked checkboxes and are interactive
in Issues and Pull Requests.

---

## Disallowed Raw HTML (GFM)

The following HTML tags are filtered (replaced with literal text) by GFM
renderers on GitHub:

`<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`, `<noembed>`,
`<noframes>`, `<script>`, `<plaintext>`

Do not use these elements in Spacecraft Software Markdown documents. If their
semantic purpose is genuinely needed, use a fenced code block with `html` tag
to display them as code.

---

## Character Escapes

Any ASCII punctuation character can be escaped with a backslash to prevent
interpretation as Markdown syntax:

```markdown
\* not italic \*
\# not a heading
\| not a table border
\\ a literal backslash
```

Full list of escapable characters (per spec): `` ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~ ``

---

## Edge Cases and Gotchas

| Scenario | Correct approach |
|----------|-----------------|
| Numbered list that starts at a number other than 1 | The first number determines the list start; subsequent numbers are ignored by spec. Use `1.` throughout or sequential numbers — be consistent. |
| Blank line inside a list item breaks tightness | Any blank line between items makes the whole list loose, wrapping all items in `<p>` tags. This is usually desirable for multi-paragraph items. |
| Nested code block inside list | Indent the fence 4 spaces beyond the list marker level to keep it inside the item. |
| Pipe in table cell | Escape as `\|`. |
| `>` inside a table cell | Does not start a blockquote inside a table cell. Use `&gt;` for the literal character. |
| Bold/italic around partial words | Works with `*` forms: `un*believ*able`. Does not work with `_` forms inside words. |
| Link with parentheses in URL | Escape unbalanced parentheses: `[label](/path/to/(file\))` or use angle-bracket autolink. |
| Reference link ID conflict | First definition wins — all subsequent definitions for the same ID are ignored. |
| ATX heading without trailing newline at EOF | Spec-legal, but always end the file with a single newline. |
| Setext heading ambiguity | A line of `---` after a paragraph is a setext H2, not a thematic break. Always separate with a blank line. |

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
