# Spacecraft Qt 6 Guidelines (QML / Qt Quick) — Full Reference

**Version:** 1.0
**Date:** 2026-08-05
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document covers the Qt Quick / QML view layer. **Qt Quick is the preferred UI layer for a new Spacecraft Qt project** — paired with Rust models behind CXX-Qt (`Spacecraft_Qt_Rust_Guidelines.md`), it keeps essentially all logic on the memory-safe side of the bridge, whereas a Widgets application drags custom painting and `QAccessibleInterface` into C++.

QML is a separate language with its own ownership semantics and its own failure modes. The rules below are not a restatement of the C++ ones.

---

## 1. Module Definition, Not Loose Files

Declare a proper QML module with `qt_add_qml_module`. It generates the type registrations, the `qmldir`, the compilation targets, and — importantly — the `all_qmllint` target that becomes the CI gate. Loose `.qml` files loaded by URL get none of that.

```cmake
qt_add_qml_module(telemetry-dashboard
    URI org.spacecraftsoftware.telemetry
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/PacketList.qml
        qml/StatusBanner.qml
    RESOURCES
        resources/steelbore.qss
)
```

```qml
// qml/Main.qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.spacecraftsoftware.telemetry   // Rust types registered via cxx-qt-build

ApplicationWindow {
    id: root
    visible: true
    width: 960; height: 640
    title: qsTr("Telemetry Dashboard")

    // The model is implemented in Rust. QML never owns domain state.
    TelemetryModel { id: telemetry }

    ColumnLayout {
        anchors.fill: parent

        StatusBanner {
            Layout.fillWidth: true
            text: telemetry.statusText
            busy: telemetry.busy
        }

        PacketList {
            Layout.fillWidth: true
            Layout.fillHeight: true
            model: telemetry
        }
    }
}
```

---

## 2. QML ↔ C++ Ownership: The Double-Ownership Crash

This is the QML-specific memory bug, and it has no analogue in the Widgets rules. The QML engine's JavaScript garbage collector and Qt's `QObject` parent tree are two independent owners. Qt decides between them by a rule that is easy to trip:

| How the object reached QML | Owner |
| :--- | :--- |
| Returned from a `Q_INVOKABLE` / slot, **no parent set** | **JavaScript GC** — QML may delete it at any time |
| Returned from a `Q_INVOKABLE` / slot, **parent set** | C++ parent |
| Returned from a `Q_PROPERTY` getter | **C++** — QML never takes ownership |
| Created by a QML declaration | The QML engine |
| Registered as a singleton | The engine |

The crash: C++ hands out an unparented `QObject` from an invokable, keeps a raw pointer to it, and the JavaScript GC frees it. The C++ pointer dangles.

```cpp
// WRONG — QML takes ownership because there is no parent, and C++ keeps
// a pointer to memory the GC will reclaim.
Q_INVOKABLE PacketDetail *detailFor(int row) {
    auto *detail = new PacketDetail(m_packets.at(row));
    m_lastDetail = detail;          // dangles as soon as QML drops the value
    return detail;
}

// CORRECT (a) — parent it, so C++ owns it unambiguously.
Q_INVOKABLE PacketDetail *detailFor(int row) {
    auto *detail = new PacketDetail(m_packets.at(row), this);   // parented
    QQmlEngine::setObjectOwnership(detail, QQmlEngine::CppOwnership);
    return detail;
}

// CORRECT (b) — hand out a value type instead. No ownership question exists.
Q_INVOKABLE QVariantMap detailFor(int row) const {
    const auto &p = m_packets.at(row);
    return {{"sensorId", p.sensorId}, {"value", p.value}};
}
```

**Prefer (b).** Returning a value type — a `QVariantMap`, a `Q_GADGET`, or a registered value type — removes the ownership question entirely. In the Rust path this is the default anyway: `cxx-qt-lib` value types cross the bridge by value.

Never call `setObjectOwnership(…, JavaScriptOwnership)` on an object that C++ also references.

---

## 3. Property Bindings and Performance

- **Declarative bindings, not imperative assignment.** Assigning to a property in JavaScript *destroys* its binding permanently. `width: parent.width / 2` re-evaluates; `onParentChanged: width = parent.width / 2` does not, and silently stops tracking.
- **No heavy work in a binding.** A binding may re-evaluate many times per frame. Parsing, formatting, or filtering inside one is a frame-budget bug — compute it in the Rust model and expose the result as a property.
- **`Loader` and `asynchronous: true`** for panels not visible at startup, so first paint is not held hostage to the whole tree.
- **Reuse delegates:** set `reuseItems: true` on `ListView` and keep delegates shallow. A delegate that instantiates dozens of items per row will not scroll at 60 Hz regardless of what the model does.
- **`required property`** in delegates rather than implicit context properties — it is faster to resolve and it fails loudly instead of silently reading `undefined`.

```qml
// qml/PacketList.qml
import QtQuick
import QtQuick.Controls

ListView {
    id: view
    clip: true
    reuseItems: true

    delegate: ItemDelegate {
        // Explicit and compile-checked by qmllint; no context-property lookup.
        required property int    index
        required property string sensorId
        required property real   value

        width: view.width
        text: qsTr("Sensor %1 — %2").arg(sensorId).arg(value.toFixed(2))

        // §18: every interactive element carries a name and a role.
        Accessible.role: Accessible.ListItem
        Accessible.name: text
    }
}
```

---

## 4. Accessibility in QML (Standard §18)

Qt Quick exposes accessibility through the `Accessible` attached property. Standard Quick Controls carry sensible defaults; anything custom-drawn — a `Canvas`, a custom `Item` with a `ShaderEffect`, a hand-built control — carries **nothing** until it is declared.

```qml
Button {
    id: refreshButton
    icon.name: "view-refresh"
    // An icon-only button announces as "button" and nothing else without this.
    Accessible.role: Accessible.Button
    Accessible.name: qsTr("Rebuild telemetry index")
    Accessible.description: qsTr("Re-scans the capture directory and reloads all packets")
    Accessible.onPressAction: clicked()
}

// Decoration is explicitly excluded so it is skipped rather than read as noise.
Rectangle {
    height: 1
    color: Theme.border
    Accessible.ignored: true
}

// State changes are ANNOUNCED, not merely repainted (Qt 6.8+).
Connections {
    target: telemetry
    function onCaptureLoaded(packets) {
        refreshButton.Accessible.announce(
            qsTr("Loaded %1 packets").arg(packets))
    }
}
```

Custom-drawn `Canvas` items must expose their contents as accessible children or a textual summary — a painted chart with no `Accessible` declaration is invisible to a screen reader, exactly as a `GtkDrawingArea` or a custom-painted `QWidget` would be (§18.3). Honour system reduced-motion independently of the §18.1 toggle before enabling any `Animation`.

**Known Qt 6.11 diagnostic — not your bug.** Rebuilding dynamic scene nodes (a `Repeater`
or a model-driven `ListView` whose delegates are recreated) makes Qt 6.11 emit repeated
stale-accessible-path warnings. The tree itself stays correct: Orca exposes and operates it
normally. Do not chase the log, and do not paper over it by pinning `Accessible` properties
onto recycled delegates — that trades a harmless warning for a genuinely wrong tree. The
§18.4 gate is an Orca run against the built application, never a clean console.

---

## 5. Theming with `steelbore` Tokens

Standard §11.1 forbids hex literals in UI logic. In QML the indirection is a singleton exposing the `steelbore` tokens, generated by `spacecraft-theme-factory` from `steelbore-color-palette`'s `assets/steelbore.toml`. **Never retype hex values from memory.**

```qml
// qml/Theme.qml  (registered as a QML singleton — generated, do not hand-edit)
pragma Singleton
import QtQuick

QtObject {
    readonly property color background:  "@STEELBORE_BACKGROUND@"
    readonly property color surface:     "@STEELBORE_SURFACE@"
    readonly property color surfaceAlt:  "@STEELBORE_SURFACE_ALT@"
    readonly property color foreground:  "@STEELBORE_FOREGROUND@"
    readonly property color accent:      "@STEELBORE_ACCENT@"
    readonly property color structure:   "@STEELBORE_STRUCTURE@"
    readonly property color success:     "@STEELBORE_SUCCESS@"
    readonly property color error:       "@STEELBORE_ERROR@"
    readonly property color warning:     "@STEELBORE_WARNING@"
    readonly property color focus:       "@STEELBORE_FOCUS@"
    readonly property color border:      "@STEELBORE_BORDER@"
}
```

```qml
Rectangle {
    color: Theme.surface
    // §11.0.1 — a surface edge against the canvas falls below the 3:1 non-text
    // floor, so a meaningful boundary is drawn rather than implied.
    border.color: Theme.border
    border.width: 1

    Label {
        // §11.0.2 — color is never the sole carrier of meaning.
        text: telemetry.busy ? qsTr("[INFO] Loading…") : qsTr("[OK] Idle")
        color: telemetry.busy ? Theme.warning : Theme.success
    }
}
```

Qt Quick Controls' Basic or Fusion style themed with these tokens is the Spacecraft default. The Material style exists and may be used where a project genuinely wants it, but Standard §13 admits platform-native design systems for native desktop toolkits — a KDE/Plasma-targeted application should follow the KDE HIG rather than Material.

---

## 6. Static Analysis & CI Gates

`qmllint` is not optional. `qt_add_qml_module` generates per-module and aggregate targets; wire the aggregate into CI.

```yaml
- name: Configure
  run: cmake -B build -DCMAKE_BUILD_TYPE=Debug
- name: QML lint
  run: cmake --build build --target all_qmllint
- name: QML format check
  run: qmlformat --verify $(find qml -name '*.qml')
- name: C++ Qt semantics
  run: clazy-standalone -p build/compile_commands.json $(find src -name '*.cpp')
```

```qml
// Silence a lint category only where it is genuinely wrong, and say why —
// never blanket-disable, and never delete the gate.
// qmllint disable unqualified
```

Add QML tests with `QUICK_TEST_MAIN`, exercising the model bindings rather than pixel layout:

```cpp
// tests/tst_qml.cpp
#include <QtQuickTest>
QUICK_TEST_MAIN(telemetry)
```

```qml
// tests/tst_status.qml
import QtQuick
import QtTest
import org.spacecraftsoftware.telemetry

TestCase {
    name: "StatusBanner"
    TelemetryModel { id: model }

    function test_error_is_text_tagged() {
        model.reload("tests/data/truncated.cap")
        // §11.0.2 — status must not be color-only.
        verify(model.statusText.startsWith("[ERROR]"))
    }
}
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Unparented `QObject` returned to QML** | Crash after garbage collection; C++ pointer dangles | Parent it and set `CppOwnership`, or return a value type. |
| **Imperative property assignment** | Binding silently stops updating | Keep the declarative binding; never assign to a bound property. |
| **Heavy work in a binding** | Frame drops that profile as "QML" with no obvious cause | Compute in the Rust model; expose the result as a property. |
| **Loose `.qml` files, no module** | No `qmllint` target, no compile-time type checks | Declare with `qt_add_qml_module`. |
| **Implicit context properties in delegates** | Silent `undefined`; slow property resolution | `required property` in every delegate. |
| **Deep delegates without `reuseItems`** | Scrolling stutters on long lists | `reuseItems: true`; flatten the delegate. |
| **Icon-only `Button`** | Screen reader announces "button" | Set `Accessible.role` and `Accessible.name`. |
| **Custom `Canvas` control** | Invisible to assistive technology | Declare accessible children or a textual summary (§18.3). |
| **Stale-accessible-path warnings (Qt 6.11)** | Repeated diagnostics while delegates are rebuilt | Known Qt behaviour with dynamic scene nodes; verify the tree with Orca (§18.4) rather than silencing the log. |
| **Hex literals in QML** | §11.1 violation; theme cannot be swapped | Reference the generated `Theme` singleton. |
| **`qmllint` warnings ignored** | Type errors surface at run time, in front of a user | Make `all_qmllint` a failing CI gate. |

---

## 8. Code Review Compliance Gate

Before merging QML, verify:
1. All QML ships inside a `qt_add_qml_module` module — no loose URL-loaded files.
2. Every `QObject` handed to QML has unambiguous ownership: parented with `CppOwnership`, or replaced by a value type.
3. No bound property is imperatively assigned; no binding performs heavy work.
4. Delegates use `required property`, are shallow, and set `reuseItems: true` on long lists.
5. Every interactive element declares `Accessible.role` and `Accessible.name`; decoration sets `Accessible.ignored`.
6. Custom-drawn items publish accessible children or a textual summary (§18.3).
7. State changes are announced, not merely repainted; reduced motion is honoured independently of the §18.1 toggle.
8. All colors come from the generated `Theme` singleton; no hex literal appears in QML (§11.1).
9. Status is never color-only — it carries `[OK]` / `[WARN]` / `[ERROR]` / `[INFO]` text (§11.0.2).
10. `all_qmllint` and `qmlformat --verify` pass; any lint suppression is narrow and justified in a comment.
11. Domain logic lives in the Rust model behind CXX-Qt, not in QML JavaScript.
