// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time formatting helpers.
//!
//! Every timestamp the tool stores, transmits, or logs is ISO 8601 in UTC with
//! a mandatory `Z` suffix (Steelbore Standard §14.2, CLI Standard §1).
//! Local time and offset notation are forbidden in all machine-readable output,
//! so this module is the single place timestamps are produced.

/// Current time as `YYYY-MM-DDTHH:MM:SSZ`: ISO 8601 in UTC, whole seconds,
/// mandatory `Z` suffix — the exact timestamp form of Steelbore Standard §14
/// and CLI Standard §1 item 1. Sub-second digits are truncated, never rounded,
/// so a timestamp never names a second that has not started yet.
///
/// # Examples
///
/// ```ignore
/// let ts = construct::time::now_iso8601();
/// assert!(ts.ends_with('Z'));
/// ```
pub(crate) fn now_iso8601() -> String {
    format_utc_seconds(jiff::Timestamp::now())
}

/// `ts` as `YYYY-MM-DDTHH:MM:SSZ`. `jiff::Timestamp` is always UTC; its
/// `Display` would add nanoseconds, so the seconds form is spelled out.
fn format_utc_seconds(ts: jiff::Timestamp) -> String {
    ts.strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_precision_with_z_suffix() {
        let ts: jiff::Timestamp = "2026-04-10T14:30:00.987654321Z".parse().unwrap();
        assert_eq!(format_utc_seconds(ts), "2026-04-10T14:30:00Z");
    }

    #[test]
    fn now_matches_the_standard_shape() {
        let now = now_iso8601();
        assert_eq!(now.len(), "YYYY-MM-DDTHH:MM:SSZ".len(), "{now}");
        assert!(now.ends_with('Z') && now.as_bytes()[10] == b'T', "{now}");
    }
}
