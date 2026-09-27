// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Perplexity consolidation: a faithful port of `perplexity-skills/build.py`.
//!
//! Perplexity rejects a skill zip with more than [`PERPLEXITY_MAX_FILES`]
//! files. A skill over that merges its per-tool `references/<tool>.md` files
//! into category files, as `build.py` does, with the category map read from
//! [`CATEGORIES_PATH`] — data, not code, so a second oversized skill needs a
//! TOML entry and nothing else. The three patterns are `build.py`'s, verbatim.
//!
//! Line handling: `build.py` splits with Python's `splitlines()`; this port
//! uses `str::lines()`. They agree on `\n` and `\r\n` and on the trailing-line
//! case, and differ only on exotic separators (`\x0b`, `\x0c`, `\x1c`–`\x1e`,
//! `\x85`, U+2028/9) that the catalogue does not contain.
//!
//! Two deliberate divergences from `build.py`'s member set: the bundle also
//! carries `LICENSE` (Standard §5.6), and it ships every other Claude-tree
//! member (a non-`.md` file under `references/`, `assets/**`) rather than only
//! the files `build.py` names. Both follow from "Claude layout".

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{Member, Members, Problem, SKILL_MD};

/// Perplexity's per-zip file limit.
pub(crate) const PERPLEXITY_MAX_FILES: usize = 100;

/// The consolidation map, relative to the catalogue root.
pub(crate) const CATEGORIES_PATH: &str = "perplexity-skills/categories.toml";

/// The only schema version this reader understands.
const SCHEMA: u32 = 1;

/// `build.py`'s `_FENCE`.
static FENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(```|~~~)").expect("static regex"));
/// `build.py`'s `_HEADING`.
static HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6}) ").expect("static regex"));
/// `build.py`'s `_LINK`.
static LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"references/([A-Za-z0-9._-]+)\.md").expect("static regex"));
/// `build.py`'s `self_check` pattern.
static ANCHORED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"references/([a-z0-9-]+)\.md#([A-Za-z0-9._-]+)").expect("static regex")
});

/// Whether `line` opens or closes a fenced code block.
pub(crate) fn is_fence(line: &str) -> bool {
    FENCE.is_match(line)
}

/// Whether `line` is an ATX heading (`#`…`######` then a space).
pub(crate) fn heading_level(line: &str) -> Option<usize> {
    HEADING.captures(line).map(|c| c[1].len())
}

/// `perplexity-skills/categories.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CategoriesFile {
    /// Must be [`SCHEMA`].
    schema: u32,
    /// Skill id → its consolidation map.
    #[serde(default)]
    pub(crate) skill: BTreeMap<String, SkillMap>,
}

/// One skill's consolidation map.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillMap {
    /// Non-tool `references/` files copied through verbatim.
    #[serde(default)]
    pub(crate) passthrough: Vec<String>,
    /// Categories, in emission order.
    pub(crate) category: Vec<Category>,
}

/// One category file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Category {
    /// File stem: `references/<slug>.md`.
    pub(crate) slug: String,
    /// The category file's H1.
    pub(crate) title: String,
    /// Tools merged into it (each a `references/<tool>.md` stem).
    pub(crate) tools: Vec<String>,
}

/// A Perplexity refusal (`perplexity` extension, `{skill, kind, detail}`).
pub(crate) fn problem(skill: &str, kind: &str, detail: &Value) -> Problem {
    Problem::conflict(
        "perplexity",
        format!("perplexity consolidation of {skill} failed: {kind}"),
        format!("$EDITOR {CATEGORIES_PATH}"),
        json!({ "skill": skill, "kind": kind, "detail": detail }),
    )
}

/// Parse the categories file text.
pub(crate) fn parse(text: &str) -> Result<CategoriesFile, String> {
    let file: CategoriesFile = toml::from_str(text).map_err(|e| e.to_string())?;
    if file.schema != SCHEMA {
        return Err(format!("schema {} is not {SCHEMA}", file.schema));
    }
    Ok(file)
}

/// Load the categories file under `repo`, or `None` when it does not exist.
///
/// # Errors
///
/// `categories_invalid` (boxed) for non-UTF-8 text, a parse error, or an
/// unknown schema; `io_failures` for any read failure other than "not found".
pub(crate) fn load(repo: &Path) -> Result<Option<CategoriesFile>, Box<Problem>> {
    let path = repo.join(CATEGORIES_PATH);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(Box::new(Problem::internal(
                "io_failures",
                format!("cannot read {CATEGORIES_PATH}: {err}"),
                json!({ "path": CATEGORIES_PATH, "error": err.to_string() }),
            )));
        }
    };
    let text = String::from_utf8(bytes).map_err(|_| {
        Box::new(problem(
            "*",
            "categories_invalid",
            &json!({ "path": CATEGORIES_PATH, "error": "not valid UTF-8" }),
        ))
    })?;
    parse(&text).map(Some).map_err(|why| {
        Box::new(problem(
            "*",
            "categories_invalid",
            &json!({ "path": CATEGORIES_PATH, "error": why }),
        ))
    })
}

/// `build.py`'s `consolidate(cat, tools)`: merge one category's tool files.
fn render_category(
    skill: &str,
    cat: &Category,
    members: &Members,
    problems: &mut Vec<Problem>,
) -> String {
    let mut out: Vec<String> = vec![format!("# {}", cat.title), String::new()];
    let mut tools: Vec<&String> = cat.tools.iter().collect();
    tools.sort();
    for tool in tools {
        let key = format!("references/{tool}.md");
        let text = members
            .get(&key)
            .map(|m| String::from_utf8_lossy(&m.bytes).into_owned())
            .unwrap_or_default();
        let raw: Vec<&str> = text.lines().collect();
        let Some(title) = raw.first().and_then(|l| l.strip_prefix("# ")) else {
            problems.push(problem(skill, "missing_h1", &json!({ "tool": tool })));
            continue;
        };
        out.push(format!("## {tool}"));
        out.push(String::new());
        out.push(format!("**{}**", title.trim()));
        out.push(String::new());
        let mut in_fence = false;
        for line in &raw[1..] {
            if is_fence(line) {
                in_fence = !in_fence;
                out.push((*line).to_owned());
                continue;
            }
            if !in_fence && heading_level(line).is_some() {
                out.push(format!("#{line}"));
            } else {
                out.push((*line).to_owned());
            }
        }
        out.push(String::new());
    }
    let mut doc = out.join("\n").trim_end_matches('\n').to_owned();
    doc.push('\n');
    doc
}

/// Validate `map` against the skill's tree: `build.py`'s `canonical_tools()`
/// equality, `tool_to_category()` uniqueness, slug shape and uniqueness, and
/// passthrough presence. Returns the tool → category-slug index.
fn check_map<'m>(
    skill: &str,
    members: &Members,
    map: &'m SkillMap,
) -> Result<BTreeMap<&'m str, &'m str>, Vec<Problem>> {
    let mut problems = Vec::new();

    // canonical_tools(): references/*.md stems minus passthrough.
    let canon: BTreeSet<String> = members
        .keys()
        .filter_map(|k| k.strip_prefix("references/"))
        .filter(|rest| !rest.contains('/') && !rest.starts_with('.'))
        .filter(|rest| !map.passthrough.iter().any(|p| p == rest))
        .filter_map(|rest| rest.strip_suffix(".md"))
        .map(str::to_owned)
        .collect();
    let mapped: BTreeSet<String> = map
        .category
        .iter()
        .flat_map(|c| c.tools.iter().cloned())
        .collect();
    if mapped != canon {
        problems.push(problem(
            skill,
            "map_out_of_sync",
            &json!({
                "in_map_not_on_disk": mapped.difference(&canon).collect::<Vec<_>>(),
                "on_disk_not_mapped": canon.difference(&mapped).collect::<Vec<_>>(),
            }),
        ));
    }

    // tool_to_category(): a tool in two categories is an error.
    let mut inv: BTreeMap<&str, &str> = BTreeMap::new();
    let mut slugs: BTreeSet<&str> = BTreeSet::new();
    for cat in &map.category {
        let slug_ok = !cat.slug.is_empty()
            && cat
                .slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !slug_ok || !slugs.insert(&cat.slug) {
            problems.push(problem(skill, "invalid_slug", &json!({ "slug": cat.slug })));
        }
        for tool in &cat.tools {
            if let Some(prev) = inv.insert(tool, &cat.slug) {
                problems.push(problem(
                    skill,
                    "duplicate_tool",
                    &json!({ "tool": tool, "categories": [prev, cat.slug] }),
                ));
            }
        }
    }
    for pass in &map.passthrough {
        if !members.contains_key(&format!("references/{pass}")) {
            problems.push(problem(
                skill,
                "missing_passthrough",
                &json!({ "file": pass }),
            ));
        }
    }
    if problems.is_empty() {
        Ok(inv)
    } else {
        Err(problems)
    }
}

/// Consolidate `members` (whose `SKILL.md` is already projected) per `map`.
///
/// # Errors
///
/// Every `perplexity` refusal found: map out of sync with the tree, a tool in
/// two categories, a bad or duplicate slug, a missing passthrough file, a tool
/// file without an H1, a dangling rewritten anchor, or a result still over
/// [`PERPLEXITY_MAX_FILES`].
pub(crate) fn consolidate(
    skill: &str,
    members: &Members,
    map: &SkillMap,
) -> Result<Members, Vec<Problem>> {
    let inv = check_map(skill, members, map)?;
    let mut problems = Vec::new();

    let docs: Vec<(String, String)> = map
        .category
        .iter()
        .map(|cat| {
            (
                cat.slug.clone(),
                render_category(skill, cat, members, &mut problems),
            )
        })
        .collect();

    // rewrite_skill(): over the whole projected SKILL.md, frontmatter included.
    let source = members
        .get(SKILL_MD)
        .map(|m| String::from_utf8_lossy(&m.bytes).into_owned())
        .unwrap_or_default();
    let skill_text = LINK
        .replace_all(&source, |c: &regex::Captures<'_>| match inv.get(&c[1]) {
            Some(cat) => format!("references/{cat}.md#{}", &c[1]),
            None => c[0].to_owned(),
        })
        .into_owned();

    // self_check(): every rewritten anchor must resolve to a `## ` heading.
    let anchors: BTreeMap<&str, BTreeSet<&str>> = docs
        .iter()
        .map(|(slug, doc)| {
            (
                slug.as_str(),
                doc.lines().filter_map(|l| l.strip_prefix("## ")).collect(),
            )
        })
        .collect();
    let mut dangling: BTreeSet<String> = BTreeSet::new();
    for c in ANCHORED.captures_iter(&skill_text) {
        let (cat, anchor) = (&c[1], &c[2]);
        match anchors.get(cat) {
            None => {
                dangling.insert(format!("{cat}.md (no such category file) #{anchor}"));
            }
            Some(set) if !set.contains(anchor) => {
                dangling.insert(format!("{cat}.md#{anchor} (no '## {anchor}' heading)"));
            }
            Some(_) => {}
        }
    }
    if !dangling.is_empty() {
        problems.push(problem(skill, "dangling_anchor", &json!(dangling)));
    }
    if !problems.is_empty() {
        return Err(problems);
    }

    let mut out: Members = members
        .iter()
        .filter(|(k, _)| {
            !k.strip_prefix("references/")
                .and_then(|r| r.strip_suffix(".md"))
                .is_some_and(|stem| inv.contains_key(stem))
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mode = members.get(SKILL_MD).map_or(0o644, |m| m.mode);
    out.insert(
        SKILL_MD.to_owned(),
        Member {
            bytes: skill_text.into_bytes(),
            mode,
        },
    );
    for (slug, doc) in docs {
        let key = format!("references/{slug}.md");
        if out.contains_key(&key) {
            problems.push(problem(skill, "slug_collision", &json!({ "path": key })));
        }
        out.insert(key, Member::text(doc));
    }
    if out.len() > PERPLEXITY_MAX_FILES {
        problems.push(problem(
            skill,
            "still_oversized",
            &json!({ "files": out.len(), "max": PERPLEXITY_MAX_FILES }),
        ));
    }
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::{consolidate, parse, Category, SkillMap, LINK};
    use crate::bundle::{Member, Members};

    fn map(cats: &[(&str, &[&str])]) -> SkillMap {
        SkillMap {
            passthrough: vec!["ATTRIBUTION.md".to_owned()],
            category: cats
                .iter()
                .map(|(slug, tools)| Category {
                    slug: (*slug).to_owned(),
                    title: slug.to_uppercase(),
                    tools: tools.iter().map(|t| (*t).to_owned()).collect(),
                })
                .collect(),
        }
    }

    fn tree(skill_md: &str, tools: &[(&str, &str)]) -> Members {
        let mut m = Members::new();
        m.insert("SKILL.md".to_owned(), Member::text(skill_md));
        m.insert("LICENSE".to_owned(), Member::text("gpl"));
        m.insert(
            "references/ATTRIBUTION.md".to_owned(),
            Member::text("# A\n"),
        );
        for (t, body) in tools {
            m.insert(format!("references/{t}.md"), Member::text(*body));
        }
        m
    }

    fn text(m: &Members, k: &str) -> String {
        String::from_utf8(m[k].bytes.clone()).unwrap()
    }

    #[test]
    fn link_regex_matches_build_py_cases() {
        let r = |s: &str| {
            LINK.replace_all(s, |c: &regex::Captures<'_>| format!("<{}>", &c[1]))
                .into_owned()
        };
        assert_eq!(r("see references/eza.md)"), "see <eza>)");
        assert_eq!(r("references/eza.mdx"), "<eza>x");
        assert_eq!(r("references/a.md.md"), "<a.md>");
    }

    #[test]
    fn consolidates_sorted_demotes_outside_fences_and_rewrites_links() {
        let m = tree(
            "---\nname: s\n---\nUse references/zz.md and references/aa.md, references/ATTRIBUTION.md.\n",
            &[
                ("zz", "# ZZ (z)\n## Usage\n```sh\n# not-a-heading\n```\n~~~\n## fenced\n~~~\n"),
                ("aa", "# AA\ntext\n\n\n"),
            ],
        );
        let out = consolidate("s", &m, &map(&[("cat", &["zz", "aa"])])).expect("ok");
        assert!(!out.contains_key("references/zz.md"));
        assert!(out.contains_key("LICENSE"));
        assert!(out.contains_key("references/ATTRIBUTION.md"));
        assert_eq!(
            text(&out, "references/cat.md"),
            "# CAT\n\n## aa\n\n**AA**\n\ntext\n\n\n\n## zz\n\n**ZZ (z)**\n\n### Usage\n```sh\n# not-a-heading\n```\n~~~\n## fenced\n~~~\n"
        );
        assert_eq!(
            text(&out, "SKILL.md"),
            "---\nname: s\n---\nUse references/cat.md#zz and references/cat.md#aa, references/ATTRIBUTION.md.\n"
        );
    }

    #[test]
    fn out_of_sync_duplicate_and_missing_h1_are_reported() {
        let m = tree("x", &[("aa", "# AA\n"), ("bb", "no h1\n")]);
        let err = consolidate("s", &m, &map(&[("c", &["aa", "cc"])])).unwrap_err();
        assert_eq!(err[0].detail["kind"], "map_out_of_sync");
        assert_eq!(err[0].detail["detail"]["in_map_not_on_disk"][0], "cc");
        assert_eq!(err[0].detail["detail"]["on_disk_not_mapped"][0], "bb");
        let dup = consolidate("s", &m, &map(&[("c", &["aa", "bb"]), ("d", &["bb"])])).unwrap_err();
        assert!(dup.iter().any(|p| p.detail["kind"] == "duplicate_tool"));
        let h1 = consolidate("s", &m, &map(&[("c", &["aa", "bb"])])).unwrap_err();
        assert_eq!(h1[0].detail["kind"], "missing_h1");
    }

    #[test]
    fn dangling_anchor_fails_the_self_check() {
        let m = tree("references/c.md#ghost\n", &[("aa", "# AA\n")]);
        let err = consolidate("s", &m, &map(&[("c", &["aa"])])).unwrap_err();
        assert_eq!(err[0].detail["kind"], "dangling_anchor");
    }

    #[test]
    fn unknown_toml_fields_and_schemas_are_rejected() {
        assert!(parse("schema = 1\nbogus = 2\n").is_err());
        assert!(parse("schema = 2\n").is_err());
        assert!(parse("schema = 1\n[skill.x]\ncategory = []\n").is_ok());
    }
}
