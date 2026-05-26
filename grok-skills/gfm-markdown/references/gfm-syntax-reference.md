# GFM Syntax Reference

This reference expands on the core GFM extensions and GitHub-specific conveniences. Use it when you need precise examples or to handle edge cases.

## 1. Tables

**Full example with alignment and escaped pipe:**

```markdown
| Column A                  | Column B (Center) | Column C (Right) |
| :------------------------ | :---------------: | ---------------: |
| Left-aligned text         | Centered content  | Right-aligned    |
| Cell with \| pipe inside  | More data         | 42               |
| Another row               | ✅ Done           | **Bold**         |
```

**Rules:**
- Delimiter row (`| --- |`) is mandatory.
- `:` at start of delimiter cell = left align
- `:` at end = right align
- `:` on both = center
- No `:` = left (default)
- Minimum 3 dashes per column recommended.
- Tables cannot contain nested tables or complex blocks.

## 2. Task Lists

**Basic + nested:**

```markdown
## Project Tasks

- [x] Set up repository
- [ ] Implement core feature
  - [x] Design API
  - [ ] Write tests
- [ ] Deploy to production
```

**Important:** The `[ ]` or `[x]` must be the first non-whitespace content after the list marker (`- ` or `* ` or `1. `).

## 3. Strikethrough

Inline only:

```markdown
The old API is now ~~deprecated~~ and will be removed in v2.0.
```

Works inside headings, lists, tables, and emphasis.

## 4. Alerts (GitHub Callouts)

**All five types:**

```markdown
> [!NOTE]
> Information that is useful but not critical.

> [!TIP]
> A helpful suggestion for users.

> [!IMPORTANT]
> Key information that requires attention.

> [!WARNING]
> Potential problems or risks.

> [!CAUTION]
> Destructive or irreversible actions.
```

**Variant with title (supported on GitHub):**

```markdown
> [!WARNING]
> **Breaking Change**
> This release removes the legacy endpoint. Migrate before upgrading.
```

## 5. Autolinks & Bare URLs

GFM automatically links:

- `https://github.com`
- `user@example.com`
- `www.example.com` (with www.)

Explicit:

```markdown
See <https://github.github.com/gfm/> for the full specification.
```

## 6. Mentions, Issues, PRs (GitHub Rendering Layer)

These are **not** part of the GFM parser spec but are transformed by GitHub's Markdown renderer:

```markdown
Thanks @octocat for the fix in #1234!
See also owner/repo#567.
```

Renders as clickable links to the user profile or issue/PR.

## 7. Emoji Shortcodes

```markdown
:rocket: :tada: :bug: :sparkles:
```

GitHub supports hundreds of shortcodes. Full list: https://github.com/ikatyang/emoji-cheat-sheet

## 8. Footnotes (GitHub-Supported Extension)

```markdown
Here is a sentence with a footnote[^1].

[^1]: This is the footnote content. It can span multiple lines
      and even contain **formatting**.
```

GitHub renders footnotes at the bottom with backlinks.

## 9. Math Support (GitHub Pages / READMEs)

```markdown
Inline math: $E = mc^2$

Display math:

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

Requires the repo to have MathJax enabled (automatic on GitHub for many files).

## 10. Common Pitfalls to Avoid

- **Do not** put blank lines inside a table cell (breaks parsing).
- **Do not** start a task list marker with spaces before `[ ]`.
- **Do not** use HTML `<input type="checkbox">` — use GFM task list syntax instead.
- **Do not** mix list types in one block (e.g., `- ` and `1. ` at same level).
- **Do** leave a blank line before and after every block element.
- **Do** test complex tables and nested lists in GitHub's preview before finalizing.

## 11. Heading Anchor Links (for TOC)

GitHub generates IDs automatically:

- `# My Section Title` → `#my-section-title`
- Use in TOC:

```markdown
## Table of Contents

- [Introduction](#introduction)
- [Installation](#installation)
```

For custom IDs (rarely needed): `## My Title {#custom-id}` (works in some processors).

## 12. Raw HTML (Limited)

GFM disallows many raw HTML tags for security. Allowed examples:

```html
<details>
<summary>Click to expand</summary>
Hidden content here.
</details>
```

Use sparingly; prefer native GFM blocks.

---

**Source:** Derived from official GFM spec v0.29-gfm (2019) + GitHub's current rendering behavior (2026). For the authoritative parser rules, always refer to https://github.github.com/gfm/