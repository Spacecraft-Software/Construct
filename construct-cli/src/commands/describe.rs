// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct describe [<noun> [<verb>]]` — emit the compact capability
//! manifest an agent fetches first for discovery. Smaller than `schema`; pure
//! query, no side effects (schema-introspection §2).

use serde_json::{json, Value};

use crate::context::Context;
use crate::manifest::{self, CommandSpec};
use crate::output::error::AppError;
use crate::output::render;

/// Build and print the capability manifest for the requested scope.
pub(crate) fn run(ctx: &Context, noun: Option<&str>, verb: Option<&str>) -> Result<(), AppError> {
    let info = manifest::tool_info();
    let specs = manifest::commands();
    let matched: Vec<&CommandSpec> = specs.iter().filter(|s| s.matches(noun, verb)).collect();

    if matched.is_empty() {
        return Err(AppError::not_found(
            ctx,
            format!("no command matches {}", filter_label(noun, verb)),
            "construct describe",
        ));
    }

    let commands: Vec<Value> = matched
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "description": s.description,
                "supports_json": s.supports_json,
                "supports_dry_run": s.supports_dry_run,
                "idempotent": s.idempotent,
                "destructive": s.destructive,
            })
        })
        .collect();

    let document = json!({
        "tool": info.name,
        "version": info.version,
        "description": info.description,
        "license": info.license,
        "homepage": info.homepage,
        "commands": commands,
        "global_flags": info.global_flags,
        "output_formats": info.output_formats,
        "mcp_available": info.mcp_available,
        "schema_command": "construct schema",
        "context_files": info.context_files,
    });

    render::emit_raw_json(ctx, &document);
    Ok(())
}

/// Human-readable description of the requested filter for error messages.
fn filter_label(noun: Option<&str>, verb: Option<&str>) -> String {
    match (noun, verb) {
        (Some(n), Some(v)) => format!("noun '{n}' verb '{v}'"),
        (Some(n), None) => format!("noun '{n}'"),
        _ => "the request".to_owned(),
    }
}
