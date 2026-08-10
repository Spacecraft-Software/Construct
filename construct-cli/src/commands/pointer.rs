// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mutable skill-tree pointer — `construct skill status` / `skill reset`,
//! and the build half of `skill sync --build`.
//!
//! Under the Home-Manager module's `mutablePointer` mode, `~/.agents/skills`
//! is a symlink to `<stateDir>/current`, and `current` is either:
//!
//! * a symlink to `<stateDir>/pinned` — "tracking the flake"; `pinned` is a
//!   Home-Manager-owned link, so the tree is GC-rooted through the generation,
//!   or
//! * a `nix build --out-link` result pointing straight into the store — "moved
//!   ahead", GC-rooted by the indirect root `--out-link` registers under
//!   `/nix/var/nix/gcroots/auto/`.
//!
//! "Moved ahead" only lasts until the next rebuild: activation re-points
//! `current` at `pinned` every time, so `flake.lock` stays authoritative and a
//! rebuild can never leave the machine on a stale tree. `skill reset` is the
//! same snap-back without waiting for one.
//!
//! Both shapes are rooted, which is the entire safety argument. The invariant
//! that keeps it true: **`current` is only ever written by `nix build
//! --out-link`, or pointed at `pinned`.** Never at a bare store path by hand —
//! that shape has no root and the next `nix-collect-garbage` deletes the tree
//! out from under every agent on the machine.

use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use serde_json::json;

use crate::cli::{PointerArgs, SyncArgs};
use crate::context::Context;
use crate::output::error::{AppError, ErrorCode};
use crate::output::{CommandOutput, HumanRender};
use crate::registry::detect::home_dir;

/// Home-relative default pointer directory. Mirrors the Home-Manager module's
/// `mutablePointer.stateDir` default — change one and you must change both.
const DEFAULT_STATE_DIR: &str = ".local/state/construct";

/// The flake output holding the whole skill tree.
const SKILLS_OUTPUT: &str = "skills";

/// Resolve the pointer path: the `--pointer` override, else the default.
fn resolve_pointer(ctx: &Context, args: &PointerArgs) -> Result<PathBuf, AppError> {
    if let Some(explicit) = &args.pointer {
        return Ok(explicit.clone());
    }
    let home = home_dir().ok_or_else(|| {
        AppError::general(
            ctx,
            ErrorCode::InternalError,
            "HOME is not set, so the default pointer path cannot be resolved",
            "construct skill status --pointer ~/.local/state/construct/current",
        )
    })?;
    Ok(home.join(DEFAULT_STATE_DIR).join("current"))
}

/// `pinned` sits beside `current` in the same state directory.
fn pinned_beside(pointer: &Path) -> PathBuf {
    pointer.with_file_name("pinned")
}

/// Where a symlink points, or `None` if it is absent or not a symlink.
fn link_target(path: &Path) -> Option<PathBuf> {
    std::fs::read_link(path).ok()
}

/// The store path a link finally resolves to, or `None` when it dangles.
fn resolved(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

/// The error for "this machine is not running the pointer layout".
fn not_pointer_managed(ctx: &Context, pointer: &Path) -> AppError {
    AppError::not_found(
        ctx,
        format!(
            "no skill pointer at '{}' — this host is not running the mutablePointer layout",
            pointer.display()
        ),
        "set spacecraft.construct.mutablePointer.enable = true, then rebuild",
    )
}

// ── status ──────────────────────────────────────────────────────────────────

/// Report where the pointer stands relative to the flake-pinned tree.
pub(crate) fn status(ctx: &Context, args: &PointerArgs) -> Result<CommandOutput, AppError> {
    let pointer = resolve_pointer(ctx, args)?;
    let pinned = pinned_beside(&pointer);

    if link_target(&pointer).is_none() {
        return Err(not_pointer_managed(ctx, &pointer));
    }

    let pinned_store = resolved(&pinned);
    let current_store = resolved(&pointer);
    // Tracking means the pointer resolves to the same tree the flake pins —
    // whether it gets there via `pinned` or by having been built from the same
    // locked revision. Comparing the resolved store paths, not the link text,
    // is what makes `sync --build` followed by no catalogue change report
    // "tracking" rather than a phantom move.
    let tracking = match (&pinned_store, &current_store) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    };
    let dangling = current_store.is_none();

    let data = json!({
        "pointer": pointer.display().to_string(),
        "pinned_link": pinned.display().to_string(),
        "pinned_store": pinned_store.as_ref().map(|p| p.display().to_string()),
        "current_store": current_store.as_ref().map(|p| p.display().to_string()),
        "tracking_flake": tracking,
        "dangling": dangling,
    });

    let state = if dangling {
        "DANGLING — target was collected; run: construct skill sync --build".to_owned()
    } else if tracking {
        "tracking the flake".to_owned()
    } else {
        "moved ahead of the flake (a rebuild resets it)".to_owned()
    };
    let human = HumanRender::Summary(vec![
        ("pointer".to_owned(), pointer.display().to_string()),
        ("state".to_owned(), state),
        (
            "pinned".to_owned(),
            display_or_dash(pinned_store.as_deref()),
        ),
        (
            "current".to_owned(),
            display_or_dash(current_store.as_deref()),
        ),
    ]);
    Ok(CommandOutput::new(data, human))
}

fn display_or_dash(path: Option<&Path>) -> String {
    path.map_or_else(|| "-".to_owned(), |p| p.display().to_string())
}

// ── reset ───────────────────────────────────────────────────────────────────

/// Point `current` back at `pinned`, i.e. back to what `flake.lock` pins.
///
/// Re-pointing at `pinned` rather than at a store path is what keeps the result
/// rooted without a fresh `nix build` — see the module docs.
pub(crate) fn reset(ctx: &Context, args: &PointerArgs) -> Result<CommandOutput, AppError> {
    let pointer = resolve_pointer(ctx, args)?;
    let pinned = pinned_beside(&pointer);

    if link_target(&pointer).is_none() {
        return Err(not_pointer_managed(ctx, &pointer));
    }
    if link_target(&pinned).is_none() {
        return Err(AppError::not_found(
            ctx,
            format!(
                "'{}' is missing — Home Manager did not render the pinned tree",
                pinned.display()
            ),
            "rebuild to restore the pinned tree, then re-run construct skill reset",
        ));
    }

    let before = resolved(&pointer);
    if ctx.dry_run {
        let human = HumanRender::Message(format!(
            "[dry-run] would point {} at {}",
            pointer.display(),
            pinned.display()
        ));
        return Ok(CommandOutput::new(
            json!({
                "pointer": pointer.display().to_string(),
                "executed": false,
            }),
            human,
        ));
    }

    replace_symlink(ctx, &pinned, &pointer)?;
    let after = resolved(&pointer);

    let data = json!({
        "pointer": pointer.display().to_string(),
        "executed": true,
        "store_before": before.as_ref().map(|p| p.display().to_string()),
        "store_after": after.as_ref().map(|p| p.display().to_string()),
        "changed": before != after,
        "reset_at": crate::time::now_iso8601(),
    });
    let human = HumanRender::Message(format!(
        "{} now tracks the flake ({})",
        pointer.display(),
        display_or_dash(after.as_deref())
    ));
    Ok(CommandOutput::new(data, human))
}

/// Atomically replace `link` with a symlink to `target`.
///
/// Written to a sibling temp name and `rename`d over, so the pointer is never
/// momentarily absent — an agent reading `~/.agents/skills` concurrently sees
/// either the old tree or the new one, never a gap.
fn replace_symlink(ctx: &Context, target: &Path, link: &Path) -> Result<(), AppError> {
    let staging = link.with_file_name(".construct-pointer.tmp");
    let _ = std::fs::remove_file(&staging);
    std::os::unix::fs::symlink(target, &staging).map_err(|e| symlink_error(ctx, link, &e))?;
    std::fs::rename(&staging, link).map_err(|e| symlink_error(ctx, link, &e))
}

fn symlink_error(ctx: &Context, link: &Path, err: &std::io::Error) -> AppError {
    AppError::general(
        ctx,
        ErrorCode::InternalError,
        format!("could not update pointer '{}': {err}", link.display()),
        "check that the state directory is writable",
    )
}

// ── build (the `sync --build` half) ─────────────────────────────────────────

/// Build the consuming flake's `skills` output and re-point `current` at it.
///
/// Two deliberate choices, both load-bearing:
///
/// 1. **The build target is the CONSUMING flake, not `github:…/Construct`.**
///    Standalone, Construct resolves its own locked nixpkgs, which the consumer
///    almost certainly has not realised — a tarball fetch and a full nixpkgs
///    instantiation ahead of what is otherwise a few-megabyte copy. Through the
///    consumer, nixpkgs is already in the store and the build is sub-second.
///
/// 2. **It builds the LOCKED revision**, because the consumer's `flake.lock` is
///    what selects it. That keeps `flake.lock` the source of truth: the pointer
///    is a fast path TO the lock, never a way around it. Anything else silently
///    desynchronises the live tree from lock-derived copies elsewhere.
pub(crate) fn build_and_point(
    ctx: &Context,
    flake_dir: &Path,
    pointer: &Path,
) -> Result<serde_json::Value, AppError> {
    let parent = pointer.parent().unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err(not_pointer_managed(ctx, pointer));
    }

    let before = resolved(pointer);
    let target = format!("{}#{SKILLS_OUTPUT}", flake_dir.display());

    let result = Proc::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "build",
            &target,
            "--out-link",
            &pointer.display().to_string(),
        ])
        .current_dir(flake_dir)
        .output();

    let output = match result {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::dependency_missing(
                ctx,
                "`nix` was not found on PATH",
                "nix --version   # install Nix, then re-run construct skill sync --build",
            ));
        }
        Err(e) => {
            return Err(AppError::general(
                ctx,
                ErrorCode::InternalError,
                format!("failed to launch nix: {e}"),
                format!("nix build {target} --out-link {}", pointer.display()),
            ));
        }
    };

    crate::output::diagnostic::emit_passthrough(ctx, &String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(AppError::general(
            ctx,
            ErrorCode::InternalError,
            format!("nix build {target} failed"),
            format!(
                "nix build {target} --out-link {}   # the consuming flake must expose a `skills` output",
                pointer.display()
            ),
        )
        .with_extension("nix_exit_code", json!(output.status.code())));
    }

    let after = resolved(pointer);
    Ok(json!({
        "pointer": pointer.display().to_string(),
        "store_before": before.as_ref().map(|p| p.display().to_string()),
        "store_after": after.as_ref().map(|p| p.display().to_string()),
        "changed": before != after,
    }))
}

/// Resolve the pointer for `skill sync --build`, honoring its `--pointer` flag.
pub(crate) fn pointer_for_sync(ctx: &Context, args: &SyncArgs) -> Result<PathBuf, AppError> {
    resolve_pointer(
        ctx,
        &PointerArgs {
            pointer: args.pointer.clone(),
        },
    )
}
