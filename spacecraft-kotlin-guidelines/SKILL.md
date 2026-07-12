---
name: spacecraft-kotlin-guidelines
description: Use for writing type-safe highly-concurrent memory-safe Kotlin code following Spacecraft Software standards. Triggers on any request involving Kotlin, JVM, .kt/.kts files, gradle.kts, Kotlin Coroutines, structured concurrency (CoroutineScope, supervisorScope), Dispatchers (Default, IO, Main), null-safety (null checks, avoiding !!), Arrow (Either), or Exposed ORM. Android-specific development (such as Jetpack Compose, wear compose, or edge-to-edge) is governed with higher dominance by the Google Android skills at android-skills/. Trigger even when implicit, e.g. "write a Kotlin coroutine", "configure build.gradle.kts", "handle exceptions in Coroutines", or "optimize collection flows". Do NOT trigger for Java unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Kotlin Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Kotlin systems engineer at Spacecraft Software specializing in type-safe, high-performance, and concurrent systems targeting JVM and Native runtimes.** Always follow these rules when writing or reviewing Kotlin code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

> [!IMPORTANT]
> **Android-Specific Development:** For any native Android development (such as Jetpack Compose, AppFunctions, edge-to-edge UI, wear-compose-m3, R8 tuning, or android-intent-security), the Google-authored skills located under `android-skills/` have **higher dominance** and take precedence over this skill. Consult Google's Android skills first.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Kotlin features robust compile-time null safety. Banish loose assertions (`!!`) and treat all external boundaries as nullable or unvalidated until successfully parsed or checked.
- **Then Performance (Priority 2).** Minimize runtime overhead by avoiding redundant object allocations in loops (use `inline` classes/functions and sequences for large collections) and preventing coroutine thread starvation.
- **Structured Concurrency (Standard §3.2).** Always tie coroutine lifecycles to structured parents. Avoid orphan coroutines, unmanaged background processes, and unhandled context cancellation bubbles.
- **Pure-first, framework isolated.** Write logic in pure Kotlin classes. Keep database libraries (Exposed ORM) or serialization engines isolated within distinct package boundaries.

## Memory Safety & Null Safety
- **Null Safety Enforcement:** The double-bang force-unwrap operator (`!!`) is strictly banned. Use optional chaining `?.`, the Elvis operator `?:` to supply fallbacks, or smart casting after explicit null checks.
- **Argument Checks:** Validate procedure pre-conditions using standard validation contracts: `requireNotNull`, `require(condition)`, or `check(condition)`.
- **Reference Management:** In long-lived callback systems, avoid holding strong references to short-lived objects (such as contexts or listeners) to prevent memory leaks.
- **Resource Lifetime:** Ensure files, streams, and database connections are closed cleanly. Use `.use { ... }` block wrappers on closable objects to guarantee auto-release.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Coroutine / Flow):**
  - **Asynchronous Non-blocking I/O:** Executing network calls, file reading/writing, or database transactions via suspending functions.
  - **Thread Offloading:** Executing blocking operations on `Dispatchers.IO` ( elastic thread pool) and CPU-bound math on `Dispatchers.Default` (core-count bounded thread pool).
  - **Isolated Concurrency:** Using `supervisorScope` to run sibling tasks concurrently, ensuring that the failure of one task does not cancel its siblings.
- **When Concurrency Hurts (Do NOT Block / Leak):**
  - **Blocking Main Thread:** Executing synchronous, blocking calls directly on the UI dispatcher (`Dispatchers.Main`), causing UI freezing.
  - **Orphaned Contexts:** Launching coroutines via `GlobalScope` or unstructured `CoroutineScope(Job())` without cancellation propagation, which leaks background memory.
  - **Mutable Shared State races:** Modifying unsynchronized variables across threads. Use `AtomicRef`, mutexes (`Mutex` from `kotlinx.coroutines.sync`), or Flows/channels.

## Mandatory Abstraction Choice
Always choose the concurrency model corresponding to the workload:
- **Compute-heavy workload:** Suspending calls offloaded using `withContext(Dispatchers.Default)`.
- **I/O or blocking workload:** Suspending calls offloaded using `withContext(Dispatchers.IO)`.
- **UI State Coordination:** Coroutines bound to `Dispatchers.Main.immediate`.
- **Sibling Failure Isolation:** Structured `supervisorScope`. Use default `coroutineScope` only when an "all-for-one" rollback policy is desired.
- **Testing:** Inject dispatchers via constructor parameters; use `StandardTestDispatcher` and `runTest` from `kotlinx-coroutines-test` for unit testing.

## Required Techniques
1. **Dispatcher Injection:** Never hardcode `Dispatchers.IO` or `Dispatchers.Default` inside class methods. Always pass a dispatcher provider interface or pass dispatchers via constructor parameters to allow test mocks.
2. **Collection Sequences:** Use `asSequence()` when chaining operators on large collections (size > 1000) to process elements lazily and avoid intermediate list allocations.
3. **Exposed Transaction Boundaries:** Wrap all database statements inside `transaction` blocks, managing database exceptions via custom mapper results.
4. **Functional Errors:** Model business exceptions functionally using `Either` from the `Arrow` library (e.g. `Either<Failure, Success>`) instead of throwing exceptions.
5. **Warnings-as-Errors:** Configure compiler flags `-Werror` inside `build.gradle.kts` files to fail builds on warnings in CI.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Kotlin ≥ 1.9.0, Kotlin Coroutines ≥ 1.7.0, Gradle ≥ 8.0.
- **Warnings-as-Errors:** Enforced compiler options `allWarningsAsErrors = true` inside `build.gradle.kts`.
- **Formatter:** Verify syntax using `ktlint` or `detekt` rules.
- **Testing:** JUnit 5 test suites combined with `MockK` for object mocking.

## Anti-Patterns (Never Do These)
- Using the `!!` operator to assert null values.
- Hardcoding `Dispatchers.IO` inside business logic layers (always inject them).
- Blocking threads inside `Dispatchers.Default` or `Dispatchers.Main`.
- Using `GlobalScope` or unmanaged coroutine scopes.
- Catching `CancellationException` without re-throwing it, which breaks coroutine cancellation.
- Hardcoding database transactions inside view controllers or API handlers.

## Pre-Commit Checklist (Verify Every Time)
- [ ] No `!!` operators left in production source code files
- [ ] Android-specific tasks deferred to `@android-skills`
- [ ] Dispatchers are injected via constructor arguments
- [ ] Long-running suspending loops check for cancellation (`ensureActive()` or `yield()`)
- [ ] Async scopes use `supervisorScope` if child failure isolation is required
- [ ] Closeable resources are managed via the `.use` block wrapper
- [ ] Gradle build passes with `allWarningsAsErrors = true` activated
- [ ] Unit tests execute successfully under `runTest` wrappers
- [ ] detekt or ktlint checks pass cleanly with zero styling violations

## References & Further Reading
- Load `references/Spacecraft_Kotlin_Guidelines.md` for full skeletons (dispatcher injected service, supervisorScope task runner, Exposed ORM transaction, Arrow Either error handler, and JUnit 5 test) when deeper patterns are needed.
- *Further reading* (consulted for background only): Kotlin Coroutines Design Document, Jetbrains Kotlin Documentation, detekt handbook, and Arrow-Kt documentation.

When the user requests Kotlin code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
