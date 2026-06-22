// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill sync` — update the `construct` flake input in a consuming
//! flake (the flake-update-only operation). This is the typed, tested successor
//! to the bravais `skills-sync` Nushell function: it runs
//! `nix flake update construct` in the target directory and **does not** run
//! `nixos-rebuild` — applying the refreshed skills is a separate, deliberate
//! step.

use std::path::PathBuf;
use std::process::Command as Proc;

use serde_json::json;

use crate::cli::SyncArgs;
use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};

/// Default consuming flake — the bravais NixOS configuration.
const DEFAULT_FLAKE_DIR: &str = "/spacecraft-software/bravais";
/// The flake input that carries the Construct skill catalogue.
const INPUT_NAME: &str = "construct";

/// Run the sync (or, under `--dry-run`, report what it would do).
pub(crate) fn run(ctx: &Context, args: &SyncArgs) -> Result<CommandOutput, AppError> {
    let flake_dir = args
        .flake_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_FLAKE_DIR));

    if !flake_dir.is_dir() {
        return Err(AppError::not_found(
            ctx,
            format!("flake directory '{}' does not exist", flake_dir.display()),
            "construct skill sync --flake-dir <existing-dir>",
        ));
    }
    let flake_dir_str = flake_dir.display().to_string();

    if ctx.dry_run {
        let data = json!({
            "flake_dir": flake_dir_str,
            "input": INPUT_NAME,
            "updated": false,
            "executed": false,
            "action": format!("nix flake update {INPUT_NAME}"),
        });
        let human = HumanRender::Message(format!(
            "[dry-run] would run: nix flake update {INPUT_NAME}  (in {flake_dir_str})"
        ));
        return Ok(CommandOutput::new(data, human));
    }

    let result = Proc::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "flake",
            "update",
            INPUT_NAME,
        ])
        .current_dir(&flake_dir)
        .output();

    let output = match result {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::dependency_missing(
                ctx,
                "`nix` was not found on PATH",
                "nix --version   # install Nix, then re-run construct skill sync",
            ));
        }
        Err(e) => {
            return Err(AppError::general(
                ctx,
                ErrorCode::InternalError,
                format!("failed to launch nix: {e}"),
                format!("cd {flake_dir_str} && nix flake update {INPUT_NAME}"),
            ));
        }
    };

    // nix logs progress to stderr; surface it only when the user asked (-v).
    if ctx.verbose > 0 {
        let progress = String::from_utf8_lossy(&output.stderr);
        if !progress.trim().is_empty() {
            eprint!("{progress}");
        }
    }

    if !output.status.success() {
        return Err(AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!(
                "nix flake update {INPUT_NAME} failed: {}",
                stderr_tail(&output.stderr)
            ),
            format!("cd {flake_dir_str} && nix flake update {INPUT_NAME}"),
        )
        .with_extension("nix_exit_code", json!(output.status.code())));
    }

    let synced_at = crate::time::now_iso8601();
    let data = json!({
        "flake_dir": flake_dir_str,
        "input": INPUT_NAME,
        "updated": true,
        "executed": true,
        "synced_at": synced_at,
    });
    let human = HumanRender::Message(format!(
        "{synced_at}  construct flake input updated in {flake_dir_str} — rebuild to apply"
    ));
    Ok(CommandOutput::new(data, human))
}

/// The last few lines of captured stderr, joined for a one-line error message.
fn stderr_tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut tail: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = tail.len().saturating_sub(3);
    tail.drain(..start);
    tail.join("; ")
}
