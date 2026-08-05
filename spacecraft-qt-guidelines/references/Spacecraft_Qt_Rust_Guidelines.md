# Spacecraft Qt 6 Guidelines (Rust / CXX-Qt) — Full Reference

**Version:** 1.0
**Date:** 2026-08-05
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document covers the **preferred** Qt 6 implementation path: application logic in Rust, exposed to Qt and QML through CXX-Qt. It mirrors the GTK skill's Rust-first posture. For the C++ surface that CXX-Qt does not cover, see `Spacecraft_Qt_Cpp_Guidelines.md`; for Qt Quick specifics see `Spacecraft_Qt_QML_Guidelines.md`.

---

## 1. Coverage Boundary — Read This First

CXX-Qt (KDAB, `MIT OR Apache-2.0`) is at **0.9.x** and has not reached 1.0. It is production-usable for the layer this skill puts in Rust, and honestly incomplete elsewhere. Knowing the line prevents both under-use and a wasted afternoon.

**Covered — put this in Rust:**

- `QObject` subclasses with properties, signals, slots, and `Q_INVOKABLE` methods
- `QAbstractItemModel` / `QAbstractListModel` implementations backing QML views
- Business logic, parsing, networking, persistence, state machines — all plain safe Rust behind the bridge
- Core value types via `cxx-qt-lib`: `QString`, `QUrl`, `QVariant`, `QVector`/`QList`, `QDateTime`, `QColor`
- Registration of Rust types into QML modules through `cxx-qt-build`

**Not covered — this stays C++:**

- Custom `QWidget` painting (`paintEvent`, `QPainter` surfaces)
- `QAccessibleInterface` subclasses for custom-painted controls
- Most non-core Qt modules (Charts, Multimedia, WebEngine, Print Support, and similar)
- Deep `QWidget` subclassing and custom layout managers
- Platform-integration plugins

**Consequence.** Prefer **Qt Quick over Widgets** in a new Spacecraft Qt project: the QML view layer plus Rust models keeps essentially all logic on the safe side of the bridge, while a Widgets application drags custom painting and accessibility interfaces into C++. Where C++ is unavoidable, it is a §3.1 exemption with ASLR and CFI, and it obeys `spacecraft-cpp-guidelines`.

---

## 2. Project Setup

```toml
# Cargo.toml
[package]
name    = "telemetry-dashboard"
version = "0.1.0"
edition = "2021"
rust-version = "1.83"

[dependencies]
cxx          = "1"
cxx-qt       = "0.9"
cxx-qt-lib   = { version = "0.9", features = ["qt_full"] }

[build-dependencies]
cxx-qt-build = { version = "0.9", features = ["link_qt_object_files"] }
```

```rust
// build.rs
fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .qt_module("Quick")
        .qml_module(cxx_qt_build::QmlModule {
            uri: "org.spacecraftsoftware.telemetry",
            rust_files: &["src/telemetry_model.rs"],
            qml_files: &["qml/Main.qml"],
            ..Default::default()
        })
        .build();
}
```

```bash
# §3.3 — audit before adding, and in CI thereafter.
cargo audit
```

---

## 3. The Bridge: Properties, Signals, Invokables

Keep the `#[cxx_qt::bridge]` **thin**. It is a declaration of what crosses into Qt's object model, not a place for logic. Everything substantial lives in ordinary safe Rust that the bridge calls.

```rust
// src/telemetry_model.rs
#[cxx_qt::bridge(cxx_file_stem = "telemetry_model")]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, packet_count)]
        #[qproperty(QString, status_text)]
        #[qproperty(bool, busy)]
        type TelemetryModel = super::TelemetryModelRust;

        /// Emitted when a capture finishes loading. Consumed by QML.
        #[qsignal]
        fn capture_loaded(self: Pin<&mut TelemetryModel>, packets: i32);

        /// Callable from QML.
        #[qinvokable]
        fn reload(self: Pin<&mut TelemetryModel>, path: &QString);
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;

/// Plain Rust state. No Qt types leak into the domain logic below this line.
#[derive(Default)]
pub struct TelemetryModelRust {
    packet_count: i32,
    status_text: QString,
    busy: bool,
    packets: Vec<crate::telemetry::Packet>,
}

impl qobject::TelemetryModel {
    pub fn reload(mut self: Pin<&mut Self>, path: &QString) {
        self.as_mut().set_busy(true);

        // Ordinary safe Rust — testable without Qt in the loop.
        match crate::telemetry::parse_capture(&path.to_string()) {
            Ok(packets) => {
                let count = packets.len() as i32;
                self.as_mut().rust_mut().packets = packets;
                self.as_mut().set_packet_count(count);
                self.as_mut().set_status_text(QString::from("[OK] Capture loaded"));
                self.as_mut().capture_loaded(count);
            }
            Err(err) => {
                // §3.1: failures are surfaced, never silently swallowed.
                // §11.0.2: status carries a text tag, not color alone.
                self.as_mut()
                    .set_status_text(QString::from(&format!("[ERROR] {err}")));
            }
        }

        self.as_mut().set_busy(false);
    }
}
```

**Rules for the bridge:**

- One `#[qobject]` per bridge module keeps generated headers readable.
- `#[qproperty]` generates the getter, setter, and `…Changed` signal; do not hand-write them.
- Never `unwrap`/`expect` on fallible input in a `#[qinvokable]` — a panic across the FFI boundary is an abort, and §3.1 forbids it on untrusted input.
- Domain types stay Rust types; convert to `QString`/`QVariant` only at the boundary.

---

## 4. Threading from the Rust Side

Qt's GUI thread rule applies identically through CXX-Qt: a `Pin<&mut QObject>` may only be touched on the thread that owns the object. CXX-Qt makes the handoff explicit with `CxxQtThread`, which queues a closure back onto the object's thread — the Rust equivalent of `QMetaObject::invokeMethod` with a queued connection.

```rust
impl qobject::TelemetryModel {
    pub fn reload_async(mut self: Pin<&mut Self>, path: &QString) {
        let path = path.to_string();
        // A handle that can be moved to another thread and used to queue work
        // back onto this object's thread.
        let qt_thread = self.qt_thread();

        self.as_mut().set_busy(true);

        // CONCURRENCY TRADE-OFF (§3.2): capture parsing is I/O- and CPU-bound
        // and blocks well past a frame budget, so it moves off the GUI thread.
        // The work is independent and touches no QObject, so a plain thread
        // plus a queued handoff is the simplest safe structure — no shared
        // mutable state, therefore no locking to get wrong.
        std::thread::spawn(move || {
            let result = crate::telemetry::parse_capture(&path);

            // Runs on the GUI thread. Mutating the QObject here is legal.
            qt_thread
                .queue(move |mut model| match result {
                    Ok(packets) => {
                        let count = packets.len() as i32;
                        model.as_mut().rust_mut().packets = packets;
                        model.as_mut().set_packet_count(count);
                        model.as_mut().set_busy(false);
                        model.as_mut().capture_loaded(count);
                    }
                    Err(err) => {
                        model.as_mut()
                            .set_status_text(QString::from(&format!("[ERROR] {err}")));
                        model.as_mut().set_busy(false);
                    }
                })
                .expect("queueing onto a live QObject thread");
        });
    }
}
```

**Never** hold a `Pin<&mut QObject>` across a thread boundary, and never wrap one in a type that asserts `Send` to make it compile.

---

## 5. Models for QML

A `QAbstractListModel` implemented in Rust is where CXX-Qt earns its place: the list data, its invariants, and its mutation logic are all memory-safe, and only the model protocol crosses the bridge.

```rust
#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = "QAbstractListModel"]
        type PacketListModel = super::PacketListModelRust;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &PacketListModel, parent: &QModelIndex) -> i32;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &PacketListModel, index: &QModelIndex, role: i32) -> QVariant;
    }
}

#[derive(Default)]
pub struct PacketListModelRust {
    packets: Vec<crate::telemetry::Packet>,
}

impl qobject::PacketListModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.packets.len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        // Bounds are checked in safe Rust; an out-of-range index yields an
        // invalid QVariant rather than reading past the end.
        let Some(packet) = self.packets.get(index.row() as usize) else {
            return QVariant::default();
        };
        match role {
            0 => QVariant::from(&QString::from(&packet.sensor_id.to_string())),
            1 => QVariant::from(&packet.value),
            _ => QVariant::default(),
        }
    }
}
```

Wrap every insertion and removal in `begin_insert_rows`/`end_insert_rows` (and the removal equivalents). Mutating the backing `Vec` without the model signals leaves the view reading stale row counts — a crash in the C++ view, not a Rust panic.

---

## 6. Testing

Domain logic is plain Rust and tests without Qt at all — which is the point of keeping the bridge thin.

```rust
#[cfg(test)]
mod tests {
    use crate::telemetry::{parse_capture, Packet};

    #[test]
    fn truncated_capture_is_an_error_not_a_panic() {
        let err = parse_capture("tests/data/truncated.cap").unwrap_err();
        assert!(matches!(err, crate::telemetry::Error::Truncated { .. }));
    }

    #[test]
    fn packet_values_are_range_checked() {
        assert!(Packet::new(1, f32::NAN).is_err());
    }
}
```

CI gates:

```yaml
- run: cargo fmt --check
- run: cargo clippy --all-targets -- -D warnings
- run: cargo audit
- run: cargo test
- run: cmake --build build --target all_qmllint
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Logic inside the bridge module** | Untestable without Qt; unreadable generated headers | Keep the bridge declarative; put logic in plain Rust it calls. |
| **`unwrap` in a `#[qinvokable]`** | Process abort on bad input — a panic cannot unwind into C++ | Return a `Result`, set an error property, surface it in the UI (§3.1). |
| **`Pin<&mut QObject>` moved to a worker** | Compile error, or corruption if forced with `unsafe` | Use `self.qt_thread()` and `queue` the mutation back. |
| **Model `Vec` mutated without row signals** | View shows stale rows, then reads out of range and crashes | Wrap mutations in `begin_insert_rows` / `end_insert_rows`. |
| **Reaching for a Widgets feature** | CXX-Qt has no binding for it | Check the coverage boundary in §1 — that layer is C++. |
| **Qt version mismatch** | Link errors in `cxx-qt-build` | Pin the Qt found by CMake to the 6.8 LTS floor; keep `cxx-qt-lib` features aligned. |
| **Blocking in a `#[qinvokable]`** | UI freezes while the invokable runs | Invokables run on the GUI thread — spawn and queue back. |

---

## 8. Code Review Compliance Gate

Before merging Rust/CXX-Qt code, verify:
1. The choice of Qt over `gtk-rs` is recorded, and every remaining C++ file carries the §3.1 exemption with ASLR + CFI.
2. The bridge module is declarative; domain logic lives in plain Rust and is unit-tested without Qt.
3. No `unwrap`/`expect` on fallible or untrusted input in any bridged function.
4. No `Pin<&mut QObject>` crosses a thread boundary; off-thread work returns via `qt_thread().queue(…)`.
5. The §3.2 concurrency trade-off is documented wherever work was moved off the GUI thread.
6. Model mutations are wrapped in the appropriate begin/end row signals.
7. Errors surface as observable state (a `[ERROR]`-tagged status property), never silently swallowed.
8. `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`, and `cargo test` are clean.
9. Any C++ in the project also passes the `Spacecraft_Qt_Cpp_Guidelines.md` gate and `spacecraft-cpp-guidelines`.
10. CXX-Qt's `MIT OR Apache-2.0` notice and Qt's `LGPL-3.0`/`GPL-3.0` texts ship in `LICENSES/` (§4.2, §4.3).
