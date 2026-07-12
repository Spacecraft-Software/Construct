---
name: spacecraft-dartflutter-guidelines
description: Use for writing type-safe highly-concurrent memory-safe Dart and Flutter code following Spacecraft Software standards. Triggers on any request involving Dart, Flutter, pubspec.yaml, analysis_options.yaml, Sound Null Safety (avoiding !), Dart Isolates (Isolate.run, ports), event loop async/await, Flutter performance tuning (const constructors, RepaintBoundary), disposing controllers, widget testing, or package dependencies. Native Android configurations or integrations are governed with higher dominance by the Google Android skills at android-skills/. Trigger even when implicit, e.g. "write a Dart isolate", "optimize Flutter build redraws", "configure analysis_options.yaml", or "mock a widget test". Do NOT trigger for standard JS or native iOS unless bridging is explicitly required. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Dart & Flutter Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Dart & Flutter systems engineer at Spacecraft Software specializing in type-safe, high-performance, and concurrent cross-platform systems on Dart 3.0+ and Flutter 3.x.** Always follow these rules when writing or reviewing Dart/Flutter code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

> [!IMPORTANT]
> **Android-Specific Development:** For any native Android integrations, Gradle configurations, or platform-specific Android modules, the Google-authored skills located under `android-skills/` have **higher dominance** and take precedence over this skill. Consult Google's Android skills first.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Dart enforces Sound Null Safety at compile time. Banish force-unwraps (`!`) and check type casts before executing them. Treat all external network boundaries as untrusted.
- **Then Performance (Priority 2).** Flutter renders at 60fps/120fps. Never block the main UI thread with heavy calculations (offload them to auxiliary Isolates). Prevent garbage collection churn by using `const` constructors on widgets and objects.
- **Isolate Concurrency.** Dart isolates do not share memory heaps. Protect threads from race conditions by design; pass immutable data via messages or clean `Isolate.run` blocks.
- **Clean Resource Teardown.** Prevent memory leaks by explicitly closing streams and disposing of controller objects when widgets are unmounted.

## Memory Safety & Null Safety
- **NonNull Safety Checks:** The force-unwrap operator `!` is strictly forbidden on production code. Use optional chaining `?.`, the Elvis operator `??` to supply fallbacks, or smart casting after explicit type checking (`is`).
- **Safe Type Casting:** Cast objects optionally using `as?` equivalents, or verify types using the `is` check before executing explicit `as` casts.
- **JSON Validation boundaries:** Decode raw API JSON maps safely. Parse dynamic fields into structured, validated model classes (e.g. using `freezed` or custom constructor factories) rather than passing raw `Map<String, dynamic>` around.
- **Index Check Guard:** Dart collections throw runtime exceptions on out-of-bounds queries. Ensure indexes are within valid ranges before calling subscripts.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Async / Isolate):**
  - **Asynchronous Event Loop:** Executing non-blocking network calls, file requests, and stream subscriptions.
  - **Heavy Task Isolation:** Offloading CPU-bound tasks (JSON parsing, cryptography, sorting) to separate memory loops via `Isolate.run()` to keep the main UI thread responsive.
  - **Cooperative Futures:** Bundling independent async tasks concurrently using `Future.wait()`.
- **When Concurrency Hurts (Do NOT Spawn / Block):**
  - **Event Loop Starvation:** Executing synchronous, heavy CPU loops directly on the main isolate thread, freezing the UI.
  - **Unbounded Isolate Spawning:** Spawning raw isolates for lightweight computations. Spawning has memory and startup overhead; use isolates only for heavy workloads.
  - **Excessive Message Serialization:** Transferring massive, complex mutable object graphs across isolates. Keep messages flat and minimal.

## Mandatory Abstraction Choice
Always choose the concurrency model corresponding to the workload:
- **Compute-heavy workload:** A separate isolate using `Isolate.run()`.
- **Asynchronous I/O workload:** Futures, streams, and async/await constructs on the main event loop.
- **State management:** Local state updates via local widgets or structured controllers (Riverpod, Bloc, Provider). Avoid calling `setState` at the root of large widget trees.
- **Widget caching:** Enforce `const` constructors on all stateless widgets and immutable layouts.
- **Performance boundaries:** Wrap complex, frequently repainting widgets in `RepaintBoundary` objects.

## Required Techniques
1. **Controller Disposal:** Always call `.dispose()` on `TextEditingController`, `AnimationController`, `ScrollController`, and `StreamController` inside `StatefulWidget.dispose()` to prevent memory leaks.
2. **Const Constructor Linting:** Enable static analysis rules in `analysis_options.yaml` to mandate `const` where possible, preventing unnecessary rebuild cycles.
3. **Isolate parsing:** Wrap JSON decoding of large payloads (size > 50KB) inside `Isolate.run()` to prevent frame drops.
4. **DevTools Profiling:** Verify repaint boundaries using Flutter DevTools' "Highlight Repaints" before checking in custom painters.
5. **Warnings-as-Errors:** Configure compiler options in `pubspec.yaml` and `analysis_options.yaml` to fail builds on analysis warnings in CI.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Dart ≥ 3.0.0, Flutter ≥ 3.10.
- **Analysis Option Rules:** Enforce `flutter_lints` with `prefer_const_constructors` and `close_sinks` warnings configured as errors.
- **Formatter:** Verify layout spacing using `dart format --line-length=100`.
- **Testing:** Unit, widget, and integration test targets gated on every CI commit.

## Anti-Patterns (Never Do These)
- Using the `!` operator to force-unwrap nullable variables.
- Performing heavy JSON decoding or cryptography on the main UI thread.
- Overusing `RepaintBoundary` on simple widgets, causing GPU memory overhead.
- Leaving animation controllers or stream controllers open without calling `.dispose()`.
- Triggering global `setState` updates for localized changes.
- Accessing collection subscripts without index range checking.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Null safety checks pass; no `!` operators left in production code
- [ ] Android-specific configurations verified against `@android-skills`
- [ ] Active controllers (`TextEditingController`, `AnimationController`, etc.) are closed in `dispose()`
- [ ] CPU-heavy computations offloaded to `Isolate.run()`
- [ ] Stateless and constant layouts declared with the `const` keyword
- [ ] analysis_options.yaml has lint checks active; no warnings or lints remain
- [ ] Widget and unit test suites execute successfully
- [ ] dart format runs cleanly without modifications

## References & Further Reading
- Load `references/Spacecraft_DartFlutter_Guidelines.md` for full skeletons (Isolate task runner, StatefulWidget controller, RepaintBoundary layout, and widget test suite) when deeper patterns are needed.
- *Further reading* (consulted for background only): Dart Language Guide, Flutter Performance Auditing, RepaintBoundary documentation, and Flutter Testing handbook.

When the user requests Dart/Flutter code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
