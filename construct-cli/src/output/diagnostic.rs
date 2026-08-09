// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Non-error diagnostics (CLI Standard `references/diagnostics.md`).
//!
//! Warnings, informational notes, and success confirmations share one type:
//! machine mode emits a single-line `{"diagnostic":{severity, code, message,
//! hint?, timestamp, command, …}}` object on stderr; human mode renders the
//! severity-tagged `[WARN] message` layout with an indented `hint:` line. The
//! severity floor (`--quiet` → errors only, agent env → `warn`+, default →
//! `ok`+, `--verbose` → `info`+) gates emission. Errors are not diagnostics —
//! they are [`crate::output::error::AppError`], which is never suppressible.

use std::io::Write as _;

use owo_colors::OwoColorize as _;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::context::Context;
use crate::output::theme;

/// The severity ladder, ordered `Info < Ok < Warn < Error` so that
/// `severity >= floor` is the emission test. The tags are the Steelbore
/// Standard §18.2.1 vocabulary; color is never the sole carrier of meaning.
#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Severity {
    /// Diagnostic narration; hidden unless `--verbose`.
    Info,
    /// Side-effect confirmation.
    Ok,
    /// Degradation or fallback the caller should know about.
    Warn,
    /// Exists for floor comparisons only — error output goes through
    /// [`crate::output::error::AppError`], never a `Diagnostic`.
    Error,
}

impl Severity {
    /// The §18.2.1 text tag. Present in every human-mode line, colored or not.
    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::Info => "[INFO]",
            Self::Ok => "[OK]",
            Self::Warn => "[WARN]",
            Self::Error => "[ERROR]",
        }
    }

    /// The theme role token for the tag (diagnostics.md §5).
    fn color(self) -> (u8, u8, u8) {
        match self {
            Self::Info => theme::STRUCTURE,
            Self::Ok => theme::SUCCESS,
            Self::Warn => theme::WARNING,
            Self::Error => theme::ERROR,
        }
    }
}

/// A non-error diagnostic bound to the invocation that produced it.
#[derive(Debug, Serialize)]
pub(crate) struct Diagnostic {
    /// `"ok"`, `"warn"`, or `"info"` — never `"error"` (that is the `error`
    /// envelope's job).
    pub(crate) severity: Severity,
    /// Stable upper-snake-case code (e.g. `TUI_FALLBACK`).
    pub(crate) code: &'static str,
    /// One-sentence, lowercase, period-free description.
    pub(crate) message: String,
    /// Optional runnable command — same contract as `error.hint`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) hint: Option<String>,
    /// ISO 8601 UTC time the diagnostic was produced.
    pub(crate) timestamp: String,
    /// The invocation that produced it.
    pub(crate) command: String,
    /// Extended structured fields (e.g. `reason`), flattened as siblings.
    #[serde(flatten)]
    pub(crate) extensions: Map<String, Value>,
}

impl Diagnostic {
    /// Construct a diagnostic bound to the current invocation context.
    ///
    /// # Panics
    ///
    /// Debug-asserts that `severity` is not [`Severity::Error`] — an
    /// error-severity diagnostic is a programming error; use `AppError`.
    pub(crate) fn new(
        ctx: &Context,
        severity: Severity,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        debug_assert!(
            severity != Severity::Error,
            "error-severity output goes through AppError, not Diagnostic"
        );
        Self {
            severity,
            code,
            message: message.into(),
            hint: None,
            timestamp: crate::time::now_iso8601(),
            command: ctx.command.clone(),
            extensions: Map::new(),
        }
    }

    /// Attach the runnable recovery/next-step command.
    pub(crate) fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Attach an extended structured field (documented in `schema`).
    pub(crate) fn with_extension(mut self, key: &str, value: Value) -> Self {
        self.extensions.insert(key.to_owned(), value);
        self
    }

    /// The single-line machine form: `{"diagnostic":{…}}`. Compact, because
    /// PowerShell fragments multi-line stderr into separate records.
    pub(crate) fn machine_line(&self) -> String {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            diagnostic: &'a Diagnostic,
        }
        serde_json::to_string(&Wrapper { diagnostic: self })
            .unwrap_or_else(|_| String::from("{\"diagnostic\":{\"severity\":\"warn\"}}"))
    }

    /// The human rendering: `[TAG] message` plus an indented `hint:` line.
    /// The tag is present with and without color (§18.2.1 — color is never
    /// the sole carrier of meaning).
    pub(crate) fn render_human(&self, color: bool) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        if color {
            let (r, g, b) = self.severity.color();
            let tag = self.severity.tag().truecolor(r, g, b).bold().to_string();
            let msg = self.message.truecolor(
                theme::FOREGROUND.0,
                theme::FOREGROUND.1,
                theme::FOREGROUND.2,
            );
            let _ = writeln!(out, "{tag} {msg}");
            if let Some(hint) = &self.hint {
                let label = "hint:".truecolor(theme::ACCENT.0, theme::ACCENT.1, theme::ACCENT.2);
                let text = hint.truecolor(theme::ACCENT.0, theme::ACCENT.1, theme::ACCENT.2);
                let _ = writeln!(out, "  {label} {text}");
            }
        } else {
            let _ = writeln!(out, "{} {}", self.severity.tag(), self.message);
            if let Some(hint) = &self.hint {
                let _ = writeln!(out, "  hint: {hint}");
            }
        }
        out
    }

    /// Emit to stderr in the form the output mode requires, gated by the
    /// severity floor. A suppressed diagnostic is simply not written — never
    /// downgraded or merged into stdout.
    pub(crate) fn emit(&self, ctx: &Context) {
        if !ctx.allows(self.severity) {
            return;
        }
        let mut stderr = std::io::stderr();
        if ctx.mode.is_machine() {
            let _ = writeln!(stderr, "{}", self.machine_line());
        } else {
            let _ = write!(stderr, "{}", self.render_human(ctx.color));
        }
        let _ = stderr.flush();
    }
}

/// Emit the `--format explore` fallback as a warn diagnostic
/// (`references/tui-explore.md` §1). Replaces the deprecated pre-v1.1.0
/// `{"warning":{…}}` shape; the hint is the working non-interactive
/// invocation the caller should use instead.
pub(crate) fn emit_tui_fallback(ctx: &Context, reason: &'static str) {
    Diagnostic::new(
        ctx,
        Severity::Warn,
        "TUI_FALLBACK",
        "interactive explore mode unavailable; falling back to `--format json`",
    )
    .with_hint("construct skill find --json")
    .with_extension("reason", Value::String(reason.to_owned()))
    .emit(ctx);
}

/// Raw passthrough of subprocess progress output to stderr. This is
/// `info`-level output (diagnostics.md §4): visible under `--verbose`,
/// suppressed otherwise. The text is forwarded verbatim — it is opaque
/// third-party output, not a diagnostic envelope.
pub(crate) fn emit_passthrough(ctx: &Context, text: &str) {
    if !ctx.allows(Severity::Info) || text.trim().is_empty() {
        return;
    }
    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "{text}");
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(hint: Option<&str>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warn,
            code: "TUI_FALLBACK",
            message: "interactive explore mode unavailable; falling back to `--format json`"
                .to_owned(),
            hint: hint.map(str::to_owned),
            timestamp: "2026-08-10T00:00:00Z".to_owned(),
            command: "construct skill find --format explore".to_owned(),
            extensions: Map::new(),
        }
    }

    #[test]
    fn machine_line_is_single_line_parseable_envelope() {
        let line = sample(Some("construct skill find --json")).machine_line();
        assert!(!line.contains('\n'), "must be single-line: {line}");
        let value: serde_json::Value = serde_json::from_str(&line).expect("valid JSON");
        assert_eq!(value["diagnostic"]["severity"], "warn");
        assert_eq!(value["diagnostic"]["code"], "TUI_FALLBACK");
        assert_eq!(value["diagnostic"]["hint"], "construct skill find --json");
        assert!(value["diagnostic"]["timestamp"]
            .as_str()
            .is_some_and(|t| t.ends_with('Z')));
    }

    #[test]
    fn machine_line_omits_absent_hint() {
        let value: serde_json::Value =
            serde_json::from_str(&sample(None).machine_line()).expect("valid JSON");
        assert!(value["diagnostic"].get("hint").is_none());
    }

    #[test]
    fn human_render_carries_tag_without_color() {
        let text = sample(Some("construct skill find --json")).render_human(false);
        assert!(text.starts_with("[WARN] "), "tag missing: {text}");
        assert!(text.contains("\n  hint: construct skill find --json"));
        assert!(!text.contains('\u{1b}'), "colorless render leaked ANSI");
    }

    #[test]
    fn human_render_keeps_tag_with_color() {
        let text = sample(None).render_human(true);
        assert!(text.contains("[WARN]"), "tag must survive coloring: {text}");
        assert!(text.contains('\u{1b}'), "colored render carries ANSI");
    }

    #[test]
    fn tags_match_the_standard_vocabulary() {
        assert_eq!(Severity::Info.tag(), "[INFO]");
        assert_eq!(Severity::Ok.tag(), "[OK]");
        assert_eq!(Severity::Warn.tag(), "[WARN]");
        assert_eq!(Severity::Error.tag(), "[ERROR]");
    }
}
