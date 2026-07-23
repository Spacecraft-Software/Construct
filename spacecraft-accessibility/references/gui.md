# GUI Accessibility — implementation reference

Standard §18.3. Read [`../SKILL.md`](../SKILL.md) first for the activation
contract.

## The model

Unlike a terminal, a GUI toolkit *can* publish an accessibility tree: a
parallel structure describing each element's **role** (what kind of thing it
is), **name** (what to call it), **state** (checked, expanded, disabled), and
**relations** (labelled-by, controls). Assistive technology reads that tree
through the platform API — UI Automation on Windows, NSAccessibility on macOS,
AT-SPI on Linux.

**The failure case is custom drawing.** A control painted onto a canvas has no
tree node unless the application publishes one. It is not "partially
accessible" — it is invisible. Anything that renders its own widgets (a game
engine UI, an immediate-mode GUI, a custom-drawn editor surface) must publish
its own accessibility tree.

## Choosing a bridge

| UI stack | Required bridge | Notes |
|----------|-----------------|-------|
| **Rust, custom-drawn UI** | **AccessKit** (Apache-2.0) | One API over UIA / NSAccessibility / AT-SPI. Already integrated in egui, Slint, Bevy, Freya, Xilem; `winit` ships an adapter |
| **GTK 4** | `GtkAccessible` | WAI-ARIA roles and states surfaced over AT-SPI |
| **Flutter** | `Semantics` / `SemanticsRole` | Maps to ARIA on web, to platform APIs elsewhere |
| **Qt** | `QAccessible` | Subclass `QAccessibleInterface` for custom widgets |

AccessKit is *required* rather than merely suggested for Rust GUI work because
Standard §3.1 already makes Rust the preferred language — which means
custom-drawn Rust UI is the most likely place for an accessibility gap to open
in this ecosystem, and AccessKit is the one bridge that closes it across all
three platforms from a single implementation.

**License check (§4.2):** AccessKit is Apache-2.0 — compatible with
GPL-3.0-or-later. Preserve its notices and ship the license text in
`LICENSES/` per §4.3.

## Requirements

- **Name and role on every interactive element.** An icon-only button with no
  accessible name announces as "button" and nothing else.
- **Decorative elements explicitly marked decorative**, so they are skipped
  rather than read as noise.
- **State changes announced**, not merely repainted. If a long operation
  finishes and only a spinner stops, nothing was communicated.
- **System preferences honored** — reduced motion and high contrast are read
  from the platform **independently of the §18.1 toggle**. The user already
  expressed that preference system-wide; do not make them express it twice.
- **Keyboard reachability and focus visibility** per §10.

## Sketches

### AccessKit (Rust)

Publish a node per element; the adapter translates to each platform API.

```rust
// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later
use accesskit::{Node, NodeId, Role};

fn export_button(id: NodeId, label: &str, pressed: bool) -> Node {
    let mut node = Node::new(Role::Button);
    // The accessible name — never leave this empty on an interactive element.
    node.set_label(label);
    if pressed {
        node.set_toggled(accesskit::Toggled::True);
    }
    node
}
```

Consult the AccessKit docs for the current API surface before writing real
code — it is pre-1.0 and node-building details move between releases.

### GTK 4

```c
gtk_accessible_update_property (GTK_ACCESSIBLE (button),
                                GTK_ACCESSIBLE_PROPERTY_LABEL, "Rebuild index",
                                -1);
```

Mark decoration so it is skipped:

```c
gtk_accessible_update_role (GTK_ACCESSIBLE (divider),
                            GTK_ACCESSIBLE_ROLE_PRESENTATION);
```

### Flutter

```dart
Semantics(
  label: 'Rebuild index',
  button: true,
  child: IconButton(icon: const Icon(Icons.refresh), onPressed: _rebuild),
)
```

Wrap purely decorative subtrees in `ExcludeSemantics`. Announce completion of
long operations rather than relying on a spinner stopping.

### Qt

```cpp
button->setAccessibleName(QStringLiteral("Rebuild index"));
button->setAccessibleDescription(QStringLiteral("Re-scans the source tree"));
```

Custom-painted widgets need a `QAccessibleInterface` subclass — setting a name
on a `QWidget` that paints its own children does not describe those children.

## Theming

Apply `steelbore-high-contrast` when accessible mode is on, and honor the
platform high-contrast preference independently. Void Navy `#000027` remains
the background in every variant — see §11.1.1 and the table in
[`../SKILL.md`](../SKILL.md).

Material Design components (§13) must be themed through the theme tokens, not
bare hex literals (§11.1) — which is precisely what makes swapping in the
high-contrast variant a one-line change.
