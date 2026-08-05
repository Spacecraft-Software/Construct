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

/// The rendered length, in characters, of a `SKILL.md` frontmatter
/// `description` — the exact string the skill loader measures (Standard §5.6).
///
/// Deliberately does not reuse [`frontmatter`], which trims for display: a
/// folded `description: >` scalar carries the trailing newline the loader
/// counts, so trimming under-reports by one character and would let a
/// description sitting exactly on the cap slip through. `.githooks/
/// check-description-length.py` counts the same way, and the two must agree.
///
/// Returns `None` when the file is unreadable, has no frontmatter, has no
/// `description`, or does not parse — none of which this cap can adjudicate.
pub(crate) fn description_len(skill_md: &Path) -> Option<usize> {
    let content = std::fs::read_to_string(skill_md).ok()?;
    let (fm, _) = split(&content)?;
    let front = serde_yaml::from_str::<Front>(fm).ok()?;
    front.description.map(|d| d.chars().count())
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
    // Inclusive of the newline before the closing fence: without it a block
    // scalar that is the *last* frontmatter key loses its trailing newline, and
    // `description_len` would under-count by one against the loader.
    let fm = &rest[..=idx];
    // Body begins after the closing fence line.
    let after = &rest[idx + 1..]; // at the closing "---"
    let body = after.split_once('\n').map_or("", |(_, b)| b);
    Some((fm, body))
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use super::description_len;

    /// `description_len` on a `SKILL.md` written to a temp file.
    fn len_of(content: &str) -> Option<usize> {
        let mut f = tempfile::NamedTempFile::new().expect("temp file");
        f.write_all(content.as_bytes()).expect("write");
        description_len(f.path())
    }

    #[test]
    fn folded_scalar_keeps_its_trailing_newline_as_the_last_key() {
        // The closing `---` fence ends the block. The loader still counts the
        // newline that terminates the folded content, so this is 4 chars.
        assert_eq!(
            len_of("---\nname: d\ndescription: >\n  abc\n---\nb\n"),
            Some(4)
        );
    }

    #[test]
    fn folded_scalar_keeps_its_trailing_newline_before_another_key() {
        // A dedented key ends the block instead. Same count either way — the
        // two shapes must not disagree, or the cap would depend on key order.
        assert_eq!(
            len_of("---\ndescription: >\n  abc\nname: d\n---\nb\n"),
            Some(4)
        );
    }

    #[test]
    fn folded_scalar_joins_wrapped_lines_with_single_spaces() {
        // "a b c\n" — raw line lengths are not the measurement.
        assert_eq!(
            len_of("---\nname: d\ndescription: >\n  a\n  b\n  c\n---\nb\n"),
            Some(6)
        );
    }

    #[test]
    fn plain_single_line_scalar_has_no_trailing_newline() {
        assert_eq!(len_of("---\nname: d\ndescription: abc\n---\nb\n"), Some(3));
    }

    #[test]
    fn absent_description_is_not_measurable() {
        assert_eq!(len_of("---\nname: d\n---\nb\n"), None);
        assert_eq!(len_of("no frontmatter here\n"), None);
    }
}
