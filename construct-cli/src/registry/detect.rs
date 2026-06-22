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

/// What occupies a candidate install path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TargetState {
    /// Nothing exists at the path — safe to create.
    Free,
    /// A symlink to `~/.agents/skills` — owned by the Construct HM module.
    HmManaged,
    /// A symlink that is not the HM-managed one.
    Linked,
    /// A real file or directory.
    Occupied,
}

/// Classify what currently exists at `target` (does not follow the final link).
pub(crate) fn classify(target: &Path) -> TargetState {
    let Ok(meta) = std::fs::symlink_metadata(target) else {
        return TargetState::Free;
    };
    if meta.file_type().is_symlink() {
        if let (Ok(dest), Some(canon)) = (std::fs::read_link(target), hm_canonical()) {
            if dest == canon {
                return TargetState::HmManaged;
            }
        }
        TargetState::Linked
    } else {
        TargetState::Occupied
    }
}

/// The absolute global skills directory for an agent, if it has one.
pub(crate) fn global_base(agent: &Agent) -> Option<PathBuf> {
    Some(home_dir()?.join(agent.global_path.as_ref()?))
}

/// Whether an agent's global skills directory is HM-managed (and therefore must
/// not be written to imperatively).
pub(crate) fn global_is_hm_managed(agent: &Agent) -> bool {
    global_base(agent).is_some_and(|base| classify(&base) == TargetState::HmManaged)
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
