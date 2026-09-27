// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Per-target `SKILL.md` frontmatter projection.
//!
//! A hybrid of parsing and raw text. `serde_yaml` validates and supplies the
//! values; the emitted text copies each kept key's **raw source block
//! byte-for-byte**, so `description` never round-trips through a YAML
//! serializer and its `>` / `>-` chomping, wrapping, and quoting survive
//! exactly. Only `metadata` is synthesized, with every value written as a JSON
//! string literal — a valid YAML double-quoted scalar by construction.
//!
//! Blocks are split with the rule `.github/check-skill-frontmatter.py` uses: a
//! block starts at a line whose first byte is not a space, tab, or `#` and that
//! contains `:`; indented and blank lines continue it. Top-level `#` comment
//! lines belong to no block and are dropped (none exist in the catalogue).
//!
//! Every projection ends in a round-trip check: the emitted frontmatter is
//! re-parsed and each kept value must equal its source value, `metadata` must
//! equal the expected merge, no other key may appear, and the rendered
//! `description` length must be unchanged. A mismatch is a generator bug and
//! is never shipped.

use serde_json::json;
use serde_yaml::{Mapping, Value};

use super::Problem;
use crate::sources::skillmd;

/// Every top-level key a root skill may carry.
const KNOWN_KEYS: &[&str] = &[
    "name",
    "description",
    "license",
    "maintainer",
    "website",
    "metadata",
    "user-invocable",
    "compatibility",
    "allowed-tools",
];

/// Top-level keys moved under `metadata:` for spec-clean targets, in order.
const MOVED_TO_METADATA: &[&str] = &["maintainer", "website"];

/// Longest skill name the Agent Skills specification allows.
const NAME_MAX: usize = 64;

/// Which frontmatter a target ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    /// Spec-clean: `name`, `description`, `license`, `compatibility`?,
    /// `allowed-tools`?, `user-invocable`? (Claude only), `metadata`?.
    Claude {
        /// Keep `user-invocable` (the Claude target; Perplexity drops it).
        user_invocable: bool,
    },
    /// `name` + `description` only.
    Grok,
    /// The single-file header: `name`, `description`, `license`.
    Header,
}

impl Profile {
    /// Raw-copied keys, in canonical emission order.
    fn kept(self) -> &'static [&'static str] {
        match self {
            Self::Claude {
                user_invocable: true,
            } => &[
                "name",
                "description",
                "license",
                "compatibility",
                "allowed-tools",
                "user-invocable",
            ],
            Self::Claude {
                user_invocable: false,
            } => &[
                "name",
                "description",
                "license",
                "compatibility",
                "allowed-tools",
            ],
            Self::Grok => &["name", "description"],
            Self::Header => &["name", "description", "license"],
        }
    }
}

/// One top-level key and its raw source text (key line through the last
/// continuation line, newline-terminated).
#[derive(Debug)]
struct Block {
    key: String,
    raw: String,
}

/// Split a frontmatter into top-level blocks. `Err` for a line the rule cannot
/// place (text at column 0 with no `:`, or indented text before any key).
fn blocks(fm: &str) -> Result<Vec<Block>, String> {
    let mut out: Vec<Block> = Vec::new();
    let mut open = false;
    for line in fm.split_inclusive('\n') {
        let first = line.as_bytes().first().copied();
        match first {
            Some(b' ' | b'\t' | b'\n' | b'\r') => {
                if let (true, Some(last)) = (open, out.last_mut()) {
                    last.raw.push_str(line);
                } else if !line.trim().is_empty() {
                    return Err("indented text before the first key".to_owned());
                }
            }
            Some(b'#') => open = false,
            Some(_) => {
                let Some((key, _)) = line.split_once(':') else {
                    return Err(format!("line without a key: {}", line.trim_end()));
                };
                out.push(Block {
                    key: key.trim().to_owned(),
                    raw: line.to_owned(),
                });
                open = true;
            }
            None => {}
        }
    }
    if let Some(last) = out.last_mut() {
        if !last.raw.ends_with('\n') {
            last.raw.push('\n');
        }
    }
    Ok(out)
}

/// A parsed frontmatter plus its raw blocks, checked against each other.
#[derive(Debug)]
struct Parsed<'a> {
    map: Mapping,
    blocks: Vec<Block>,
    body: &'a str,
}

/// Parse and block-split `text`, or describe why it cannot be projected.
fn parse(text: &str) -> Result<Parsed<'_>, String> {
    let (fm, body) = skillmd::split(text).ok_or_else(|| "no frontmatter".to_owned())?;
    let map = skillmd::parse_mapping(fm).map_err(|e| e.reason().to_owned())?;
    let blocks = blocks(fm).map_err(|why| format!("unsupported frontmatter layout: {why}"))?;
    let mut map_keys: Vec<String> = Vec::new();
    for key in map.keys() {
        match key {
            Value::String(k) => map_keys.push(k.clone()),
            _ => return Err("unsupported frontmatter layout: non-string key".to_owned()),
        }
    }
    let mut block_keys: Vec<String> = blocks.iter().map(|b| b.key.clone()).collect();
    map_keys.sort();
    block_keys.sort();
    if map_keys != block_keys {
        return Err(
            "unsupported frontmatter layout: raw keys do not match the parsed keys".to_owned(),
        );
    }
    Ok(Parsed { map, blocks, body })
}

/// Whether `name` is a valid Agent Skills name: lowercase alphanumeric runs
/// joined by single hyphens, at most [`NAME_MAX`] characters.
pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= NAME_MAX
        && name.split('-').all(|run| {
            !run.is_empty()
                && run
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Refusal for a frontmatter that cannot be projected.
fn layout_problem(skill: &str, why: &str) -> Problem {
    Problem::conflict(
        "unsupported_frontmatter",
        format!("{skill}/SKILL.md frontmatter cannot be projected: {why}"),
        format!("$EDITOR {skill}/SKILL.md"),
        json!({ "skill": skill, "error": why }),
    )
}

/// Checks shared by root and Grok-native skills: `name` and `description`.
fn validate_identity(skill: &str, p: &Parsed<'_>, out: &mut Vec<Problem>) {
    match p.map.get("name") {
        Some(Value::String(name)) if name == skill && valid_name(name) => {}
        other => {
            let found = match other {
                Some(Value::String(s)) => s.clone(),
                Some(_) => "<not a string>".to_owned(),
                None => "<missing>".to_owned(),
            };
            out.push(Problem::conflict(
                "invalid_name",
                format!(
                    "{skill}: name '{found}' must equal the directory and match ^[a-z0-9]+(-[a-z0-9]+)*$ (max {NAME_MAX})"
                ),
                format!("$EDITOR {skill}/SKILL.md"),
                json!({ "skill": skill, "name": found }),
            ));
        }
    }
    if !matches!(p.map.get("description"), Some(Value::String(_))) {
        out.push(Problem::conflict(
            "invalid_frontmatter",
            format!("{skill}: SKILL.md has no string description"),
            format!("$EDITOR {skill}/SKILL.md"),
            json!({ "skill": skill, "error": "missing description" }),
        ));
    }
}

/// Validate a root skill's frontmatter for every spec-clean projection.
pub(crate) fn validate(skill: &str, text: &str) -> Vec<Problem> {
    let p = match parse(text) {
        Ok(p) => p,
        Err(why) => return vec![layout_problem(skill, &why)],
    };
    let mut out = Vec::new();
    validate_identity(skill, &p, &mut out);
    for block in &p.blocks {
        if !KNOWN_KEYS.contains(&block.key.as_str()) {
            out.push(Problem::conflict(
                "unknown_frontmatter_keys",
                format!("{skill}: unknown frontmatter key '{}'", block.key),
                format!("$EDITOR {skill}/SKILL.md"),
                json!({ "skill": skill, "key": block.key }),
            ));
        }
    }
    if let Some(v) = p.map.get("allowed-tools") {
        let ok = match v {
            Value::String(_) => true,
            Value::Sequence(items) => items.iter().all(|i| matches!(i, Value::String(_))),
            _ => false,
        };
        if !ok {
            out.push(layout_problem(
                skill,
                "allowed-tools must be a string or a list of strings",
            ));
        }
    }
    if let Err(problem) = metadata(skill, &p.map) {
        out.push(problem);
    }
    out
}

/// Validate a Grok-native skill (projected to `name` + `description` only).
pub(crate) fn validate_grok(skill: &str, text: &str) -> Vec<Problem> {
    match parse(text) {
        Ok(p) => {
            let mut out = Vec::new();
            validate_identity(skill, &p, &mut out);
            out
        }
        Err(why) => vec![layout_problem(skill, &why)],
    }
}

/// The merged `metadata` entries: the source `metadata` in source order, then
/// the top-level keys in [`MOVED_TO_METADATA`] that are present. Nothing is
/// invented for a skill that has none.
fn metadata(skill: &str, map: &Mapping) -> Result<Vec<(String, String)>, Problem> {
    let invalid = |key: &str| {
        Problem::conflict(
            "invalid_metadata",
            format!("{skill}: metadata value '{key}' must be a string"),
            format!("$EDITOR {skill}/SKILL.md"),
            json!({ "skill": skill, "key": key }),
        )
    };
    let mut entries: Vec<(String, String)> = Vec::new();
    match map.get("metadata") {
        None | Some(Value::Null) => {}
        Some(Value::Mapping(m)) => {
            for (k, v) in m {
                match (k, v) {
                    (Value::String(k), Value::String(v)) => entries.push((k.clone(), v.clone())),
                    (Value::String(k), _) => return Err(invalid(k)),
                    _ => return Err(invalid("<non-string key>")),
                }
            }
        }
        Some(_) => return Err(invalid("metadata")),
    }
    for key in MOVED_TO_METADATA {
        match map.get(*key) {
            None => {}
            Some(Value::String(v)) => match entries.iter().find(|(k, _)| k == key) {
                Some((_, existing)) if existing == v => {}
                Some(_) => {
                    return Err(Problem::conflict(
                        "metadata_conflict",
                        format!("{skill}: top-level {key} differs from metadata.{key}"),
                        format!("$EDITOR {skill}/SKILL.md"),
                        json!({ "skill": skill, "key": key }),
                    ))
                }
                None => entries.push(((*key).to_owned(), v.clone())),
            },
            Some(_) => return Err(invalid(key)),
        }
    }
    Ok(entries)
}

/// A `metadata` key as YAML: plain when it is a simple identifier, otherwise
/// a JSON (YAML double-quoted) string.
fn yaml_key(key: &str) -> String {
    let plain = key
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if plain {
        key.to_owned()
    } else {
        serde_json::Value::String(key.to_owned()).to_string()
    }
}

/// Project `text` (a whole `SKILL.md`) to `profile`, returning the new
/// `SKILL.md` with the body bytes untouched.
///
/// # Errors
///
/// A [`Problem`] when the source cannot be projected (already reported by
/// [`validate`] for root skills) or when the round-trip check fails.
pub(crate) fn project(skill: &str, text: &str, profile: Profile) -> Result<String, Problem> {
    let p = parse(text).map_err(|why| layout_problem(skill, &why))?;
    let mut out = String::from("---\n");
    for key in profile.kept() {
        if let Some(block) = p.blocks.iter().find(|b| b.key == *key) {
            out.push_str(&block.raw);
        }
    }
    let meta = if matches!(profile, Profile::Claude { .. }) {
        metadata(skill, &p.map)?
    } else {
        Vec::new()
    };
    if !meta.is_empty() {
        out.push_str("metadata:\n");
        for (k, v) in &meta {
            out.push_str("  ");
            out.push_str(&yaml_key(k));
            out.push_str(": ");
            out.push_str(&serde_json::Value::String(v.clone()).to_string());
            out.push('\n');
        }
    }
    out.push_str("---\n");
    out.push_str(p.body);
    roundtrip(skill, text, &out, &p.map, profile, &meta)?;
    Ok(out)
}

/// Verify the emitted frontmatter against the source (see the module docs).
fn roundtrip(
    skill: &str,
    source: &str,
    emitted: &str,
    src: &Mapping,
    profile: Profile,
    meta: &[(String, String)],
) -> Result<(), Problem> {
    let fail = |key: &str| {
        Problem::internal(
            "roundtrip_failures",
            format!("frontmatter rewrite changed {key} in {skill}"),
            json!({ "skill": skill, "key": key }),
        )
    };
    let (fm, _) = skillmd::split(emitted).ok_or_else(|| fail("<frontmatter>"))?;
    let got = skillmd::parse_mapping(fm).map_err(|_| fail("<frontmatter>"))?;
    let mut expected_keys: Vec<&str> = Vec::new();
    for key in profile.kept() {
        if let Some(value) = src.get(*key) {
            expected_keys.push(key);
            if got.get(*key) != Some(value) {
                return Err(fail(key));
            }
        }
    }
    if !meta.is_empty() {
        expected_keys.push("metadata");
        let want: Mapping = meta
            .iter()
            .map(|(k, v)| (Value::String(k.clone()), Value::String(v.clone())))
            .collect();
        if got.get("metadata") != Some(&Value::Mapping(want)) {
            return Err(fail("metadata"));
        }
    }
    if got.len() != expected_keys.len() {
        return Err(fail("<extra keys>"));
    }
    let before = skillmd::field_len(source, "description").map_err(|_| fail("description"))?;
    let after = skillmd::field_len(emitted, "description").map_err(|_| fail("description"))?;
    if before != after {
        return Err(fail("description"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{blocks, project, validate, Profile};

    const LATE_DESC: &str = "---\nname: late-desc\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\ndescription: >-\n  Folded text\n  across lines: with colon.\n\n  Second paragraph.\n---\n\n# Body\n";

    #[test]
    fn claude_projection_reorders_and_moves_maintainer_under_metadata() {
        let out = project(
            "late-desc",
            LATE_DESC,
            Profile::Claude {
                user_invocable: true,
            },
        )
        .expect("projects");
        assert_eq!(
            out,
            "---\nname: late-desc\ndescription: >-\n  Folded text\n  across lines: with colon.\n\n  Second paragraph.\nlicense: GPL-3.0-or-later\nmetadata:\n  maintainer: \"M H <m@h.org>\"\n  website: \"https://x.org/\"\n---\n\n# Body\n"
        );
    }

    #[test]
    fn grok_and_header_profiles_keep_only_their_keys() {
        let grok = project("late-desc", LATE_DESC, Profile::Grok).unwrap();
        assert!(grok.starts_with("---\nname: late-desc\ndescription: >-\n"));
        assert!(!grok.contains("license"));
        let header = project("late-desc", LATE_DESC, Profile::Header).unwrap();
        assert!(header.contains("license: GPL-3.0-or-later\n---\n"));
        assert!(!header.contains("metadata"));
    }

    #[test]
    fn user_invocable_is_kept_for_claude_only() {
        let src =
            "---\nname: dual\ndescription: >\n  d\nuser-invocable: false\nlicense: MIT\n---\nb\n";
        let claude = project(
            "dual",
            src,
            Profile::Claude {
                user_invocable: true,
            },
        )
        .unwrap();
        assert!(claude.contains("user-invocable: false\n"));
        let perplexity = project(
            "dual",
            src,
            Profile::Claude {
                user_invocable: false,
            },
        )
        .unwrap();
        assert!(!perplexity.contains("user-invocable"));
    }

    #[test]
    fn metadata_merge_keeps_source_entries_first_and_escapes_values() {
        let src = "---\nname: m\ndescription: d\nmaintainer: \"A \\\"q\\\" \\\\ b: c\"\nmetadata:\n  spdx: GPL-3.0-or-later\n  author: Someone\n---\nb\n";
        let out = project(
            "m",
            src,
            Profile::Claude {
                user_invocable: true,
            },
        )
        .expect("round-trips");
        assert!(out.contains(
            "metadata:\n  spdx: \"GPL-3.0-or-later\"\n  author: \"Someone\"\n  maintainer: \"A \\\"q\\\" \\\\ b: c\"\n"
        ));
    }

    #[test]
    fn unknown_key_conflict_and_non_string_metadata_are_refused() {
        let unknown = validate("u", "---\nname: u\ndescription: d\nlicence: x\n---\n");
        assert_eq!(unknown[0].key, "unknown_frontmatter_keys");
        let conflict = validate(
            "c",
            "---\nname: c\ndescription: d\nmaintainer: a\nmetadata:\n  maintainer: b\n---\n",
        );
        assert_eq!(conflict[0].key, "metadata_conflict");
        let nonstring = validate(
            "n",
            "---\nname: n\ndescription: d\nmetadata:\n  n: 3\n---\n",
        );
        assert_eq!(nonstring[0].key, "invalid_metadata");
        let badname = validate("Bad", "---\nname: Bad\ndescription: d\n---\n");
        assert_eq!(badname[0].key, "invalid_name");
    }

    #[test]
    fn block_splitter_matches_the_python_rule() {
        let fm = "name: a\ndescription: >\n  x\n\n  y\n# comment\nmetadata:\n  k: v\n";
        let b = blocks(fm).unwrap();
        let keys: Vec<&str> = b.iter().map(|b| b.key.as_str()).collect();
        assert_eq!(keys, vec!["name", "description", "metadata"]);
        assert_eq!(b[1].raw, "description: >\n  x\n\n  y\n");
        assert_eq!(b[2].raw, "metadata:\n  k: v\n");
    }
}
