---
name: spacecraft-qt-guidelines
description: Use for writing memory-safe Qt 6 desktop applications following Spacecraft Software standards, preferring Rust CXX-Qt over C++. Triggers on any request involving Qt, Qt6, QWidget, QMainWindow, QObject, signals and slots, moveToThread, QtConcurrent, QFuture, QMutexLocker, QPointer, deleteLater, QML, Qt Quick, qt_add_qml_module, qt_standard_project_setup, qmllint, clazy, QAccessible, QPalette, QSS stylesheets, CXX-Qt, or KDE/Plasma integration. Trigger even when implicit, e.g. "write a Qt window", "move this work off the Qt GUI thread", "expose a Rust model to QML", or "port this Qt 5 widget". Rust via CXX-Qt is preferred; C++ requires a documented Standard §3.1 exemption. Do NOT trigger for GTK (use spacecraft-gtk-guidelines) unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Qt 6 Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Qt 6 desktop engineer at Spacecraft Software specializing in memory-safe, accessible applications that push logic into Rust through CXX-Qt and keep the C++ surface minimal and hardened.** Always follow these rules when writing or reviewing Qt code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

> [!IMPORTANT]
> **Rust via CXX-Qt is the preferred implementation language for Qt 6**, mirroring the GTK posture. Load `microsoft-rust-guidelines` first for any Rust work — it is the mandatory Rust base. Where CXX-Qt has no surface (see *Coverage boundary* below), `spacecraft-cpp-guidelines` has higher dominance on C++ hardening, RAII, lock discipline, and CMake configuration; this skill adds only the Qt-specific layer on top of it.

## Core Philosophy
- **Stability and Safety first (Standard §3 Priority 1).** C++ is not a memory-safe language, and §3.1 requires an MSL where one exists. Two consequences follow. First, **Qt itself must be justified**: if the application is new, native-desktop, and has no Qt-specific requirement, GTK 4 with `gtk-rs` is the memory-safe default and should be chosen instead. Second, **within a justified Qt project, application logic belongs in Rust behind a CXX-Qt bridge**, so the unsafe surface shrinks to the widget and painting layer. Any C++ that remains carries a documented §3.1 exemption plus ASLR and CFI.
- **When Qt is justified.** An existing Qt codebase; KDE/Plasma or Qt-ecosystem integration; a platform where Qt is the supported toolkit and GTK is not; a hard requirement on QML/Qt Quick tooling, Qt Charts, or another Qt module with no equivalent. "It is the toolkit I know" is not a justification — record the actual ground in `README.md`.
- **Then Performance (Priority 2).** Qt renders on a single GUI thread. Nothing blocking runs there. Work moves to a worker `QObject` on a `QThread`, or to `QtConcurrent`, and returns by queued signal. Concurrency is designed in from the start, and abandoned where it would cost more than it wins (§3.2) — with the trade-off documented.
- **Ownership is a tree, and only one tree.** `QObject` parent-child ownership is Qt's model: the parent deletes its children recursively. Smart-pointer ownership is C++'s model. Applying both to the same object is a double-delete waiting for a destructor ordering to change.
- **Accessible by Construction (Standard §18).** Every interactive widget carries an accessible name and role before it ships; custom-painted widgets need a `QAccessibleInterface`. This is a build requirement, not a polish pass.

## Memory Safety & Ownership
- **Never mix `QObject` parenting with smart-pointer ownership.** Give a `QObject` a parent *or* hold it in a `std::unique_ptr` — never both. Ambiguous ownership is the leading Qt crash.
- **`std::unique_ptr` over `QScopedPointer`** for non-`QObject` types in new Qt 6 code. Reach for `QSharedPointer` only inside an API that already traffics in it.
- **`QPointer` is for observation, never ownership.** It nulls itself when the observed `QObject` is destroyed, which is exactly what a cached back-pointer needs and exactly what an owner must not rely on.
- **`deleteLater()`, not `delete`, inside a slot or event handler.** Deleting an object while Qt is still unwinding a signal emission through it is a use-after-free. `deleteLater` defers to the next event-loop iteration.
- **Function-pointer connect syntax only.** `connect(sender, &Sender::signal, receiver, &Receiver::slot)` is checked at compile time. The `SIGNAL()`/`SLOT()` macros resolve at run time and fail silently into a warning nobody reads.
- **Always pass a context object when connecting to a lambda.** `connect(sender, &Sender::signal, receiver, [=]{…})` ties the connection to `receiver`'s lifetime. A three-argument lambda connect outlives its captures and fires into freed memory.
- **Rust side: the bridge is the safety boundary.** CXX-Qt generates the `QObject` glue; keep the `#[cxx_qt::bridge]` thin and the logic in plain safe Rust behind it. Audit every crate with `cargo audit` (§3.3).

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Move / Queue):**
  - **Worker object on a `QThread`:** a `QObject` holding the long task, `moveToThread`-ed, driven and answered by queued signals. This is the sanctioned pattern.
  - **`QtConcurrent::run` + `QFutureWatcher`:** for a self-contained computation whose result is delivered by signal without blocking the GUI thread.
  - **`QPromise`** for progress reporting and exception propagation out of a worker.
- **When Concurrency Hurts (Do NOT Subclass / Block):**
  - **Subclassing `QThread` and putting slots on it:** a `QThread` instance lives in the thread that *created* it, so its queued slots run in the old thread — the exact opposite of the intent. Use a worker object.
  - **`QFuture::waitForFinished()` on the GUI thread:** blocks the event loop; the UI freezes for the duration. Watch with `QFutureWatcher` instead.
  - **Touching widgets from a worker thread:** all `QWidget` access is GUI-thread-only. Cross back with a queued connection or `QMetaObject::invokeMethod`.
  - **Unnamed lock guards:** `QMutexLocker(&m);` constructs a temporary that is destroyed at the end of the full expression, releasing the mutex immediately and leaving the critical section unprotected. Always name the guard.
  - **Mutating shared containers across threads** without synchronization — return results and merge on one thread instead.

## Mandatory Abstraction Choice
Always choose the abstraction corresponding to the task:
- **Application logic, models, state:** Rust behind `#[cxx_qt::bridge]`. This is the default.
- **UI definition:** QML with `qt_add_qml_module` when the project is Quick-based; `.ui` files compiled by `uic` when it is Widgets-based.
- **C++ surface:** only where CXX-Qt has no coverage — custom `QWidget::paintEvent`, `QAccessibleInterface` subclasses, and Qt modules the bindings do not wrap.
- **Threading:** worker `QObject` + `moveToThread` + queued signals; or `QtConcurrent::run` + `QFutureWatcher`. Never a `QThread` subclass.
- **Locking:** a **named** `QMutexLocker`; `std::scoped_lock` for multiple mutexes in a fixed order.
- **Accessibility bridge:** `QAccessible` names and roles on standard widgets; a `QAccessibleInterface` subclass for anything custom-painted (§18.3).
- **Theme:** a named `steelbore` theme applied through `QPalette` roles plus a token-substituted `steelbore.qss` — never hex literals in widget code (§11.1).
- **Packaging:** Flatpak manifest, `.desktop` entry, icons; file access through xdg-desktop-portal (§3.3 sandboxing).

## Required Techniques
1. **One ownership model per object:** parent it, or `unique_ptr` it. Never both. `QPointer` only to observe.
2. **PMF connect with a context object:** `connect(sender, &S::sig, receiver, &R::slot)`; for lambdas always supply the receiver as context.
3. **Worker-object threading:** `QObject` worker + `moveToThread(&thread)`, results returned by queued signal; document the §3.2 trade-off.
4. **Named lock guards:** `QMutexLocker locker(&m_mutex);` — an unnamed guard is a data race, not a lock.
5. **Deprecation and cast gates:** define `QT_DISABLE_DEPRECATED_UP_TO` to the supported Qt floor and `QT_NO_CAST_FROM_ASCII` so every user-visible string goes through `tr()`.
6. **Accessible name and role on every interactive widget:** `setAccessibleName` / `setAccessibleDescription`; `QAccessibleInterface` for custom paint. Verify with Orca (§18.4).
7. **Theme tokens only:** `QPalette` roles bound to `steelbore` tokens and a generated `steelbore.qss`; a bare hex literal is a §11.1 violation.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Qt **6.8 LTS** (supported to 2029-10-08); Qt 6.11 is current, and 6.12 is the next LTS. CMake ≥ 3.21, C++20. CXX-Qt 0.9.x, Rust 1.83+.
- **CMake shape:** `qt_standard_project_setup()`, `qt_add_executable`, `qt_add_qml_module`. Never hand-rolled `moc` invocations; never `qmake` in new projects.
- **C++ gates:** everything in `spacecraft-cpp-guidelines` — `-Wall -Wextra -Wpedantic -Werror`, `-D_GLIBCXX_ASSERTIONS` / `-D_LIBCPP_HARDENING_MODE`, `-fhardened`, sanitizers in Debug. Plus `clazy` (Qt-semantic warnings) and `clang-tidy`, both failing the build.
- **QML gates:** the `all_qmllint` target generated by `qt_add_qml_module`, and `qmlformat --verify`.
- **Rust gates:** `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo audit`, `cargo test`.
- **Flags:** note every applied and every disabled optimization flag (§3.2). Per §3.2.1, on NixOS `-flto` requires `-fuse-ld=mold` (preferred) or `-fuse-ld=bfd`.
- **Accessibility gate:** exercise the built application with Orca before release (§18.4).

## Anti-Patterns (Never Do These)
- Giving a `QObject` both a parent and a smart-pointer owner.
- `delete` on a `QObject` from inside a slot or event handler instead of `deleteLater()`.
- `SIGNAL()` / `SLOT()` string macros in new code.
- Connecting to a lambda without a context object.
- Subclassing `QThread` and adding slots to the subclass.
- `QFuture::waitForFinished()` or any blocking call on the GUI thread.
- Touching a `QWidget` from a worker thread.
- `QMutexLocker(&m);` as an unnamed temporary.
- Acquiring multiple mutexes without a fixed global order.
- Writing hex color literals into widget code or QSS instead of `steelbore` tokens.
- Shipping a custom-painted widget with no `QAccessibleInterface`.
- Starting a new Qt project in C++ without filing the §3.1 exemption — and starting a new native desktop project in Qt at all without recording why not GTK/`gtk-rs`.

## Pre-Commit Checklist (Verify Every Time)
- [ ] The choice of Qt over `gtk-rs` is recorded, and the §3.1 C++ exemption is filed with ASLR + CFI enabled
- [ ] Application logic lives in Rust behind CXX-Qt; the C++ surface is limited to what the bindings do not cover
- [ ] Every `QObject` has exactly one owner — parent or smart pointer, never both
- [ ] `deleteLater()` is used in slots and event handlers; no bare `delete` on a live `QObject`
- [ ] All connections use PMF syntax; every lambda connection supplies a context object
- [ ] No `QThread` subclass carries slots; workers use `moveToThread` and queued signals
- [ ] No blocking call and no widget access occurs off the GUI thread
- [ ] Every `QMutexLocker` is named; multi-mutex sections use a fixed order or `std::scoped_lock`
- [ ] `QT_DISABLE_DEPRECATED_UP_TO` and `QT_NO_CAST_FROM_ASCII` are defined
- [ ] Every interactive widget has an accessible name and role; custom-painted widgets subclass `QAccessibleInterface`
- [ ] All colors come from `steelbore` `QPalette` roles and QSS tokens — no hex literals
- [ ] `clazy`, `clang-tidy`, `qmllint`, and `-Werror` are clean; Rust gates pass
- [ ] Applied and disabled compiler flags are both documented (§3.2)

## References & Further Reading
- Load `references/Spacecraft_Qt_Rust_Guidelines.md` for the preferred path — CXX-Qt bridges, properties, signals, invokables, QML models from Rust, `cxx-qt-build` CMake wiring, and the honest coverage boundary.
- Load `references/Spacecraft_Qt_Cpp_Guidelines.md` for the C++ surface — ownership, connect discipline, worker threading, hardened CMake, accessibility, theming, packaging.
- Load `references/Spacecraft_Qt_QML_Guidelines.md` for Qt Quick — QML↔C++ ownership, `qt_add_qml_module`, `qmllint`, `Accessible` attached properties, Quick Controls theming.
- Cross-reference `spacecraft-accessibility-support` for the §18 bridge table, activation contract, and audit gates — this skill does not restate them.
- Cross-reference `steelbore-color-palette` for palette values and `spacecraft-theme-factory` for emitting `steelbore.qss`. Never retype hex values from memory.
- **Licensing (§4.2).** Open-source Qt 6 is `LGPL-3.0` for most modules, with **fourteen modules available only under `GPL-3.0`** — Qt Canvas Painter, Qt CoAP, Qt Graphs, Qt GRPC, Qt HTTP Server, Qt Lottie Animation, Qt MQTT, Qt Network Authorization, Qt Qml Compiler, Qt Quick 3D, Qt Quick 3D Physics, Qt Quick Timeline, Qt Virtual Keyboard, Qt Wayland Compositor. All are compatible with a `GPL-3.0-or-later` project; a GPL-only module forecloses any future relicensing, so record which modules are linked. CXX-Qt is `MIT OR Apache-2.0`. Preserve upstream notices and ship each distinct license text in `LICENSES/` (§4.3).
- *Further reading* (consulted for background only): the Qt 6 API reference, the Qt threading and accessibility documentation, the CXX-Qt book, and the KDE Human Interface Guidelines.

When the user requests Qt code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft desktop engineer would ship.
