// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Filesystem detection for the registry: where an agent's skills directory
//! lives, whether an agent looks installed, and — critically — whether a target
//! directory is **managed by the Construct Home-Manager module** (a symlink to
//! `~/.agents/skills`). The HM-managed check is what keeps the imperative
//! installer from clobbering the declarative install (the coexistence rule).

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::registry::Agent;

/// The user's home directory, if known.
pub(crate) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// The canonical Construct skills store the HM module symlinks agent paths to.
fn hm_canonical() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".agents/skills"))
}

/// Root of the Nix store. Anything resolving under it is read-only, so an
/// imperative install there can only fail with `EROFS`.
///
/// `NIX_STORE_DIR` is the environment variable Nix itself uses to relocate the
/// store; honoring it keeps the check correct on a non-default installation.
/// `/nix/store` is the upstream default and the value on every Steelbore host.
fn store_root() -> PathBuf {
    std::env::var_os("NIX_STORE_DIR").map_or_else(|| PathBuf::from("/nix/store"), PathBuf::from)
}

/// What occupies a candidate install path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TargetState {
    /// Nothing exists at the path — safe to create.
    Free,
    /// A symlink to `~/.agents/skills` — owned by the Construct HM module.
    HmManaged,
    /// A symlink resolving into the Nix store — declaratively owned, read-only.
    StoreBacked,
    /// A symlink that is neither HM-managed nor store-backed.
    Linked,
    /// A real file or directory.
    Occupied,
}

/// Whether a state means "some declarative layer owns this path, do not write."
///
/// Both variants are refusals for the same reason and must never be checked
/// individually — an equality test against `HmManaged` alone silently lets the
/// store-backed shape through.
pub(crate) fn is_declarative(state: TargetState) -> bool {
    matches!(state, TargetState::HmManaged | TargetState::StoreBacked)
}

/// Classify what currently exists at `target`.
///
/// A symlink is compared against `~/.agents/skills` first, then *fully
/// resolved* to catch the store. Resolution matters because the HM module owns
/// `~/.agents/skills` itself: for the agents whose `global_path` **is**
/// `.agents/skills`, the naive comparison tests the path against itself and
/// never matches, so without the resolving step those agents are classified
/// `Linked` and the refusals in `install` do not fire.
pub(crate) fn classify(target: &Path) -> TargetState {
    let Ok(meta) = std::fs::symlink_metadata(target) else {
        return TargetState::Free;
    };
    if !meta.file_type().is_symlink() {
        return TargetState::Occupied;
    }
    if let (Ok(dest), Some(canon)) = (std::fs::read_link(target), hm_canonical()) {
        if dest == canon {
            return TargetState::HmManaged;
        }
    }
    // `canonicalize` follows the whole chain, so an indirection through a
    // pointer directory still lands on the store. It fails on a dangling link,
    // which is not store-backed — fall through to `Linked`.
    match std::fs::canonicalize(target) {
        Ok(resolved) if resolved.starts_with(store_root()) => TargetState::StoreBacked,
        _ => TargetState::Linked,
    }
}

/// The absolute global skills directory for an agent, if it has one.
pub(crate) fn global_base(agent: &Agent) -> Option<PathBuf> {
    Some(home_dir()?.join(agent.global_path.as_ref()?))
}

/// Whether an agent's global skills directory is declaratively managed (and
/// therefore must not be written to imperatively).
///
/// Covers both the direct `~/.agents/skills` symlink and the store-backed
/// shape; the name is kept because on a Construct host both *are* the HM
/// module's doing, and it backs the stable `hm_managed` field in `agent list`.
pub(crate) fn global_is_hm_managed(agent: &Agent) -> bool {
    global_base(agent).is_some_and(|base| is_declarative(classify(&base)))
}

/// Heuristic "is this agent installed?": does its config directory (the
/// `global_path` minus the trailing `skills` segment) exist? This mirrors the
/// spirit of vercel-labs' per-agent `detectInstalled` without porting ~70
/// bespoke checks; it can yield false positives for agents whose root is a
/// shared dir like `.config`.
pub(crate) fn detect_installed(agent: &Agent) -> bool {
    let Some(home) = home_dir() else {
        return false;
    };
    let Some(global) = agent.global_path.as_ref() else {
        return false;
    };
    let root = global.strip_suffix("/skills").unwrap_or(global);
    !root.is_empty() && home.join(root).exists()
}
