// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The JSON output envelope (CLI Standard §6): every machine-mode response is a
//! single document of the shape `{ "metadata": { … }, "data": … }`. Building it
//! goes through this module so no command can accidentally serialize bare data.

use serde::Serialize;
use serde_json::Value;

use crate::context::Context;

/// The `metadata` block carried by every enveloped response.
#[derive(Debug, Serialize)]
pub(crate) struct Metadata {
    /// Binary name.
    pub(crate) tool: &'static str,
    /// Tool version (semver).
    pub(crate) version: &'static str,
    /// Full invocation, normalized.
    pub(crate) command: String,
    /// ISO 8601 UTC response time.
    pub(crate) timestamp: String,
    /// Pagination block, when the command paginates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pagination: Option<Pagination>,
    /// Detected agent harness label (informational), when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_agent: Option<String>,
    /// Present and `true` only on `--dry-run`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) dry_run: bool,
}

/// Pagination metadata for list commands.
#[derive(Debug, Serialize)]
pub(crate) struct Pagination {
    pub(crate) page: u32,
    pub(crate) per_page: u32,
    pub(crate) total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_token: Option<String>,
}

/// Build the metadata block for the current invocation.
pub(crate) fn metadata(ctx: &Context) -> Metadata {
    Metadata {
        tool: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        command: ctx.command.clone(),
        timestamp: crate::time::now_iso8601(),
        pagination: None,
        tool_agent: detect_tool_agent(),
        dry_run: ctx.dry_run,
    }
}

/// Wrap a `data` payload in the full `{ metadata, data }` envelope, consuming
/// `data` (it moves directly into the document).
pub(crate) fn wrap(ctx: &Context, data: Value) -> Value {
    let mut document = serde_json::Map::with_capacity(2);
    document.insert(
        "metadata".to_owned(),
        serde_json::to_value(metadata(ctx)).unwrap_or(Value::Null),
    );
    document.insert("data".to_owned(), data);
    Value::Object(document)
}

/// Detect the agent harness from informational environment variables. These
/// never change the output format on their own (that is `AI_AGENT`'s job); they
/// are recorded for telemetry/debugging only.
fn detect_tool_agent() -> Option<String> {
    const PROBES: [(&str, &str); 3] = [
        ("CLAUDECODE", "claude-code"),
        ("CURSOR_AGENT", "cursor"),
        ("GEMINI_CLI", "gemini-cli"),
    ];
    PROBES
        .iter()
        .find(|(var, _)| std::env::var_os(var).is_some())
        .map(|(_, label)| (*label).to_owned())
}
