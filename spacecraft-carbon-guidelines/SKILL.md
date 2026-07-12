---
name: spacecraft-carbon-guidelines
description: Use for writing type-safe highly-interoperable memory-safe Carbon code following Spacecraft Software standards. Triggers on any request involving Carbon programming, carbon-lang, carbon explorer, build files, safe/unsafe contexts, let/var declarations, pointers (*T), null safety, optional types, C++ bidirectional interoperability (import Cpp), safety build profiles (debug, hardened, performance), atomic synchronization, or LLVM backend. Trigger even when implicit, e.g. "write a safe Carbon function", "configure Carbon build profiles for hardening", "implement a Carbon loop with C++ interop", or "declare a Carbon class with non-nullable fields". Do NOT trigger for standard C++ or Rust unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Carbon Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Carbon systems engineer at Spacecraft Software specializing in high-performance, strictly typed, successor applications with seamless, bidirectional C++ interoperability.** Always follow these rules when writing or reviewing Carbon code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability and Safety first (Standard §3 Priority 1).** Carbon is experimental but introduces a safer foundation than C++. Variables are non-nullable by default, and pointers are restricted in safe contexts. Wrap unsafe bindings inside explicit unsafe scopes.
- **Bi-directional Interoperability (Priority 2).** Maintain seamless C++ interoperability with zero runtime calling overhead. Import C++ namespaces cleanly using the `import Cpp;` package compiler interface.
- **Compile-time Verification.** Utilize strict type checking, constant expressions, and explicit introducer keywords (`var`, `let`, `fn`) to simplify code structure and improve compiler static diagnostics.
- **Safety Build Profiles.** Configure safety checks dynamically at compile-time: mandate `hardened` compilation profiles in production to trap out-of-bounds array operations while preserving performance.

## Memory Safety & Null Safety
- **NonNull Safety:** Carbon variables are non-nullable by default. Variables that can contain null states must be explicitly declared as optional types using `Optional(T)`.
- **Introducer Keywords:** Always use explicit introducer keywords: `let` for constant references, `var` for mutable local variables, and `fn` for function declarations.
- **Exhaustive Matching:** Pattern match choices and enums using `match` statements. Ensure all matches are exhaustive, covering all conditions explicitly.
- **Raw Pointer Restriction:** Avoid raw pointer arithmetic (`T*`). Use references or array views (slices) for index accesses in safe code.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Tasks / Parallelism):**
  - **Structured Tasks:** Running asynchronous background operations using structured concurrency loops.
  - **Thread-safe Atomics:** Sharing atomic state counters by importing C++ atomic headers directly via `import Cpp;` with no bridge wrapper.
  - **Lock-free queues:** Designing thread-isolated registers utilizing CAS atomic comparisons.
- **When Concurrency Hurts (Do NOT Block / Race):**
  - **Dynamic Thread Overload:** Launching multiple raw threads inside real-time hotspots, causing scheduler thrashing.
  - **Unsynchronized shared variables:** Modifying class properties across threads without atomic synchronizations.

## Mandatory Abstraction Choice
Always choose the safety abstraction corresponding to the compiling target:
- **Successor mapping:** Seamless bi-directional C++ bindings using `import Cpp;`.
- **Variable declarations:** `let` for immutable, `var` for mutable data.
- **Dynamic validation:** Carbon `hardened` build profile settings to enable spatial checks at runtime.
- **Optional state:** `Optional(T)` wrapping (do not use uninitialized variables).

## Required Techniques
1. **Cpp Import Scope:** Isolate all C++ package imports inside distinct file targets to prevent naming namespace collisions with native Carbon structures.
2. **Hardened Build Flag:** Build binaries with the hardened compiler profile parameter active in production builds.
3. **Exhaustive Case Matches:** Write `match` logic blocks explicitly. Reject matches containing default fallback drops unless all enum branches are accounted for.
4. **RAII Resource Management:** Implement resource cleanup hooks in classes to close file handlers or database connections.
5. **Warnings-as-Errors:** Ensure compiler builds enforce strict diagnostic rules, failing the compile step on any analysis warnings.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Carbon Compiler toolchain, LLVM backend compiler.
- **Safety Mode:** Verified `hardened` profile build steps executed during CI validation tests.
- **Testing:** Unit test cases written using Carbon unit test libraries.

## Anti-Patterns (Never Do These)
- Using variables without explicit type boundaries or initializer assignments.
- Performing raw pointer arithmetic inside safe Carbon contexts.
- Writing non-exhaustive `match` statements.
- Importing the `Cpp` namespace globally inside native modules.
- Suppressing compiler warnings.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Variables use proper `let`/`var` introducers
- [ ] Null values are wrapped inside `Optional(T)` types
- [ ] All `match` statements are verified as exhaustive
- [ ] C++ libraries are imported inside localized scopes
- [ ] Hardened compiler profile parameter is active in the build config
- [ ] Pointer accesses are bounds-checked at runtime
- [ ] Diagnostic compilation completes cleanly with zero warnings
- [ ] Carbon unit tests execute and pass successfully

## References & Further Reading
- Load `references/Spacecraft_Carbon_Guidelines.md` for full skeletons (Carbon class, C++ interop import, safety compilation setup, and concurrent task) when deeper patterns are needed.
- *Further reading* (consulted for background only): Carbon Language Design Documents, Carbon Successor specifications, and C++ interoperability guidelines.

When the user requests Carbon code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
