// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The agent registry: the set of AI coding agents `construct` can install
//! skills into. The data lives in `registry/agents.toml` (derived from
//! vercel-labs/skills; see `CREDITS.md`) and is embedded into the binary at
//! compile time so the tool is self-contained. Adding an agent is a one-entry
//! edit to that file — no code change.

pub(crate) mod detect;

use serde::{Deserialize, Serialize};

/// The embedded registry source, parsed at runtime into [`Agent`]s.
const AGENTS_TOML: &str = include_str!("../../registry/agents.toml");

/// How an agent lays out installed skills on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SkillFormat {
    /// Skills live at `<path>/<skill-name>/SKILL.md` (the common case).
    Directory,
    /// `SKILL.md` lives at the path root (e.g. Grok's flat bundles).
    Flat,
}

/// One agent's install targets.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Agent {
    /// Stable agent identifier (e.g. `claude-code`, `cursor`).
    pub(crate) id: String,
    /// Human-friendly name.
    pub(crate) display_name: String,
    /// Project-relative skills directory (e.g. `.claude/skills`).
    pub(crate) project_path: String,
    /// Home-relative global skills directory, if the agent supports one.
    #[serde(default)]
    pub(crate) global_path: Option<String>,
    /// On-disk skill layout.
    pub(crate) format: SkillFormat,
}

/// Internal shape of `agents.toml` (`[[agent]]` array of tables).
#[derive(Debug, Deserialize)]
struct RegistryFile {
    agent: Vec<Agent>,
}

/// All registered agents.
///
/// The registry ships embedded in the binary, so a parse failure is a
/// programming error (bad data committed), not a runtime condition — it panics
/// per M-PANIC-ON-BUG. `tests::registry_parses_and_is_unique` guards it.
pub(crate) fn all() -> Vec<Agent> {
    let parsed: RegistryFile =
        toml::from_str(AGENTS_TOML).expect("embedded registry/agents.toml is valid");
    parsed.agent
}

/// Find an agent by id.
#[allow(
    dead_code,
    reason = "consumed by the installer (`skill add`) in this phase's next step"
)]
pub(crate) fn find(agents: &[Agent], id: &str) -> Option<Agent> {
    agents.iter().find(|a| a.id == id).cloned()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{all, SkillFormat};

    #[test]
    fn registry_parses_and_is_unique() {
        let agents = all();
        assert!(
            agents.len() >= 70,
            "expected the full agent registry, got {}",
            agents.len()
        );
        let ids: HashSet<&String> = agents.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), agents.len(), "duplicate agent ids in registry");
        // Every agent has a non-empty id and project path.
        for a in &agents {
            assert!(!a.id.is_empty(), "empty agent id");
            assert!(
                !a.project_path.is_empty(),
                "empty project_path for {}",
                a.id
            );
        }
        // Grok is the flat-format exception and must be present.
        let grok = agents
            .iter()
            .find(|a| a.id == "grok")
            .expect("grok present");
        assert_eq!(grok.format, SkillFormat::Flat);
    }
}
