// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Black-box tests for `construct skill build`, all offline, against a small
//! catalogue generated in a temp directory. No fixture is committed: a
//! `SKILL.md` or `LICENSE` stub in the tree would sit in the path of the
//! repository's own description, frontmatter, license, and REUSE gates.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::io::{Cursor, Read as _};
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;
use zip::ZipArchive;

/// The palette source bytes the fixture carries.
const PALETTE: &str = "# palette\n[steelbore]\nvoid-navy = \"#000027\"\n";

fn write(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

fn bin() -> Command {
    Command::cargo_bin("construct").expect("binary builds")
}

/// Set mode 0755 at runtime: git and packaging may not preserve exec bits.
#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// A `SKILL.md` with the typical fleet frontmatter.
fn typical(name: &str, body: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: >\n  {name} does a thing.\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\n---\n{body}"
    )
}

/// The generated fixture catalogue (see the design's §13.1).
fn catalogue() -> TempDir {
    let t = TempDir::new().expect("temp catalogue");
    let p = t.path();

    write(
        p,
        "alpha/SKILL.md",
        &typical(
            "alpha",
            "\n# Alpha\n\nRead [the guide](references/guide.md#usage) and `references/other.md`.\n",
        ),
    );
    write(p, "alpha/LICENSE", "gpl text\n");
    write(
        p,
        "alpha/references/guide.md",
        "# Guide\n\n## Usage\n\nData lives in [x](../assets/x.json).\nSee [the palette](../../steelbore-color-palette/SKILL.md), [back](../SKILL.md), [lic](../LICENSE).\n",
    );
    write(p, "alpha/references/other.md", "# Other\n\ntext\n");
    write(p, "alpha/assets/x.json", "{\"k\": 1}\n");
    write(p, "alpha/assets/run.sh", "#!/bin/sh\necho hi\n");
    make_executable(&p.join("alpha/assets/run.sh"));

    write(
        p,
        "late-desc/SKILL.md",
        "---\nname: late-desc\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\ndescription: >-\n  Described last,\n  with a colon: here.\n---\nbody\n",
    );
    write(p, "late-desc/LICENSE", "gpl text\n");

    write(
        p,
        "dual/SKILL.md",
        "---\nname: dual\ndescription: >-\n  Dual licensed.\nlicense: GPL-3.0-or-later OR MIT\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\nuser-invocable: false\n---\nbody\n",
    );
    write(p, "dual/LICENSE.GPL", "gpl text\n");
    write(p, "dual/LICENSE.MIT", "mit text\n");
    write(p, "dual/CREDITS.md", "# Credits\n\nUpstream.\n");

    write(
        p,
        "meta-merge/SKILL.md",
        "---\nname: meta-merge\ndescription: d\nlicense: GPL-3.0-or-later\nmaintainer: M H <m@h.org>\nwebsite: https://x.org/\nmetadata:\n  spdx: \"SPDX-License-Identifier: GPL-3.0-or-later\"\n  author: Someone\n---\nbody\n",
    );
    write(p, "meta-merge/LICENSE", "gpl text\n");

    write(
        p,
        "meta-only/SKILL.md",
        "---\nname: meta-only\ndescription: d\nlicense: GPL-3.0-or-later\nmetadata:\n  author: Someone\n---\nbody\n",
    );
    write(p, "meta-only/LICENSE", "gpl text\n");

    write(
        p,
        "spacecraft-brand-guidelines/SKILL.md",
        &typical("spacecraft-brand-guidelines", "body\n"),
    );
    write(p, "spacecraft-brand-guidelines/LICENSE", "gpl text\n");

    write(
        p,
        "steelbore-color-palette/SKILL.md",
        &typical("steelbore-color-palette", "body\n"),
    );
    write(p, "steelbore-color-palette/LICENSE", "gpl text\n");
    write(p, "steelbore-color-palette/assets/steelbore.toml", PALETTE);

    write(
        p,
        "grok-skills/grokky/SKILL.md",
        "---\nname: grokky\ndescription: A Grok-native skill.\n---\nbody\n",
    );
    write(p, "grok-skills/grokky/LICENSE", "gpl text\n");
    write(p, "grok-skills/grokky/assets/t.md", "# T\n");

    // Decoys directly inside excluded directories: if an exclusion is lost,
    // they surface as skills (and fail the name gate), so these tests fail.
    write(
        p,
        "Excluded/SKILL.md",
        "---\nname: decoy\ndescription: d\n---\n",
    );
    write(
        p,
        "perplexity-skills/SKILL.md",
        "---\nname: decoy\ndescription: d\n---\n",
    );
    write(
        p,
        "android-skills/vendored/SKILL.md",
        "---\nname: vendored\ndescription: d\n---\n",
    );
    write(
        p,
        "perplexity-skills/categories.toml",
        &categories_toml(&["big"]),
    );
    t
}

/// A categories map covering the generated `big` skill's 105 tools in three
/// categories, for each skill id in `skills`.
fn categories_toml(skills: &[&str]) -> String {
    let mut out = String::from("schema = 1\n");
    for skill in skills {
        let _ = write!(
            out,
            "\n[skill.{skill}]\npassthrough = [\"ATTRIBUTION.md\"]\n"
        );
        for (slug, range) in [("first", 0..35), ("second", 35..70), ("third", 70..105)] {
            let tools: Vec<String> = range.map(|i| format!("\"t{i:03}\"")).collect();
            let _ = write!(
                out,
                "\n[[skill.{skill}.category]]\nslug = \"{slug}\"\ntitle = \"{} Things\"\ntools = [{}]\n",
                slug.to_uppercase(),
                tools.join(", ")
            );
        }
    }
    out
}

/// A skill with 108 files: SKILL.md, LICENSE, ATTRIBUTION.md, 105 tools.
fn big(p: &Path) {
    let mut links = String::new();
    for i in 0..105 {
        let _ = writeln!(links, "- [t{i:03}](references/t{i:03}.md)");
    }
    write(
        p,
        "big/SKILL.md",
        &typical("big", &format!("\n# Big\n\n{links}")),
    );
    write(p, "big/LICENSE", "gpl text\n");
    write(
        p,
        "big/references/ATTRIBUTION.md",
        "# Attribution\n\nverbatim\n",
    );
    for i in 0..105 {
        write(
            p,
            &format!("big/references/t{i:03}.md"),
            &format!("# Tool {i}\n\n## Usage\n\n```sh\n# not-a-heading\n```\n"),
        );
    }
}

/// Run `construct skill build` against `repo` with extra args.
fn build(repo: &Path, extra: &[&str]) -> assert_cmd::assert::Assert {
    bin()
        .args(["skill", "build", "--repo", repo.to_str().unwrap(), "--json"])
        .args(extra)
        .assert()
}

/// The success envelope's `data`.
fn data(assert: &assert_cmd::assert::Assert) -> Value {
    let v: Value = serde_json::from_slice(&assert.get_output().stdout).expect("JSON envelope");
    v["data"].clone()
}

/// The structured error on stderr (diagnostic lines may precede it).
fn error(assert: &assert_cmd::assert::Assert) -> Value {
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    let line = stderr
        .lines()
        .find(|l| l.starts_with("{\"error\""))
        .unwrap_or_else(|| panic!("no structured error in: {stderr}"));
    let v: Value = serde_json::from_str(line).expect("single-line JSON error");
    v["error"].clone()
}

/// Names in a zip, in archive order.
fn zip_names(path: &Path) -> Vec<String> {
    let bytes = fs::read(path).unwrap();
    let mut a = ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..a.len())
        .map(|i| a.by_index(i).unwrap().name().to_owned())
        .collect()
}

/// One member of a zip as text.
fn zip_text(path: &Path, name: &str) -> String {
    let bytes = fs::read(path).unwrap();
    let mut a = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut f = a.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// The parsed frontmatter of a `SKILL.md` text, keys in order.
fn frontmatter_keys(text: &str) -> Vec<String> {
    let rest = text.strip_prefix("---\n").unwrap();
    let end = rest.find("\n---\n").unwrap();
    let map: serde_yaml::Mapping = serde_yaml::from_str(&rest[..=end]).unwrap();
    map.keys().map(|k| k.as_str().unwrap().to_owned()).collect()
}

/// Every file under `dir`, relative, sorted.
fn tree(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.insert(
                    path.strip_prefix(dir)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    out
}

#[test]
fn build_all_targets_writes_expected_layout() {
    let cat = catalogue();
    let p = cat.path();
    let d = data(&build(p, &[]).success());
    assert_eq!(d["written"], true);
    assert_eq!(d["skills"], 8, "7 root + 1 grok-native");

    let root = [
        "alpha",
        "dual",
        "late-desc",
        "meta-merge",
        "meta-only",
        "spacecraft-brand-guidelines",
        "steelbore-color-palette",
    ];
    let expect = |exts: &[&str], extra: &[&str]| -> BTreeSet<String> {
        let mut s: BTreeSet<String> = root
            .iter()
            .chain(extra)
            .flat_map(|n| exts.iter().map(move |e| format!("{n}.{e}")))
            .collect();
        s.insert(".construct-build".to_owned());
        s
    };
    assert_eq!(tree(&p.join("dist/claude")), expect(&["zip", "skill"], &[]));
    assert_eq!(
        tree(&p.join("dist/grok")),
        expect(&["zip", "skill"], &["grokky"])
    );
    assert_eq!(tree(&p.join("dist/perplexity")), expect(&["zip"], &[]));
    assert_eq!(tree(&p.join("dist/single-file")), expect(&["md"], &[]));

    // Claude is nested with directory entries; .skill has none.
    assert_eq!(
        zip_names(&p.join("dist/claude/alpha.zip")),
        vec![
            "alpha/",
            "alpha/LICENSE",
            "alpha/SKILL.md",
            "alpha/assets/",
            "alpha/assets/run.sh",
            "alpha/assets/x.json",
            "alpha/references/",
            "alpha/references/guide.md",
            "alpha/references/other.md",
        ]
    );
    assert!(zip_names(&p.join("dist/claude/alpha.skill"))
        .iter()
        .all(|n| !n.ends_with('/')));
    // Grok is flat.
    let grok = zip_names(&p.join("dist/grok/alpha.zip"));
    assert!(grok.contains(&"SKILL.md".to_owned()), "{grok:?}");
    assert!(grok.iter().all(|n| !n.starts_with("alpha/")));
    // Perplexity: nested, files only.
    assert!(zip_names(&p.join("dist/perplexity/alpha.zip"))
        .iter()
        .all(|n| n.starts_with("alpha/") && !n.ends_with('/')));
}

#[test]
fn build_is_byte_reproducible() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &[]).success();
    let first: Vec<(String, Vec<u8>)> = tree(&p.join("dist"))
        .into_iter()
        .map(|f| {
            let bytes = fs::read(p.join("dist").join(&f)).unwrap();
            (f, bytes)
        })
        .collect();
    build(p, &[]).success();
    for (f, bytes) in &first {
        assert_eq!(
            &fs::read(p.join("dist").join(f)).unwrap(),
            bytes,
            "{f} changed between builds"
        );
    }
    assert_eq!(first.len(), tree(&p.join("dist")).len());
}

/// Run git in `repo` with no system or global config, so the host's
/// `core.fileMode` and identity never leak into a test.
fn git(repo: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}

/// Stage the whole fixture in a fresh index, recording `run.sh` with the
/// given index mode regardless of its working-tree mode.
fn index_with_run_sh(repo: &Path, chmod: &str) {
    git(repo, &["init", "-q"]);
    git(repo, &["add", "-A"]);
    git(repo, &["update-index", chmod, "alpha/assets/run.sh"]);
}

fn run_sh_mode(repo: &Path) -> Option<u32> {
    let bytes = fs::read(repo.join("dist/claude/alpha.zip")).unwrap();
    let mut a = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let f = a.by_name("alpha/assets/run.sh").unwrap();
    f.unix_mode().map(|m| m & 0o777)
}

#[cfg(unix)]
#[test]
fn exec_bit_comes_from_the_git_index() {
    use std::os::unix::fs::PermissionsExt as _;
    let cat = catalogue();
    let p = cat.path();
    // Disk 0644, index 100755: only the index can make it 0755.
    fs::set_permissions(
        p.join("alpha/assets/run.sh"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    index_with_run_sh(p, "--chmod=+x");
    build(p, &["--target", "claude"]).success();
    assert_eq!(run_sh_mode(p), Some(0o755));
}

/// The working-tree exec bit is host-dependent (`core.fileMode = false`), so
/// it never reaches an archive: disk 0755 with index 100644 ships 0644.
#[test]
fn working_tree_exec_bit_is_not_shipped() {
    let cat = catalogue();
    let p = cat.path();
    make_executable(&p.join("alpha/assets/run.sh"));
    index_with_run_sh(p, "--chmod=-x");
    build(p, &["--target", "claude"]).success();
    assert_eq!(run_sh_mode(p), Some(0o644));

    // Outside a git work tree every file is 0644, whatever its disk mode.
    fs::remove_dir_all(p.join(".git")).unwrap();
    build(p, &["--target", "claude"]).success();
    assert_eq!(run_sh_mode(p), Some(0o644));
}

#[test]
fn claude_frontmatter_is_spec_clean() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &["--target", "claude,perplexity"]).success();

    let dual = zip_text(&p.join("dist/claude/dual.zip"), "dual/SKILL.md");
    assert_eq!(
        frontmatter_keys(&dual),
        vec![
            "name",
            "description",
            "license",
            "user-invocable",
            "metadata"
        ]
    );
    assert!(dual
        .contains("metadata:\n  maintainer: \"M H <m@h.org>\"\n  website: \"https://x.org/\"\n"));
    assert!(dual.ends_with("---\nbody\n"), "body untouched: {dual}");

    let perplexity = zip_text(&p.join("dist/perplexity/dual.zip"), "dual/SKILL.md");
    assert_eq!(
        frontmatter_keys(&perplexity),
        vec!["name", "description", "license", "metadata"]
    );

    let merged = zip_text(&p.join("dist/claude/meta-merge.zip"), "meta-merge/SKILL.md");
    assert!(merged.contains("metadata:\n  spdx: \"SPDX-License-Identifier: GPL-3.0-or-later\"\n  author: \"Someone\"\n  maintainer: \"M H <m@h.org>\"\n  website: \"https://x.org/\"\n"));

    let only = zip_text(&p.join("dist/claude/meta-only.zip"), "meta-only/SKILL.md");
    assert!(only.contains("metadata:\n  author: \"Someone\"\n---\n"));

    // Description moved up and byte-identical, chomping and all.
    let late = zip_text(&p.join("dist/claude/late-desc.zip"), "late-desc/SKILL.md");
    assert!(late.starts_with(
        "---\nname: late-desc\ndescription: >-\n  Described last,\n  with a colon: here.\nlicense:"
    ));
}

#[test]
fn grok_frontmatter_is_name_and_description_only() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &["--target", "grok"]).success();
    for skill in ["alpha", "dual", "grokky"] {
        let text = zip_text(&p.join(format!("dist/grok/{skill}.zip")), "SKILL.md");
        assert_eq!(
            frontmatter_keys(&text),
            vec!["name", "description"],
            "{skill}"
        );
    }
    let dual = zip_names(&p.join("dist/grok/dual.zip"));
    assert!(dual.contains(&"LICENSE.GPL".to_owned()) && dual.contains(&"LICENSE.MIT".to_owned()));
    assert!(dual.contains(&"CREDITS.md".to_owned()));
}

#[test]
fn palette_vendored_and_identical() {
    let cat = catalogue();
    let p = cat.path();
    let d = data(&build(p, &[]).success());
    let brand = "spacecraft-brand-guidelines";
    assert_eq!(
        zip_text(
            &p.join(format!("dist/claude/{brand}.zip")),
            &format!("{brand}/assets/steelbore.toml")
        ),
        PALETTE
    );
    assert_eq!(
        zip_text(
            &p.join(format!("dist/grok/{brand}.skill")),
            "assets/steelbore.toml"
        ),
        PALETTE
    );
    assert_eq!(
        zip_text(
            &p.join(format!("dist/perplexity/{brand}.zip")),
            &format!("{brand}/assets/steelbore.toml")
        ),
        PALETTE
    );
    let single = fs::read_to_string(p.join(format!("dist/single-file/{brand}.md"))).unwrap();
    assert!(single.contains(&format!(
        "<a id=\"ref-assets-steelbore-toml\"></a>\n## assets/steelbore.toml\n\n```toml\n{PALETTE}```\n"
    )));
    // A non-consumer carries no palette.
    assert!(!zip_names(&p.join("dist/claude/alpha.zip"))
        .iter()
        .any(|n| n.ends_with("steelbore.toml")));
    let vendored = d["palette_vendored"].as_array().unwrap();
    assert_eq!(
        vendored.len(),
        6,
        "zip+skill claude, zip+skill grok, perplexity, md"
    );
    assert!(vendored
        .iter()
        .all(|v| v["verified"] == true && v["skill"] == brand));
}

#[test]
fn palette_source_missing_is_not_found() {
    let cat = catalogue();
    let p = cat.path();
    fs::remove_file(p.join("steelbore-color-palette/assets/steelbore.toml")).unwrap();
    let a = build(p, &[]).code(3);
    assert_eq!(error(&a)["code"], "NOT_FOUND");
    assert!(!p.join("dist").exists());
}

/// Only an absent palette is "not found"; any other read failure (here
/// EISDIR) is an I/O error, never a `git checkout` hint.
#[test]
fn palette_source_unreadable_is_io_error() {
    let cat = catalogue();
    let p = cat.path();
    let src = p.join("steelbore-color-palette/assets/steelbore.toml");
    fs::remove_file(&src).unwrap();
    fs::create_dir_all(&src).unwrap();
    let e = error(&build(p, &[]).code(1));
    assert_ne!(e["code"], "NOT_FOUND");
    assert!(e["io_failures"].is_array(), "{e}");
    assert!(!p.join("dist").exists());
}

#[test]
fn perplexity_consolidates_over_100() {
    let cat = catalogue();
    let p = cat.path();
    big(p);
    let d = data(&build(p, &["--target", "perplexity"]).success());
    let c = &d["consolidated"][0];
    assert_eq!(c["skill"], "big");
    assert_eq!(c["files_before"], 108);
    assert_eq!(c["files_after"], 6);
    assert_eq!(c["categories"], 3);
    assert_eq!(
        d["consolidated"].as_array().unwrap().len(),
        1,
        "small skills untouched"
    );

    let zip = p.join("dist/perplexity/big.zip");
    assert_eq!(
        zip_names(&zip),
        vec![
            "big/LICENSE",
            "big/SKILL.md",
            "big/references/ATTRIBUTION.md",
            "big/references/first.md",
            "big/references/second.md",
            "big/references/third.md",
        ]
    );
    let skill = zip_text(&zip, "big/SKILL.md");
    assert!(skill.contains("- [t000](references/first.md#t000)\n"));
    assert!(skill.contains("- [t104](references/third.md#t104)\n"));
    assert_eq!(
        zip_text(&zip, "big/references/ATTRIBUTION.md"),
        "# Attribution\n\nverbatim\n"
    );
    let first = zip_text(&zip, "big/references/first.md");
    assert!(first.starts_with(
        "# FIRST Things\n\n## t000\n\n**Tool 0**\n\n\n### Usage\n\n```sh\n# not-a-heading\n```\n\n## t001\n"
    ));
}

#[test]
fn perplexity_refuses_oversized_without_map() {
    let cat = catalogue();
    let p = cat.path();
    big(p);
    write(p, "perplexity-skills/categories.toml", "schema = 1\n");
    let a = build(p, &["--target", "perplexity"]).code(5);
    let e = error(&a);
    assert_eq!(e["code"], "CONFLICT");
    assert_eq!(e["perplexity"][0]["kind"], "oversized_without_map");
    assert!(!p.join("dist").exists());
}

#[test]
fn perplexity_categories_file_missing_is_not_found() {
    let cat = catalogue();
    let p = cat.path();
    big(p);
    fs::remove_file(p.join("perplexity-skills/categories.toml")).unwrap();
    let a = build(p, &["--target", "perplexity"]).code(3);
    assert!(error(&a)["hint"]
        .as_str()
        .unwrap()
        .contains("perplexity-skills/categories.toml"));
}

#[test]
fn perplexity_categories_file_unreadable_is_io_error() {
    let cat = catalogue();
    let p = cat.path();
    big(p);
    let map = p.join("perplexity-skills/categories.toml");
    fs::remove_file(&map).unwrap();
    fs::create_dir_all(&map).unwrap();
    let e = error(&build(p, &["--target", "perplexity"]).code(1));
    assert!(e["io_failures"].is_array(), "{e}");
}

#[test]
fn perplexity_refuses_map_out_of_sync() {
    let cat = catalogue();
    let p = cat.path();
    big(p);
    fs::remove_file(p.join("big/references/t000.md")).unwrap();
    write(p, "big/references/extra.md", "# Extra\n");
    let a = build(p, &["--target", "perplexity"]).code(5);
    let detail = &error(&a)["perplexity"][0];
    assert_eq!(detail["kind"], "map_out_of_sync");
    assert_eq!(detail["detail"]["in_map_not_on_disk"][0], "t000");
    assert_eq!(detail["detail"]["on_disk_not_mapped"][0], "extra");
}

#[test]
fn single_file_inlines_references_and_assets() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &["--target", "single-file"]).success();
    let doc = fs::read_to_string(p.join("dist/single-file/alpha.md")).unwrap();
    assert_eq!(
        frontmatter_keys(&doc),
        vec!["name", "description", "license"]
    );
    assert!(doc.contains(
        "Read [the guide](#ref-references-guide-md) and [`references/other.md`](#ref-references-other-md)."
    ));
    assert!(doc.contains(
        "<a id=\"ref-references-guide-md\"></a>\n## references/guide.md\n\n## Guide\n\n### Usage\n\nData lives in [x](#ref-assets-x-json).\n"
    ));
    assert!(doc.contains(
        "<a id=\"ref-assets-run-sh\"></a>\n## assets/run.sh\n\n```sh\n#!/bin/sh\necho hi\n```\n"
    ));
    assert!(doc.contains("```json\n{\"k\": 1}\n```\n"));
    assert!(
        doc.ends_with(
            "\n## License\n\n<a id=\"ref-license\"></a>\n### LICENSE\n\n```text\ngpl text\n```\n"
        ),
        "license text closes the document"
    );
    assert!(doc.contains(
        "See the palette (the `steelbore-color-palette` skill), [back](#ref-skill-md), [lic](#ref-license).\n"
    ));
    assert_eq!(relative_links(&doc), Vec::<String>::new());
    // Every rewritten link resolves.
    for id in doc.split("](#").skip(1).map(|s| &s[..s.find(')').unwrap()]) {
        assert!(
            doc.contains(&format!("<a id=\"{id}\"></a>")),
            "dangling #{id}"
        );
    }
    let dual = fs::read_to_string(p.join("dist/single-file/dual.md")).unwrap();
    assert!(dual.contains("<a id=\"ref-credits-md\"></a>\n## CREDITS.md\n\n## Credits\n"));
    assert!(dual.contains("### LICENSE.GPL\n\n```text\ngpl text\n```\n"));
    assert!(dual.contains("### LICENSE.MIT\n\n```text\nmit text\n```\n"));
}

#[test]
fn description_cap_refuses_everything() {
    let cat = catalogue();
    let p = cat.path();
    write(
        p,
        "alpha/SKILL.md",
        &format!(
            "---\nname: alpha\ndescription: {}\nlicense: GPL-3.0-or-later\n---\nbody\n",
            "x".repeat(1001)
        ),
    );
    let a = build(p, &[]).code(5);
    let e = error(&a);
    assert_eq!(e["code"], "CONFLICT");
    assert_eq!(e["oversized_skills"][0]["skill"], "alpha");
    assert_eq!(e["oversized_skills"][0]["over_by"], 1);
    assert!(!p.join("dist").exists(), "all-or-nothing: nothing written");
}

#[test]
fn invalid_frontmatter_refused() {
    let cat = catalogue();
    let p = cat.path();
    write(
        p,
        "alpha/SKILL.md",
        "---\nname: alpha\ndescription: Use it: now\n---\nbody\n",
    );
    let e = error(&build(p, &[]).code(5));
    assert_eq!(e["invalid_frontmatter"][0]["skill"], "alpha");
}

#[test]
fn compatibility_cap_refused() {
    let cat = catalogue();
    let p = cat.path();
    write(
        p,
        "alpha/SKILL.md",
        &format!(
            "---\nname: alpha\ndescription: d\nlicense: GPL-3.0-or-later\ncompatibility: {}\n---\nbody\n",
            "c".repeat(501)
        ),
    );
    let e = error(&build(p, &[]).code(5));
    assert_eq!(e["oversized_compatibility"][0]["skill"], "alpha");
    assert_eq!(e["oversized_compatibility"][0]["chars"], 501);
}

#[test]
fn compatibility_and_allowed_tools_are_kept_for_claude() {
    let cat = catalogue();
    let p = cat.path();
    write(
        p,
        "alpha/SKILL.md",
        "---\nname: alpha\ndescription: d\nallowed-tools: Bash(git:*) Read\ncompatibility: Requires git\nlicense: GPL-3.0-or-later\n---\nbody\n",
    );
    build(p, &["--target", "claude"]).success();
    let text = zip_text(&p.join("dist/claude/alpha.zip"), "alpha/SKILL.md");
    assert_eq!(
        frontmatter_keys(&text),
        vec![
            "name",
            "description",
            "license",
            "compatibility",
            "allowed-tools"
        ]
    );
}

#[test]
fn unknown_key_refused() {
    let cat = catalogue();
    let p = cat.path();
    write(
        p,
        "alpha/SKILL.md",
        "---\nname: alpha\ndescription: d\nlicence: GPL\n---\nbody\n",
    );
    let e = error(&build(p, &[]).code(5));
    assert_eq!(e["unknown_frontmatter_keys"][0]["key"], "licence");
}

#[cfg(unix)]
#[test]
fn symlink_refused() {
    let cat = catalogue();
    let p = cat.path();
    std::os::unix::fs::symlink("x.json", p.join("alpha/assets/link.json")).unwrap();
    let e = error(&build(p, &[]).code(5));
    assert_eq!(e["symlinks"][0]["path"], "assets/link.json");
}

#[test]
fn missing_license_refused() {
    let cat = catalogue();
    let p = cat.path();
    fs::remove_file(p.join("alpha/LICENSE")).unwrap();
    let e = error(&build(p, &[]).code(5));
    assert_eq!(e["missing_license"][0]["skill"], "alpha");
}

#[test]
fn dry_run_writes_nothing_but_validates() {
    let cat = catalogue();
    let p = cat.path();
    let d = data(&build(p, &["--dry-run"]).success());
    assert_eq!(d["written"], false);
    assert!(d["bundles"].as_array().is_some_and(|b| !b.is_empty()));
    assert!(!p.join("dist").exists());

    write(
        p,
        "alpha/SKILL.md",
        &format!(
            "---\nname: alpha\ndescription: {}\n---\nbody\n",
            "x".repeat(1001)
        ),
    );
    build(p, &["--dry-run"]).code(5);
    assert!(!p.join("dist").exists());
}

#[test]
fn out_dir_not_owned_refused_unless_force() {
    let cat = catalogue();
    let p = cat.path();
    write(p, "dist/claude/stray.txt", "not ours");
    let e = error(&build(p, &["alpha", "--target", "claude"]).code(5));
    assert_eq!(e["out_dir_not_owned"][0]["target"], "claude");
    // The hint reruns this exact invocation, not a full build from the default repo.
    assert_eq!(
        e["hint"],
        format!(
            "construct skill build alpha --target claude --repo {} --force",
            p.display()
        )
    );
    assert!(p.join("dist/claude/stray.txt").exists());

    build(p, &["--target", "claude", "--force"]).success();
    assert!(!p.join("dist/claude/stray.txt").exists());
    assert!(p.join("dist/claude/alpha.zip").exists());
}

/// A partial `--force` build into a foreign directory must not adopt it:
/// otherwise the next plain full build would delete the foreign files.
#[test]
fn partial_force_build_never_adopts_a_foreign_dir() {
    let cat = catalogue();
    let p = cat.path();
    write(p, "dist/claude/mine.txt", "not ours");
    build(p, &["alpha", "--target", "claude", "--force"]).success();
    assert!(p.join("dist/claude/alpha.zip").exists());
    assert!(
        !p.join("dist/claude/.construct-build").exists(),
        "a foreign dir gets no ownership marker"
    );
    let e = error(&build(p, &["alpha", "--target", "claude"]).code(5));
    assert_eq!(e["out_dir_not_owned"][0]["target"], "claude");
    // The hint reruns this exact invocation, not a full build from the default repo.
    assert_eq!(
        e["hint"],
        format!(
            "construct skill build alpha --target claude --repo {} --force",
            p.display()
        )
    );
    assert_eq!(
        fs::read_to_string(p.join("dist/claude/mine.txt")).unwrap(),
        "not ours"
    );
}

#[test]
fn full_build_removes_stale_bundles() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &["--target", "claude"]).success();
    assert!(p.join("dist/claude/late-desc.zip").exists());
    fs::remove_dir_all(p.join("late-desc")).unwrap();
    build(p, &["--target", "claude"]).success();
    assert!(!p.join("dist/claude/late-desc.zip").exists());
    assert!(p.join("dist/claude/alpha.zip").exists());
}

#[test]
fn partial_build_leaves_other_skills() {
    let cat = catalogue();
    let p = cat.path();
    build(p, &["--target", "claude"]).success();
    let before = fs::read(p.join("dist/claude/dual.zip")).unwrap();
    fs::remove_file(p.join("dist/claude/alpha.zip")).unwrap();
    let d = data(&build(p, &["alpha", "--target", "claude"]).success());
    assert_eq!(d["skills"], 1);
    assert!(p.join("dist/claude/alpha.zip").exists());
    assert_eq!(fs::read(p.join("dist/claude/dual.zip")).unwrap(), before);
}

#[test]
fn unknown_skill_is_not_found() {
    let cat = catalogue();
    let p = cat.path();
    let e = error(&build(p, &["nope", "alpha", "gone"]).code(3));
    assert_eq!(e["code"], "NOT_FOUND");
    assert_eq!(e["unknown_skills"], serde_json::json!(["nope", "gone"]));
    // A Grok-native skill is unknown to a build without the grok target.
    build(p, &["grokky", "--target", "claude"]).code(3);
    build(p, &["grokky", "--target", "grok"]).success();
}

#[test]
fn bad_target_is_usage_error() {
    let cat = catalogue();
    build(cat.path(), &["--target", "bogus"]).code(2);
}

#[test]
fn missing_repo_is_not_found() {
    bin()
        .args(["skill", "build", "--repo", "/no/such/catalogue", "--json"])
        .assert()
        .code(3);
}

#[test]
fn machine_error_is_single_line_json() {
    let cat = catalogue();
    let p = cat.path();
    fs::remove_file(p.join("alpha/LICENSE")).unwrap();
    let a = build(p, &[]).code(5);
    let stderr = String::from_utf8_lossy(&a.get_output().stderr).into_owned();
    let line = stderr
        .lines()
        .find(|l| l.starts_with("{\"error\""))
        .expect("error line");
    let v: Value = serde_json::from_str(line).expect("the error is one JSON line");
    assert_eq!(v["error"]["exit_code"], 5);
    assert!(v["error"]["hint"].as_str().is_some_and(|h| !h.is_empty()));
    assert!(v["error"]["timestamp"]
        .as_str()
        .is_some_and(|t| t.ends_with('Z')));
}

/// Every relative link target in a markdown document, outside fenced blocks
/// and inline code spans: inline `](target)` links and `[label]: target`
/// reference definitions. External (`scheme:`), `#fragment`, and absolute
/// targets are not relative.
fn relative_links(doc: &str) -> Vec<String> {
    fn relative(target: &str) -> bool {
        let target = target.trim_start_matches('<');
        let external = target.split_once(':').is_some_and(|(scheme, _)| {
            !scheme.is_empty()
                && scheme
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'.' | b'-'))
        });
        !(target.is_empty() || target.starts_with(['#', '/']) || external)
    }
    /// The line with every inline code span removed.
    fn without_code(line: &str) -> String {
        let mut out = String::new();
        let mut rest = line;
        while let Some(i) = rest.find('`') {
            out.push_str(&rest[..i]);
            let run = rest[i..].bytes().take_while(|b| *b == b'`').count();
            let ticks = &rest[i..i + run];
            let after = &rest[i + run..];
            // The closing run must be exactly as long as the opening one.
            let close = after.match_indices(ticks).find(|(j, _)| {
                !after[j + run..].starts_with('`') && (*j == 0 || !after[..*j].ends_with('`'))
            });
            if let Some((j, _)) = close {
                rest = &after[j + run..];
            } else {
                out.push_str(&rest[i..]);
                rest = "";
            }
        }
        out.push_str(rest);
        out
    }
    let mut found = Vec::new();
    let mut fence: Option<String> = None;
    for line in doc.lines() {
        let trimmed = line.trim_start();
        let marker: String = trimmed
            .chars()
            .take_while(|c| *c == '`' || *c == '~')
            .collect();
        if marker.len() >= 3
            && marker
                .chars()
                .all(|c| c == marker.chars().next().unwrap_or('`'))
        {
            match &fence {
                None => fence = Some(marker),
                Some(open) if marker.starts_with(open.as_str()) && trimmed.trim() == marker => {
                    fence = None;
                }
                Some(_) => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let text = without_code(line);
        for (i, _) in text.match_indices("](") {
            let target: String = text[i + 2..]
                .chars()
                .take_while(|c| *c != ')' && !c.is_whitespace())
                .collect();
            if relative(&target) {
                found.push(format!("{line} -> {target}"));
            }
        }
        let def = text.trim_start();
        if let (Some(rest), false) = (def.strip_prefix('['), def.starts_with("[^")) {
            if let Some((_, after)) = rest.split_once("]:") {
                let target = after.split_whitespace().next().unwrap_or("");
                if relative(target) {
                    found.push(format!("{line} -> {target}"));
                }
            }
        }
    }
    found
}

/// The real catalogue's single-file render leaves no relative link behind.
/// Ignored by default for the same reason as `real_catalogue_builds_clean`.
#[test]
#[ignore = "reads the real catalogue; set CONSTRUCT_REPO"]
fn real_catalogue_single_file_has_no_relative_links() {
    let Ok(repo) = std::env::var("CONSTRUCT_REPO") else {
        return;
    };
    let out = TempDir::new().expect("temp out");
    build(
        Path::new(&repo),
        &[
            "--target",
            "single-file",
            "--out",
            out.path().to_str().unwrap(),
        ],
    )
    .success();
    let dir = out.path().join("single-file");
    let mut offenders = Vec::new();
    let mut docs = 0;
    for entry in fs::read_dir(&dir).expect("single-file output") {
        let path = entry.unwrap().path();
        let doc = fs::read_to_string(&path).unwrap();
        docs += 1;
        for hit in relative_links(&doc) {
            offenders.push(format!("{}: {hit}", path.display()));
        }
    }
    assert!(docs > 0, "no single-file documents in {}", dir.display());
    // The dual-licensed Microsoft base carries the MIT permission notice.
    let ms = fs::read_to_string(dir.join("microsoft-rust-guidelines.md")).unwrap();
    for needle in [
        "### LICENSE.GPL\n",
        "### LICENSE.MIT\n",
        "Permission is hereby granted, free of charge",
    ] {
        assert!(
            ms.contains(needle),
            "microsoft-rust-guidelines.md lacks {needle:?}"
        );
    }
    assert!(
        offenders.is_empty(),
        "relative links remain:\n{}",
        offenders.join("\n")
    );
}

/// The real catalogue passes every gate for every target. Reads files outside
/// the crate, so it is ignored by default (the Nix sandbox sees only
/// `construct-cli/`). Run by hand:
/// `CONSTRUCT_REPO=/spacecraft-software/construct cargo test -- --ignored`.
#[test]
#[ignore = "reads the real catalogue; set CONSTRUCT_REPO"]
fn real_catalogue_builds_clean() {
    let Ok(repo) = std::env::var("CONSTRUCT_REPO") else {
        return;
    };
    let d = data(&build(Path::new(&repo), &["--dry-run"]).success());
    assert_eq!(d["written"], false);
    assert!(d["consolidated"]
        .as_array()
        .is_some_and(|c| c.iter().any(|x| x["skill"] == "spacecraft-cli-preference")));
}
