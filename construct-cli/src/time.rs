// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time formatting helpers.
//!
//! Every timestamp the tool stores, transmits, or logs is ISO 8601 in UTC with
//! a mandatory `Z` suffix (Spacecraft Software Standard §12.2, CLI Standard §1).
//! Local time and offset notation are forbidden in all machine-readable output,
//! so this module is the single place timestamps are produced.

/// Current time as an ISO 8601 / RFC 3339 string in UTC with a `Z` suffix.
///
/// # Examples
///
/// ```ignore
/// let ts = construct::time::now_iso8601();
/// assert!(ts.ends_with('Z'));
/// ```
pub(crate) fn now_iso8601() -> String {
    // `jiff::Timestamp` is always UTC and its `Display` emits RFC 3339 with the
    // `Z` suffix, which is exactly the format the Standard mandates.
    jiff::Timestamp::now().to_string()
}
