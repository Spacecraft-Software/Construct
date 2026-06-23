// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Steelbore six-token color palette (Steelbore Standard §11).
//!
//! Colors are referenced through these named tokens, never as inline hex
//! literals, so a future theme can be substituted in one place. Each token is
//! an RGB triple consumed by `owo-colors`' `truecolor`.

/// Mandatory background / neutral chrome.
pub(crate) const VOID_NAVY: (u8, u8, u8) = (0x00, 0x00, 0x27);
/// Primary text, warnings, hints.
pub(crate) const MOLTEN_AMBER: (u8, u8, u8) = (0xD9, 0x8E, 0x32);
/// Accent / informational / structural.
pub(crate) const STEEL_BLUE: (u8, u8, u8) = (0x4B, 0x7E, 0xB0);
/// Success / safe status.
pub(crate) const RADIUM_GREEN: (u8, u8, u8) = (0x50, 0xFA, 0x7B);
/// Errors / warning status.
pub(crate) const RED_OXIDE: (u8, u8, u8) = (0xFF, 0x5C, 0x5C);
/// Data values / links / info.
pub(crate) const LIQUID_COOLANT: (u8, u8, u8) = (0x8B, 0xE9, 0xFD);

/// Silence "unused" while later phases (TUI, richer rendering) adopt the rest of
/// the palette. Referencing every token keeps them live and documents intent.
#[allow(
    dead_code,
    reason = "full palette is part of the public theme; consumed incrementally across phases"
)]
pub(crate) const PALETTE: [(u8, u8, u8); 6] = [
    VOID_NAVY,
    MOLTEN_AMBER,
    STEEL_BLUE,
    RADIUM_GREEN,
    RED_OXIDE,
    LIQUID_COOLANT,
];
