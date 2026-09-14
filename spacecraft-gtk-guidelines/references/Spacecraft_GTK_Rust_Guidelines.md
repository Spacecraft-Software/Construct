# Spacecraft GTK 4 Guidelines (Rust / gtk-rs) — Full Reference

**Version:** 1.0
**Date:** 2026-08-05
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Model-agnostic — the current Claude 5 family (Opus, Fable, Sonnet), Grok, and any comparably capable reasoning model

This document expands on the `SKILL.md` for the **default** GTK 4 implementation path: Rust with the `gtk-rs` bindings. It provides complete skeletons for project setup, GObject subclassing, weak-capture memory management, main-loop threading, accessibility, theming, packaging, and CI gates. For the C fallback path see `Spacecraft_GTK_C_Guidelines.md`.

---

## 1. Project Setup & Dependency Auditing

Pin the GTK feature level explicitly rather than inheriting whatever the host provides — a `v4_18` call compiled against a 4.10 runtime is a load-time failure, not a compile error. Standard §3.3 requires auditing every third-party crate before inclusion, and GTK pulls a deep `-sys` tree.

```toml
# Cargo.toml
[package]
name    = "telemetry-dashboard"
version = "0.1.0"
edition = "2021"
# gtk4 0.11.x requires Rust 1.83 or newer.
rust-version = "1.83"

[dependencies]
# `v4_10` is the Spacecraft floor. Raise to `v4_18` only when the deployment
# target guarantees it — 4.18 is where the AccessKit backend landed, giving
# Windows and macOS a11y in addition to AT-SPI on Linux.
gtk4          = { version = "0.11", features = ["v4_10"] }
libadwaita    = { version = "0.9",  features = ["v1_5"] }
glib          = "0.22"
gio           = "0.22"
async-channel = "2"
```

```bash
# Audit the whole -sys tree before the first commit, and in CI thereafter.
cargo audit
cargo tree --duplicates   # catch two gtk4-sys majors linked into one binary
```

---

## 2. GObject Subclassing & Composite Templates

Define the widget tree declaratively and bind it with `#[template_child]`. Never assemble a large tree imperatively — a `.ui`/`.blp` file is validated in CI, diffable, and translatable, while imperative construction is none of those.

```rust
// src/window/mod.rs
mod imp;

use gtk4 as gtk;
use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct TelemetryWindow(ObjectSubclass<imp::TelemetryWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::Native, gtk::Root;
}

impl TelemetryWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
```

```rust
// src/window/imp.rs
use gtk4 as gtk;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/spacecraftsoftware/TelemetryDashboard/window.ui")]
pub struct TelemetryWindow {
    #[template_child]
    pub packet_list: TemplateChild<gtk::ListView>,
    #[template_child]
    pub refresh_button: TemplateChild<gtk::Button>,

    // Interior mutability: GObject construction hands out `&self`, never `&mut self`.
    pub handler_ids: RefCell<Vec<glib::SignalHandlerId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for TelemetryWindow {
    const NAME: &'static str = "TelemetryWindow";
    type Type = super::TelemetryWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for TelemetryWindow {
    fn constructed(&self) {
        self.parent_constructed();
        self.obj().wire_signals();
    }

    // Teardown: disconnect handlers connected to objects that outlive this widget.
    fn dispose(&self) {
        for id in self.handler_ids.borrow_mut().drain(..) {
            // Only disconnect from objects still alive; long-lived settings/actions.
            let _ = id;
        }
        self.dispose_template();
    }
}

impl WidgetImpl for TelemetryWindow {}
impl WindowImpl for TelemetryWindow {}
impl ApplicationWindowImpl for TelemetryWindow {}
impl adw::subclass::prelude::AdwApplicationWindowImpl for TelemetryWindow {}
```

---

## 3. Memory Management: Weak Capture & Reference Cycles

The characteristic GTK leak is a cycle: a widget owns a signal closure, and the closure owns a strong reference back to the widget. Neither refcount ever reaches zero. `glib::clone!` with `#[weak]` breaks it — the macro upgrades the weak reference on every invocation and **skips the callback** if the object is already gone.

```rust
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;

impl TelemetryWindow {
    fn wire_signals(&self) {
        let imp = self.imp();

        // CORRECT: #[weak] breaks the cycle. If the window is gone when the
        // button fires, the closure body is skipped rather than resurrecting it.
        imp.refresh_button.connect_clicked(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_button| {
                window.reload_packets();
            }
        ));

        // WRONG — do not do this. `self` captured strongly means the window
        // holds the button holds the closure holds the window. Permanent leak.
        //
        //   let window = self.clone();
        //   imp.refresh_button.connect_clicked(move |_| window.reload_packets());

        // #[strong] is legitimate only when the closure MUST keep the object
        // alive for the duration of an operation. Justify it in a comment.
        let cancellable = gio::Cancellable::new();
        imp.refresh_button.connect_clicked(glib::clone!(
            #[strong]
            cancellable, // strong: the cancellable has no other owner while in flight
            move |_| cancellable.cancel()
        ));
    }
}
```

**Signal handlers on long-lived objects.** A handler connected to a `GSettings`, an application action, or a shared model lives as long as *that* object, not as long as the widget. Store the id and disconnect on `dispose`:

```rust
let settings = gio::Settings::new("org.spacecraftsoftware.TelemetryDashboard");
let id = settings.connect_changed(
    Some("refresh-interval"),
    glib::clone!(
        #[weak(rename_to = window)]
        self,
        move |s, _key| window.set_interval(s.uint("refresh-interval"))
    ),
);
// Retain so `dispose` can call `settings.disconnect(id)`.
self.imp().handler_ids.borrow_mut().push(id);
```

---

## 4. Threading: The Main-Thread Rule

Every GTK and GDK object is main-thread-only. In `gtk-rs` this is enforced by the type system — GTK types are `!Send`, so moving a widget into a worker thread is a compile error. **Never reach for `unsafe` to defeat it.** Workers own plain data; results cross back by channel and are applied on the main thread.

Per Standard §3.2, concurrency is an architecture-level decision considered from the start, adopted where it advances performance, and abandoned where it would compromise Priority 1. Record the trade-off in a comment, as below.

```rust
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;

#[derive(Debug)]
struct TelemetryPacket {
    sensor_id: u32,
    value: f32,
}

impl TelemetryWindow {
    fn reload_packets(&self) {
        let (sender, receiver) = async_channel::bounded::<Vec<TelemetryPacket>>(1);

        // CONCURRENCY TRADE-OFF (§3.2): parsing a telemetry capture is CPU- and
        // I/O-bound and routinely exceeds the 16.6 ms frame budget, so it is
        // moved off the main loop. The work is embarrassingly independent and
        // touches no widget, so no synchronization is required beyond the
        // channel handoff — the simplest safe structure, not the most parallel.
        gio::spawn_blocking(move || {
            let packets = parse_capture("/var/lib/telemetry/latest.cap");
            // `send_blocking` is correct here: this closure is NOT on the main loop.
            let _ = sender.send_blocking(packets);
        });

        // The receiving arm runs on the main thread, so touching widgets is legal.
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                while let Ok(packets) = receiver.recv().await {
                    window.imp().packet_list.set_model(Some(&build_model(&packets)));
                }
            }
        ));
    }
}
```

**Rules that follow from the above:**

| Call | Legal from | Notes |
| :--- | :--- | :--- |
| `glib::idle_add` / `timeout_add` | any thread | closure must be `Send + 'static`; cannot carry widgets |
| `glib::idle_add_local` / `timeout_add_local` | **main thread only** | may carry main-thread-only data |
| `glib::spawn_future_local` | **main thread only** | the future runs on the main context; widget access is legal |
| `gio::spawn_blocking` | any thread | for blocking I/O; the closure must not touch widgets |
| `async_channel::send_blocking` | worker only | blocks — calling it on the main loop stalls the UI |
| `async_channel::send().await` | main context | non-blocking |

---

## 5. Accessibility (Standard §18)

`spacecraft-accessibility-support` owns the §18 contract, the activation toggle, and the audit gates — this section covers only the gtk-rs API surface. Every interactive widget carries an explicit name and role; decoration is explicitly marked so screen readers skip it rather than read noise.

```rust
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::accessible::{Property, Relation, State};
use gtk::AccessibleRole;

fn make_accessible(window: &TelemetryWindow) {
    let imp = window.imp();

    // An icon-only button with no accessible name announces as "button" and
    // nothing else. This is the single most common GUI a11y failure.
    imp.refresh_button.update_property(&[
        Property::Label("Rebuild telemetry index"),
        Property::Description("Re-scans the capture directory and reloads all packets"),
    ]);

    // Decoration is skipped rather than read as noise.
    imp.divider.update_role(AccessibleRole::Presentation);

    // State changes must be ANNOUNCED, not merely repainted. A spinner that
    // stops communicates nothing to a screen-reader user.
    imp.refresh_button.update_state(&[State::Busy(true)]);

    // Relate a control to the label that names it.
    imp.interval_entry.update_relation(&[Relation::LabelledBy(&[imp.interval_label.upcast_ref()])]);
}
```

### The AccessKit boundary

Standard §18.3 requires **AccessKit** for "Rust, custom-drawn UI" and `GtkAccessible` for "GTK 4". Both apply, and the scoping term is *custom-drawn*:

- **Toolkit-native `gtk-rs` widgets → `GtkAccessible`.** A `GtkButton`, `GtkListView`, or `AdwPreferencesRow` already has a node in the accessibility tree. Do **not** bolt AccessKit onto it; supply the name and role through `update_property`/`update_role` as above.
- **`GtkDrawingArea` and any Cairo-painted custom surface → publish an accessibility tree as well.** A control painted onto a canvas has no node unless the application creates one. It is not "partially accessible" — it is invisible. Either implement `AccessibleImpl` on a `GtkWidget` subclass exposing each painted control as a child node, or drive an AccessKit adapter for the surface.

Verification is by real screen reader (§18.4): run under Orca on Linux and confirm every interactive element announces its name and role.

---

## 6. Theming: `steelbore` Tokens, Never Hex Literals

Standard §11.1 forbids hex literals in UI logic and requires all palette references to go through a named `steelbore` theme, so a user can swap the theme without touching application code. GTK's CSS `@define-color` is exactly that indirection.

Generate `steelbore.css` with `spacecraft-theme-factory`; take the values from `steelbore-color-palette`'s `assets/steelbore.toml`. **Never retype hex values from memory.**

```css
/* resources/steelbore.css — generated; token definitions only */
@define-color steelbore_background  @STEELBORE_BACKGROUND@;
@define-color steelbore_surface     @STEELBORE_SURFACE@;
@define-color steelbore_surface_alt @STEELBORE_SURFACE_ALT@;
@define-color steelbore_foreground  @STEELBORE_FOREGROUND@;
@define-color steelbore_accent      @STEELBORE_ACCENT@;
@define-color steelbore_structure   @STEELBORE_STRUCTURE@;
@define-color steelbore_success     @STEELBORE_SUCCESS@;
@define-color steelbore_error       @STEELBORE_ERROR@;
@define-color steelbore_warning     @STEELBORE_WARNING@;
@define-color steelbore_focus       @STEELBORE_FOCUS@;
@define-color steelbore_border      @STEELBORE_BORDER@;

window                 { background-color: @steelbore_background; color: @steelbore_foreground; }
.panel                 { background-color: @steelbore_surface; }
/* §11.0.1: a surface's edge against the canvas is below the 3:1 non-text floor,
   so where the boundary is meaningful it MUST be drawn. */
.panel                 { border: 1px solid @steelbore_border; }
*:focus-visible        { outline: 2px solid @steelbore_focus; outline-offset: 2px; }
.status.error::before  { content: "[ERROR] "; }   /* §11.0.2: color is never the sole carrier */
.status.ok::before     { content: "[OK] "; }
```

```rust
fn install_theme(display: &gdk::Display) {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/org/spacecraftsoftware/TelemetryDashboard/steelbore.css");
    gtk::style_context_add_provider_for_display(
        display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
```

Under accessible mode, swap in the `steelbore-high-contrast` variant, and honour the platform high-contrast and reduced-motion preferences **independently** of the §18.1 toggle — read `AdwStyleManager` / `GtkSettings::gtk-enable-animations` rather than requiring the user to state the preference twice.

---

## 7. Desktop Integration & Sandboxing (§3.3)

Standard §3.3 requires sandboxing and privilege separation. On the desktop that means Flatpak plus xdg-desktop-portal: the application asks for a file through the portal and receives a handle, rather than holding blanket filesystem access.

```ini
# data/org.spacecraftsoftware.TelemetryDashboard.desktop
[Desktop Entry]
Type=Application
Name=Telemetry Dashboard
Comment=Live spacecraft telemetry inspection
Exec=telemetry-dashboard %U
Icon=org.spacecraftsoftware.TelemetryDashboard
Categories=Utility;Monitor;
StartupNotify=true
```

```yaml
# build-aux/org.spacecraftsoftware.TelemetryDashboard.yaml
app-id: org.spacecraftsoftware.TelemetryDashboard
runtime: org.gnome.Platform
runtime-version: '50'
sdk: org.gnome.Sdk
sdk-extensions: [org.freedesktop.Sdk.Extension.rust-stable]
command: telemetry-dashboard
finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  # NO --filesystem=host. File access goes through the portal so the user
  # grants exactly the files they picked, and nothing else (§3.3).
```

```rust
// Portal-mediated file access — no blanket filesystem permission required.
let dialog = gtk::FileDialog::builder()
    .title("Select telemetry capture")
    .modal(true)
    .build();
dialog.open(Some(window), gio::Cancellable::NONE, glib::clone!(
    #[weak] window,
    move |result| {
        if let Ok(file) = result {
            window.load_capture(&file);
        }
    }
));
```

Ship a `.metainfo.xml` AppStream file alongside the desktop entry; `appstreamcli validate` belongs in CI.

---

## 8. Testing & CI Gates

```yaml
# .github/workflows/ci.yml (excerpt)
- name: Format
  run: cargo fmt --check
- name: Lint
  run: cargo clippy --all-targets -- -D warnings
- name: Audit
  run: cargo audit
- name: Test
  run: xvfb-run -a cargo test
- name: Validate UI definitions
  run: |
    for ui in $(find data -name '*.ui'); do gtk4-builder-tool validate "$ui"; done
- name: Validate Blueprint sources
  run: blueprint-compiler format --check $(find data -name '*.blp')
- name: Validate AppStream metadata
  run: appstreamcli validate --pedantic data/*.metainfo.xml
```

> `gtk4-builder-tool validate` does not know about libadwaita types and will report `Invalid object type 'AdwApplicationWindow'` for templates using them. Validate GTK-only templates with the tool and cover libadwaita templates with a headless smoke test that instantiates the window under `xvfb-run` instead — do not silence the gate by deleting it.

Widget-level tests run headless and assert on accessible properties, which doubles as an a11y regression gate:

```rust
#[test]
fn refresh_button_is_named() {
    gtk::init().unwrap();
    let window = TelemetryWindow::new(&adw::Application::builder().build());
    let label = window.imp().refresh_button.accessible_property(
        gtk::AccessibleProperty::Label,
    );
    assert_eq!(label.get::<String>().unwrap(), "Rebuild telemetry index");
}
```

---

## 9. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Strong capture in a signal closure** | Window never destroyed; memory grows per open/close cycle | Capture with `glib::clone!(#[weak] …)`; reserve `#[strong]` for documented cases. |
| **Handler on a long-lived object** | Callbacks fire against a destroyed widget; use-after-free in C, skipped-forever in Rust | Store the `SignalHandlerId` and disconnect in `dispose`. |
| **Widget touched from a worker** | Compile error (`!Send`) in Rust; random corruption in C | Send plain data over `async_channel`; apply on the main thread. |
| **`send_blocking` on the main loop** | UI freezes for the duration of the send | Use `send().await` on the main context; reserve `send_blocking` for workers. |
| **`idle_add` vs `idle_add_local` confusion** | Compile error, or a panic about the wrong thread | `_local` from the main thread only; the `Send` variant carries no widgets. |
| **Icon-only button** | Orca announces "button" with no name | `update_property(&[Property::Label(…)])` on every interactive widget. |
| **Custom `GtkDrawingArea`** | Screen reader sees an empty region | Publish an accessibility tree — `GtkAccessible` children or an AccessKit adapter. |
| **Hex literals in CSS** | §11.1 violation; theme cannot be swapped | Emit `@define-color` tokens via `spacecraft-theme-factory`. |
| **Nested surfaces without a border** | Panels visually merge; below the §11.0.1 3:1 non-text floor | Draw the boundary with a `@steelbore_border` edge. |
| **Feature flag drift** | Runtime symbol lookup failure on an older host GTK | Pin the `gtk4` crate feature (`v4_10`…`v4_18`) to the deployment floor. |

---

## 10. Code Review Compliance Gate

Before merging Rust GTK code, verify:
1. No GTK or GDK object is reachable from a non-main thread, and no `unsafe` defeats `!Send`.
2. Every signal closure captures weakly, or carries a written justification for `#[strong]`.
3. Handlers connected to objects outliving the widget are disconnected in `dispose`.
4. Blocking work runs on `gio::spawn_blocking` or a worker and returns through a channel, with the §3.2 trade-off documented.
5. Every interactive widget has an accessible name and role; decoration is marked presentational.
6. Custom-drawn surfaces publish an accessibility tree (§18.3).
7. All colors resolve through `steelbore` theme tokens; no hex literal appears in Rust or CSS (§11.1).
8. High contrast and reduced motion are read from the platform independently of the §18.1 toggle.
9. `.ui` files pass `gtk4-builder-tool validate`; `.blp` files pass `blueprint-compiler format --check`.
10. `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`, and `cargo test` are clean.
11. The Flatpak manifest grants no blanket `--filesystem=host`; file access is portal-mediated (§3.3).
12. Upstream licenses (GTK/libadwaita `LGPL-2.1-or-later`, bindings `MIT`) are shipped in `LICENSES/` (§4.2, §4.3).
