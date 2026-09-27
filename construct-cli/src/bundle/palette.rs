// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Steelbore palette vendoring for bundles that leave the catalogue.
//!
//! `spacecraft-brand-guidelines` and `spacecraft-accessibility-support` read
//! every palette value from the sibling `steelbore-color-palette` skill. A
//! bundle installed on its own may not have that sibling, so every non-source
//! target carries a byte-identical copy at `assets/steelbore.toml` — read only
//! when the sibling is absent. The copy is verified against the source after
//! it is encoded and again after it is written; any difference fails the build.

use serde_json::json;

use super::{single, sink, Member, Members, PaletteCheck, Problem};

/// Skills that receive a vendored palette (maintainer decision, M1).
pub(crate) const PALETTE_CONSUMERS: &[&str] = &[
    "spacecraft-accessibility-support",
    "spacecraft-brand-guidelines",
];

/// The palette source, relative to the catalogue root.
pub(crate) const PALETTE_SOURCE: &str = "steelbore-color-palette/assets/steelbore.toml";

/// Where the vendored copy lands inside a consumer bundle.
pub(crate) const PALETTE_MEMBER: &str = "assets/steelbore.toml";

/// The single-file anchor of [`PALETTE_MEMBER`].
const PALETTE_ANCHOR: &str = "ref-assets-steelbore-toml";

/// Whether `skill` receives a vendored palette.
pub(crate) fn is_consumer(skill: &str) -> bool {
    PALETTE_CONSUMERS.contains(&skill)
}

/// Insert the palette into a consumer's members.
///
/// # Errors
///
/// `palette_collision` when the skill already carries its own
/// `assets/steelbore.toml` — never overwritten silently.
pub(crate) fn vendor(skill: &str, members: &mut Members, palette: &[u8]) -> Result<(), Problem> {
    if members.contains_key(PALETTE_MEMBER) {
        return Err(Problem::conflict(
            "palette_collision",
            format!("{skill} already carries {PALETTE_MEMBER}; the vendored copy would replace it"),
            format!("rm {skill}/{PALETTE_MEMBER}"),
            json!({ "skill": skill }),
        ));
    }
    members.insert(
        PALETTE_MEMBER.to_owned(),
        Member {
            bytes: palette.to_vec(),
            mode: 0o644,
        },
    );
    Ok(())
}

/// Whether the palette inside `artifact` is byte-identical to `palette`.
pub(crate) fn verify(check: &PaletteCheck, artifact: &[u8], palette: &[u8]) -> bool {
    match check {
        PaletteCheck::Zip { prefix } => {
            sink::read_member(artifact, &format!("{prefix}{PALETTE_MEMBER}")).as_deref()
                == Some(palette)
        }
        PaletteCheck::Markdown => {
            let Ok(doc) = std::str::from_utf8(artifact) else {
                return false;
            };
            let Some(body) = single::extract_fenced(doc, PALETTE_ANCHOR) else {
                return false;
            };
            // The renderer adds a final newline only when the source lacks one.
            body.as_bytes() == palette
                || (!palette.ends_with(b"\n")
                    && body.strip_suffix('\n').map(str::as_bytes) == Some(palette))
        }
    }
}
