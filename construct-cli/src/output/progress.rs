// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Progress feedback for long operations, on **stderr only** (CLI Standard
//! rule 7).
//!
//! A [`Progress`] is a determinate bar ([`Progress::bar`]: a position out of a
//! known total). Subprocess waits use [`step`] / [`step_command`] instead —
//! static lines, never animated, because a subprocess may prompt on the
//! terminal. What a bar draws is
//! decided once per invocation and carried on the [`Context`] as a [`Style`]:
//!
//! - [`Style::Off`] — the default everywhere except a human terminal. The
//!   handle is a no-op and not one byte reaches stdout or stderr, so every
//!   piped, `--json`, agent, CI, `--quiet`, and `TERM=dumb` run is
//!   byte-identical to a build without this module. See [`enabled`].
//! - [`Style::Animated`] — a braille spinner (and, for a bar, a 20-cell bar
//!   and percentage) redrawn in place on one line, in the `accent` role token
//!   when color is on. The line is erased when the operation ends.
//! - [`Style::Accessible`] — Standard §18.2 accessible mode: no animation, no
//!   glyph spinner, no cursor movement. Progress is a plain, append-only
//!   `Working: <label>… 40%` line, emitted only when the percentage has risen
//!   and at most once per second, then `Working: <label>… done`. Append-only
//!   is deliberate: a screen reader re-reads a repainted region
//!   (`spacecraft-accessibility-support` `references/cli-tui.md`, "linear,
//!   append-only output").
//!
//! Every other stderr writer (diagnostics, errors, subprocess passthrough)
//! goes through [`write_stderr`], which erases a live animated line first and
//! redraws it afterwards, so a progress line and a `[WARN]` line never share a
//! row. One progress line is live at a time; a handle created while another is
//! live is a no-op, keeping the outer operation's line intact.
//!
//! Hand-rolled rather than `indicatif`: the accessible-mode line, the
//! injected-clock rate limit, and the diagnostic suspend hook would all be
//! custom on top of it, and it would add `console`, `unicode-width`, and
//! `portable-atomic` to the locked tree for a spinner and a bar.

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use owo_colors::OwoColorize as _;

use crate::context::Context;
use crate::output::theme;

/// Spinner frame period: 10 frames per second reads as motion without
/// measurable CPU cost.
const TICK: Duration = Duration::from_millis(100);

/// Minimum spacing between accessible-mode lines — Standard §18.2 caps the
/// static progress line at one rewrite per second.
const A11Y_INTERVAL: Duration = Duration::from_secs(1);

/// Braille spinner frames (animated mode only; never in accessible mode,
/// where screen readers would pronounce each glyph).
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Width of the animated bar, in cells.
const BAR_CELLS: u64 = 20;

/// Terminal width assumed when the real width cannot be read.
const FALLBACK_COLUMNS: usize = 80;

/// ANSI: carriage return + erase the whole line. Used only in animated mode.
const CLEAR_LINE: &str = "\r\x1b[2K";

/// How progress is drawn for this invocation. Resolved once in
/// [`Context::from_cli`] and never re-read from the environment.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Style {
    /// No progress output at all.
    Off,
    /// In-place spinner / bar; `color` follows the §6 color cascade.
    Animated {
        /// Paint the spinner and bar in the `accent` role token.
        color: bool,
    },
    /// Standard §18.2 static, append-only, rate-limited lines.
    Accessible,
}

/// Whether progress output is allowed at all. **Every** condition must hold:
/// the output mode is human (not a machine format, not the explore TUI, not
/// the agent/CI cascade — those resolve to JSON before this is asked), stderr
/// is a terminal, `--quiet` is off, and `TERM` is not `dumb` (no cursor
/// movement). Pure so each condition is unit-tested on its own.
pub(crate) fn enabled(human_mode: bool, stderr_tty: bool, quiet: bool, dumb_term: bool) -> bool {
    human_mode && stderr_tty && !quiet && !dumb_term
}

/// Pick the [`Style`] from the gate and the §18.1 accessible-mode toggle.
pub(crate) fn style(enabled: bool, accessible: bool, color: bool) -> Style {
    match (enabled, accessible) {
        (false, _) => Style::Off,
        (true, true) => Style::Accessible,
        (true, false) => Style::Animated { color },
    }
}

// ── the live line ───────────────────────────────────────────────────────────

/// State of the one live progress line. Guarded by [`LIVE`], which is also
/// the lock every stderr write takes, so drawing and diagnostics serialize.
#[derive(Debug)]
struct Live {
    /// Owner handle id; a stale ticker or handle whose id differs is ignored.
    id: u64,
    style: Style,
    label: String,
    message: String,
    pos: u64,
    /// `None` for a spinner.
    total: Option<u64>,
    frame: usize,
    /// Whether an animated line is currently on screen (needs erasing).
    drawn: bool,
    throttle: Throttle,
}

/// The single live progress line, if any. A process-global is the point: the
/// diagnostic and error emitters must find the line without threading a
/// handle through every call site.
static LIVE: Mutex<Option<Live>> = Mutex::new(None);

/// Source of handle ids; `0` is reserved for the no-op handle.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Lock [`LIVE`], recovering from poisoning — a panic mid-draw must not turn
/// every later diagnostic into a second panic.
fn live() -> MutexGuard<'static, Option<Live>> {
    LIVE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Write to stderr without interleaving with a live progress line: an
/// animated line is erased first and redrawn after `write` returns. With no
/// live line this is exactly a locked stderr write — same bytes as before
/// this module existed. Write errors are ignored, as everywhere on stderr.
pub(crate) fn write_stderr(write: impl FnOnce(&mut dyn Write)) {
    let mut guard = live();
    let mut stderr = std::io::stderr().lock();
    if let Some(line) = guard.as_mut() {
        if line.drawn {
            let _ = stderr.write_all(CLEAR_LINE.as_bytes());
            line.drawn = false;
        }
    }
    write(&mut stderr);
    if let Some(line) = guard.as_mut() {
        draw(line, &mut stderr);
    }
    let _ = stderr.flush();
}

/// A handle to a progress line. Dropping it without [`Progress::finish`]
/// (an early `?` return, a panic unwinding) erases the animated line and
/// leaves no `done` claim behind.
pub(crate) struct Progress {
    /// `0` for a no-op handle.
    id: u64,
    ticker: Option<(Sender<()>, JoinHandle<()>)>,
    finished: bool,
}

impl std::fmt::Debug for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Progress")
            .field("id", &self.id)
            .field("ticker", &self.ticker.is_some())
            .field("finished", &self.finished)
            .finish()
    }
}

impl Progress {
    /// A determinate bar of `total` steps.
    pub(crate) fn bar(ctx: &Context, label: impl Into<String>, total: u64) -> Self {
        Self::start(ctx.progress, label.into(), Some(total))
    }

    /// A handle that draws nothing.
    fn noop() -> Self {
        Self {
            id: 0,
            ticker: None,
            finished: true,
        }
    }

    fn start(style: Style, label: String, total: Option<u64>) -> Self {
        if style == Style::Off {
            return Self::noop();
        }
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        {
            let mut guard = live();
            if guard.is_some() {
                // Nested operation: keep the outer line; this one is silent.
                return Self::noop();
            }
            let mut line = Live {
                id,
                style,
                label,
                message: String::new(),
                pos: 0,
                total,
                frame: 0,
                drawn: false,
                throttle: Throttle::default(),
            };
            let mut stderr = std::io::stderr().lock();
            if style == Style::Accessible {
                let pct = total.map(|t| percent(0, t));
                line.throttle.tick(Instant::now(), pct.unwrap_or(0));
                let _ = writeln!(stderr, "{}", a11y_line(&line.label, pct));
            } else {
                draw(&mut line, &mut stderr);
            }
            let _ = stderr.flush();
            *guard = Some(line);
        };
        let ticker = matches!(style, Style::Animated { .. })
            .then(|| spawn_ticker(id))
            .flatten();
        Self {
            id,
            ticker,
            finished: false,
        }
    }

    /// Advance a bar by `n` steps.
    pub(crate) fn inc(&self, n: u64) {
        self.update(|line| line.pos = line.pos.saturating_add(n));
    }

    /// Replace the trailing detail text (current skill, target, step).
    pub(crate) fn set_message(&self, message: impl Into<String>) {
        let message = message.into();
        self.update(move |line| line.message = message);
    }

    /// Mark the operation complete: the animated line is erased; the
    /// accessible line ends with `done`.
    pub(crate) fn finish(mut self) {
        self.finished = true;
        // Drop does the rest.
    }

    fn update(&self, change: impl FnOnce(&mut Live)) {
        if self.id == 0 {
            return;
        }
        let mut guard = live();
        let Some(line) = guard.as_mut().filter(|l| l.id == self.id) else {
            return;
        };
        change(line);
        let mut stderr = std::io::stderr().lock();
        match line.style {
            Style::Accessible => {
                if let Some(total) = line.total {
                    let pct = percent(line.pos, total);
                    if line.throttle.tick(Instant::now(), pct) {
                        let _ = writeln!(stderr, "{}", a11y_line(&line.label, Some(pct)));
                    }
                }
            }
            Style::Animated { .. } => draw(line, &mut stderr),
            Style::Off => {}
        }
        let _ = stderr.flush();
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if self.id == 0 {
            return;
        }
        {
            let mut guard = live();
            if guard.as_ref().is_some_and(|l| l.id == self.id) {
                if let Some(line) = guard.take() {
                    let mut stderr = std::io::stderr().lock();
                    if line.drawn {
                        let _ = stderr.write_all(CLEAR_LINE.as_bytes());
                    }
                    if line.style == Style::Accessible && self.finished {
                        let _ = writeln!(stderr, "{}", a11y_done(&line.label));
                    }
                    let _ = stderr.flush();
                }
            }
        }
        if let Some((stop, handle)) = self.ticker.take() {
            let _ = stop.send(());
            let _ = handle.join();
        }
    }
}

/// How an interactive [`step`] ended.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Outcome {
    Started,
    Done,
    Failed,
}

/// The static line for an interactive step: `Working: <label>…`, then
/// `Working: <label>… done` or `… failed`. Plain text, no escapes.
fn step_line(label: &str, outcome: Outcome) -> String {
    match outcome {
        Outcome::Started => format!("Working: {label}…"),
        Outcome::Done => format!("Working: {label}… done"),
        Outcome::Failed => format!("Working: {label}… failed"),
    }
}

/// Run `work` — a step that may **prompt on the terminal** (git push, gh pr
/// create, git clone/pull: anything that can ask for credentials on the
/// TTY) — with no animation at all. One complete static line is written
/// before `work` starts and one after it ends (`done` on `Ok`, `failed` on
/// `Err`); nothing is redrawn in between, so a credential prompt stays
/// readable. Printed only when progress is enabled (any non-[`Style::Off`]
/// style); animated and accessible mode render it identically.
pub(crate) fn step<T, E>(
    ctx: &Context,
    label: impl Into<String>,
    work: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if ctx.progress == Style::Off {
        return work();
    }
    let label = label.into();
    let line = |outcome| {
        let text = step_line(&label, outcome);
        write_stderr(|stderr| {
            let _ = writeln!(stderr, "{text}");
        });
    };
    line(Outcome::Started);
    let result = work();
    line(if result.is_ok() {
        Outcome::Done
    } else {
        Outcome::Failed
    });
    result
}

/// [`step`] for a subprocess: `failed` covers both a launch error and a
/// non-zero exit status.
pub(crate) fn step_command(
    ctx: &Context,
    label: impl Into<String>,
    command: &mut std::process::Command,
) -> std::io::Result<std::process::Output> {
    let mut raw = None;
    let _ = step(ctx, label, || match command.output() {
        Ok(out) if out.status.success() => {
            raw = Some(Ok(out));
            Ok(())
        }
        other => {
            raw = Some(other);
            Err(())
        }
    });
    raw.unwrap_or_else(|| Err(std::io::Error::other("subprocess did not run")))
}

/// Animate the spinner every [`TICK`] until told to stop. Returns `None`
/// when the thread cannot be spawned; the line then redraws only on updates.
fn spawn_ticker(id: u64) -> Option<(Sender<()>, JoinHandle<()>)> {
    let (stop, stopped) = mpsc::channel::<()>();
    let handle = std::thread::Builder::new()
        .name("construct-progress".to_owned())
        .spawn(move || loop {
            match stopped.recv_timeout(TICK) {
                Err(RecvTimeoutError::Timeout) => {
                    let mut guard = live();
                    let Some(line) = guard.as_mut().filter(|l| l.id == id) else {
                        return;
                    };
                    line.frame = line.frame.wrapping_add(1);
                    let mut stderr = std::io::stderr().lock();
                    draw(line, &mut stderr);
                    let _ = stderr.flush();
                }
                Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
            }
        })
        .ok()?;
    Some((stop, handle))
}

/// Redraw an animated line in place. A no-op for the other styles.
fn draw(line: &mut Live, out: &mut dyn Write) {
    let Style::Animated { color } = line.style else {
        return;
    };
    let text = animated_line(line, color, columns());
    let _ = write!(out, "{CLEAR_LINE}{text}");
    line.drawn = true;
}

/// Current terminal width, falling back to [`FALLBACK_COLUMNS`].
fn columns() -> usize {
    ratatui::crossterm::terminal::size()
        .ok()
        .map(|(cols, _)| usize::from(cols))
        .filter(|&c| c > 0)
        .unwrap_or(FALLBACK_COLUMNS)
}

// ── rendering (pure) ────────────────────────────────────────────────────────

/// Percentage `pos / total`, clamped to 0–100; an empty total is complete.
fn percent(pos: u64, total: u64) -> u8 {
    if total == 0 {
        return 100;
    }
    let pct = pos.min(total).saturating_mul(100) / total;
    u8::try_from(pct).unwrap_or(100)
}

/// The animated line, truncated to `columns - 1` characters so it never
/// wraps (a wrapped line cannot be erased with one `\r\x1b[2K`). Color is
/// applied after truncation, so escape bytes never count against the width.
fn animated_line(line: &Live, color: bool, columns: usize) -> String {
    let spin = SPINNER[line.frame % SPINNER.len()].to_string();
    let (bar_fill, bar_empty, counts) = match line.total {
        Some(total) => {
            let pct = percent(line.pos, total);
            let filled = u64::from(pct) * BAR_CELLS / 100;
            let fill = "█".repeat(usize::try_from(filled).unwrap_or(0));
            let empty = "░".repeat(usize::try_from(BAR_CELLS - filled).unwrap_or(0));
            (
                fill,
                empty,
                format!(" {pct:>3}% {}/{total}", line.pos.min(total)),
            )
        }
        None => (String::new(), String::new(), String::new()),
    };
    let has_bar = line.total.is_some();
    let detail = if line.message.is_empty() {
        String::new()
    } else {
        format!(" · {}", line.message)
    };

    // Plain layout: "<spin> <label> [<bar>] <pct>% <pos>/<total> · <detail>".
    let head = format!("{spin} {}", line.label);
    let bar_plain = if has_bar {
        format!(" [{bar_fill}{bar_empty}]")
    } else {
        String::new()
    };
    let plain = format!("{head}{bar_plain}{counts}{detail}");
    let budget = columns.saturating_sub(1);
    if plain.chars().count() > budget || !color {
        return truncate(&plain, budget);
    }

    let (r, g, b) = theme::ACCENT;
    let bar_colored = if has_bar {
        format!(" [{}{bar_empty}]", bar_fill.truecolor(r, g, b))
    } else {
        String::new()
    };
    format!(
        "{} {}{bar_colored}{counts}{detail}",
        spin.truecolor(r, g, b),
        line.label
    )
}

/// Cut `text` to at most `max` characters.
fn truncate(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// An accessible-mode progress line: `Working: <label>… 40%`, or
/// `Working: <label>…` for an indeterminate operation.
fn a11y_line(label: &str, pct: Option<u8>) -> String {
    match pct {
        Some(pct) => format!("Working: {label}… {pct}%"),
        None => format!("Working: {label}…"),
    }
}

/// The closing accessible-mode line.
fn a11y_done(label: &str) -> String {
    format!("Working: {label}… done")
}

/// Accessible-mode rate limit: a line is emitted only when the percentage has
/// risen (monotonic) **and** at least [`A11Y_INTERVAL`] has passed since the
/// previous one. The clock is a parameter so tests need not sleep.
#[derive(Debug, Default)]
struct Throttle {
    last_at: Option<Instant>,
    last_pct: Option<u8>,
}

impl Throttle {
    /// Record `pct` at `now`; `true` when a line should be written.
    fn tick(&mut self, now: Instant, pct: u8) -> bool {
        let first = self.last_at.is_none();
        let rose = self.last_pct.is_none_or(|last| pct > last);
        let spaced = self
            .last_at
            .is_none_or(|at| now.saturating_duration_since(at) >= A11Y_INTERVAL);
        if first || (rose && spaced) {
            self.last_at = Some(now);
            self.last_pct = Some(pct);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_only_when_every_condition_holds() {
        assert!(enabled(true, true, false, false));
    }

    #[test]
    fn machine_or_agent_mode_disables() {
        // Agent/CI env and piped stdout resolve to JSON: human_mode is false.
        assert!(!enabled(false, true, false, false));
    }

    #[test]
    fn non_tty_stderr_disables() {
        assert!(!enabled(true, false, false, false));
    }

    #[test]
    fn quiet_disables() {
        assert!(!enabled(true, true, true, false));
    }

    #[test]
    fn dumb_term_disables() {
        assert!(!enabled(true, true, false, true));
    }

    #[test]
    fn style_follows_gate_then_accessible_toggle() {
        assert_eq!(style(false, true, true), Style::Off);
        assert_eq!(style(false, false, true), Style::Off);
        assert_eq!(style(true, true, true), Style::Accessible);
        assert_eq!(style(true, false, false), Style::Animated { color: false });
        assert_eq!(style(true, false, true), Style::Animated { color: true });
    }

    #[test]
    fn noop_handle_is_inert() {
        let p = Progress::start(Style::Off, "x".to_owned(), Some(3));
        p.inc(1);
        p.set_message("y");
        assert_eq!(p.id, 0);
        p.finish();
        assert!(live().is_none());
    }

    #[test]
    fn a11y_lines_are_plain_text() {
        assert_eq!(
            a11y_line("building bundles", Some(40)),
            "Working: building bundles… 40%"
        );
        assert_eq!(
            a11y_line("updating flake", None),
            "Working: updating flake…"
        );
        assert_eq!(
            a11y_done("building bundles"),
            "Working: building bundles… done"
        );
        for text in [a11y_line("x", Some(1)), a11y_done("x")] {
            assert!(text.is_ascii() || text.contains('…'));
            assert!(!text.contains('\u{1b}') && !text.contains('\r'));
            assert!(!text.chars().any(|c| SPINNER.contains(&c)));
        }
    }

    #[test]
    fn throttle_caps_at_one_line_per_second() {
        let t0 = Instant::now();
        let mut th = Throttle::default();
        assert!(th.tick(t0, 0), "first line always emitted");
        assert!(!th.tick(t0 + Duration::from_millis(300), 10));
        assert!(!th.tick(t0 + Duration::from_millis(999), 20));
        assert!(th.tick(t0 + Duration::from_secs(1), 30));
        assert!(!th.tick(t0 + Duration::from_millis(1500), 40));
        assert!(th.tick(t0 + Duration::from_millis(2100), 50));
    }

    #[test]
    fn throttle_is_monotonic() {
        let t0 = Instant::now();
        let mut th = Throttle::default();
        assert!(th.tick(t0, 50));
        // A second later but no rise (or a fall): nothing emitted.
        assert!(!th.tick(t0 + Duration::from_secs(2), 50));
        assert!(!th.tick(t0 + Duration::from_secs(3), 40));
        assert!(th.tick(t0 + Duration::from_secs(4), 51));
    }

    #[test]
    fn interactive_step_lines_are_static_text() {
        assert_eq!(
            step_line("pushing feat/x", Outcome::Started),
            "Working: pushing feat/x…"
        );
        assert_eq!(
            step_line("pushing feat/x", Outcome::Done),
            "Working: pushing feat/x… done"
        );
        assert_eq!(
            step_line("pushing feat/x", Outcome::Failed),
            "Working: pushing feat/x… failed"
        );
        for o in [Outcome::Started, Outcome::Done, Outcome::Failed] {
            let t = step_line("x", o);
            assert!(
                !t.contains('\r') && !t.contains('\u{1b}'),
                "no redraw bytes"
            );
            assert!(!t.chars().any(|c| SPINNER.contains(&c)), "no spinner glyph");
        }
    }

    #[test]
    fn percent_is_clamped() {
        assert_eq!(percent(0, 10), 0);
        assert_eq!(percent(4, 10), 40);
        assert_eq!(percent(15, 10), 100);
        assert_eq!(percent(0, 0), 100);
    }

    fn sample(total: Option<u64>, pos: u64, message: &str) -> Live {
        Live {
            id: 1,
            style: Style::Animated { color: false },
            label: "building bundles".to_owned(),
            message: message.to_owned(),
            pos,
            total,
            frame: 0,
            drawn: false,
            throttle: Throttle::default(),
        }
    }

    #[test]
    fn animated_bar_layout() {
        let text = animated_line(&sample(Some(10), 4, "claude/foo"), false, 200);
        assert_eq!(
            text,
            "⠋ building bundles [████████░░░░░░░░░░░░]  40% 4/10 · claude/foo"
        );
    }

    #[test]
    fn animated_line_never_wraps() {
        let text = animated_line(&sample(Some(10), 4, "a-very-long-skill-name"), true, 30);
        assert_eq!(text.chars().count(), 29);
        assert!(!text.contains('\n'));
    }

    #[test]
    fn animated_color_uses_escapes_only_when_asked() {
        let plain = animated_line(&sample(None, 0, ""), false, 200);
        assert_eq!(plain, "⠋ building bundles");
        let colored = animated_line(&sample(None, 0, ""), true, 200);
        assert!(colored.contains('\u{1b}'));
        assert!(colored.contains("building bundles"));
    }
}
