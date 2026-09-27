// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The single-file target: one self-contained `<name>.md` per skill, for
//! platforms with no skill loader (Gemini Gems, `MiniMax`).
//!
//! Layout, in fixed order: a short YAML header (`name`, `description`,
//! `license`, raw source blocks), the `SKILL.md` body, every `references/**`
//! file, every `assets/**` file, `CREDITS.md`, then a closing `## License`
//! section. Each inlined file gets an explicit `<a id="ref-…">` anchor and a
//! `## <relative path>` heading. Markdown references are inlined with
//! headings demoted one level (capped at `######`); every other text file
//! goes in a fenced block one backtick longer than the longest run it
//! contains. Binary files are skipped and reported.
//!
//! The document is the unit of distribution on these platforms, so it
//! carries its license text (Standard §5.6): every root `LICENSE` and
//! `LICENSE.<TAG>` file is appended verbatim under `## License`, one
//! `### <file>` subsection each, in a fenced `text` block. A dual-licensed
//! skill therefore ships both texts — the MIT permission notice included.
//!
//! Links are rewritten fence-aware in the body and every inlined markdown
//! file, so no relative link survives outside a fenced block:
//!
//! - a link (or reference definition) that resolves to an inlined member, or
//!   an inline code span whose entire content is such a path, points at its
//!   anchor. A `#frag` on such a link is dropped (inlined headings are
//!   demoted, so the original slug is not trustworthy) and counted. Code spans
//!   also resolve relative to the skill root, because prose names files
//!   root-relative (`` `references/x.md` ``) wherever it sits;
//! - a link back to the skill's own `SKILL.md` (`../SKILL.md` from a
//!   reference) points at the body's `<a id="ref-skill-md">` anchor;
//! - a link to a sibling catalogue skill (`../<skill>/SKILL.md`,
//!   `../<skill>/`) becomes plain text naming it: the link text alone when it
//!   already is the skill's name, otherwise `text (the `<skill>` skill)`;
//! - every other relative link — a binary asset, an upstream path such as
//!   `../libs/ux` — keeps its text and loses the link, and a
//!   reference definition that would dangle is dropped. Both are counted.
//!
//! External (`scheme:`), pure-fragment, and absolute links are left alone.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde_json::json;

use super::consolidate::heading_level;
use super::{Members, Problem, SKILL_MD};
use crate::sources::skillmd;

/// A rendered single-file document.
#[derive(Debug)]
pub(crate) struct Rendered {
    /// The document text.
    pub(crate) text: String,
    /// Inlined sections (the body counts as one).
    pub(crate) sections: usize,
    /// Link fragments dropped by rewriting.
    pub(crate) fragments_dropped: usize,
    /// Relative links with no target in the document, reduced to text.
    pub(crate) unlinked: usize,
    /// Members left out because they are binary.
    pub(crate) skipped: Vec<String>,
}

/// How a member is inlined.
#[derive(Debug)]
enum Kind {
    Markdown,
    Fenced(&'static str),
}

/// The anchor id of an inlined path: `ref-` plus the lowercased path with
/// every character outside `[a-z0-9]` mapped to `-`.
pub(crate) fn anchor(path: &str) -> String {
    let mapped: String = path
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("ref-{mapped}")
}

/// Whether a member is a root license file (`LICENSE` or `LICENSE.<TAG>`).
fn is_license(path: &str) -> bool {
    path == "LICENSE" || (path.starts_with("LICENSE.") && !path.contains('/'))
}

/// Append `text` in a fenced block one backtick longer than its longest run.
fn push_fenced(doc: &mut String, info: &str, text: &str) {
    let fence = fence_for(text);
    let _ = write!(doc, "{fence}{info}\n{text}");
    if !text.ends_with('\n') {
        doc.push('\n');
    }
    doc.push_str(&fence);
    doc.push('\n');
}

/// The fenced-block info string for a path's extension.
fn info_string(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, ext)| ext) {
        Some("toml") => "toml",
        Some("json") => "json",
        Some("css") => "css",
        Some("sh") => "sh",
        Some("md") => "markdown",
        Some("texi") => "texinfo",
        Some("scm") => "scheme",
        _ => "text",
    }
}

/// A backtick fence one longer than the longest backtick run in `content`,
/// and at least three.
fn fence_for(content: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in content.chars() {
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat((longest + 1).max(3))
}

/// Resolve `rel` against the directory `base` (`""` for the skill root),
/// normalizing `.` and `..`. `None` when it escapes the skill root or is
/// absolute.
fn resolve(base: &str, rel: &str) -> Option<String> {
    if rel.starts_with('/') || rel.is_empty() {
        return None;
    }
    let mut parts: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

/// Whether a link target is external (`scheme:`) rather than a path.
fn is_external(target: &str) -> bool {
    let Some((scheme, _)) = target.split_once(':') else {
        return false;
    };
    let mut bytes = scheme.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'.' | b'-'))
}

/// A line split into text and inline-code segments.
fn segments(line: &str) -> Vec<(bool, &str)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run_start = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let n = i - run_start;
        // Find a closing run of exactly `n` backticks.
        let mut j = i;
        let mut close = None;
        while j < bytes.len() {
            if bytes[j] == b'`' {
                let s = j;
                while j < bytes.len() && bytes[j] == b'`' {
                    j += 1;
                }
                if j - s == n {
                    close = Some(j);
                    break;
                }
            } else {
                j += 1;
            }
        }
        if let Some(end) = close {
            if run_start > start {
                out.push((false, &line[start..run_start]));
            }
            out.push((true, &line[run_start..end]));
            start = end;
            i = end;
        }
    }
    if start < line.len() {
        out.push((false, &line[start..]));
    }
    out
}

/// The byte ranges `[start, end)` of every inline code span in a line.
fn code_spans(line: &str) -> Vec<(usize, usize)> {
    let mut at = 0;
    let mut out = Vec::new();
    for (code, seg) in segments(line) {
        if code {
            out.push((at, at + seg.len()));
        }
        at += seg.len();
    }
    out
}

/// The end of the code span starting exactly at `i`, if one does.
fn span_end(spans: &[(usize, usize)], i: usize) -> Option<usize> {
    spans
        .binary_search_by_key(&i, |(s, _)| *s)
        .ok()
        .map(|k| spans[k].1)
}

/// An inline link `[text](inner)` found in a line.
struct Inline {
    /// Byte range of the link text (between the brackets).
    text: (usize, usize),
    /// Byte range of the destination and optional title (between the parens).
    inner: (usize, usize),
    /// One past the closing `)`.
    end: usize,
}

/// Parse an inline link whose `[` sits at `open`. Code spans are opaque
/// inside the link text, brackets and parentheses nest, and `\` escapes the
/// next byte. `None` when the brackets are not an inline link.
fn parse_inline(line: &str, open: usize, spans: &[(usize, usize)]) -> Option<Inline> {
    let b = line.as_bytes();
    let mut j = open + 1;
    let mut depth = 0usize;
    let close_bracket = loop {
        if j >= b.len() {
            return None;
        }
        if let Some(e) = span_end(spans, j) {
            j = e;
            continue;
        }
        match b[j] {
            b'\\' if b.get(j + 1) != Some(&b'`') => j += 2,
            b'[' => {
                depth += 1;
                j += 1;
            }
            b']' if depth == 0 => break j,
            b']' => {
                depth -= 1;
                j += 1;
            }
            _ => j += 1,
        }
    };
    if b.get(close_bracket + 1) != Some(&b'(') {
        return None;
    }
    let start = close_bracket + 2;
    let mut k = start;
    let mut depth = 0usize;
    let close_paren = loop {
        if k >= b.len() {
            return None;
        }
        match b[k] {
            b'\\' if b.get(k + 1) != Some(&b'`') => k += 2,
            b'(' => {
                depth += 1;
                k += 1;
            }
            b')' if depth == 0 => break k,
            b')' => {
                depth -= 1;
                k += 1;
            }
            _ => k += 1,
        }
    };
    Some(Inline {
        text: (open + 1, close_bracket),
        inner: (start, close_paren),
        end: close_paren + 1,
    })
}

/// The byte range of the destination in a reference definition line
/// (`[label]: target "title"`), or `None` when the line is not one.
/// Footnote definitions (`[^n]: …`) are not links and are not matched.
fn definition(line: &str) -> Option<(usize, usize)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent..];
    if indent > 3 || !rest.starts_with('[') || rest.starts_with("[^") {
        return None;
    }
    let close = rest.find("]:")?;
    if close < 2 || rest[1..close].contains(['[', ']']) {
        return None;
    }
    let after = &rest[close + 2..];
    let start = indent + close + 2 + (after.len() - after.trim_start().len());
    let len = line[start..]
        .find(char::is_whitespace)
        .unwrap_or(line.len() - start);
    (len > 0).then_some((start, start + len))
}

/// A fenced code block's opening run: its character and length.
#[derive(Clone, Copy)]
struct Fence {
    ch: u8,
    len: usize,
}

impl Fence {
    /// The fence run a line opens with (≤ 3 spaces of indent, ≥ 3 of one
    /// character). A backtick fence's info string may not hold a backtick.
    fn open(line: &str) -> Option<Self> {
        let rest = line.trim_start_matches(' ');
        if line.len() - rest.len() > 3 {
            return None;
        }
        let ch = *rest
            .as_bytes()
            .first()
            .filter(|c| matches!(c, b'`' | b'~'))?;
        let len = rest.bytes().take_while(|b| *b == ch).count();
        let valid = len >= 3 && !(ch == b'`' && rest[len..].contains('`'));
        valid.then_some(Self { ch, len })
    }

    /// Whether `line` closes this fence: the same character, a run at least
    /// as long, and nothing after it but whitespace. A line with an info
    /// string (```` ```sh ````) never closes a fence.
    fn closed_by(self, line: &str) -> bool {
        Self::open(line).is_some_and(|f| {
            f.ch == self.ch
                && f.len >= self.len
                && line.trim_start_matches(' ')[f.len..].trim().is_empty()
        })
    }
}

/// Where a relative link lands once the skill is one document.
enum Dest<'a> {
    /// An inlined member, or the body itself: link to its anchor.
    Anchor(&'a str),
    /// A sibling skill in the catalogue: name it in plain text.
    Sibling(String),
    /// Nothing in the document: keep the link text, drop the link.
    Gone,
}

/// Link-rewriting state for one document.
struct Rewriter<'a> {
    /// The skill being rendered.
    skill: &'a str,
    /// Inlined path → anchor id (`SKILL.md` maps to the body's anchor).
    anchors: &'a BTreeMap<String, String>,
    /// Every root skill in the catalogue.
    siblings: &'a BTreeSet<String>,
    /// Fragments dropped from links retargeted at an anchor.
    dropped: usize,
    /// Relative links reduced to plain text.
    unlinked: usize,
}

impl<'a> Rewriter<'a> {
    /// Classify a link destination. `None` leaves it untouched: external
    /// (`scheme:`), a pure `#fragment`, or an absolute path.
    fn dest(&self, base: &str, url: &str) -> Option<Dest<'a>> {
        let url = url
            .strip_prefix('<')
            .and_then(|u| u.strip_suffix('>'))
            .unwrap_or(url);
        let path = url.split_once('#').map_or(url, |(p, _)| p);
        if path.is_empty() || path.starts_with('/') || is_external(url) {
            return None;
        }
        let anchors: &'a BTreeMap<String, String> = self.anchors;
        if let Some(a) = resolve(base, path).and_then(|p| anchors.get(&p)) {
            return Some(Dest::Anchor(a));
        }
        match self.sibling(base, path) {
            Some(name) if name == self.skill => anchors.get(SKILL_MD).map(|a| Dest::Anchor(a)),
            Some(name) => Some(Dest::Sibling(name.to_owned())),
            None => Some(Dest::Gone),
        }
    }

    /// The catalogue skill `rel` names when it climbs exactly one level out
    /// of the skill root: `../<skill>`, `../<skill>/`, or
    /// `../<skill>/SKILL.md` (from `references/`, one more `../`).
    fn sibling<'p>(&self, base: &str, rel: &'p str) -> Option<&'p str> {
        let mut depth = base.split('/').filter(|s| !s.is_empty()).count();
        let mut ups = 0usize;
        let mut parts: Vec<&str> = Vec::new();
        for seg in rel.split('/') {
            match seg {
                "" | "." => {}
                ".." if parts.pop().is_some() => {}
                ".." if depth > 0 => depth -= 1,
                ".." => ups += 1,
                s => parts.push(s),
            }
        }
        match (ups, parts.as_slice()) {
            (1, [name] | [name, SKILL_MD]) if self.siblings.contains(*name) => Some(name),
            _ => None,
        }
    }

    /// Emit one inline link, rewritten for the single document.
    fn link(&mut self, line: &str, image: bool, l: &Inline, base: &str, out: &mut String) {
        let text = &line[l.text.0..l.text.1];
        let inner = &line[l.inner.0..l.inner.1];
        let (url, title) = match inner.find(char::is_whitespace) {
            Some(ws) => (&inner[..ws], &inner[ws..]),
            None => (inner, ""),
        };
        let label = self.line(text, base, true);
        match self.dest(base, url) {
            None => {
                if image {
                    out.push('!');
                }
                let _ = write!(out, "[{label}]({inner})");
            }
            // An image of an inlined text file becomes a plain link to it.
            Some(Dest::Anchor(a)) => {
                if url.contains('#') {
                    self.dropped += 1;
                }
                let _ = write!(out, "[{label}](#{a}{title})");
            }
            Some(Dest::Sibling(name)) => {
                self.unlinked += 1;
                if label.trim_matches('`') == name {
                    out.push_str(&label);
                } else {
                    let _ = write!(out, "{label} (the `{name}` skill)");
                }
            }
            Some(Dest::Gone) => {
                self.unlinked += 1;
                out.push_str(&label);
            }
        }
    }

    /// Emit one inline code span; a span whose whole content is a path to an
    /// inlined member becomes a link to it (never inside link text, and
    /// never for `SKILL.md`, which prose names generically).
    fn code_span(&self, line: &str, (s, e): (usize, usize), base: &str, in_link: bool) -> String {
        let seg = &line[s..e];
        let ticks = seg.bytes().take_while(|b| *b == b'`').count();
        let content = &seg[ticks..seg.len() - ticks];
        let bracketed = line[..s].ends_with('[') || line[e..].starts_with(']');
        if in_link || bracketed || content.contains(char::is_whitespace) {
            return seg.to_owned();
        }
        let lookup = |b: &str| {
            resolve(b, content)
                .filter(|p| p != SKILL_MD)
                .and_then(|p| self.anchors.get(&p))
        };
        match lookup(base).or_else(|| lookup("")) {
            Some(a) => format!("[{seg}](#{a})"),
            None => seg.to_owned(),
        }
    }

    /// Rewrite one non-fenced line (or a link text, when `in_link`).
    fn line(&mut self, line: &str, base: &str, in_link: bool) -> String {
        let spans = code_spans(line);
        let b = line.as_bytes();
        let mut out = String::with_capacity(line.len());
        let mut plain = 0;
        let mut i = 0;
        while i < b.len() {
            if let Some(e) = span_end(&spans, i) {
                out.push_str(&line[plain..i]);
                out.push_str(&self.code_span(line, (i, e), base, in_link));
                i = e;
                plain = e;
                continue;
            }
            match b[i] {
                b'\\' if b.get(i + 1) != Some(&b'`') => i += 2,
                b'!' | b'[' => {
                    let image = b[i] == b'!';
                    let open = if image { i + 1 } else { i };
                    let parsed = (b.get(open) == Some(&b'['))
                        .then(|| parse_inline(line, open, &spans))
                        .flatten();
                    if let Some(l) = parsed {
                        out.push_str(&line[plain..i]);
                        self.link(line, image, &l, base, &mut out);
                        i = l.end;
                        plain = l.end;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        out.push_str(&line[plain.min(line.len())..]);
        out
    }

    /// Rewrite a markdown document fence-aware; `demote` shifts headings
    /// outside fences down one level, capped at `######`. Fences follow
    /// `CommonMark` — the renderer's reading decides which links are live —
    /// not the open/close toggle `consolidate` keeps for `build.py` fidelity.
    /// A reference
    /// definition that resolves to an inlined member is retargeted; one that
    /// would dangle is dropped (its `[label]` then reads as plain text).
    fn document(&mut self, text: &str, base: &str, demote: bool) -> String {
        let mut out = String::with_capacity(text.len() + 64);
        let mut fence: Option<Fence> = None;
        for line in text.split_inclusive('\n') {
            let stripped = line.trim_end_matches(['\n', '\r']);
            match fence {
                Some(f) => {
                    if f.closed_by(stripped) {
                        fence = None;
                    }
                    out.push_str(line);
                    continue;
                }
                None => {
                    if let Some(f) = Fence::open(stripped) {
                        fence = Some(f);
                        out.push_str(line);
                        continue;
                    }
                }
            }
            if let Some((s, e)) = definition(stripped) {
                let url = &stripped[s..e];
                match self.dest(base, url) {
                    None => out.push_str(line),
                    Some(Dest::Anchor(a)) => {
                        if url.contains('#') {
                            self.dropped += 1;
                        }
                        let _ = write!(out, "{}#{a}{}", &line[..s], &line[e..]);
                    }
                    Some(Dest::Sibling(_) | Dest::Gone) => self.unlinked += 1,
                }
                continue;
            }
            let rewritten = self.line(line, base, false);
            if demote && heading_level(stripped).is_some_and(|n| n < 6) {
                out.push('#');
            }
            out.push_str(&rewritten);
        }
        out
    }
}

/// The parent directory of a member path (`""` at the skill root).
fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// The members a single-file document inlines, split by how they are placed.
struct Inlined<'m> {
    /// References, assets, then `CREDITS.md`, each with how it is inlined.
    sections: Vec<(&'m str, Kind, &'m str)>,
    /// Root license files, appended verbatim under `## License`.
    licenses: Vec<(&'m str, &'m str)>,
    /// Members left out because they are binary.
    skipped: Vec<String>,
}

/// Sort the members into the document's fixed section order.
fn inlined(members: &Members) -> Inlined<'_> {
    // Inlined sections in fixed order.
    let mut order: Vec<&String> = members
        .keys()
        .filter(|k| k.starts_with("references/"))
        .collect();
    order.extend(members.keys().filter(|k| k.starts_with("assets/")));
    if let Some((k, _)) = members.get_key_value("CREDITS.md") {
        order.push(k);
    }

    let mut skipped = Vec::new();
    let mut sections: Vec<(&str, Kind, &str)> = Vec::new();
    for path in order {
        let bytes = &members[path].bytes;
        let text = match std::str::from_utf8(bytes) {
            Ok(t) if !t.contains('\0') => t,
            _ => {
                skipped.push(path.clone());
                continue;
            }
        };
        let kind = if (path.starts_with("references/") || path == "CREDITS.md")
            && std::path::Path::new(path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        {
            Kind::Markdown
        } else {
            Kind::Fenced(info_string(path))
        };
        sections.push((path.as_str(), kind, text));
    }
    // License texts close the document, verbatim (§5.6 license carriage).
    let mut licenses: Vec<(&str, &str)> = Vec::new();
    for path in members.keys().filter(|k| is_license(k)) {
        match std::str::from_utf8(&members[path].bytes) {
            Ok(t) if !t.contains('\0') => licenses.push((path.as_str(), t)),
            _ => skipped.push(path.clone()),
        }
    }
    Inlined {
        sections,
        licenses,
        skipped,
    }
}

/// Render one skill. `projected` is the `SKILL.md` already projected to the
/// header profile; `members` are the collected (and palette-vendored) files;
/// `siblings` names every root skill in the catalogue, so a link to one can
/// be told apart from any other link that leaves the skill.
///
/// # Errors
///
/// `anchor_collision` when two paths map to one anchor, and
/// `single_file_dangling_anchor` when an emitted `#ref-…` link has no target.
pub(crate) fn render(
    skill: &str,
    projected: &str,
    members: &Members,
    siblings: &BTreeSet<String>,
) -> Result<Rendered, Vec<Problem>> {
    let (fm, body) = skillmd::split(projected).unwrap_or(("", projected));

    let Inlined {
        sections,
        licenses,
        skipped,
    } = inlined(members);

    let mut anchors: BTreeMap<String, String> = BTreeMap::new();
    let mut by_anchor: BTreeMap<String, &str> = BTreeMap::new();
    let mut problems = Vec::new();
    let inlined = sections
        .iter()
        .map(|(path, _, _)| *path)
        .chain(licenses.iter().map(|(path, _)| *path));
    for path in inlined {
        let a = anchor(path);
        if let Some(prev) = by_anchor.insert(a.clone(), path) {
            problems.push(Problem::conflict(
                "anchor_collision",
                format!("{skill}: {prev} and {path} both map to #{a}"),
                format!("$EDITOR {skill}/{path}"),
                json!({ "skill": skill, "anchor": a, "paths": [prev, path] }),
            ));
        }
        anchors.insert(path.to_owned(), a);
    }
    if !problems.is_empty() {
        return Err(problems);
    }

    // The body's own anchor: the target of every link back to `SKILL.md`.
    // No member can claim it — inlined paths all sit under `references/`,
    // `assets/`, or are `CREDITS.md` or a root license file.
    let top = anchor(SKILL_MD);
    anchors.insert(SKILL_MD.to_owned(), top.clone());

    let mut rw = Rewriter {
        skill,
        anchors: &anchors,
        siblings,
        dropped: 0,
        unlinked: 0,
    };
    let mut doc = String::from("---\n");
    doc.push_str(fm);
    let _ = writeln!(doc, "---\n<a id=\"{top}\"></a>");
    doc.push_str(&rw.document(body, "", false));
    if !doc.ends_with('\n') {
        doc.push('\n');
    }
    for (path, kind, text) in &sections {
        let _ = write!(doc, "\n<a id=\"{}\"></a>\n## {path}\n\n", anchors[*path]);
        match kind {
            Kind::Markdown => doc.push_str(&rw.document(text, parent(path), true)),
            Kind::Fenced(info) => push_fenced(&mut doc, info, text),
        }
        if !doc.ends_with('\n') {
            doc.push('\n');
        }
    }
    if !licenses.is_empty() {
        doc.push_str("\n## License\n");
        for (path, text) in &licenses {
            let _ = write!(doc, "\n<a id=\"{}\"></a>\n### {path}\n\n", anchors[*path]);
            push_fenced(&mut doc, "text", text);
        }
    }

    self_check(skill, &doc)?;

    Ok(Rendered {
        text: doc,
        sections: sections.len() + licenses.len() + 1,
        fragments_dropped: rw.dropped,
        unlinked: rw.unlinked,
        skipped,
    })
}

/// Every emitted `](#ref-…)` link must have its `<a id>` in the same
/// document (the single-file mirror of `build.py`'s anchor self-check).
fn self_check(skill: &str, doc: &str) -> Result<(), Vec<Problem>> {
    let mut dangling: Vec<String> = Vec::new();
    let mut rest = doc;
    while let Some(i) = rest.find("](#ref-") {
        rest = &rest[i + 3..];
        let end = rest.find(')').unwrap_or(rest.len());
        let id = &rest[..end];
        if !doc.contains(&format!("<a id=\"{id}\"></a>")) && !dangling.iter().any(|d| d == id) {
            dangling.push(id.to_owned());
        }
    }
    if dangling.is_empty() {
        return Ok(());
    }
    Err(vec![Problem::conflict(
        "single_file_dangling_anchor",
        format!(
            "{skill}: single-file links point at missing anchors: {}",
            dangling.join(", ")
        ),
        format!("$EDITOR {skill}/{SKILL_MD}"),
        json!({ "skill": skill, "anchors": dangling }),
    )])
}

/// The content of the fenced block under the `<a id="{anchor}">` section, or
/// `None` when the section or its fence is missing.
pub(crate) fn extract_fenced<'a>(doc: &'a str, anchor_id: &str) -> Option<&'a str> {
    let marker = format!("<a id=\"{anchor_id}\"></a>\n");
    let start = doc.find(&marker)? + marker.len();
    let rest = &doc[start..];
    // `## path\n\n` then the opening fence line.
    let rest = &rest[rest.find("\n\n")? + 2..];
    let (open, after) = rest.split_once('\n')?;
    let fence: String = open.chars().take_while(|c| *c == '`').collect();
    if fence.len() < 3 {
        return None;
    }
    let mut offset = 0;
    for line in after.split_inclusive('\n') {
        if line.trim_end_matches('\n') == fence {
            return Some(&after[..offset]);
        }
        offset += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{anchor, extract_fenced, fence_for, render, resolve};
    use crate::bundle::{Member, Members};

    fn siblings(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| (*n).to_owned()).collect()
    }

    fn members(files: &[(&str, &str)]) -> Members {
        files
            .iter()
            .map(|(k, v)| ((*k).to_owned(), Member::text(*v)))
            .collect()
    }

    #[test]
    fn fence_outgrows_the_longest_backtick_run() {
        assert_eq!(fence_for("plain"), "```");
        assert_eq!(fence_for("a ```` b"), "`````");
    }

    #[test]
    fn resolve_normalizes_and_refuses_escapes() {
        assert_eq!(
            resolve("references", "../assets/x.json").as_deref(),
            Some("assets/x.json")
        );
        assert_eq!(
            resolve("references", "./other.md").as_deref(),
            Some("references/other.md")
        );
        assert_eq!(resolve("", "../other/SKILL.md"), None);
    }

    #[test]
    fn renders_sections_rewrites_links_and_leaves_fences_alone() {
        let body = "\n# Title\n\nSee [guide](references/guide.md#usage) and `references/guide.md`.\n[ext](https://x.org/references/guide.md)\n```\n[in](references/guide.md)\n```\n";
        let m = members(&[
            ("SKILL.md", "unused"),
            ("LICENSE", "gpl"),
            (
                "references/guide.md",
                "# Guide\n\nUses [x](../assets/x.json).\n###### deep\n",
            ),
            ("assets/x.json", "{\"a\": \"```\"}"),
            ("CREDITS.md", "# Credits\n"),
        ]);
        let projected = format!("---\nname: s\ndescription: d\nlicense: GPL\n---\n{body}");
        let r = render("s", &projected, &m, &siblings(&["s"])).expect("renders");
        let t = &r.text;
        assert!(t.starts_with(
            "---\nname: s\ndescription: d\nlicense: GPL\n---\n<a id=\"ref-skill-md\"></a>\n\n# Title\n"
        ));
        assert!(t.contains("See [guide](#ref-references-guide-md) and [`references/guide.md`](#ref-references-guide-md)."));
        assert!(t.contains("[ext](https://x.org/references/guide.md)"));
        assert!(t.contains("```\n[in](references/guide.md)\n```\n"));
        assert!(t.contains("<a id=\"ref-references-guide-md\"></a>\n## references/guide.md\n\n## Guide\n\nUses [x](#ref-assets-x-json).\n###### deep\n"));
        assert!(t.contains("## assets/x.json\n\n````json\n{\"a\": \"```\"}\n````\n"));
        assert!(t.contains("## CREDITS.md\n\n## Credits\n"));
        assert!(
            t.ends_with(
                "\n## License\n\n<a id=\"ref-license\"></a>\n### LICENSE\n\n```text\ngpl\n```\n"
            ),
            "{t}"
        );
        assert_eq!(r.fragments_dropped, 1);
        assert_eq!(
            extract_fenced(t, "ref-assets-x-json"),
            Some("{\"a\": \"```\"}\n")
        );
    }

    #[test]
    fn every_license_file_closes_the_document_and_links_anchor_there() {
        let body =
            "\nSee [`LICENSE.GPL`](LICENSE.GPL), `LICENSE.MIT`, and [both](./LICENSE.MIT).\n";
        let m = members(&[
            ("LICENSE.GPL", "GNU GENERAL PUBLIC LICENSE\n"),
            (
                "LICENSE.MIT",
                "Permission is hereby granted, free of charge",
            ),
            ("references/r.md", "# R\n\nSee [mit](../LICENSE.MIT).\n"),
            ("CREDITS.md", "# Credits\n"),
        ]);
        let projected = format!("---\nname: s\nlicense: GPL-3.0-or-later OR MIT\n---\n{body}");
        let r = render("s", &projected, &m, &siblings(&["s"])).expect("renders");
        let t = &r.text;
        assert!(t.contains(concat!(
            "See [`LICENSE.GPL`](#ref-license-gpl), [`LICENSE.MIT`](#ref-license-mit), ",
            "and [both](#ref-license-mit).\n",
        )));
        assert!(t.contains("See [mit](#ref-license-mit).\n"));
        assert!(t.ends_with(concat!(
            "## CREDITS.md\n\n## Credits\n",
            "\n## License\n",
            "\n<a id=\"ref-license-gpl\"></a>\n### LICENSE.GPL\n\n",
            "```text\nGNU GENERAL PUBLIC LICENSE\n```\n",
            "\n<a id=\"ref-license-mit\"></a>\n### LICENSE.MIT\n\n",
            "```text\nPermission is hereby granted, free of charge\n```\n",
        )));
        assert_eq!(r.unlinked, 0);
        assert_eq!(r.sections, 5);
        assert_eq!(
            extract_fenced(t, "ref-license-mit"),
            Some("Permission is hereby granted, free of charge\n")
        );
    }

    #[test]
    fn relative_links_never_dangle() {
        let body = concat!(
            "\n# Top\n\n",
            "Per [The Standard](../std/SKILL.md) and [`fmt`](../fmt/SKILL.md) and [palette](../pal/).\n",
            "See [`LICENSE`](../LICENSE), [ux](../libs/ux), [docs](../docs/#M-X), ",
            "![alt](pic.png), [ghost](references/none.md), [me](../s/SKILL.md#top).\n",
            "Keep [web](https://x.org/a.md), [here](#local), [abs](/etc/x), and `SKILL.md`.\n",
            "```\n[fenced](../std/SKILL.md)\n```\n",
        );
        let reference = concat!(
            "# Ref\n\nRead [`../SKILL.md`](../SKILL.md) first; ",
            "[`spacecraft-x`](../../std/SKILL.md) too, and [lic](../../LICENSE).\n",
            "Use [M-A] and [M-B].\n\n",
            "[M-A]: ../ux/#M-A\n",
            "[M-B]: ./other.md#frag \"t\"\n",
            "[M-C]: https://x.org/\n",
            "[^1]: a footnote\n",
        );
        let m = members(&[
            ("references/r.md", reference),
            ("references/other.md", "# Other\n"),
        ]);
        let projected = format!("---\nname: s\n---\n{body}");
        let r =
            render("s", &projected, &m, &siblings(&["s", "std", "fmt", "pal"])).expect("renders");
        let t = &r.text;
        assert!(
            t.contains(concat!(
                "Per The Standard (the `std` skill) and `fmt` and palette (the `pal` skill).\n",
                "See `LICENSE`, ux, docs, alt, ghost, [me](#ref-skill-md).\n",
                "Keep [web](https://x.org/a.md), [here](#local), [abs](/etc/x), and `SKILL.md`.\n",
                "```\n[fenced](../std/SKILL.md)\n```\n",
            )),
            "{t}"
        );
        assert!(t.contains(concat!(
            "Read [`../SKILL.md`](#ref-skill-md) first; ",
            "`spacecraft-x` (the `std` skill) too, and lic.\n",
            "Use [M-A] and [M-B].\n\n",
            "[M-B]: #ref-references-other-md \"t\"\n",
            "[M-C]: https://x.org/\n",
            "[^1]: a footnote\n",
        )));
        assert!(!t.contains("[M-A]:"));
        assert!(t.contains("---\n<a id=\"ref-skill-md\"></a>\n"));
        // ../LICENSE, ../libs/ux, ../docs/, pic.png, none.md, ../std (ref),
        // ../../LICENSE, [M-A], and the three sibling links in the body.
        assert_eq!(r.unlinked, 11);
        // ../s/SKILL.md#top and ./other.md#frag.
        assert_eq!(r.fragments_dropped, 2);
    }

    #[test]
    fn fences_follow_commonmark() {
        let body = concat!(
            "\n````markdown\n```sh\n[a](../x)\n```\n[b](../x)\n````\n",
            "```markdown\n```sh\n[c](../x)\n```\n[d](../x)\n",
            "~~~\n```\n[e](../x)\n~~~\n",
        );
        let projected = format!("---\nname: s\n---\n{body}");
        let r = render("s", &projected, &Members::new(), &BTreeSet::new()).expect("renders");
        // An info string never closes a fence, a longer fence outlives shorter
        // runs, and `~~~` ignores backticks.
        assert!(r.text.contains("```sh\n[a](../x)\n```\n[b](../x)\n````\n"));
        assert!(r.text.contains("```sh\n[c](../x)\n```\nd\n"));
        assert!(r.text.contains("~~~\n```\n[e](../x)\n~~~\n"));
        assert_eq!(r.unlinked, 1);
    }

    #[test]
    fn anchor_collision_is_refused() {
        let m = members(&[("references/a-b.md", "x"), ("references/a_b.md", "y")]);
        let err = render("s", "---\nname: s\n---\n", &m, &BTreeSet::new()).unwrap_err();
        assert_eq!(err[0].key, "anchor_collision");
        assert_eq!(anchor("references/a-b.md"), "ref-references-a-b-md");
    }

    #[test]
    fn dangling_anchor_is_refused() {
        let err = render(
            "s",
            "---\nname: s\n---\n[x](#ref-ghost)\n",
            &Members::new(),
            &BTreeSet::new(),
        )
        .unwrap_err();
        assert_eq!(err[0].key, "single_file_dangling_anchor");
    }
}
