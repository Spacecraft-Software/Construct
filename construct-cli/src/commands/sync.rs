// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! `construct skill sync` — update the `construct` flake input in a consuming
//! flake (the flake-update-only operation). This is the typed, tested successor
//! to the bravais `skills-sync` Nushell function: it runs
//! `nix flake update construct` in the target directory and **does not** run
//! `nixos-rebuild` — applying the refreshed skills is a separate, deliberate
//! step. The core [`flake_update`] is reused by `skill ship`.

use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use serde_json::json;

use crate::cli::SyncArgs;
use crate::commands::pointer;
use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};

/// Default consuming flake — the bravais NixOS configuration.
pub(crate) const DEFAULT_FLAKE_DIR: &str = "/spacecraft-software/bravais";
/// The flake input that carries the Construct skill catalogue.
pub(crate) const INPUT_NAME: &str = "construct";

/// Run the sync (or, under `--dry-run`, report what it would do).
pub(crate) fn run(ctx: &Context, args: &SyncArgs) -> Result<CommandOutput, AppError> {
    let flake_dir = args
        .flake_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_FLAKE_DIR));
    let flake_dir_str = flake_dir.display().to_string();

    if ctx.dry_run {
        if !flake_dir.is_dir() {
            return Err(missing_dir(ctx, &flake_dir));
        }
        let mut action = format!("nix flake update {INPUT_NAME}");
        if args.build {
            action.push_str(", then nix build .#skills --out-link <pointer>");
        }
        let data = json!({
            "flake_dir": flake_dir_str,
            "input": INPUT_NAME,
            "updated": false,
            "executed": false,
            "build": args.build,
            "action": action,
        });
        let human = HumanRender::Message(format!(
            "[dry-run] would run: {action}  (in {flake_dir_str})"
        ));
        return Ok(CommandOutput::new(data, human));
    }

    // `--no-update` only reaches here alongside `--build` (clap `requires`), so
    // skipping the update always still leaves something to do.
    let synced_at = if args.no_update {
        if !flake_dir.is_dir() {
            return Err(missing_dir(ctx, &flake_dir));
        }
        None
    } else {
        Some(flake_update(ctx, &flake_dir)?)
    };

    if !args.build {
        let stamp = synced_at.clone().unwrap_or_else(crate::time::now_iso8601);
        let data = json!({
            "flake_dir": flake_dir_str,
            "input": INPUT_NAME,
            "updated": true,
            "executed": true,
            "build": false,
            "synced_at": stamp,
        });
        let human = HumanRender::Message(format!(
            "{stamp}  construct flake input updated in {flake_dir_str} — rebuild to apply"
        ));
        return Ok(CommandOutput::new(data, human));
    }

    let pointer = pointer::pointer_for_sync(ctx, args)?;
    let moved = pointer::build_and_point(ctx, &flake_dir, &pointer)?;
    let stamp = synced_at.unwrap_or_else(crate::time::now_iso8601);
    let changed = moved
        .get("changed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let data = json!({
        "flake_dir": flake_dir_str,
        "input": INPUT_NAME,
        "updated": !args.no_update,
        "executed": true,
        "build": true,
        "synced_at": stamp,
        "pointer": moved,
    });
    let human = HumanRender::Message(if changed {
        format!(
            "{stamp}  skills live at {} — no rebuild needed",
            moved
                .get("store_after")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("<unknown>")
        )
    } else {
        format!("{stamp}  skills already up to date — pointer unchanged")
    });
    Ok(CommandOutput::new(data, human))
}

/// Run `nix flake update construct` in `flake_dir`, returning the ISO 8601 UTC
/// timestamp of the update. Shared by `skill sync` and `skill ship`.
pub(crate) fn flake_update(ctx: &Context, flake_dir: &Path) -> Result<String, AppError> {
    if !flake_dir.is_dir() {
        return Err(missing_dir(ctx, flake_dir));
    }
    let flake_dir_str = flake_dir.display().to_string();

    let result = Proc::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "flake",
            "update",
            INPUT_NAME,
        ])
        .current_dir(flake_dir)
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

    // nix logs progress to stderr; it is info-level passthrough, visible only
    // when the severity floor admits it (`--verbose`, diagnostics.md §4).
    crate::output::diagnostic::emit_passthrough(ctx, &String::from_utf8_lossy(&output.stderr));

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

    Ok(crate::time::now_iso8601())
}

/// The "flake dir does not exist" error.
fn missing_dir(ctx: &Context, flake_dir: &Path) -> AppError {
    AppError::not_found(
        ctx,
        format!("flake directory '{}' does not exist", flake_dir.display()),
        "construct skill sync --flake-dir <existing-dir>",
    )
}

/// The last few lines of captured stderr, joined for a one-line error message.
fn stderr_tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut tail: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = tail.len().saturating_sub(3);
    tail.drain(..start);
    tail.join("; ")
}
