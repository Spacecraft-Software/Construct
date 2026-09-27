// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Gemini app target: what the Gemini skill upload accepts.
//!
//! Every rule here was established by a real upload error, not by
//! documentation:
//!
//! - `SKILL.md` must sit at the zip root ("One of the uploaded files must be a
//!   SKILL.md file"), so the layout is flat, as for Grok.
//! - Only [`ALLOWED_EXTENSIONS`] are accepted ("unsupported file type"). Any
//!   other member gets `.txt` appended — `LICENSE` ships as `LICENSE.txt`,
//!   `assets/steelbore.toml` as `assets/steelbore.toml.txt` — and every
//!   markdown link inside the skill that points at a renamed member is
//!   rewritten to the new name. Only links change: a code span or prose that
//!   names a file, and any link that leaves the skill, is left as written.
//! - The per-skill file count is limited ("too many files"; the exact number
//!   is undocumented). A bundle over [`GEMINI_MAX_FILES`] is refused.
//!
//! The `name` rule (`^[a-z0-9]+(-[a-z0-9]+)*$`) is enforced before any target
//! runs, by the frontmatter identity check every root skill passes.

use std::collections::BTreeMap;

use serde_json::json;

use super::single::{self, Fence};
use super::{Member, Members, Problem, SKILL_MD};

/// The only file extensions the Gemini app accepts in a skill upload.
pub(crate) const ALLOWED_EXTENSIONS: &[&str] = &["csv", "py", "txt", "md"];

/// The per-bundle file limit this target enforces. The platform's own limit
/// is undocumented; 100 matches Perplexity's and is known to upload.
pub(crate) const GEMINI_MAX_FILES: usize = 100;

/// The suffix appended to a member whose extension the platform refuses.
const PLATFORM_SUFFIX: &str = ".txt";

/// Whether the file name at the end of `path` carries an allowed extension.
/// Matching is exact (lowercase): an upper-case `.MD` is renamed too, which
/// costs nothing and never risks a refusal.
pub(crate) fn allowed(path: &str) -> bool {
    let file = path.rsplit_once('/').map_or(path, |(_, f)| f);
    file.rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && ALLOWED_EXTENSIONS.contains(&ext))
}

/// The in-bundle path of a source member: unchanged when allowed, otherwise
/// with `.txt` appended.
///
/// License files follow the same rule — `LICENSE` becomes `LICENSE.txt` and
/// `LICENSE.MIT` becomes `LICENSE.MIT.txt`. Standard §4.3's "no extension"
/// rule governs license files in a repository; this is a platform bundle,
/// and Standard §5.6 license carriage (the text ships in every bundle) is
/// what it must keep. The text itself is byte-identical.
pub(crate) fn platform_path(path: &str) -> String {
    if allowed(path) {
        path.to_owned()
    } else {
        format!("{path}{PLATFORM_SUFFIX}")
    }
}

/// A Gemini refusal (`gemini` extension, `{skill, kind, detail}`).
fn problem(skill: &str, kind: &str, message: String, detail: &serde_json::Value) -> Problem {
    Problem::conflict(
        "gemini",
        message,
        format!("construct skill build {skill} --target gemini --dry-run --verbose"),
        json!({ "skill": skill, "kind": kind, "detail": detail }),
    )
}

/// Rename every member the platform refuses, rewrite the links that pointed
/// at one, and enforce the file limit. `members` carry the projected
/// `SKILL.md` and any consolidation already applied.
///
/// # Errors
///
/// `rename_collision` when a renamed path already exists in the skill, and
/// `too_many_files` when the bundle is still over [`GEMINI_MAX_FILES`].
pub(crate) fn transform(skill: &str, members: &Members) -> Result<Members, Vec<Problem>> {
    let renames: BTreeMap<&str, String> = members
        .keys()
        .filter(|k| !allowed(k))
        .map(|k| (k.as_str(), platform_path(k)))
        .collect();

    let mut problems = Vec::new();
    for (from, to) in &renames {
        if members.contains_key(to) {
            problems.push(problem(
                skill,
                "rename_collision",
                format!(
                    "{skill}: {from} would be renamed to {to}, which the skill already carries"
                ),
                &json!({ "from": from, "to": to }),
            ));
        }
    }
    if members.len() > GEMINI_MAX_FILES {
        problems.push(problem(
            skill,
            "too_many_files",
            format!(
                "{skill}: the gemini bundle has {} files, over the limit of {GEMINI_MAX_FILES}",
                members.len()
            ),
            &json!({ "files": members.len(), "max": GEMINI_MAX_FILES }),
        ));
    }
    if !problems.is_empty() {
        return Err(problems);
    }

    let mut out = Members::new();
    for (path, member) in members {
        let key = renames
            .get(path.as_str())
            .cloned()
            .unwrap_or_else(|| path.clone());
        let member = if key.rsplit_once('.').map(|(_, e)| e) == Some("md") && !renames.is_empty() {
            match std::str::from_utf8(&member.bytes) {
                Ok(text) => {
                    let rewritten = if path == SKILL_MD {
                        rewrite_skill_md(text, &renames)
                    } else {
                        rewrite_links(text, single::parent(path), &renames)
                    };
                    Member {
                        bytes: rewritten.into_bytes(),
                        mode: member.mode,
                    }
                }
                Err(_) => member.clone(),
            }
        } else {
            member.clone()
        };
        out.insert(key, member);
    }
    Ok(out)
}

/// Rewrite `SKILL.md`'s body only: the frontmatter (and so the description)
/// stays byte-identical.
fn rewrite_skill_md(text: &str, renames: &BTreeMap<&str, String>) -> String {
    let split = text
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n").map(|i| 4 + i + 5));
    match split {
        Some(at) => {
            let mut out = text[..at].to_owned();
            out.push_str(&rewrite_links(&text[at..], "", renames));
            out
        }
        None => rewrite_links(text, "", renames),
    }
}

/// Rewrite every inline link and reference definition outside fenced code
/// whose destination resolves (against `base`) to a renamed member.
pub(crate) fn rewrite_links(text: &str, base: &str, renames: &BTreeMap<&str, String>) -> String {
    let mut out = String::with_capacity(text.len() + 16);
    let mut fence: Option<Fence> = None;
    for line in text.split_inclusive('\n') {
        let stripped = line.trim_end_matches(['\n', '\r']);
        if let Some(f) = fence {
            if f.closed_by(stripped) {
                fence = None;
            }
            out.push_str(line);
            continue;
        }
        if let Some(f) = Fence::open(stripped) {
            fence = Some(f);
            out.push_str(line);
            continue;
        }
        if let Some((s, e)) = single::definition(stripped) {
            match rewrite_url(base, &stripped[s..e], renames) {
                Some(url) => {
                    out.push_str(&line[..s]);
                    out.push_str(&url);
                    out.push_str(&line[e..]);
                }
                None => out.push_str(line),
            }
            continue;
        }
        out.push_str(&rewrite_line(line, base, renames));
    }
    out
}

/// Rewrite the inline links of one line (or of a link's text). Code spans are
/// copied verbatim.
fn rewrite_line(line: &str, base: &str, renames: &BTreeMap<&str, String>) -> String {
    let spans = single::code_spans(line);
    let b = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut plain = 0;
    let mut i = 0;
    while i < b.len() {
        if let Some(e) = single::span_end(&spans, i) {
            i = e;
            continue;
        }
        match b[i] {
            b'\\' if b.get(i + 1) != Some(&b'`') => i += 2,
            b'[' => match single::parse_inline(line, i, &spans) {
                Some(l) => {
                    out.push_str(&line[plain..i]);
                    let text = &line[l.text.0..l.text.1];
                    let inner = &line[l.inner.0..l.inner.1];
                    let (url, title) = match inner.find(char::is_whitespace) {
                        Some(ws) => (&inner[..ws], &inner[ws..]),
                        None => (inner, ""),
                    };
                    out.push('[');
                    // An image inside a link's text is a link too.
                    out.push_str(&rewrite_line(text, base, renames));
                    out.push_str("](");
                    match rewrite_url(base, url, renames) {
                        Some(new) => {
                            out.push_str(&new);
                            out.push_str(title);
                        }
                        None => out.push_str(inner),
                    }
                    out.push(')');
                    i = l.end;
                    plain = l.end;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
    out.push_str(&line[plain.min(line.len())..]);
    out
}

/// The rewritten destination when `url` resolves inside the skill to a
/// renamed member; `None` leaves it untouched (external, fragment-only,
/// absolute, outside the skill, or not renamed). Only the path gains the
/// suffix: angle brackets and a `#fragment` are kept.
fn rewrite_url(base: &str, url: &str, renames: &BTreeMap<&str, String>) -> Option<String> {
    let (open, inner, close) = match url.strip_prefix('<').and_then(|u| u.strip_suffix('>')) {
        Some(u) => ("<", u, ">"),
        None => ("", url, ""),
    };
    let (path, frag) = inner
        .split_once('#')
        .map_or((inner, None), |(p, f)| (p, Some(f)));
    if path.is_empty() || path.starts_with('/') || single::is_external(inner) {
        return None;
    }
    let resolved = single::resolve(base, path)?;
    let to = renames.get(resolved.as_str())?;
    let suffix = &to[resolved.len()..];
    let frag = frag.map_or_else(String::new, |f| format!("#{f}"));
    Some(format!("{open}{path}{suffix}{frag}{close}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renames(pairs: &[&'static str]) -> BTreeMap<&'static str, String> {
        pairs.iter().map(|p| (*p, platform_path(p))).collect()
    }

    #[test]
    fn extension_rule() {
        assert!(allowed("SKILL.md"));
        assert!(allowed("references/a.py"));
        assert!(allowed("data/t.csv"));
        assert!(!allowed("LICENSE"));
        assert!(!allowed("assets/steelbore.toml"));
        assert!(!allowed("assets/.md"));
        assert!(!allowed("x.MD"));
        assert_eq!(platform_path("LICENSE"), "LICENSE.txt");
        assert_eq!(platform_path("LICENSE.MIT"), "LICENSE.MIT.txt");
        assert_eq!(platform_path("assets/build.sh"), "assets/build.sh.txt");
        assert_eq!(platform_path("references/r.md"), "references/r.md");
    }

    #[test]
    fn rewrites_links_only() {
        let r = renames(&["assets/x.toml", "LICENSE"]);
        let text = "See [x](../assets/x.toml#top), `../assets/x.toml`, [l](<../LICENSE>).\n\
                    [other](../../palette/assets/x.toml) and [ext](https://h/assets/x.toml)\n\
                    [def]: ../assets/x.toml \"t\"\n\
                    ```\n[x](../assets/x.toml)\n```\n\
                    [![img](../assets/x.toml)](../LICENSE)\n";
        let out = rewrite_links(text, "references", &r);
        assert_eq!(
            out,
            "See [x](../assets/x.toml.txt#top), `../assets/x.toml`, [l](<../LICENSE.txt>).\n\
             [other](../../palette/assets/x.toml) and [ext](https://h/assets/x.toml)\n\
             [def]: ../assets/x.toml.txt \"t\"\n\
             ```\n[x](../assets/x.toml)\n```\n\
             [![img](../assets/x.toml.txt)](../LICENSE.txt)\n"
        );
    }

    #[test]
    fn skill_md_frontmatter_untouched() {
        let r = renames(&["LICENSE"]);
        let text = "---\nname: a\ndescription: see [l](LICENSE)\n---\n[l](LICENSE)\n";
        assert_eq!(
            rewrite_skill_md(text, &r),
            "---\nname: a\ndescription: see [l](LICENSE)\n---\n[l](LICENSE.txt)\n"
        );
    }

    #[test]
    fn collision_and_limit_refused() {
        let mut m = Members::new();
        m.insert("LICENSE".to_owned(), Member::text("a"));
        m.insert("LICENSE.txt".to_owned(), Member::text("b"));
        let e = transform("s", &m).unwrap_err();
        assert_eq!(e[0].detail["kind"], "rename_collision");

        let mut m = Members::new();
        for i in 0..=GEMINI_MAX_FILES {
            m.insert(format!("references/{i}.md"), Member::text("x"));
        }
        let e = transform("s", &m).unwrap_err();
        assert_eq!(e[0].detail["kind"], "too_many_files");
    }
}
