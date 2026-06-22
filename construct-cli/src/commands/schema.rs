// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct schema [<noun> [<verb>]]` — emit JSON Schema (Draft 2020-12) for
//! the whole tool or a single command. The `parameters` object of each command
//! is directly usable as an LLM function-calling tool definition. Derived from
//! [`crate::manifest`] so it never drifts from the runtime parser.

use serde_json::{json, Value};

use crate::context::Context;
use crate::manifest::{self, CommandSpec};
use crate::output::error::AppError;
use crate::output::render;

/// The JSON Schema dialect we emit.
const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Build and print the schema document for the requested scope.
pub(crate) fn run(ctx: &Context, noun: Option<&str>, verb: Option<&str>) -> Result<(), AppError> {
    let specs = manifest::commands();
    let matched: Vec<&CommandSpec> = specs.iter().filter(|s| s.matches(noun, verb)).collect();

    if matched.is_empty() {
        return Err(AppError::not_found(
            ctx,
            format!("no command matches {}", filter_label(noun, verb)),
            "construct schema",
        ));
    }

    let document = if noun.is_some() && verb.is_some() && matched.len() == 1 {
        let mut single = command_schema(matched[0]);
        if let Value::Object(map) = &mut single {
            map.insert("$schema".to_owned(), json!(DRAFT));
        }
        single
    } else {
        let info = manifest::tool_info();
        json!({
            "$schema": DRAFT,
            "tool": info.name,
            "version": info.version,
            "description": info.description,
            "commands": matched.iter().map(|s| command_schema(s)).collect::<Vec<_>>(),
        })
    };

    render::emit_raw_json(ctx, &document);
    Ok(())
}

/// The per-command schema object (schema-introspection §1).
fn command_schema(spec: &CommandSpec) -> Value {
    let exit_codes: serde_json::Map<String, Value> = spec
        .exit_codes
        .iter()
        .map(|(code, desc)| (code.clone(), Value::String(desc.clone())))
        .collect();
    let examples: Vec<Value> = spec
        .examples
        .iter()
        .map(|(command, description)| json!({ "command": command, "description": description }))
        .collect();

    json!({
        "command": spec.name,
        "description": spec.description,
        "parameters": spec.parameters,
        "output_schema": {
            "type": "object",
            "properties": {
                "metadata": { "type": "object" },
                "data": spec.output_data,
            }
        },
        "exit_codes": Value::Object(exit_codes),
        "examples": examples,
        "supports_json": spec.supports_json,
        "supports_dry_run": spec.supports_dry_run,
        "idempotent": spec.idempotent,
        "destructive": spec.destructive,
    })
}

/// Human-readable description of the requested filter for error messages.
fn filter_label(noun: Option<&str>, verb: Option<&str>) -> String {
    match (noun, verb) {
        (Some(n), Some(v)) => format!("noun '{n}' verb '{v}'"),
        (Some(n), None) => format!("noun '{n}'"),
        _ => "the request".to_owned(),
    }
}
