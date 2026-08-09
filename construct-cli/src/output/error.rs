// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Structured errors (CLI Standard §1 item 8, exit-codes-errors reference).
//!
//! Every non-zero exit in machine mode emits a single-line JSON object to
//! stderr with the `error.{code, exit_code, message, hint, timestamp, command,
//! docs_url}` shape. The `hint` is always a runnable command, never prose, so an
//! agent can self-correct ("tips thinking").

use std::io::Write as _;

use owo_colors::OwoColorize as _;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::context::Context;
use crate::output::mode::OutputMode;
use crate::output::theme;

/// The canonical, stable error-code enum (exit-codes-errors §3). Tool-specific
/// codes may be added later; each must be documented in `construct schema`.
#[allow(
    dead_code,
    reason = "canonical CLI Standard error-code set; some codes are first constructed in later phases"
)]
#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ErrorCode {
    NotFound,
    PermissionDenied,
    InvalidArgument,
    MissingArgument,
    Conflict,
    Timeout,
    NetworkError,
    DependencyMissing,
    InternalError,
    FeatureUnavailable,
}

/// A structured, agent-actionable error.
#[derive(Debug, Serialize)]
pub(crate) struct AppError {
    /// Stable upper-snake-case code.
    pub(crate) code: ErrorCode,
    /// Process exit code; matches the canonical map (§4).
    pub(crate) exit_code: i32,
    /// One-sentence, period-free description.
    pub(crate) message: String,
    /// A runnable recovery command — never prose.
    pub(crate) hint: String,
    /// ISO 8601 UTC time the error was produced.
    pub(crate) timestamp: String,
    /// The invocation that failed.
    pub(crate) command: String,
    /// Optional docs link for this error class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) docs_url: Option<String>,
    /// Extended structured fields (e.g. `retry_after`), flattened as siblings.
    #[serde(flatten)]
    pub(crate) extensions: Map<String, Value>,
}

impl AppError {
    /// Construct an error bound to the current invocation context.
    pub(crate) fn new(
        ctx: &Context,
        code: ErrorCode,
        exit_code: i32,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code,
            exit_code,
            message: message.into(),
            hint: hint.into(),
            timestamp: crate::time::now_iso8601(),
            command: ctx.command.clone(),
            docs_url: None,
            extensions: Map::new(),
        }
    }

    /// Attach an extended structured field (documented in `schema`).
    pub(crate) fn with_extension(mut self, key: &str, value: Value) -> Self {
        self.extensions.insert(key.to_owned(), value);
        self
    }

    /// A required external tool is missing from `PATH` (exit 127).
    pub(crate) fn dependency_missing(
        ctx: &Context,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(ctx, ErrorCode::DependencyMissing, 127, message, hint)
    }

    /// A referenced resource does not exist (exit 3).
    pub(crate) fn not_found(
        ctx: &Context,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(ctx, ErrorCode::NotFound, 3, message, hint)
    }

    /// An argument failed validation (exit 2).
    #[allow(dead_code, reason = "input-validation helper consumed by later phases")]
    pub(crate) fn invalid_argument(
        ctx: &Context,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(ctx, ErrorCode::InvalidArgument, 2, message, hint)
    }

    /// A general subprocess/operation failure (exit 1).
    pub(crate) fn general(
        ctx: &Context,
        code: ErrorCode,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(ctx, code, 1, message, hint)
    }

    /// An unexpected internal failure (exit 1).
    pub(crate) fn internal(
        ctx: &Context,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::new(ctx, ErrorCode::InternalError, 1, message, hint)
    }

    /// A usage error raised before a full [`Context`] exists (clap parse
    /// failure). Reconstructs `command` from the raw arguments.
    pub(crate) fn usage_early(message: &str) -> Self {
        let mut args: Vec<String> = std::env::args().collect();
        if let Some(first) = args.first_mut() {
            "construct".clone_into(first);
        }
        Self {
            code: ErrorCode::InvalidArgument,
            exit_code: 2,
            message: message.to_owned(),
            hint: "construct --help".to_owned(),
            timestamp: crate::time::now_iso8601(),
            command: args.join(" "),
            docs_url: None,
            extensions: Map::new(),
        }
    }

    /// Serialize as a single-line `{ "error": … }` object on stderr. Compact,
    /// because PowerShell fragments multi-line stderr into separate records.
    pub(crate) fn emit_to_stderr(&self) {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            error: &'a AppError,
        }
        let line = serde_json::to_string(&Wrapper { error: self })
            .unwrap_or_else(|_| String::from("{\"error\":{\"code\":\"INTERNAL_ERROR\"}}"));
        let mut stderr = std::io::stderr();
        let _ = writeln!(stderr, "{line}");
        let _ = stderr.flush();
    }

    /// Render the error for a human terminal in the unified diagnostic layout
    /// (diagnostics.md §5): the `[ERROR]` tag first — carried with and without
    /// color, since color is never the sole carrier of meaning (§18.2.1) —
    /// then the message in the default foreground, then the indented `hint:`.
    /// Content matches the structured form.
    pub(crate) fn render_human(&self, color: bool) -> String {
        if color {
            let tag = "[ERROR]"
                .truecolor(theme::ERROR.0, theme::ERROR.1, theme::ERROR.2)
                .bold()
                .to_string();
            let msg = self.message.truecolor(
                theme::FOREGROUND.0,
                theme::FOREGROUND.1,
                theme::FOREGROUND.2,
            );
            let label = "hint:".truecolor(theme::ACCENT.0, theme::ACCENT.1, theme::ACCENT.2);
            let hint = self
                .hint
                .truecolor(theme::ACCENT.0, theme::ACCENT.1, theme::ACCENT.2);
            format!("{tag} {msg}\n  {label} {hint}\n")
        } else {
            format!("[ERROR] {}\n  hint: {}\n", self.message, self.hint)
        }
    }

    /// Write the human rendering to stderr.
    fn emit_human(&self, color: bool) {
        let mut stderr = std::io::stderr();
        let _ = write!(stderr, "{}", self.render_human(color));
        let _ = stderr.flush();
    }
}

/// Emit an error in the form appropriate to the output mode and return its exit
/// code. Machine modes get the structured JSON object; human modes get the
/// tagged rendering. Errors never consult the severity floor.
pub(crate) fn report(err: &AppError, mode: OutputMode) -> i32 {
    if mode.is_machine() {
        err.emit_to_stderr();
    } else {
        err.emit_human(mode == OutputMode::HumanWithColor);
    }
    err.exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AppError {
        AppError {
            code: ErrorCode::NotFound,
            exit_code: 3,
            message: "skill `foo` does not exist".to_owned(),
            hint: "construct skill find --json".to_owned(),
            timestamp: "2026-08-10T00:00:00Z".to_owned(),
            command: "construct skill use foo".to_owned(),
            docs_url: None,
            extensions: Map::new(),
        }
    }

    #[test]
    fn human_render_carries_error_tag_without_color() {
        let text = sample().render_human(false);
        assert!(text.starts_with("[ERROR] "), "tag missing: {text}");
        assert!(text.contains("\n  hint: construct skill find --json"));
        assert!(!text.contains('\u{1b}'), "colorless render leaked ANSI");
    }

    #[test]
    fn human_render_keeps_tag_with_color() {
        let text = sample().render_human(true);
        assert!(
            text.contains("[ERROR]"),
            "tag must survive coloring: {text}"
        );
        assert!(text.contains('\u{1b}'), "colored render carries ANSI");
    }
}
