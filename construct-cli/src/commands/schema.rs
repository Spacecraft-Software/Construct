// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct schema [<noun> [<verb>]]` — emit JSON Schema (Draft 2020-12) for
//! the whole tool or a single command. The `parameters` object of each command
//! is directly usable as an LLM function-calling tool definition. Derived from
//! [`crate::manifest`] so it never drifts from the runtime parser.

use serde_json::{json, Value};

use crate::cli::SchemaProvider;
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

    let single = noun.is_some() && verb.is_some() && matched.len() == 1;
    if let Some((provider, _)) = ctx.schema_provider {
        render::emit_raw_json(ctx, &provider_document(provider, &matched, single));
        return Ok(());
    }

    let document = if single {
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

/// The LLM tool name for a command: its path in snake case
/// (`construct skill list` → `construct_skill_list`), which satisfies every
/// provider's `^[a-zA-Z0-9_-]{1,64}$` and MCP's no-hyphen convention.
pub(crate) fn tool_name(spec: &CommandSpec) -> String {
    spec.name.replace([' ', '-'], "_")
}

/// `construct schema --format <provider>` (agentic-cli `llm-tool-calling.md`
/// §2–§6). One command (`schema <noun> <verb>`) renders as one tool; anything
/// wider renders as the whole set in the provider's container shape.
fn provider_document(provider: SchemaProvider, specs: &[&CommandSpec], single: bool) -> Value {
    match provider {
        SchemaProvider::Anthropic => {
            let tools: Vec<Value> = specs
                .iter()
                .map(|s| {
                    json!({
                        "name": tool_name(s),
                        "description": s.description,
                        "input_schema": s.parameters,
                    })
                })
                .collect();
            one_or_all(tools, single)
        }
        SchemaProvider::OpenAi => {
            let tools: Vec<Value> = specs
                .iter()
                .map(|s| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool_name(s),
                            "description": s.description,
                            "parameters": openai_strict(&s.parameters),
                            "strict": true,
                        }
                    })
                })
                .collect();
            one_or_all(tools, single)
        }
        SchemaProvider::Gemini => json!({
            "function_declarations": specs
                .iter()
                .map(|s| json!({
                    "name": tool_name(s),
                    "description": s.description,
                    "parameters": gemini(&s.parameters),
                }))
                .collect::<Vec<_>>(),
        }),
        SchemaProvider::Mcp => json!({
            "tools": specs
                .iter()
                .map(|s| json!({
                    "name": tool_name(s),
                    "description": s.description,
                    "inputSchema": s.parameters,
                }))
                .collect::<Vec<_>>(),
        }),
    }
}

/// A lone tool object for a single command, the array otherwise.
fn one_or_all(mut tools: Vec<Value>, single: bool) -> Value {
    if single && tools.len() == 1 {
        tools.pop().unwrap_or(Value::Null)
    } else {
        Value::Array(tools)
    }
}

/// `format` values OpenAI strict mode accepts; any other is dropped.
const OPENAI_FORMATS: [&str; 9] = [
    "date-time",
    "time",
    "date",
    "duration",
    "email",
    "hostname",
    "ipv4",
    "ipv6",
    "uuid",
];

/// OpenAI strict mode: every object lists all its properties as `required`
/// and sets `additionalProperties: false`; an optional property becomes
/// nullable instead; `default` and unsupported `format`s are dropped.
fn openai_strict(schema: &Value) -> Value {
    let Value::Object(map) = schema else {
        return schema.clone();
    };
    let mut out = map.clone();
    out.remove("default");
    if out
        .get("format")
        .and_then(Value::as_str)
        .is_some_and(|f| !OPENAI_FORMATS.contains(&f))
    {
        out.remove("format");
    }
    if let Some(items) = map.get("items") {
        out.insert("items".to_owned(), openai_strict(items));
    }
    if let Some(Value::Object(props)) = map.get("properties") {
        let required: Vec<&str> = map
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let mut strict = serde_json::Map::with_capacity(props.len());
        for (key, prop) in props {
            let prop = openai_strict(prop);
            let prop = if required.contains(&key.as_str()) {
                prop
            } else {
                nullable(prop)
            };
            strict.insert(key.clone(), prop);
        }
        out.insert(
            "required".to_owned(),
            Value::Array(props.keys().cloned().map(Value::String).collect()),
        );
        out.insert("properties".to_owned(), Value::Object(strict));
        out.insert("additionalProperties".to_owned(), Value::Bool(false));
    }
    Value::Object(out)
}

/// `schema` with `null` added to its `type` (and `enum`, when it has one).
fn nullable(mut schema: Value) -> Value {
    if let Value::Object(map) = &mut schema {
        let null = Value::from("null");
        match map.get_mut("type") {
            Some(Value::String(t)) => {
                let t = Value::String(std::mem::take(t));
                map.insert("type".to_owned(), Value::Array(vec![t, null.clone()]));
            }
            Some(Value::Array(types)) if !types.contains(&null) => types.push(null.clone()),
            _ => {}
        }
        if let Some(Value::Array(values)) = map.get_mut("enum") {
            if !values.contains(&Value::Null) {
                values.push(Value::Null);
            }
        }
    }
    schema
}

/// Gemini `function_declarations` parameters: upper-case type names, a
/// `["T", "null"]` union collapsed to `T` plus `nullable`, `enum` kept on
/// strings only, and the keywords Gemini rejects (`additionalProperties`,
/// `default`, combinators, unsupported `format`s) dropped.
fn gemini(schema: &Value) -> Value {
    let Value::Object(map) = schema else {
        return schema.clone();
    };
    let mut out = serde_json::Map::new();
    let mut is_string = false;
    match map.get("type") {
        Some(Value::String(t)) => {
            is_string = t == "string";
            out.insert("type".to_owned(), Value::String(t.to_uppercase()));
        }
        Some(Value::Array(types)) => {
            if let Some(t) = types
                .iter()
                .filter_map(Value::as_str)
                .find(|t| *t != "null")
            {
                is_string = t == "string";
                out.insert("type".to_owned(), Value::String(t.to_uppercase()));
            }
            if types.iter().any(|t| t == "null") {
                out.insert("nullable".to_owned(), Value::Bool(true));
            }
        }
        _ => {}
    }
    for (key, value) in map {
        match key.as_str() {
            "description" | "minimum" | "maximum" | "minItems" | "maxItems" | "required" => {
                out.insert(key.clone(), value.clone());
            }
            "enum" if is_string => {
                out.insert(key.clone(), value.clone());
            }
            "format" if value == "date-time" => {
                out.insert(key.clone(), value.clone());
            }
            "items" => {
                out.insert(key.clone(), gemini(value));
            }
            "properties" => {
                if let Value::Object(props) = value {
                    let props = props.iter().map(|(k, v)| (k.clone(), gemini(v))).collect();
                    out.insert(key.clone(), Value::Object(props));
                }
            }
            _ => {}
        }
    }
    Value::Object(out)
}

/// Human-readable description of the requested filter for error messages.
fn filter_label(noun: Option<&str>, verb: Option<&str>) -> String {
    match (noun, verb) {
        (Some(n), Some(v)) => format!("noun '{n}' verb '{v}'"),
        (Some(n), None) => format!("noun '{n}'"),
        _ => "the request".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["name"],
            "properties": {
                "name": { "type": "string", "pattern": "^[a-z]+$", "description": "n" },
                "dir": { "type": "string", "format": "uri-reference", "description": "d" },
                "target": { "type": "array", "items": { "type": "string", "enum": ["a", "b"] }, "description": "t" },
                "all": { "type": "boolean", "default": false, "description": "a" }
            }
        })
    }

    #[test]
    fn openai_strict_requires_everything_and_nulls_optionals() {
        let s = openai_strict(&params());
        assert_eq!(s["additionalProperties"], false);
        assert_eq!(s["required"].as_array().map(Vec::len), Some(4));
        assert_eq!(s["properties"]["name"]["type"], "string");
        assert_eq!(s["properties"]["dir"]["type"], json!(["string", "null"]));
        assert!(s["properties"]["dir"].get("format").is_none());
        assert!(s["properties"]["all"].get("default").is_none());
        assert_eq!(s["properties"]["target"]["type"], json!(["array", "null"]));
    }

    #[test]
    fn gemini_upper_cases_and_drops_unsupported_keywords() {
        let g = gemini(&params());
        assert_eq!(g["type"], "OBJECT");
        assert!(g.get("additionalProperties").is_none());
        assert_eq!(g["properties"]["target"]["items"]["type"], "STRING");
        assert_eq!(
            g["properties"]["target"]["items"]["enum"],
            json!(["a", "b"])
        );
        assert!(g["properties"]["dir"].get("format").is_none());
        assert!(g["properties"]["all"].get("default").is_none());
        assert_eq!(
            gemini(&json!({ "type": ["string", "null"] }))["nullable"],
            true
        );
    }
}
