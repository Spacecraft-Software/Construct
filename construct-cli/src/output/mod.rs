// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The output contract: mode detection ([`mode`]), the `{ metadata, data }`
//! envelope ([`envelope`]), structured errors ([`error`]), the Steelbore color
//! [`theme`], and the [`render`]er that turns a [`CommandOutput`] into bytes on
//! stdout. Printing happens **only** in this module (CLI Standard §7).

pub(crate) mod envelope;
pub(crate) mod error;
pub(crate) mod mode;
pub(crate) mod render;
pub(crate) mod theme;

use serde_json::Value;

/// What a data-returning command hands back for rendering.
///
/// `data` is the machine-mode `data` payload; `human` is how the same result is
/// drawn on a TTY. Self-describing commands (`schema`, `describe`) bypass this
/// and call [`render::emit_raw_json`] directly.
#[derive(Debug)]
pub(crate) struct CommandOutput {
    /// The `data` field of the JSON envelope.
    pub(crate) data: Value,
    /// The human-mode rendering of the same result.
    pub(crate) human: HumanRender,
}

/// How a [`CommandOutput`] is drawn in human (TTY) mode.
#[allow(
    dead_code,
    reason = "Summary/Table renderings are used by the list/agent commands in later phases"
)]
#[derive(Debug)]
pub(crate) enum HumanRender {
    /// Aligned key/value lines.
    Summary(Vec<(String, String)>),
    /// A bordered table with a header row.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// One or more ready-to-print message lines.
    Message(String),
}

impl CommandOutput {
    /// Construct a command output from its machine payload and human rendering.
    pub(crate) fn new(data: Value, human: HumanRender) -> Self {
        Self { data, human }
    }
}
