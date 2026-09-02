# Spacecraft GTK 4 Guidelines (C) — Full Reference

**Version:** 1.0
**Date:** 2026-08-05
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Model-agnostic — the current Claude 5 family (Opus, Fable, Sonnet), Grok, and any comparably capable reasoning model

This document covers the **fallback** GTK 4 implementation path: C with GObject. It is a full peer to `Spacecraft_GTK_Rust_Guidelines.md`, not a stub — existing GTK code, every GNOME platform library, and most examples in the wild are C, and that code still has to be correct and hardened.

It adds the GObject-specific layer on top of `spacecraft-clang-guidelines`, which retains higher dominance on C hardening, bounded flow, explicit `<stdint.h>` types, sanitizers, and build configuration. **Do not duplicate those rules here — load that skill.**

---

## 1. Choosing C: The §3.1 Exemption

Standard §3.1 states that memory-safe languages are always preferred, and that "if an MSL alternative exists, it must be chosen unless a documented technical exemption is filed." For GTK 4, `gtk-rs` **is** that alternative. Writing new GTK code in C is therefore an exception that must be justified in writing, not a default.

Grounds that ordinarily justify the exemption:

- **Existing C codebase.** Extending an established C GTK application; a partial rewrite would be riskier than the C it replaces.
- **GNOME platform library.** Authoring a library that must expose a stable C ABI and GObject-Introspection typelib for consumption by Python, JavaScript, and Vala.
- **Binding gap.** A required GNOME library has no maintained Rust binding and writing one is out of scope.
- **Toolchain constraint.** A target platform where the Rust toolchain is unavailable or unqualified.

"Familiarity", "it is only a small utility", and "the examples are in C" are **not** grounds.

Where the exemption is filed, §3.1's mandatory mitigations attach:

```cmake
# ASLR: position-independent executable, so the loader can randomize the layout.
add_compile_options(-fPIE)
add_link_options(-pie)

# CFI: control-flow integrity wherever the toolchain supports it.
if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    add_compile_options(-fsanitize=cfi -fvisibility=hidden -flto)
    add_link_options(-fsanitize=cfi -flto)
elseif(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    add_compile_options(-fcf-protection=full)
endif()
```

Record the exemption in the project's `README.md` alongside the §5.2 posture section, naming which ground applies.

---

## 2. Reference Counting: Floating References

Every `GtkWidget` derives from `GInitiallyUnowned` and is created with a **floating** reference — an unowned reference that the first container to take the widget "sinks" into a real one. This is the single most misunderstood part of GObject, and it produces a leak that looks like the opposite of a leak.

```c
#include <gtk/gtk.h>

void demonstrate_floating_references(GtkWidget *box)
{
    /* Freshly constructed: refcount 1, FLOATING. */
    GtkWidget *button = gtk_button_new_with_label ("Rebuild index");

    /* Parenting sinks the floating reference. The box now owns it.
       Do NOT unref after this — the box holds the only reference. */
    gtk_box_append (GTK_BOX (box), button);

    /* WRONG, and the classic leak:

         GtkWidget *orphan = gtk_button_new_with_label ("Never parented");
         g_object_unref (orphan);        // does NOT free it

       g_object_unref on a still-floating reference does not release the
       object, because nobody ever owned it. The memory leaks. */

    /* CORRECT when taking ownership outside a container: sink explicitly. */
    g_autoptr(GtkWidget) detached = g_object_ref_sink (gtk_label_new ("Detached"));
    /* `detached` is now a normal owned reference; g_autoptr releases it at
       scope exit. On an already-sunken object, g_object_ref_sink is
       equivalent to g_object_ref, so this is always safe. */
}
```

**Rule:** immediately after `g_object_new` on a type deriving from `GInitiallyUnowned`, either parent it or `g_object_ref_sink` it. Never leave a widget in the floating state past the end of the function that created it.

---

## 3. Automatic Cleanup: `g_autoptr` and `g_clear_object`

Manual `g_object_unref` on every exit path is where use-after-free and double-free enter a C GTK codebase. GLib's cleanup attributes remove the class of bug entirely and should be used unconditionally.

```c
#include <gtk/gtk.h>
#include <gio/gio.h>

gboolean load_capture (const char *path, GError **error)
{
    /* Released automatically at every scope exit, including early returns
       and the error path. No goto-cleanup ladder needed. */
    g_autoptr(GFile) file = g_file_new_for_path (path);
    g_autoptr(GBytes) bytes = NULL;
    g_autofree char *summary = NULL;

    bytes = g_file_load_bytes (file, NULL, NULL, error);
    if (bytes == NULL)
        return FALSE;   /* `file` released here automatically */

    summary = g_strdup_printf ("Loaded %" G_GSIZE_FORMAT " bytes",
                               g_bytes_get_size (bytes));
    g_message ("%s", summary);

    return TRUE;        /* all three released here */
}

/* For struct members and long-lived pointers, g_clear_object both unrefs
   and NULLs, so a stale pointer cannot be dereferenced afterwards. */
static void
telemetry_window_dispose (GObject *object)
{
    TelemetryWindow *self = TELEMETRY_WINDOW (object);

    g_clear_object (&self->settings);
    g_clear_object (&self->capture_model);
    g_clear_pointer (&self->capture_path, g_free);

    G_OBJECT_CLASS (telemetry_window_parent_class)->dispose (object);
}
```

---

## 4. Weak References: `GWeakRef`, Not `g_object_weak_ref`

GObject offers three weak-reference mechanisms and only one is thread-safe. This matters whenever the final `g_object_unref` might happen on a thread other than the observer's.

| API | Thread-safe | Use |
| :--- | :--- | :--- |
| `g_object_weak_ref` | **No** | Legacy; avoid in any threaded program |
| `g_object_add_weak_pointer` | **No** | Legacy; avoid in any threaded program |
| `GWeakRef` | **Yes** | The correct choice — upgrade is atomic with respect to invalidation |

```c
#include <glib-object.h>

typedef struct {
    GWeakRef window;      /* does not keep the window alive */
    guint    source_id;
} PollContext;

static gboolean
poll_telemetry (gpointer user_data)
{
    PollContext *ctx = user_data;

    /* Atomic weak-to-strong upgrade. Returns NULL if the window is gone;
       there is no window in which the object is destroyed between the
       liveness check and the ref. */
    g_autoptr(TelemetryWindow) window = g_weak_ref_get (&ctx->window);
    if (window == NULL)
        return G_SOURCE_REMOVE;   /* window died; stop polling */

    telemetry_window_refresh (window);
    return G_SOURCE_CONTINUE;
}

static void
poll_context_free (gpointer data)
{
    PollContext *ctx = data;
    g_weak_ref_clear (&ctx->window);
    g_free (ctx);
}

void
start_polling (TelemetryWindow *window)
{
    PollContext *ctx = g_new0 (PollContext, 1);
    g_weak_ref_init (&ctx->window, window);
    ctx->source_id = g_timeout_add_full (G_PRIORITY_DEFAULT, 1000,
                                         poll_telemetry, ctx, poll_context_free);
}
```

---

## 5. Signal Handlers: Lifetime and Disconnection

A signal handler keeps its closure — and everything the closure references — alive for the lifetime of the **emitting** object. Connecting a widget's method to a long-lived `GSettings` or application action, then destroying the widget, leaves a handler that fires against freed memory.

```c
/* PREFERRED: g_signal_connect_object ties the connection to the lifetime of
   `self`. When `self` is finalized the handler is disconnected automatically,
   and the handler is not invoked during `self`'s disposal. */
g_signal_connect_object (self->settings, "changed::refresh-interval",
                         G_CALLBACK (on_interval_changed), self,
                         G_CONNECT_DEFAULT);

/* When plain g_signal_connect is unavoidable, retain the id and disconnect. */
self->changed_id = g_signal_connect (self->settings, "changed::theme",
                                     G_CALLBACK (on_theme_changed), self);

static void
telemetry_window_dispose (GObject *object)
{
    TelemetryWindow *self = TELEMETRY_WINDOW (object);

    if (self->changed_id != 0 && self->settings != NULL) {
        g_clear_signal_handler (&self->changed_id, self->settings);
    }
    g_clear_object (&self->settings);

    G_OBJECT_CLASS (telemetry_window_parent_class)->dispose (object);
}
```

`g_clear_signal_handler` both disconnects and zeroes the id, so a second `dispose` pass (GObject permits `dispose` to run more than once) cannot double-disconnect.

---

## 6. Threading: The Main-Thread Rule in C

C has no `!Send` to enforce it, so the main-thread restriction is entirely on the author. Every GTK and GDK call belongs on the thread that iterates the main context. Worker threads own plain data and hand results back through a main-context source.

```c
#include <gtk/gtk.h>

typedef struct {
    GWeakRef  window;
    GPtrArray *packets;   /* plain data — no widgets cross the boundary */
} LoadResult;

/* Runs on the MAIN thread: touching widgets here is legal. */
static gboolean
apply_result (gpointer user_data)
{
    LoadResult *result = user_data;
    g_autoptr(TelemetryWindow) window = g_weak_ref_get (&result->window);

    if (window != NULL)
        telemetry_window_set_packets (window, result->packets);

    return G_SOURCE_REMOVE;
}

/* Runs on a WORKER thread: not one GTK call appears in this function. */
static void
load_capture_thread (GTask        *task,
                     gpointer      source_object,
                     gpointer      task_data,
                     GCancellable *cancellable)
{
    const char *path = task_data;
    GPtrArray *packets = parse_capture (path);   /* pure computation */

    LoadResult *result = g_new0 (LoadResult, 1);
    g_weak_ref_init (&result->window, source_object);
    result->packets = packets;

    /* Marshal back to the main context. g_idle_add_full runs `apply_result`
       on the thread iterating the default main context — the main thread. */
    g_idle_add_full (G_PRIORITY_DEFAULT_IDLE, apply_result, result, load_result_free);
    g_task_return_boolean (task, TRUE);
}
```

**Prefer `GTask` over raw `GThread`.** `GTask` integrates with `GCancellable`, carries errors, and its thread pool avoids per-operation thread creation — which, per §3.2, is where naive concurrency stops advancing performance and starts costing it. Per §3.2 the trade-off must be documented: state in a comment why the work was moved off the main loop and why the chosen structure is the simplest safe one.

---

## 7. Accessibility in C (Standard §18)

Same contract as the Rust path; see `spacecraft-accessibility-support` for the §18 rules themselves.

```c
gtk_accessible_update_property (GTK_ACCESSIBLE (self->refresh_button),
                                GTK_ACCESSIBLE_PROPERTY_LABEL,
                                "Rebuild telemetry index",
                                GTK_ACCESSIBLE_PROPERTY_DESCRIPTION,
                                "Re-scans the capture directory and reloads all packets",
                                -1);

/* Decoration is skipped rather than read as noise. */
gtk_accessible_update_role (GTK_ACCESSIBLE (self->divider),
                            GTK_ACCESSIBLE_ROLE_PRESENTATION);

/* State changes are announced, not merely repainted. */
gtk_accessible_update_state (GTK_ACCESSIBLE (self->refresh_button),
                             GTK_ACCESSIBLE_STATE_BUSY, TRUE,
                             -1);
```

A `GtkDrawingArea` painting its own controls is invisible to assistive technology unless the application publishes nodes for them — implement `GtkAccessible` on a widget subclass exposing each painted control. Verify with Orca (§18.4).

---

## 8. Hardened Build Configuration

This section is the GTK-specific delta only. The full C hardening flag set, sanitizer policy, `clang-tidy` configuration, and CERT C / MISRA C rules live in `spacecraft-clang-guidelines` — load it.

```meson
# meson.build
project('telemetry-dashboard', 'c',
  version: '0.1.0',
  default_options: [
    'c_std=c11',
    'warning_level=3',
    'werror=true',
    'b_pie=true',          # ASLR (§3.1 mandatory mitigation)
  ])

gtk_dep = dependency('gtk4', version: '>= 4.10')
adw_dep = dependency('libadwaita-1', version: '>= 1.5')

cc = meson.get_compiler('c')

# §3.2 — every applied flag is noted, and every disabled flag with its reason.
hardening = cc.get_supported_arguments([
  '-fstack-protector-strong',   # applied: stack smashing detection
  '-D_FORTIFY_SOURCE=3',        # applied: fortified libc calls
  '-fcf-protection=full',       # applied: CFI (§3.1) on GCC
  '-fno-omit-frame-pointer',    # applied: usable backtraces in crash reports
])
add_project_arguments(hardening, language: 'c')

# DISABLED: -flto.
#   Reason (§3.2.1): under NixOS the GCC LTO plugin is not on the default
#   linker search path, so -flto fails to link unless paired with
#   -fuse-ld=mold (preferred) or -fuse-ld=bfd. Stability (P1) outranks
#   Performance (P2), so LTO stays off until the linker is pinned.
#
# DISABLED: -march=native.
#   Reason: distribution and Flatpak builds run on unknown hardware; a
#   native-tuned binary would fault on older targets.

if get_option('buildtype') == 'debug'
  add_project_arguments(['-fsanitize=address,undefined'], language: 'c')
  add_project_link_arguments(['-fsanitize=address,undefined'], language: 'c')
endif
```

Run the GLib test suites under the sanitizers, and set `G_DEBUG=fatal-warnings` in CI so a `g_warning` from GTK fails the build rather than scrolling past.

```bash
G_DEBUG=fatal-warnings ASAN_OPTIONS=detect_leaks=1 xvfb-run -a meson test -C build
gtk4-builder-tool validate data/window.ui
```

---

## 9. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Unparented widget** | Steady leak; `g_object_unref` appears to do nothing | Parent it, or take ownership with `g_object_ref_sink`. |
| **Unref after parenting** | Crash on window close; refcount underflow | The container owns it — do not unref after `gtk_box_append`. |
| **`g_object_weak_ref` across threads** | Race between destruction and callback; use-after-free | Use `GWeakRef` — its upgrade is atomic with respect to invalidation. |
| **Handler on a long-lived object** | Callback fires against freed widget memory | `g_signal_connect_object`, or retain the id and `g_clear_signal_handler`. |
| **GTK call from a worker thread** | Sporadic corruption, X/Wayland protocol errors, no compiler complaint | Marshal to the main context with `g_idle_add_full`; workers touch data only. |
| **Manual unref ladder** | Leak on an early-return or error path | `g_autoptr` / `g_autofree` / `g_clear_object` on every owned pointer. |
| **`dispose` running twice** | Double-disconnect or double-unref | Use the `g_clear_*` family, which zeroes as it releases. |
| **`g_warning` ignored in CI** | Latent GTK misuse ships | `G_DEBUG=fatal-warnings` in the test environment. |
| **LTO on NixOS** | Link failure: LTO plugin not found | §3.2.1 — pair `-flto` with `-fuse-ld=mold`, or document it disabled. |
| **Raw `int` sizes** | Ambiguous width across targets | `spacecraft-clang-guidelines`: explicit `<stdint.h>` types. |

---

## 10. Code Review Compliance Gate

Before merging C GTK code, verify:
1. The §3.1 technical exemption for choosing C is filed, and ASLR (`-fPIE`/`-pie`) plus CFI are enabled.
2. Every widget created is parented or `g_object_ref_sink`-ed before its creating function returns.
3. No `g_object_unref` follows a successful parenting call.
4. Every owned pointer uses `g_autoptr`/`g_autofree`/`g_clear_object`; no manual unref ladder remains.
5. Weak observation uses `GWeakRef`, never `g_object_weak_ref` or `g_object_add_weak_pointer`.
6. Handlers on longer-lived objects use `g_signal_connect_object`, or are cleared in `dispose`.
7. No GTK or GDK call appears on a worker thread; results marshal back through the main context.
8. The §3.2 concurrency trade-off is documented where work was moved off the main loop.
9. Every interactive widget has an accessible name and role; custom-drawn surfaces publish nodes (§18.3).
10. Colors resolve through `steelbore` CSS tokens; no hex literal appears in C or CSS (§11.1).
11. Applied and disabled compiler flags are both documented, with the §3.2.1 NixOS LTO caveat honoured.
12. Tests run with `G_DEBUG=fatal-warnings` and the sanitizers enabled, and pass.
13. Everything in the `spacecraft-clang-guidelines` compliance gate also passes — it governs the C, this list governs the GObject layer.
