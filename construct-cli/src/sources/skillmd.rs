// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Minimal `SKILL.md` parsing: split the YAML frontmatter from the body, read
//! `name` / `description` (for `skill find`), and return the body (for
//! `skill use`). YAML is parsed with `serde_yaml`, so folded `description: >`
//! scalars join correctly.
//!
//! The parse is strict. A plain-scalar `description:` that contains `: `
//! (`… the house look: slide decks …`) is a YAML error, not a string, and every
//! strict consumer — this crate, `PyYAML`, Nushell's `from yaml`, the skill
//! loaders — rejects the whole frontmatter. [`frontmatter`] degrades to
//! `(None, None)` so `skill find` still lists the skill; [`description_len`]
//! reports the failure so the §5.6 gate in `skill ship` refuses it instead of
//! exempting it.

use std::fmt;
use std::path::Path;

use serde::Deserialize;
use serde_yaml::Value;

/// Maximum rendered length of a skill's frontmatter `description` (Standard
/// §5.6).
///
/// The consuming skill loader rejects anything over **1024** characters at
/// install time — after the bundles are built and pushed — so the cap sits at
/// 1000 for a 24-character margin covering encoding and trailing-newline edge
/// cases. Raising it past the loader's limit would ship bundles that cannot be
/// installed. `.githooks/check-description-length.py` enforces the same number
/// in CI and in the pre-commit hook; changing one without the other lets a
/// bundle pass one gate and fail the next.
pub(crate) const DESCRIPTION_CAP: usize = 1000;

/// Maximum length of a skill's frontmatter `compatibility` field.
///
/// The Agent Skills specification limits `compatibility` to 500 characters.
/// Several Construct skills set it (`spacecraft-cli-shell`,
/// `spacecraft-cli-preference`, `spacecraft-missing-pkg`); `skill ship` and
/// `skill build` refuse one over the cap, measured with the same rule as
/// [`DESCRIPTION_CAP`].
pub(crate) const COMPATIBILITY_CAP: usize = 500;

/// The frontmatter fields we care about.
#[derive(Debug, Default, Deserialize)]
struct Front {
    name: Option<String>,
    description: Option<String>,
}

/// A `SKILL.md` whose frontmatter is present but is not valid YAML.
///
/// Carries the parser's own diagnostic, which names the line and column it
/// stopped at, so the author can find the offending character without
/// re-parsing by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvalidFrontmatter {
    reason: String,
}

impl InvalidFrontmatter {
    /// The parser's diagnostic, including the line and column it stopped at.
    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for InvalidFrontmatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frontmatter is not valid YAML: {}", self.reason)
    }
}

impl std::error::Error for InvalidFrontmatter {}

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
/// Returns `Ok(None)` when the file is unreadable, has no frontmatter, or has
/// no `description` — none of which this cap can adjudicate — and `Err` when
/// the frontmatter is there but does not parse.
///
/// The `Err` is deliberate. A strict loader sees no `description` at all for
/// such a skill, so there is nothing the cap could have measured; folding that
/// case into `None` made an unparseable frontmatter an *exemption* from the
/// gate, and one skill shipped through it that way. The caller must refuse it.
pub(crate) fn description_len(skill_md: &Path) -> Result<Option<usize>, InvalidFrontmatter> {
    let Ok(content) = std::fs::read_to_string(skill_md) else {
        return Ok(None);
    };
    field_len(&content, "description")
}

/// The rendered length, in characters, of the string-valued frontmatter field
/// `key` in the `SKILL.md` text `content`.
///
/// The one counting rule behind every frontmatter cap: `chars().count()` of the
/// parsed, **untrimmed** string (see [`description_len`] for why untrimmed).
/// `Ok(None)` when there is no frontmatter or no such key; `Err` when the
/// frontmatter does not parse, or when `key` is present but is not a string —
/// a strict loader has no string to measure there either.
pub(crate) fn field_len(content: &str, key: &str) -> Result<Option<usize>, InvalidFrontmatter> {
    let Some((fm, _)) = split(content) else {
        return Ok(None);
    };
    let parsed = parse_mapping(fm)?;
    match parsed.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.chars().count())),
        Some(_) => Err(InvalidFrontmatter {
            reason: format!("`{key}` is not a string"),
        }),
    }
}

/// Parse a frontmatter block into an order-preserving YAML mapping.
///
/// An empty frontmatter is an empty mapping; anything that parses to a
/// non-mapping value (a bare scalar, a sequence) is invalid.
pub(crate) fn parse_mapping(fm: &str) -> Result<serde_yaml::Mapping, InvalidFrontmatter> {
    let value = serde_yaml::from_str::<Value>(fm).map_err(|err| InvalidFrontmatter {
        reason: err.to_string(),
    })?;
    match value {
        Value::Mapping(map) => Ok(map),
        Value::Null => Ok(serde_yaml::Mapping::new()),
        _ => Err(InvalidFrontmatter {
            reason: "frontmatter is not a mapping".to_owned(),
        }),
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
pub(crate) fn split(content: &str) -> Option<(&str, &str)> {
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

    use super::{description_len, InvalidFrontmatter};

    /// The description that shipped unparseable: 870 characters, wrapped at 78
    /// columns exactly as `spacecraft-brand-guidelines/SKILL.md` now carries
    /// them. The `: ` in the third line is the whole story — a plain scalar
    /// stops being a string there; a folded block scalar does not.
    const BRAND_DESCRIPTION_LINES: &[&str] = &[
        "Applies the Spacecraft Software brand — the Steelbore palette family",
        "(Standard §11) and the §12 FOSS-licensed typography — to any artifact that",
        "should carry the house look: slide decks, diagrams, SVGs, dashboards,",
        "marketing pages, READMEs, or UI mockups. Triggers on brand colours, house",
        "style, visual formatting, \"make this on-brand\", token roles (canvas,",
        "surface, foreground, accent, structure, status), or the Share Tech Mono /",
        "Inconsolata pairing. This skill names tokens and their roles; it never",
        "carries values — every hex, RGB triple, and contrast ratio is read from",
        "`steelbore-color-palette`'s `assets/steelbore.toml`, the single source",
        "(§11.4). Do NOT use it to generate editor or terminal themes (use",
        "`spacecraft-theme-factory`), to author documents (use",
        "`spacecraft-document-format`), or to pick an accessible variant (use",
        "`spacecraft-accessibility-support`).",
    ];

    /// `description_len` on a `SKILL.md` written to a temp file.
    fn len_of(content: &str) -> Result<Option<usize>, InvalidFrontmatter> {
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
            Ok(Some(4))
        );
    }

    #[test]
    fn folded_scalar_keeps_its_trailing_newline_before_another_key() {
        // A dedented key ends the block instead. Same count either way — the
        // two shapes must not disagree, or the cap would depend on key order.
        assert_eq!(
            len_of("---\ndescription: >\n  abc\nname: d\n---\nb\n"),
            Ok(Some(4))
        );
    }

    #[test]
    fn folded_scalar_joins_wrapped_lines_with_single_spaces() {
        // "a b c\n" — raw line lengths are not the measurement.
        assert_eq!(
            len_of("---\nname: d\ndescription: >\n  a\n  b\n  c\n---\nb\n"),
            Ok(Some(6))
        );
    }

    #[test]
    fn plain_single_line_scalar_has_no_trailing_newline() {
        assert_eq!(
            len_of("---\nname: d\ndescription: abc\n---\nb\n"),
            Ok(Some(3))
        );
    }

    #[test]
    fn absent_description_is_not_measurable() {
        assert_eq!(len_of("---\nname: d\n---\nb\n"), Ok(None));
        assert_eq!(len_of("no frontmatter here\n"), Ok(None));
    }

    #[test]
    fn plain_scalar_containing_colon_space_is_invalid_frontmatter() {
        // `description: … look: slide decks …` reads as a nested mapping value
        // to every strict YAML parser, and the whole frontmatter is rejected.
        // Before this was an `Err`, the gate took "cannot measure" as "nothing
        // to measure" and the skill shipped.
        let plain = BRAND_DESCRIPTION_LINES.join(" ");
        assert!(plain.contains(": slide decks"), "fixture lost its `: `");
        let err = len_of(&format!("---\nname: d\ndescription: {plain}\n---\nb\n"))
            .expect_err("a plain scalar containing `: ` must not parse");
        assert!(
            err.reason().contains("mapping values are not allowed"),
            "unexpected diagnostic: {err}"
        );
    }

    #[test]
    fn folded_scalar_of_the_same_870_characters_measures_870() {
        // Identical text as a `>-` block scalar: the wrapped lines fold with
        // single spaces, the strip indicator drops the final newline, and the
        // `: ` is ordinary content. 870 is what the plain scalar rendered to
        // where it happened to be read leniently, so nothing changes for the
        // consumers that already saw it.
        let mut block = String::new();
        for line in BRAND_DESCRIPTION_LINES {
            block.push_str("  ");
            block.push_str(line);
            block.push('\n');
        }
        assert_eq!(
            len_of(&format!("---\nname: d\ndescription: >-\n{block}---\nb\n")),
            Ok(Some(870))
        );
    }
}
