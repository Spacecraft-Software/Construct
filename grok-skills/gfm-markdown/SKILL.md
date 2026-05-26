---
name: gfm-markdown
description: Use for creating editing or validating GitHub Flavored Markdown GFM documents such as READMEs wikis issue templates PR descriptions technical docs or changelogs — applies full GFM syntax including tables task lists strikethrough alerts footnotes and ensures perfect rendering on GitHub
---

# GFM Markdown Skill

## Purpose
This skill ensures all Markdown output targeting GitHub or GFM renderers (GitHub.com, GitHub Pages, wikis, issues, PRs) uses correct syntax and follows professional documentation standards. The base model knows generic Markdown; this skill adds GFM-specific extensions, GitHub rendering quirks, and reusable templates.

## Activation Triggers
- User asks to "create a README", "write GFM doc", "make Markdown for GitHub", "format as Markdown", "update CHANGELOG.md", etc.
- Any request for .md file output intended for GitHub ecosystem.

## Mandatory Rules for All GFM Documents

1. **Document Structure**
   - Always start with a single `# Title` (H1).
   - Use `##`, `###`, `####` hierarchically — never skip levels (H1 → H3 invalid).
   - End long documents with `---` and a "Footer" section if appropriate.
   - Add a Table of Contents for docs > 500 words using manual links (GitHub auto-generates heading anchors: lowercase-with-hyphens).

2. **Tables (GFM Extension)**
   - Always include a delimiter row with alignment colons.
   - Example:
     ```markdown
     | Feature       | Status     | Notes                  |
     | ------------- | :--------: | ---------------------- |
     | Tables        | ✅         | Full GFM support       |
     | Task lists    | ✅         | Interactive checkboxes |
     ```
   - Escape literal `|` inside cells as `\|`.
   - Keep cell content short; wrap long text or split tables.
   - No HTML tables unless absolutely necessary (GFM supports basic `<table>` but prefer native).

3. **Task Lists (GFM Extension)**
   - Use at the **start** of a list item paragraph only:
     ```markdown
     - [ ] Unchecked item
     - [x] Completed item
     - [X] Also completed (case-insensitive)
     ```
   - Supports nesting but avoid deep nesting for UX.
   - Rendered as native checkboxes on GitHub.

4. **Strikethrough (GFM Extension)**
   - Use exactly two tildes: `~~deprecated feature~~`
   - Works inline in paragraphs, lists, tables.

5. **Alerts / Callouts (GitHub-Specific, Widely Supported)**
   - Use blockquote syntax with type:
     ```markdown
     > [!NOTE]
     > This is a neutral informational message.
     >
     > [!TIP]
     > Pro tip for users.
     >
     > [!IMPORTANT]
     > Critical information users must not miss.
     >
     > [!WARNING]
     > Potential risks or breaking changes.
     >
     > [!CAUTION]
     > Destructive actions — use with extreme care.
     ```
   - Place alerts immediately after relevant heading or paragraph.
   - Do **not** nest alerts.

6. **Code Blocks & Syntax Highlighting**
   - Always use fenced blocks with language identifier:
     ```bash
     #!/bin/bash
     echo "Hello GFM"
     ```
   - Supported languages include: bash, python, javascript, json, yaml, rust, go, etc.
   - For no highlighting: ```text or ```none
   - Use ` ```diff ` for patches.

7. **Links, Autolinks & References**
   - Bare URLs and emails auto-link: https://example.com or user@example.com
   - Descriptive links: [GitHub GFM Spec](https://github.github.com/gfm/)
   - Reference-style (preferred for repeated links):
     ```markdown
     See the [spec][gfm] for details.
     ...
     [gfm]: https://github.github.com/gfm/ "Official GFM Specification"
     ```

8. **Images**
   - Always include descriptive alt text: `![Screenshot of the dashboard showing metrics](assets/dashboard.png)`
   - For GitHub repos: use relative paths (`docs/images/foo.png`) or `https://raw.githubusercontent.com/owner/repo/branch/path`
   - Add title in quotes if helpful: `![alt](url "Title on hover")`

9. **Mentions, Issues & PR References (GitHub Rendering)**
   - `@username` → links to profile
   - `#123` or `owner/repo#123` → links to issue/PR
   - Use in context: "Thanks to @octocat for reporting #42"

10. **YAML Frontmatter (for GitHub Pages / Jekyll / Obsidian / etc.)**
    - Optional but recommended for published docs:
      ```yaml
      ---
      title: "My Awesome Document"
      author: "Mohamed"
      date: 2026-05-25
      tags: [gfm, documentation, github]
      ---
      ```
    - Frontmatter must be the very first content (no blank lines before `---`).

11. **General Formatting**
    - One blank line between all block elements (paragraphs, lists, code, tables, alerts).
    - Soft-wrap source lines at ~80-100 characters for readability.
    - Use em-dashes (—) and en-dashes (–) via Unicode or `&mdash;` (but prefer Unicode).
    - For math: GitHub supports LaTeX in `$...$` or `$$...$$` blocks (display mode) — use when needed.

## Output Workflow
1. Choose or copy the most relevant template from `assets/`.
2. Fill in content following the rules above.
3. Review for GFM compliance (tables aligned, task markers correct, alerts properly typed).
4. If the user will copy-paste to GitHub, remind them to use the preview tab.
5. For very long docs, suggest splitting into `docs/` folder with `SUMMARY.md` or GitHub wiki structure.

## Templates Available
- `assets/README-template.md` — Standard open-source project README
- `assets/CHANGELOG-template.md` — Keep a Changelog format (https://keepachangelog.com)
- `assets/pr-description-template.md` — Professional pull request body
- `assets/technical-doc-template.md` — General long-form technical document with TOC

Copy the chosen template and customize. Never start from blank unless the user explicitly requests a minimal file.

## References
- `references/gfm-syntax-reference.md` — Comprehensive syntax examples, edge cases, and common pitfalls.
- Official spec: https://github.github.com/gfm/ (Version 0.29-gfm, 2019; GitHub has since added alerts and other conveniences).

Always produce documents that look professional both in raw source and when rendered on GitHub.