# AppImage — download-and-run application bundle

Band B, tier 7 — **after Flatpak, before Snap.** An AppImage is a single
self-contained executable that carries the app and its dependencies. There is
**no manager to install and nothing to detect with `command -v`**: you reach for
AppImage when upstream publishes an `*.AppImage` release asset, the host can run
it, and no higher-ranked manager (Guix → … → Flatpak) has the tool.

Mostly a GUI / portable-app format, like Flatpak and Snap. A single AppImage
*can* be used disposably (download to a temp dir, run, delete), but it ranks in
Band B because it is a packaged-application download reached only when the
ephemeral and earlier permanent managers come up empty.

## Syntax

```bash
# 1. Download the upstream release asset
curl -L -o /tmp/<tool>.AppImage <release-asset-url>

# 2. Make it executable
chmod +x /tmp/<tool>.AppImage

# 3. Run it
/tmp/<tool>.AppImage [<args>]
```

### No-FUSE fallback (extract instead of mount)

AppImages mount themselves through **FUSE** at runtime. On hosts without FUSE
(many containers, some minimal systems) the direct run fails with a
`dlopen(): error loading libfuse.so.2` / `fuse: failed to … /dev/fuse` message.
Extract and run the payload directly instead:

```bash
/tmp/<tool>.AppImage --appimage-extract        # unpacks ./squashfs-root/
./squashfs-root/AppRun [<args>]
```

(Or run the upstream binary found under `squashfs-root/usr/bin/`.) The
`--appimage-extract` route needs no FUSE and no root.

### Key notes

- **No sudo, no system files.** Everything lives in the file you downloaded (and
  `./squashfs-root/` if you extracted). Nothing touches `/usr`.
- **Verify the source.** Download only from the project's official releases
  (GitHub Releases, the project site). Prefer an asset with a published checksum
  or signature and verify it when one is offered.
- **Integration is optional.** Tools like `appimaged` / AppImageLauncher can
  register an AppImage with the desktop, but that creates durable state — skip it
  for a one-off run.

## Lookup

- Upstream project releases (GitHub Releases / the project's download page) are
  the authoritative source for the correct `*.AppImage` asset URL.
- Catalogue: <https://appimage.github.io/> (AppImageHub) — searchable index of
  apps that ship an AppImage.

Confirm an `*.AppImage` asset actually exists for the tool before choosing this
tier; if none does, fall through to Snap. See [lookup.md](lookup.md).

## Cleanup

```bash
rm -f /tmp/<tool>.AppImage          # the bundle itself
rm -rf ./squashfs-root              # only if you used --appimage-extract
```

Because the whole app is one file, removing it removes the install completely —
no leftover system state. Tell the user where the file landed so they can delete
it (or keep it) at will.
