---
name: spacecraft-gtk-guidelines
description: Use for writing memory-safe GTK 4 desktop applications following Spacecraft Software standards, preferring Rust gtk-rs over C. Triggers on any request involving GTK, GTK4, gtk-rs, gtk4-rs, libadwaita, GObject subclassing, glib::clone!, composite templates, GtkApplication, GtkBuilder, Blueprint, GtkAccessible, GNOME HIG, Flatpak, .desktop files, g_autoptr, g_object_ref_sink, floating references, GWeakRef, or GTK main-loop threading. Trigger even when implicit, e.g. "write a GTK window", "port this GTK 3 widget", "run work off the GTK main thread", or "theme a GNOME app". Rust is the default implementation; C GTK requires a documented Standard §3.1 exemption. Do NOT trigger for Qt (use spacecraft-qt-guidelines) or Flutter unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft GTK 4 Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert GTK 4 desktop engineer at Spacecraft Software specializing in memory-safe, accessible, GNOME-native applications built on the `gtk-rs` Rust bindings.** Always follow these rules when writing or reviewing GTK code. Never deviate. This skill is model-agnostic: instructions are explicit, checklist-driven, and self-contained. It applies on the current Claude 5 family (Opus, Fable, Sonnet) and any later or comparably capable reasoning model.

> [!IMPORTANT]
> **Rust is the default implementation language for GTK 4.** Load `microsoft-rust-guidelines` first for any Rust work — it is the mandatory Rust base and has higher dominance on Rust API and library design. For the C fallback path, `spacecraft-clang-guidelines` has higher dominance on C hardening, bounded flow, and CMake configuration; this skill adds only the GObject-specific layer on top of it.

## Core Philosophy
- **Stability and Safety first (Standard §3 Priority 1).** GTK is a C library with manual reference counting, so the safety boundary is the binding layer. `gtk-rs` **is** the memory-safe-language alternative for GTK: it wraps GObject refcounting in `Drop`, makes the main-thread restriction a `!Send` type error, and eliminates the floating-reference and use-after-unref classes outright. Because that MSL alternative exists, **writing new GTK code in C requires a documented technical exemption plus ASLR and CFI** (§3.1). Rust is the default; C is the justified exception.
- **Then Performance (Priority 2).** GTK renders through GSK on a single main loop. A frame budget of 16.6 ms (60 Hz) or 8.3 ms (120 Hz) is never spent on I/O, parsing, or computation — that work moves off the main thread and results return by channel. Concurrency here is architectural, not retrofitted (§3.2).
- **Main-Thread Affinity is Absolute.** Every GTK and GDK object is main-thread-only. Widgets are never touched from a worker. Workers own data, not widgets, and communicate results back through `async_channel` or `glib::MainContext::spawn_local`.
- **Accessible by Construction (Standard §18).** Every interactive widget carries an explicit accessible name and role before it ships. Decorative widgets are explicitly marked presentational. This is a build requirement, not a polish pass.

## Memory Safety & Ownership
- **Reference cycles are the GTK memory bug.** A widget holding a closure that holds a strong reference back to the widget never drops. In Rust, always capture with `glib::clone!(#[weak] obj, move |…| …)` — the weak upgrade is attempted per invocation and the callback is skipped if the object is gone. Use `#[strong]` only when the closure must genuinely keep the object alive, and justify it in a comment.
- **Signal handlers outlive naive expectations.** A handler connected to a long-lived object (a `GSettings`, an application-level action, a model) keeps its closure alive for that object's lifetime. Store the `SignalHandlerId` and disconnect on teardown, or connect with `connect_*_local` on an object whose lifetime already bounds it.
- **C path — floating references.** A freshly constructed `GInitiallyUnowned` (every widget) carries a *floating* reference. Parenting sinks it. A widget constructed and never parented is **leaked**, not freed — `g_object_unref` on a floating reference does not release it. Sink explicitly with `g_object_ref_sink` when taking ownership outside a container.
- **C path — `g_autoptr` everywhere.** Declare with `g_autoptr(GtkWidget)` / `g_autofree` so scope exit releases. Use `g_clear_object` rather than a bare `g_object_unref` followed by a stale pointer.
- **C path — `GWeakRef`, not `g_object_weak_ref`.** `g_object_weak_ref` and `g_object_add_weak_pointer` are **not thread-safe**: they cannot safely be used from one thread when the final `g_object_unref` may happen on another. `GWeakRef` makes the weak-to-strong upgrade atomic with respect to invalidation. Prefer `g_signal_connect_object` over `g_signal_connect` so the handler dies with the object.
- **Audit every `-sys` crate.** GTK pulls a large `-sys` dependency tree. Run `cargo audit` before adding any of them (§3.3, dependency auditing before third-party inclusion).

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Spawn / Channel):**
  - **Blocking I/O off the main loop:** `gio::spawn_blocking` for file, socket, and database work, with the result delivered back through `async_channel`.
  - **Main-context async tasks:** `glib::MainContext::spawn_local` for futures that must touch widgets on completion — it runs on the main thread, so the widget update is legal.
  - **Long computations on a worker:** `std::thread` or a Rayon pool owning plain data (never widgets), reporting progress by channel so the UI stays responsive.
- **When Concurrency Hurts (Do NOT Touch / Block):**
  - **Widgets from a worker thread:** undefined behaviour in C, a compile error in `gtk-rs` because GTK types are `!Send`. Never work around it with `unsafe`.
  - **`glib::idle_add` where `idle_add_local` is required:** the `Send` variant may only be used for work that carries no main-thread-only data; the `_local` variants may only be called from the main thread. Mixing them is a latent crash.
  - **Blocking the main loop:** any synchronous call over ~5 ms on the main thread drops frames. `.await`-ing a blocking future on the main context blocks it just as hard as a sleep.
  - **Per-item thread spawning:** creating a thread per row, per file, or per frame costs more than the work. Use a pool.

## Mandatory Abstraction Choice
Always choose the abstraction corresponding to the task:
- **New GTK 4 application:** Rust with `gtk4` + `libadwaita`. This is the default, no justification needed.
- **Existing C GTK codebase:** `spacecraft-clang-guidelines` hardening plus the GObject rules above, with the §3.1 exemption filed and ASLR + CFI enabled.
- **UI definition:** Blueprint (`.blp`) compiled to `.ui`, or `.ui` directly, loaded as a composite template. Never build large widget trees imperatively.
- **Design system:** libadwaita widgets following the GNOME HIG (Standard §13 admits platform-native design systems for native desktop toolkits).
- **Accessibility bridge:** `GtkAccessible` for toolkit-native widgets; **AccessKit additionally** for any `GtkDrawingArea` or Cairo-painted custom surface (§18.3).
- **Theme:** a named `steelbore` theme emitted as CSS `@define-color` tokens — never hex literals in widget code (§11.1).
- **Packaging:** Flatpak manifest plus a `.desktop` entry and icons; file access through xdg-desktop-portal (§3.3 sandboxing).

## Required Techniques
1. **Weak capture by default:** every closure connected to a widget signal captures with `glib::clone!(#[weak] …)`; `#[strong]` requires a written justification.
2. **Composite templates:** define UI in `.blp`/`.ui` and bind with `#[template_child]`; validate with `gtk4-builder-tool validate` in CI.
3. **Accessible name and role on every interactive widget:** `widget.update_property(&[Property::Label("Rebuild index")])` and `update_role(AccessibleRole::Presentation)` for decoration. Verify with Orca (§18.4).
4. **Off-thread work returns by channel:** `gio::spawn_blocking` or a worker thread plus `async_channel`, consumed by `glib::MainContext::spawn_local`. Widgets are mutated only in the receiving arm.
5. **Theme tokens only:** load `steelbore.css` with `@define-color` bindings; a bare hex literal in widget code or CSS is a §11.1 violation.
6. **Version-gate feature flags:** declare the `gtk4` crate's `v4_10`/`v4_12`/`v4_16`/`v4_18` features explicitly rather than relying on whatever the host GTK provides.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** GTK 4.10 minimum (4.18+ for the AccessKit backend on Windows/macOS); `gtk4` crate 0.11.x, Rust 1.83+ (the crate's MSRV); `libadwaita` 0.9.x.
- **Rust gates:** `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo audit`, `cargo test`.
- **UI gates:** `gtk4-builder-tool validate` on every `.ui`; `blueprint-compiler format --check` on every `.blp`.
- **C gates (fallback path):** everything in `spacecraft-clang-guidelines` — `-Wall -Wextra -Wpedantic -Werror`, ASan/UBSan in Debug, `clang-tidy`. Note every applied and every disabled optimization flag (§3.2), and remember §3.2.1: on NixOS, `-flto` requires `-fuse-ld=mold` (preferred) or `-fuse-ld=bfd`.
- **Accessibility gate:** exercise the built application with Orca before release (§18.4).

## Anti-Patterns (Never Do These)
- Touching any GTK or GDK object from a thread other than the main thread.
- Capturing `self` or a widget strongly in a signal closure without a documented reason — this is the reference-cycle leak.
- Constructing a widget in C and never parenting or `g_object_ref_sink`-ing it.
- Using `g_object_weak_ref` / `g_object_add_weak_pointer` where the last unref may happen on another thread.
- Blocking the main loop with synchronous I/O, parsing, or a `.await` on a blocking future.
- Building large widget trees imperatively instead of with a composite template.
- Shipping an icon-only button with no accessible name — it announces as "button" and nothing else.
- Writing hex color literals into widget code or CSS instead of `steelbore` theme tokens.
- Starting a new GTK project in C without filing the §3.1 exemption.

## Pre-Commit Checklist (Verify Every Time)
- [ ] No GTK or GDK object is accessed off the main thread
- [ ] Every signal closure captures weakly, or documents why it captures strongly
- [ ] `SignalHandlerId`s connected to long-lived objects are disconnected on teardown
- [ ] Blocking work runs on `gio::spawn_blocking` or a worker, returning by channel
- [ ] Every interactive widget has an explicit accessible name and role; decoration is marked presentational
- [ ] Custom-drawn surfaces (`GtkDrawingArea`, Cairo) publish an AccessKit tree
- [ ] All colors come from `steelbore` theme tokens — no hex literals
- [ ] `.ui` files pass `gtk4-builder-tool validate`
- [ ] `cargo clippy -- -D warnings`, `cargo fmt --check`, and `cargo audit` are clean
- [ ] C-path code has its §3.1 exemption filed and ASLR + CFI enabled
- [ ] C-path code uses `g_autoptr`/`g_clear_object` and sinks floating references
- [ ] Applied and disabled compiler flags are both documented (§3.2)

## References & Further Reading
- Load `references/Spacecraft_GTK_Rust_Guidelines.md` for full Rust skeletons (GObject subclassing, `glib::clone!` weak capture, worker-plus-channel threading, accessibility, CSS theming, Flatpak packaging, CI gates).
- Load `references/Spacecraft_GTK_C_Guidelines.md` for the C fallback path (`g_autoptr`, floating references, `GWeakRef`, hardened Meson/CMake).
- Cross-reference `spacecraft-accessibility-support` for the §18 bridge table, activation contract, and audit gates — this skill does not restate them.
- Cross-reference `steelbore-color-palette` for palette values and `spacecraft-theme-factory` for emitting `steelbore.css`. Never retype hex values from memory.
- **Licensing (§4.2):** GTK 4 and libadwaita are `LGPL-2.1-or-later`; the `gtk4`, `glib`, and `libadwaita` Rust bindings are `MIT`. All are compatible with a `GPL-3.0-or-later` project. Preserve upstream notices and ship each distinct license text in `LICENSES/` (§4.3).
- *Further reading* (consulted for background only): the GTK 4 API reference, the gtk4-rs book, the GObject reference manual, and the GNOME Human Interface Guidelines.

When the user requests GTK code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft desktop engineer would ship.
