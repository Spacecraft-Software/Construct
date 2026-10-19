// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct mcp` — every command as an MCP tool, over stdio (CLI Standard
//! `mcp-surface.md`; agentic-cli §6).
//!
//! Newline-delimited JSON-RPC 2.0 on stdin/stdout; stdout carries frames and
//! nothing else, and the server stops when stdin closes. Tool definitions are
//! derived from [`crate::manifest`], the same source as `schema` and
//! `describe`, and each argument is mapped onto the clap flag or positional
//! of the same name, so the three surfaces cannot drift.
//!
//! **Lazy schemas.** `tools/list` advertises each tool's name, one-line
//! description, and capability hints, with a bare `{"type": "object"}` input
//! schema (MCP requires the key). `tools/get` returns one tool's full input
//! and output schemas, exit codes, and examples.
//!
//! **Execution.** `tools/call` runs the matching command as a child process of
//! this same binary, from an argv array (never a shell string), with `--json`
//! and stdin closed. In-process dispatch would let a handler's stdout corrupt
//! the frame stream; the child inherits every CLI rule unchanged — validation,
//! the `--yes` gate on `skill delete`, the structured error contract.

use std::io::{BufRead as _, Read as _};
use std::process::{Command as Process, Stdio};

use serde_json::{json, Map, Value};

use crate::cli::McpArgs;
use crate::commands::schema::tool_name;
use crate::context::Context;
use crate::manifest::{self, CommandSpec};
use crate::output::diagnostic::{Diagnostic, Severity};
use crate::output::render;

/// The protocol revision answered when the client does not name one.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC: invalid JSON.
const PARSE_ERROR: i64 = -32700;
/// JSON-RPC: not a request object.
const INVALID_REQUEST: i64 = -32600;
/// JSON-RPC: unknown method.
const METHOD_NOT_FOUND: i64 = -32601;
/// JSON-RPC: bad `params` (unknown tool, unknown or mistyped argument).
const INVALID_PARAMS: i64 = -32602;
/// MCP server-error range: the command ran and failed (`mcp-surface.md` §7).
const TOOL_FAILED: i64 = -32000;

/// Global flags a tool call may set, beyond the command's own parameters.
const GLOBAL_KEYS: [&str; 3] = ["fields", "dry_run", "yes"];

/// Serve until stdin closes.
pub(crate) fn run(ctx: &Context, _args: &McpArgs) {
    let tools: Vec<CommandSpec> = manifest::commands()
        .into_iter()
        .filter(|s| s.verb != "mcp")
        .collect();
    Diagnostic::new(
        ctx,
        Severity::Info,
        "MCP_READY",
        format!("serving {} tools over stdio", tools.len()),
    )
    .emit(ctx);

    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut line = Vec::new();
    loop {
        line.clear();
        // A read error is treated like EOF: there is no way to answer it.
        match input.by_ref().read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let text = String::from_utf8_lossy(&line);
        if text.trim().is_empty() {
            continue;
        }
        if let Some(frame) = handle_line(&text, &tools) {
            render::write_frame(&frame);
        }
    }
}

/// Answer one input line; `None` for a notification (no `id`).
fn handle_line(text: &str, tools: &[CommandSpec]) -> Option<Value> {
    let request: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(e) => return Some(error_frame(&Value::Null, PARSE_ERROR, &e.to_string(), None)),
    };
    let Some(object) = request.as_object() else {
        return Some(error_frame(
            &Value::Null,
            INVALID_REQUEST,
            "a request is a JSON object (batches are not supported)",
            None,
        ));
    };
    let id = object.get("id").cloned()?;
    let method = object.get("method").and_then(Value::as_str).unwrap_or("");
    let params = object.get("params").cloned().unwrap_or(Value::Null);
    Some(match respond(method, &params, tools) {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(Failure {
            code,
            message,
            data,
        }) => error_frame(&id, code, &message, data),
    })
}

/// A JSON-RPC error to send back.
struct Failure {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl Failure {
    fn params(message: impl Into<String>, hint: &str) -> Self {
        Self {
            code: INVALID_PARAMS,
            message: message.into(),
            data: Some(json!({ "hint": hint })),
        }
    }
}

/// Dispatch one request method.
fn respond(method: &str, params: &Value, tools: &[CommandSpec]) -> Result<Value, Failure> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL_VERSION),
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": env!("CARGO_PKG_NAME"), "version": env!("CARGO_PKG_VERSION") },
            "instructions": "tools/list carries names and descriptions only. Call tools/get with {\"name\": <tool>} for a tool's full input schema before tools/call.",
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": tools.iter().map(advertisement).collect::<Vec<_>>(),
        })),
        "tools/get" => find(tools, params).map(full_definition),
        "tools/call" => {
            let spec = find(tools, params)?;
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            let argv = build_argv(spec, &arguments)?;
            call(&argv)
        }
        _ => Err(Failure {
            code: METHOD_NOT_FOUND,
            message: format!("method '{method}' is not supported"),
            data: Some(json!({ "hint": "tools/list" })),
        }),
    }
}

/// The tool `params.name` names.
fn find<'a>(tools: &'a [CommandSpec], params: &Value) -> Result<&'a CommandSpec, Failure> {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    tools
        .iter()
        .find(|s| tool_name(s) == name)
        .ok_or_else(|| Failure::params(format!("unknown tool '{name}'"), "tools/list"))
}

/// MCP capability hints. A command that accepts `--dry-run` writes
/// (validation-safety §4 requires it of every writer), so the rest are reads.
fn annotations(spec: &CommandSpec) -> Value {
    json!({
        "readOnlyHint": !spec.supports_dry_run,
        "destructiveHint": spec.destructive,
        "idempotentHint": spec.idempotent,
    })
}

/// The `tools/list` entry: name, description, hints, and a bare input schema.
fn advertisement(spec: &CommandSpec) -> Value {
    json!({
        "name": tool_name(spec),
        "description": spec.description,
        "inputSchema": { "type": "object" },
        "annotations": annotations(spec),
    })
}

/// The `tools/get` answer: everything `construct schema <noun> <verb>` says,
/// in MCP's key spelling, with the callable global flags folded in.
fn full_definition(spec: &CommandSpec) -> Value {
    let exit_codes: Map<String, Value> = spec
        .exit_codes
        .iter()
        .map(|(code, desc)| (code.clone(), Value::String(desc.clone())))
        .collect();
    json!({
        "name": tool_name(spec),
        "description": spec.description,
        "inputSchema": input_schema(spec),
        "outputSchema": {
            "type": "object",
            "properties": { "metadata": { "type": "object" }, "data": spec.output_data },
        },
        "annotations": annotations(spec),
        "exit_codes": exit_codes,
        "examples": spec
            .examples
            .iter()
            .map(|(command, description)| json!({ "command": command, "description": description }))
            .collect::<Vec<_>>(),
    })
}

/// The command's parameters plus the global flags a call may set.
fn input_schema(spec: &CommandSpec) -> Value {
    let mut schema = spec.parameters.clone();
    if let Some(Value::Object(props)) = schema.get_mut("properties") {
        if spec.supports_json {
            props.insert(
                "fields".to_owned(),
                json!({ "type": "array", "items": { "type": "string" }, "description": "Restrict data to these fields (token economy)" }),
            );
        }
        if spec.supports_dry_run {
            props.insert(
                "dry_run".to_owned(),
                json!({ "type": "boolean", "default": false, "description": "Return the action plan without changing anything" }),
            );
        }
        if spec.destructive {
            props.insert(
                "yes".to_owned(),
                json!({ "type": "boolean", "default": false, "description": "Confirm the destructive operation; refused without it" }),
            );
        }
    }
    schema
}

/// The manifest key for a clap argument: a positional's id, else its long
/// flag with `_` for `-` (`--flake-dir` → `flake_dir`).
pub(crate) fn arg_key(arg: &clap::Arg) -> String {
    match arg.get_long() {
        Some(long) if !arg.is_positional() => long.replace('-', "_"),
        _ => arg.get_id().to_string(),
    }
}

/// Translate tool arguments into the command's argv, flags first, then `--`
/// and the positionals in order (so a value starting with `-` stays a value),
/// then `--json`. Every key must name a real argument of that command.
fn build_argv(spec: &CommandSpec, arguments: &Value) -> Result<Vec<String>, Failure> {
    let schema_hint = format!("tools/get {{\"name\": \"{}\"}}", tool_name(spec));
    let Some(arguments) = arguments.as_object() else {
        return Err(Failure::params("arguments must be an object", &schema_hint));
    };
    let root = crate::cli::command();
    let path: Vec<&str> = spec.name.split(' ').skip(1).collect();
    let Some(leaf) = path
        .iter()
        .try_fold(&root, |cmd, name| cmd.find_subcommand(name))
    else {
        return Err(Failure::params(
            format!("no command for '{}'", spec.name),
            "tools/list",
        ));
    };

    let mut argv: Vec<String> = path.iter().map(|s| (*s).to_owned()).collect();
    let mut positionals: Vec<(usize, Vec<String>)> = Vec::new();
    for (key, value) in arguments {
        let mistyped =
            || Failure::params(format!("argument '{key}' has the wrong type"), &schema_hint);
        if GLOBAL_KEYS.contains(&key.as_str()) {
            match (key.as_str(), value) {
                ("fields", _) => argv.push(format!(
                    "--fields={}",
                    strings(value).ok_or_else(mistyped)?.join(",")
                )),
                (_, Value::Bool(true)) => argv.push(format!("--{}", key.replace('_', "-"))),
                (_, Value::Bool(false) | Value::Null) => {}
                _ => return Err(mistyped()),
            }
            continue;
        }
        let Some(arg) = leaf
            .get_arguments()
            .find(|a| !a.is_global_set() && arg_key(a) == *key)
        else {
            return Err(Failure::params(
                format!("unknown argument '{key}'"),
                &schema_hint,
            ));
        };
        if value.is_null() {
            continue;
        }
        if arg.is_positional() {
            let index = arg.get_index().unwrap_or(usize::MAX);
            positionals.push((index, strings(value).ok_or_else(mistyped)?));
            continue;
        }
        let long = arg.get_long().unwrap_or(key);
        if arg.get_action().takes_values() {
            for item in strings(value).ok_or_else(mistyped)? {
                argv.push(format!("--{long}={item}"));
            }
        } else {
            match value {
                Value::Bool(true) => argv.push(format!("--{long}")),
                Value::Bool(false) => {}
                _ => return Err(mistyped()),
            }
        }
    }
    argv.push("--json".to_owned());
    positionals.sort_by_key(|(index, _)| *index);
    if !positionals.is_empty() {
        argv.push("--".to_owned());
        argv.extend(positionals.into_iter().flat_map(|(_, values)| values));
    }
    Ok(argv)
}

/// A string, number, or array of them, as argv strings.
fn strings(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::String(s) => Some(vec![s.clone()]),
        Value::Number(n) => Some(vec![n.to_string()]),
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// Run `construct <argv>` and map its outcome: stdout's envelope on success,
/// the structured error from stderr (`mcp-surface.md` §7) on failure.
fn call(argv: &[String]) -> Result<Value, Failure> {
    let exe = std::env::current_exe().map_err(|e| Failure {
        code: TOOL_FAILED,
        message: "INTERNAL_ERROR".to_owned(),
        data: Some(json!({ "message": format!("cannot locate the construct binary: {e}") })),
    })?;
    let output = Process::new(exe)
        .args(argv)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Failure {
            code: TOOL_FAILED,
            message: "INTERNAL_ERROR".to_owned(),
            data: Some(json!({ "message": format!("cannot run construct: {e}") })),
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if output.status.success() {
        let mut result =
            json!({ "content": [{ "type": "text", "text": stdout }], "isError": false });
        if let Ok(document @ Value::Object(_)) = serde_json::from_str::<Value>(&stdout) {
            result["structuredContent"] = document;
        }
        return Ok(result);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let error = stderr
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|v| v.get("error").cloned());
    Err(match error {
        Some(error) => Failure {
            code: TOOL_FAILED,
            message: error["code"]
                .as_str()
                .unwrap_or("GENERAL_FAILURE")
                .to_owned(),
            data: Some(error),
        },
        None => Failure {
            code: TOOL_FAILED,
            message: "GENERAL_FAILURE".to_owned(),
            data: Some(json!({
                "exit_code": output.status.code(),
                "message": stderr.trim(),
            })),
        },
    })
}

/// A JSON-RPC error frame.
fn error_frame(id: &Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut error = json!({ "code": code, "message": message });
    if let Some(data) = data {
        error["data"] = data;
    }
    json!({ "jsonrpc": "2.0", "id": id, "error": error })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(name: &str) -> CommandSpec {
        manifest::commands()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("no spec {name}"))
    }

    #[test]
    fn argv_maps_flags_positionals_and_globals() {
        let argv = build_argv(
            &spec("construct skill build"),
            &json!({ "skills": ["-odd"], "target": ["claude", "grok"], "dry_run": true }),
        )
        .unwrap_or_else(|f| panic!("{}", f.message));
        assert_eq!(
            argv,
            [
                "skill",
                "build",
                "--dry-run",
                "--target=claude",
                "--target=grok",
                "--json",
                "--",
                "-odd"
            ]
        );
    }

    #[test]
    fn argv_refuses_unknown_and_mistyped_arguments() {
        let list = spec("construct skill list");
        assert!(build_argv(&list, &json!({ "bogus": 1 })).is_err());
        assert!(build_argv(&list, &json!({ "global": "yes" })).is_err());
        assert!(build_argv(&list, &json!([])).is_err());
    }

    #[test]
    fn list_is_lazy_and_get_is_full() {
        let tools = vec![spec("construct skill delete")];
        let list =
            respond("tools/list", &Value::Null, &tools).unwrap_or_else(|f| panic!("{}", f.message));
        let entry = &list["tools"][0];
        assert_eq!(entry["name"], "construct_skill_delete");
        assert_eq!(entry["inputSchema"], json!({ "type": "object" }));
        assert_eq!(entry["annotations"]["destructiveHint"], true);
        let full = respond(
            "tools/get",
            &json!({ "name": "construct_skill_delete" }),
            &tools,
        )
        .unwrap_or_else(|f| panic!("{}", f.message));
        assert!(full["inputSchema"]["properties"]["yes"].is_object());
        assert!(full["inputSchema"]["properties"]["skills"].is_object());
    }

    #[test]
    fn notifications_get_no_answer_and_bad_json_does() {
        assert!(handle_line(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            &[]
        )
        .is_none());
        let frame = handle_line("{not json", &[]).unwrap_or_default();
        assert_eq!(frame["error"]["code"], PARSE_ERROR);
        let frame =
            handle_line(r#"{"jsonrpc":"2.0","id":7,"method":"nope"}"#, &[]).unwrap_or_default();
        assert_eq!(frame["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(frame["id"], 7);
    }
}
