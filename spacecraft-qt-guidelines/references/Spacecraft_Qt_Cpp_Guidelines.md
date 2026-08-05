# Spacecraft Qt 6 Guidelines (C++) — Full Reference

**Version:** 1.0
**Date:** 2026-08-05
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document covers the Qt 6 **C++ surface** — the layer CXX-Qt does not reach, and every existing Qt codebase. It adds the Qt-specific layer on top of `spacecraft-cpp-guidelines`, which retains higher dominance on C++ hardening flags, RAII, the Rule of Zero/Five, and general lock discipline. **Do not duplicate those rules here — load that skill.**

Every file governed by this document is a Standard §3.1 exemption: C++ is not a memory-safe language, and a memory-safe alternative exists for most of what it is doing. Keep this surface as small as `Spacecraft_Qt_Rust_Guidelines.md` §1 allows.

---

## 1. The §3.1 Exemption and Its Mitigations

Two separate justifications are owed, and they are not the same argument:

1. **Why Qt rather than GTK 4 with `gtk-rs`.** GTK plus `gtk-rs` is the memory-safe default for a new native desktop application. Choosing Qt needs a ground: an existing Qt codebase, KDE/Plasma integration, a platform where Qt is the supported toolkit, or a hard dependency on a Qt module with no equivalent.
2. **Why this file is C++ rather than Rust behind CXX-Qt.** Custom painting, `QAccessibleInterface`, an unbound Qt module, or deep `QWidget` subclassing.

Record both in `README.md` beside the §5.2 posture section. Then apply §3.1's mandatory mitigations:

```cmake
# ASLR (§3.1)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
add_link_options(-pie)

# CFI (§3.1), where the toolchain supports it
if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    add_compile_options(-fsanitize=cfi -fvisibility=hidden -flto)
    add_link_options(-fsanitize=cfi -flto)
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    add_compile_options(-fcf-protection=full)
endif()
```

---

## 2. Ownership: One Model Per Object

Qt's `QObject` parent-child tree and C++ smart pointers are two complete ownership systems. Applying both to one object is the leading Qt crash — the destructor order eventually changes and the object is deleted twice.

```cpp
#include <QtWidgets>
#include <memory>

class TelemetryWindow : public QMainWindow
{
    Q_OBJECT
public:
    explicit TelemetryWindow(QWidget *parent = nullptr)
        : QMainWindow(parent)
    {
        // CORRECT: parented. The QObject tree owns it; do not wrap in a
        // smart pointer and do not delete it manually.
        m_refreshButton = new QPushButton(tr("Rebuild telemetry index"), this);

        // CORRECT: a non-QObject helper is owned by unique_ptr.
        m_parser = std::make_unique<CaptureParser>();

        // WRONG — do not do this. Two owners, one object:
        //
        //   auto owned = std::make_unique<QPushButton>(tr("Bad"), this);
        //
        // The parent deletes the button when the window dies; the unique_ptr
        // then deletes it again. Pick one owner.

        // CORRECT: QPointer OBSERVES. It nulls itself on destruction, which
        // is exactly what a cached back-pointer needs — and exactly what an
        // owner must not rely on.
        m_activeDialog = nullptr;
    }

private:
    QPushButton *m_refreshButton = nullptr;          // owned by the QObject tree
    std::unique_ptr<CaptureParser> m_parser;         // owned by unique_ptr
    QPointer<QDialog> m_activeDialog;                // observed, not owned
};
```

**Deletion inside a slot.** Deleting a `QObject` while Qt is still unwinding a signal emission through it is a use-after-free. `deleteLater()` defers to the next event-loop iteration:

```cpp
connect(m_dialog, &QDialog::finished, this, [this](int result) {
    handleResult(result);
    m_dialog->deleteLater();   // NOT `delete m_dialog;`
    m_dialog = nullptr;
});
```

Prefer `std::unique_ptr` over `QScopedPointer` in new Qt 6 code; reserve `QSharedPointer` for APIs that already traffic in it. Follow `spacecraft-cpp-guidelines` on the Rule of Zero / Rule of Five, and use `Q_DISABLE_COPY_MOVE` on `QObject` subclasses — they are non-copyable by design and the macro makes the intent a compile error rather than a surprise.

---

## 3. Signals and Slots: Compile-Time Connections

```cpp
// CORRECT: pointer-to-member-function syntax. Checked at compile time —
// a renamed signal is a build error, not a silent runtime warning.
connect(m_refreshButton, &QPushButton::clicked,
        this,            &TelemetryWindow::reloadPackets);

// CORRECT: a lambda WITH a context object. The connection is tied to
// `this`; when `this` dies the connection is severed automatically.
connect(m_worker, &TelemetryWorker::packetsReady,
        this,     [this](const QVector<Packet> &packets) { applyPackets(packets); });

// WRONG — a three-argument lambda connect has no context object. The
// connection outlives `this`, and the captured `this` dangles:
//
//   connect(m_worker, &TelemetryWorker::packetsReady,
//           [this](const QVector<Packet> &p) { applyPackets(p); });

// WRONG — string-based macros resolve at run time. A typo produces a
// console warning nobody reads and a connection that never fires:
//
//   connect(m_refreshButton, SIGNAL(clicked()), this, SLOT(reloadPackts()));
```

Retain the `QMetaObject::Connection` when a connection must be severed before either object dies:

```cpp
m_settingsConnection = connect(m_settings, &Settings::intervalChanged,
                               this, &TelemetryWindow::setInterval);
// later
disconnect(m_settingsConnection);
```

---

## 4. Threading: Worker Objects, Never `QThread` Subclasses

A `QThread` instance **lives in the thread that created it**, so slots added to a `QThread` subclass execute in the *old* thread — precisely the opposite of the intent. The sanctioned pattern is a worker `QObject` moved onto a thread and driven by queued signals.

```cpp
class TelemetryWorker : public QObject
{
    Q_OBJECT
public slots:
    void loadCapture(const QString &path)
    {
        // Runs on the worker thread. Not one QWidget call appears here.
        QVector<Packet> packets = CaptureParser::parse(path);
        emit packetsReady(packets);       // queued across the thread boundary
    }

signals:
    void packetsReady(const QVector<Packet> &packets);
};

class TelemetryWindow : public QMainWindow
{
    Q_OBJECT
public:
    explicit TelemetryWindow(QWidget *parent = nullptr) : QMainWindow(parent)
    {
        // CONCURRENCY TRADE-OFF (§3.2): capture parsing is I/O- and CPU-bound
        // and exceeds the frame budget, so it moves off the GUI thread. One
        // persistent worker thread is used rather than a pool — captures are
        // loaded one at a time, so extra parallelism would add synchronization
        // cost without reducing latency. Revisit if batch loading is added.
        auto *worker = new TelemetryWorker;          // no parent: it is moved
        worker->moveToThread(&m_workerThread);

        connect(&m_workerThread, &QThread::finished, worker, &QObject::deleteLater);
        connect(this,   &TelemetryWindow::loadRequested, worker, &TelemetryWorker::loadCapture);
        connect(worker, &TelemetryWorker::packetsReady,  this,   &TelemetryWindow::applyPackets);

        m_workerThread.start();
    }

    ~TelemetryWindow() override
    {
        // Always quit and join. A running QThread destroyed here terminates
        // the process, the Qt analogue of the std::thread trap in
        // spacecraft-cpp-guidelines.
        m_workerThread.quit();
        m_workerThread.wait();
    }

private:
    QThread m_workerThread;
};
```

**`QtConcurrent` for self-contained computations:**

```cpp
auto *watcher = new QFutureWatcher<QVector<Packet>>(this);
connect(watcher, &QFutureWatcherBase::finished, this, [this, watcher] {
    applyPackets(watcher->result());
    watcher->deleteLater();
});
watcher->setFuture(QtConcurrent::run(&CaptureParser::parse, path));

// WRONG — blocks the event loop; the UI freezes for the whole parse:
//
//   auto packets = QtConcurrent::run(&CaptureParser::parse, path).result();
//   // .result() and .waitForFinished() both block. Watch instead.
```

**Locking.** The unnamed-guard trap from `spacecraft-cpp-guidelines` has an exact Qt form:

```cpp
// WRONG — a temporary destroyed at the end of the full expression. The mutex
// is released immediately and the "critical section" below is unprotected.
//
//   QMutexLocker(&m_mutex);
//   m_packets.append(packet);

// CORRECT — named, so it lives to the end of the scope.
QMutexLocker locker(&m_mutex);
m_packets.append(packet);
```

Acquire multiple mutexes in a fixed global order, or with `std::scoped_lock`, and never invoke an unknown callback or emit a signal while holding a lock — a queued connection is safe, but a direct one runs arbitrary code inside the critical section.

---

## 5. Hardened CMake Configuration

```cmake
cmake_minimum_required(VERSION 3.21)
project(TelemetryDashboard LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Widgets Quick)
qt_standard_project_setup()          # AUTOMOC, RPATH, and project defaults

qt_add_executable(telemetry-dashboard
    src/main.cpp
    src/telemetry_window.cpp
)

target_compile_definitions(telemetry-dashboard PRIVATE
    # Deprecated API is an error, pinned to the supported Qt floor (6.8 LTS).
    QT_DISABLE_DEPRECATED_UP_TO=0x060800
    # Force every user-visible string through tr(); catches untranslated text
    # and implicit Latin-1 conversions at compile time.
    QT_NO_CAST_FROM_ASCII
    QT_NO_CAST_TO_ASCII
    # Keep Q_ASSERT live in release builds — a failed invariant should abort
    # loudly rather than continue into undefined behaviour (§3.1).
    QT_FORCE_ASSERTS
)

# §3.2 — every applied flag noted, every disabled flag noted with its reason.
target_compile_options(telemetry-dashboard PRIVATE
    -Wall -Wextra -Wpedantic -Werror   # applied: no warning ships
    -fstack-protector-strong           # applied: stack smashing detection
    -fno-omit-frame-pointer            # applied: usable crash backtraces
)

if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    target_compile_options(telemetry-dashboard PRIVATE -fhardened)
    target_compile_definitions(telemetry-dashboard PRIVATE _GLIBCXX_ASSERTIONS)
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    target_compile_definitions(telemetry-dashboard PRIVATE
        _LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_EXTENSIVE)
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_options(telemetry-dashboard PRIVATE -fsanitize=address,undefined)
    target_link_options(telemetry-dashboard    PRIVATE -fsanitize=address,undefined)
endif()

# DISABLED: -flto.
#   Reason (§3.2.1): under NixOS the GCC LTO plugin is not on the default
#   linker search path, so -flto fails to link unless paired with
#   -fuse-ld=mold (preferred) or -fuse-ld=bfd. Stability (P1) outranks
#   Performance (P2) — LTO stays off until the linker is pinned.
#
# DISABLED: -march=native.
#   Reason: distribution and Flatpak builds target unknown hardware.

target_link_libraries(telemetry-dashboard PRIVATE Qt6::Widgets Qt6::Quick)
```

Never hand-invoke `moc`, `uic`, or `rcc` — `qt_standard_project_setup()` wires them. Never introduce `qmake` into a new project.

---

## 6. Accessibility (Standard §18)

`spacecraft-accessibility-support` owns the §18 contract; this covers the Qt API surface only.

```cpp
m_refreshButton->setAccessibleName(tr("Rebuild telemetry index"));
m_refreshButton->setAccessibleDescription(
    tr("Re-scans the capture directory and reloads all packets"));

// State changes are ANNOUNCED, not merely repainted. Qt 6.8 added the
// announcement channel; a stopped spinner communicates nothing on its own.
QAccessibleAnnouncementEvent event(m_refreshButton, tr("Telemetry index rebuilt"));
QAccessible::updateAccessibility(&event);
```

**Custom-painted widgets are the failure case.** Setting a name on a `QWidget` that paints its own children does not describe those children — to assistive technology they do not exist. Subclass `QAccessibleInterface` and publish them:

```cpp
class PacketGraphAccessible : public QAccessibleWidget
{
public:
    explicit PacketGraphAccessible(PacketGraph *widget)
        : QAccessibleWidget(widget, QAccessible::Graphic) {}

    int childCount() const override
    { return graph()->seriesCount(); }

    QAccessibleInterface *child(int index) const override
    { return new PacketSeriesAccessible(graph(), index); }

    QString text(QAccessible::Text t) const override
    {
        if (t == QAccessible::Name)
            return PacketGraph::tr("Telemetry packet graph");
        return QAccessibleWidget::text(t);
    }

private:
    PacketGraph *graph() const { return static_cast<PacketGraph *>(widget()); }
};

// Register the factory at startup.
QAccessible::installFactory([](const QString &key, QObject *object) -> QAccessibleInterface * {
    if (key == QLatin1String("PacketGraph"))
        return new PacketGraphAccessible(static_cast<PacketGraph *>(object));
    return nullptr;
});
```

On Linux the `qt-at-spi` bridge plugin exposes this over AT-SPI to Orca; on Windows it maps to MSAA/UIA and on macOS to NSAccessibility. Verify with a real screen reader (§18.4).

---

## 7. Theming: `steelbore` Tokens, Never Hex Literals

Standard §11.1 requires all palette references to go through a named `steelbore` theme so a user can swap it without touching application code. In Qt that means a `QPalette` built from tokens plus a generated `steelbore.qss` — never a hex literal in widget code.

Generate the stylesheet with `spacecraft-theme-factory`; take values from `steelbore-color-palette`'s `assets/steelbore.toml`. **Never retype hex values from memory.**

```cpp
// src/theme/steelbore.cpp — the single place palette values enter the program.
QPalette buildSteelborePalette(const SteelboreTokens &t)
{
    QPalette p;
    p.setColor(QPalette::Window,          t.background);
    p.setColor(QPalette::Base,            t.surfaceAlt);
    p.setColor(QPalette::AlternateBase,   t.surface);
    p.setColor(QPalette::WindowText,      t.foreground);
    p.setColor(QPalette::Text,            t.foreground);
    p.setColor(QPalette::Highlight,       t.accent);
    p.setColor(QPalette::Link,            t.structure);
    p.setColor(QPalette::Mid,             t.border);
    return p;
}
```

```css
/* resources/steelbore.qss — generated; tokens substituted at build time. */
QMainWindow          { background: @STEELBORE_BACKGROUND@; color: @STEELBORE_FOREGROUND@; }
QFrame#panel         { background: @STEELBORE_SURFACE@;
                       /* §11.0.1: a surface edge against the canvas is below the
                          3:1 non-text floor, so a meaningful boundary is drawn. */
                       border: 1px solid @STEELBORE_BORDER@; }
*:focus              { outline: 2px solid @STEELBORE_FOCUS@; }
QLabel#statusError   { color: @STEELBORE_ERROR@; }   /* paired with an "[ERROR] " text tag */
```

Under accessible mode apply the `steelbore-high-contrast` variant, and honour the platform high-contrast and reduced-motion preferences **independently** of the §18.1 toggle.

---

## 8. Desktop Integration & Sandboxing (§3.3)

```ini
# data/org.spacecraftsoftware.TelemetryDashboard.desktop
[Desktop Entry]
Type=Application
Name=Telemetry Dashboard
Exec=telemetry-dashboard %U
Icon=org.spacecraftsoftware.TelemetryDashboard
Categories=Utility;Monitor;
StartupNotify=true
```

```yaml
# build-aux/org.spacecraftsoftware.TelemetryDashboard.yaml
app-id: org.spacecraftsoftware.TelemetryDashboard
runtime: org.kde.Platform
runtime-version: '6.8'
sdk: org.kde.Sdk
command: telemetry-dashboard
finish-args:
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  # NO --filesystem=host. QFileDialog routes through xdg-desktop-portal under
  # Flatpak, so the user grants exactly the files they picked (§3.3).
```

`QFileDialog` uses the portal automatically inside a Flatpak sandbox provided the platform theme plugin is present — do not bypass it with a custom file browser.

---

## 9. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Parent plus smart pointer** | Double free on shutdown | One owner per object — parent, or `unique_ptr`, never both. |
| **`delete` inside a slot** | Use-after-free while the emission unwinds | `deleteLater()`. |
| **Lambda connect without context** | Callback fires into a destroyed receiver | Pass the receiver as the third argument. |
| **`SIGNAL()`/`SLOT()` macros** | Connection silently never fires | Pointer-to-member-function syntax; it is compile-checked. |
| **`QThread` subclass with slots** | Slots run in the creating thread; no speedup, subtle races | Worker `QObject` + `moveToThread` + queued signals. |
| **`QThread` destroyed while running** | `QThread: Destroyed while thread is still running`, then abort | `quit()` then `wait()` in the owner's destructor. |
| **`waitForFinished()` on the GUI thread** | UI freezes for the whole computation | `QFutureWatcher::finished`. |
| **Unnamed `QMutexLocker`** | Data race with a lock apparently present | Name the guard: `QMutexLocker locker(&m);`. |
| **Signal emitted under a direct connection while locked** | Deadlock or reentrancy through the critical section | Emit outside the lock, or use a queued connection. |
| **Widget touched from a worker** | Random crashes, painting artifacts | Queued signal or `QMetaObject::invokeMethod`. |
| **Custom-painted widget** | Screen reader reports one empty region | Subclass `QAccessibleInterface` and publish children. |
| **Hex literals in QSS** | §11.1 violation; theme cannot be swapped | Generate `steelbore.qss` from tokens. |
| **LTO on NixOS** | Link failure: LTO plugin not found | §3.2.1 — pair with `-fuse-ld=mold`, or document it disabled. |

---

## 10. Code Review Compliance Gate

Before merging Qt C++ code, verify:
1. Both justifications are recorded — Qt over `gtk-rs`, and C++ over CXX-Qt — and ASLR + CFI are enabled.
2. Every `QObject` has exactly one owner; `QPointer` is used only to observe.
3. No bare `delete` on a live `QObject` inside a slot or event handler.
4. All connections use PMF syntax, and every lambda connection supplies a context object.
5. No `QThread` subclass carries slots; every `QThread` is `quit()`-ed and `wait()`-ed by its owner.
6. No blocking call and no widget access occurs off the GUI thread.
7. Every `QMutexLocker` is named; multi-mutex sections use a fixed order; no unknown callback runs under a lock.
8. The §3.2 concurrency trade-off is documented where work moved off the GUI thread.
9. `QT_DISABLE_DEPRECATED_UP_TO`, `QT_NO_CAST_FROM_ASCII`, and `QT_FORCE_ASSERTS` are defined.
10. Every interactive widget has an accessible name and role; custom-painted widgets ship a `QAccessibleInterface`.
11. Colors resolve through `steelbore` `QPalette` roles and QSS tokens; no hex literal appears in C++ or QSS (§11.1).
12. `clazy`, `clang-tidy`, and `-Werror` are clean; applied and disabled flags are both documented.
13. Everything in the `spacecraft-cpp-guidelines` compliance gate also passes — it governs the C++, this list governs the Qt layer.
