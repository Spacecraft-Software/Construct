// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct agent list` — enumerate every agent in the registry with its
//! install paths, a best-effort installed flag, and whether its global skills
//! directory is managed by the Construct Home-Manager module (in which case
//! global installs are refused; see `install`).

use serde_json::{json, Value};

use crate::context::Context;
use crate::output::{CommandOutput, HumanRender};
use crate::registry::{self, detect};

/// List all registered agents.
pub(crate) fn list(_ctx: &Context) -> CommandOutput {
    let agents = registry::all();
    let mut records = Vec::with_capacity(agents.len());
    let mut rows = Vec::with_capacity(agents.len());

    for agent in &agents {
        let installed = detect::detect_installed(agent);
        let hm_managed = detect::global_is_hm_managed(agent);
        records.push(json!({
            "id": agent.id,
            "display_name": agent.display_name,
            "project_path": agent.project_path,
            "global_path": agent.global_path,
            "format": agent.format,
            "installed": installed,
            "hm_managed": hm_managed,
        }));
        rows.push(vec![
            agent.id.clone(),
            if installed {
                "yes".to_owned()
            } else {
                "-".to_owned()
            },
            if hm_managed {
                "hm".to_owned()
            } else {
                "-".to_owned()
            },
            agent
                .global_path
                .clone()
                .unwrap_or_else(|| "(project-only)".to_owned()),
        ]);
    }

    let human = HumanRender::Table {
        headers: vec![
            "AGENT".to_owned(),
            "INSTALLED".to_owned(),
            "HM".to_owned(),
            "GLOBAL PATH".to_owned(),
        ],
        rows,
    };
    CommandOutput::new(Value::Array(records), human)
}
