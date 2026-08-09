// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `steelbore` theme: the Steelbore 2 (Steelbore Modern) role tokens of
//! Standard §11.1. This release is construct's "next minor release" that the
//! v1.33 grandfathering clause pointed at, so the legacy six-token palette is
//! replaced by the full eleven-role contract.
//!
//! Colors are referenced through these named role tokens, never as inline hex
//! literals, so a future theme can be substituted in one place. Each token is
//! an RGB triple consumed by `owo-colors`' `truecolor`. Values mirror the
//! canonical `steelbore-color-palette` skill's `steelbore.toml`.

/// `background` — Void Navy. Mandatory canvas / neutral chrome.
pub(crate) const BACKGROUND: (u8, u8, u8) = (0x00, 0x00, 0x27);
/// `surface` — Quantum Blue. Elevated panel/card fill; never a text color.
pub(crate) const SURFACE: (u8, u8, u8) = (0x0E, 0x2A, 0x47);
/// `surface-alt` — Deep Matrix. Code/terminal well fill; never a text color.
pub(crate) const SURFACE_ALT: (u8, u8, u8) = (0x0B, 0x1A, 0x12);
/// `foreground` — Platinum Mist. Body text / default readout.
pub(crate) const FOREGROUND: (u8, u8, u8) = (0xD9, 0xDE, 0xE5);
/// `accent` — Plasma Orange. Primary accent; the `hint:` color.
pub(crate) const ACCENT: (u8, u8, u8) = (0xFF, 0x5E, 0x00);
/// `structure` — Pulse Violet. Structure / links / `[INFO]` diagnostics.
pub(crate) const STRUCTURE: (u8, u8, u8) = (0x8A, 0x6C, 0xFF);
/// `success` — Acid Lime. `[OK]` diagnostics / safe status.
pub(crate) const SUCCESS: (u8, u8, u8) = (0xB4, 0xFF, 0x00);
/// `error` — Mars Red. `[ERROR]` diagnostics.
pub(crate) const ERROR: (u8, u8, u8) = (0xFF, 0x3B, 0x3B);
/// `warning` — Plasma Magenta. `[WARN]` diagnostics.
pub(crate) const WARNING: (u8, u8, u8) = (0xE4, 0x45, 0xFF);
/// `focus` — Acid Lime. Visible focus indicator (alias of `success`).
pub(crate) const FOCUS: (u8, u8, u8) = SUCCESS;
/// `border` — Pulse Violet. Boundaries and table chrome (alias of `structure`).
pub(crate) const BORDER: (u8, u8, u8) = STRUCTURE;

/// Silence "unused" while later phases (TUI, richer rendering) adopt the rest of
/// the palette. Referencing every token keeps them live and documents intent.
#[allow(
    dead_code,
    reason = "full eleven-role theme is part of the public contract; consumed incrementally across phases"
)]
pub(crate) const PALETTE: [(u8, u8, u8); 11] = [
    BACKGROUND,
    SURFACE,
    SURFACE_ALT,
    FOREGROUND,
    ACCENT,
    STRUCTURE,
    SUCCESS,
    ERROR,
    WARNING,
    FOCUS,
    BORDER,
];
