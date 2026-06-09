# Cleanup & Fallback Guidance

## Cleanup summary

Band A leaves nothing to clean up — Guix/Nix stores are garbage-collected later
by `guix gc` / `nix-collect-garbage`, and the one-shot runners (`npx`/`uvx`)
only populate a throwaway cache. The Band B (permanent) managers do need
cleanup:

| Manager  | Remove a package                                          | Scope            |
|----------|-----------------------------------------------------------|------------------|
| Cargo    | `cargo uninstall <crate>`                                 | User-local       |
| Homebrew | `brew uninstall <formula>`                                | User-local       |
| Flatpak  | `flatpak uninstall --user <app-id>`                       | User-local       |
| AppImage | `rm <tool>.AppImage` (and `rm -rf squashfs-root` if extracted) | The file you downloaded |
| Snap     | `sudo snap remove <pkg>` (or `--purge` to wipe data)      | System-wide      |

**Whenever a Band B (permanent) manager was used, tell the user what was
installed and how to remove it.**

---

## When the package isn't in any manager in the chain

If lookups across Guix, Nix, the one-shot runners, Cargo, Homebrew, Flatpak,
AppImage, and Snap all come up empty:

1. **If it's a Node / Python / other-ecosystem package, re-check the one-shot
   runner tier first.** `npx`/`uvx` are Band A tier 3, not a fallback — a tool
   published to npm or PyPI should have been provisioned there (bootstrapping
   the runtime via `guix shell node -- npx <pkg>` if needed). Revisit
   [oneshot.md](oneshot.md) before concluding the package is unavailable.
2. **If it's a Rust crate not yet on crates.io**, try a Git source:
   ```bash
   cargo install --git https://github.com/<owner>/<repo>
   ```
3. **If upstream ships a release binary or `*.AppImage`** but it's in no
   manager, an AppImage (Band B tier 7) or a direct binary download is the
   remaining option — see [appimage.md](appimage.md).
4. **Report the gap to the user** with the searches you performed and the
   URLs you consulted — makes it easy to double-check or report the
   missing package upstream.

## Hard prohibitions

Do **not** fall back to:

- **System-distro managers:** `apt`, `dnf`, `yum`, `pacman`, `zypper`,
  `emerge`, `xbps`, etc. These require root, touch `/usr`, and leave
  durable, hard-to-reverse changes.
- **Global language package installs:** `pip install` outside a venv,
  `npm install -g`, `gem install` without a user prefix.

If a user explicitly requests one of the above anyway, explain the
skill's policy, document the command they would need to run, and let
them make the call.
