// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Minimal `SKILL.md` parsing: split the YAML frontmatter from the body, read
//! `name` / `description` (for `skill find`), and return the body (for
//! `skill use`). YAML is parsed with `serde_yaml`, so folded `description: >`
//! scalars join correctly.

use std::path::Path;

use serde::Deserialize;

/// The frontmatter fields we care about.
#[derive(Debug, Default, Deserialize)]
struct Front {
    name: Option<String>,
    description: Option<String>,
}

/// Parsed `(name, description)` from a `SKILL.md` frontmatter block.
pub(crate) fn frontmatter(skill_md: &Path) -> (Option<String>, Option<String>) {
    let content = std::fs::read_to_string(skill_md).unwrap_or_default();
    let Some((fm, _)) = split(&content) else {
        return (None, None);
    };
    match serde_yaml::from_str::<Front>(fm) {
        Ok(front) => (front.name, front.description.map(|d| d.trim().to_owned())),
        Err(_) => (None, None),
    }
}

/// The markdown body of a `SKILL.md` (everything after the frontmatter), or the
/// whole file when there is no frontmatter.
pub(crate) fn body(skill_md: &Path) -> String {
    let content = std::fs::read_to_string(skill_md).unwrap_or_default();
    match split(&content) {
        Some((_, body)) => body.trim_start().to_owned(),
        None => content,
    }
}

/// Split `---\n<frontmatter>\n---\n<body>` into `(frontmatter, body)`.
fn split(content: &str) -> Option<(&str, &str)> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let idx = rest.find("\n---")?;
    let fm = &rest[..idx];
    // Body begins after the closing fence line.
    let after = &rest[idx + 1..]; // at the closing "---"
    let body = after.split_once('\n').map_or("", |(_, b)| b);
    Some((fm, body))
}
