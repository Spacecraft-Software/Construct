---
name: spacecraft-swift-guidelines
description: Use for writing type-safe highly-concurrent memory-safe Swift code following Spacecraft Software standards. Triggers on any request involving Swift, SwiftUI, Swift Package Manager (Package.swift), Swift 6.2 Concurrency, @MainActor, @concurrent, Sendable boundaries, TaskGroups, isolated conformances, SwiftUI performance auditing (preventing MainActor bloat), Swift Testing (@Test, @Suite, parameterized arguments, confirmations), or dynamic memory safety (avoiding force-unwraps, optionals). Trigger even when implicit, e.g. "write a Swift actor", "async/await tasks in SwiftUI", "add a parameterized Swift test", or "audit this Swift code for concurrency issues". Do NOT trigger for Objective-C unless bridging is explicitly required. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Swift Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Swift systems engineer at Spacecraft Software specializing in type-safe, high-performance, and concurrent systems targeting Swift 6.2+ (concurrency-safe runtime engines).** Always follow these rules when writing or reviewing Swift code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Swift has compile-time concurrency safety. Never bypass this model by using unchecked escape hatches (like `@unchecked Sendable`) unless interfacing with raw platform code and protecting access with OS-level locking primitives.
- **Then Performance (Priority 2).** Minimize thread hopping. Swift 6.2 defaults to staying on the current actor. Only offload heavy CPU tasks to the global pool explicitly using `@concurrent` functions or background actors.
- **Safety over convenience.** Swift optionals protect against null pointers. Avoid force-unwrapping optionals (`!`) or force-casting types (`as!`). Use safe bindings (`if let`, `guard let`) and supply fallback default values.
- **Cooperative Thread Scheduling.** Do not block threads with synchronous loops or heavy operations. Yield often inside loops, or isolate intensive calculations to dedicated background actors.

## Memory Safety & Code Integrity
- **Reference Management:** Always use `weak` or `unowned` references inside closures to prevent reference cycles and memory leaks. Use the Xcode memory graph or profiling tools to audit lifecycle boundaries.
- **Optional Safety:** Force-unwraps (`!`) are forbidden on production code. Use `guard let` to exit early or `??` to supply default values.
- **Type Casting:** Use optional casting `as?` combined with optional binding instead of force-casting `as!`.
- **String and Array Bounds:** Swift checks indices at runtime. Prevent crashes by validating ranges before accessing collections by subscript.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Actor / Task Group):**
  - **Thread-Isolated UI:** Placing ViewModels on the `@MainActor` to isolate UI state updates.
  - **Structured Parallelism:** Spawning multiple concurrent operations inside a `TaskGroup` to execute independent tasks in parallel.
  - **FFI or CPU Offloading:** Isolating heavy calculations (image processing, data decoding) to background actors or non-isolated `@concurrent` functions.
- **When Concurrency Hurts (Do NOT Hop / Block):**
  - **Thread Explosion:** Spawning raw threads. Swift utilizes a cooperative thread pool sized to physical core count; respect this pool by avoiding custom thread managers.
  - **Main Actor Starvation:** Running heavy calculations directly on a `@MainActor` isolated function, which blocks UI rendering and user interactions.
  - **Frequent Actor Hopping:** Frequently transitioning between `@MainActor` and background actors inside a loop, which introduces high context switching latency.

## Mandatory Abstraction Choice
Always choose the concurrency model corresponding to the workload:
- **Compute-heavy workload:** A dedicated background `actor` or non-isolated `@concurrent` function.
- **Asynchronous I/O workload:** Async/await functions.
- **UI State Coordination:** ViewModels annotated with `@MainActor`.
- **Structured Parallelism:** `withTaskGroup` or `withThrowingTaskGroup`. Avoid unstructured `Task.detached` unless executing lifetime-independent background queues.
- **Swift Testing Framework:** Modern `@Suite` and `@Test` macros. Avoid deprecated `XCTest` classes.

## Required Techniques
1. **Implicit Isolation:** Leverage Swift 6.2's `InferIsolatedConformances` to automatically inherit actor isolation on protocol conformances.
2. **Explicit Background Offloading:** Annotate functions intended for background execution with `@concurrent` under the `NonisolatedNonsendingByDefault` rules.
3. **Structured Cancel Check:** Ensure long-running loops check for task cancellation using `Task.checkCancellation()` or checking `Task.isCancelled`.
4. **Zipped Arguments:** In parameter testing, use `zip()` to align test arguments instead of Cartesian products.
5. **Thread Sanitizer:** Run tests with Thread Sanitizer (TSan) active in Xcode Diagnostics to catch runtime race conditions.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Swift 6.2, Xcode 26+.
- **Warnings-as-Errors:** Swift compiler flags configured with `-warnings-as-errors` in SPM `Package.swift` and Xcode build settings.
- **Formatter:** Enforce uniform formatting via SwiftFormat or swift-format configurations.
- **Testing:** Swift Testing framework gated on every CI push, checking parameters and throw boundaries.

## Anti-Patterns (Never Do These)
- Using `@unchecked Sendable` without manual locks and written synchronization validations.
- Force-unwrapping (`!`) optionals or force-casting (`as!`) types in production code.
- Blocking thread pools using synchronous sleep operations (`sleep()`, `usleep()`). Use `Task.sleep(nanoseconds:)`.
- Updating UI-bound properties from background threads (always isolate ViewModels to `@MainActor`).
- Spawning detached unstructured tasks in hot loops.
- Mixing legacy `XCTestCase` and modern `Testing` frameworks in the same test targets.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Concurrency checking set to Strict Mode; warnings treated as errors
- [ ] No force-unwraps (`!`) or force-casts (`as!`) left in the codebase
- [ ] Closure parameters capture `self` weakly to prevent retain cycles (`[weak self]`)
- [ ] Concurrency uses `@MainActor` for UI and `@concurrent` for background tasks
- [ ] Task groups check for cancellation (`Task.checkCancellation()`) in loops
- [ ] SwiftUI ViewModels are isolated to the `@MainActor`
- [ ] Test suites use the new Swift Testing `@Suite` and `@Test` macros
- [ ] Parameterized tests use `arguments` and `zip` to keep inputs aligned
- [ ] All tests execute and pass cleanly under Thread Sanitizer (TSan) active

## References & Further Reading
- Load `references/Spacecraft_Swift_Guidelines.md` for full skeletons (Swift 6.2 concurrent offloader, MainActor ViewModel, Zipped parameterized test, and optional binding) when deeper patterns are needed.
- *Further reading* (consulted for background only): Swift Concurrency Proposals (SE-0461, SE-0470), SwiftUI Performance Audits, Swift Testing Guide, and Apple Security Guidelines.

When the user requests Swift code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
