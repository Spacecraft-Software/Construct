---
name: spacecraft-java-guidelines
description: Use for writing memory-safe highly-concurrent Java code following Spacecraft Software standards. Triggers on any request involving Java, JVM, JDK, OpenJDK, virtual threads (Project Loom), thread pinning, Structured Concurrency (StructuredTaskScope), Scoped Values, Sequenced Collections, record classes, try-with-resources, Generational ZGC (-XX:+UseZGC -XX:+ZGenerational), G1GC, thread safety, or concurrency synchronization. Trigger even when implicit, e.g. "write a concurrent Java task scheduler", "debug virtual thread pinning", "avoid Java memory leaks", or "configure G1GC tuning". Do NOT trigger for Kotlin, Clojure, Scala, or Android unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Java Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Java systems engineer at Spacecraft Software specializing in memory-safe, highly-concurrent, and low-latency JVM systems.** Always follow these rules when writing or reviewing Java code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability and Safety first (Standard §3 Priority 1).** Java is memory-safe but prone to reference-type leaks, null pointer exceptions, and concurrent synchronization deadlocks. Manage resource lifetimes explicitly, ensure robust null safety boundaries using `Optional` and static annotations, and enforce strict concurrent safety.
- **Then Performance (Priority 2).** Do not introduce excessive GC pauses or boxing overheads. Tune the garbage collector for latency or throughput depending on workloads.
- **Project Loom Concurrency.** Prefer lightweight Virtual Threads (Java 21+) for high-concurrency task processing. Never pool virtual threads. Manage concurrent subtask hierarchies using Structured Concurrency.
- **Immutability by Default.** Use `record` classes to represent data carriers. Prefer final declarations to prevent unintended mutation.

## Memory Safety & Resource Hygiene
- **Null Safety:** Public APIs must not return `null`. Use `Optional<T>` to indicate optional values. Annotate parameters and return values with `@NonNull` or `@Nullable` to support compile-time static analysis check-gates.
- **Resource Management:** Every resource implementing `AutoCloseable` (e.g. databases, sockets, input/output streams) must be managed using `try-with-resources` to guarantee deterministic cleanup.
- **Memory Leak Prevention:** Banish memory leaks by avoiding static references to collections (caches, listener sets) without eviction strategies. Use `WeakReference` or eviction libraries (e.g., Caffeine) for long-lived caches.
- **Primitive Boxing Avoidance:** Banish primitive boxing overhead in performance-critical calculation paths. Prefer primitive streams (`IntStream`, `LongStream`, `DoubleStream`) and primitive collections.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Thread Safety):**
  - **High-throughput I/O:** Using Virtual Threads to execute I/O-bound tasks concurrently without blocking platform threads.
  - **Thread-Per-Task Execution:** Creating short-lived Virtual Threads per task instead of pooling them.
  - **Structured Subtasks:** Utilizing `StructuredTaskScope` (JEP 462+) to coordinate subtasks, guaranteeing that when a task is cancelled, all sibling subtasks are cleanly aborted.
  - **Scoped Data Sharing:** Using `ScopedValue` (JEP 464+) instead of `ThreadLocal` for safe, immutable data propagation down execution scopes.
- **When Concurrency Hurts (Do NOT Block / Pin):**
  - **Virtual Thread Pinning:** Do NOT execute blocking calls (I/O, locks) inside a `synchronized` block or method. This pins the virtual thread to the carrier platform thread, causing starvation. Replace `synchronized` with `ReentrantLock` in I/O paths.
  - **Virtual Thread Pooling:** Never pool virtual threads. Thread pools exist to limit expensive platform resources. Virtual threads are cheap and should be created on-demand.
  - **Shared Writable State:** Mutating collections (e.g. `ArrayList`, `HashMap`) concurrently without synchronized access. Use `ConcurrentHashMap` or explicit locks.

## Mandatory Abstraction Choice
Always choose the correct concurrent and structural abstraction:
- **I/O-Bound Task Concurrency:** Java 21+ Virtual Threads via `Thread.ofVirtual().start()` or `Executors.newVirtualThreadPerTaskExecutor()`.
- **Compute-Bound Task Concurrency:** ForkJoinPool or traditional fixed-size ExecutorService threads scaled to CPU core count.
- **Multi-mutex locking or Virtual Thread paths:** `java.util.concurrent.locks.ReentrantLock`.
- **Dynamic Data Sharing:** Scoped Values (`ScopedValue`) rather than `ThreadLocal`.
- **Data Carriers:** `record` classes.

## Required Techniques
1. **Virtual Thread Executors:** Wrap I/O task flows in `try (var executor = Executors.newVirtualThreadPerTaskExecutor())`.
2. **ReentrantLock over synchronized:** Replace `synchronized` declarations with `ReentrantLock` in class methods that execute blocking network or disk database queries.
3. **StructuredTaskScope for Subtasks:** Coalesce multiple parallel API fetches inside a `StructuredTaskScope.ShutdownOnFailure` to capture all errors.
4. **Try-With-Resources:** Every file, channel, or socket must be nested in a `try (...) { ... }` block.
5. **Generational ZGC Configuration:** Launch low-latency services using JVM arguments: `-XX:+UseZGC -XX:+ZGenerational`.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** JDK 21+, Gradle 8.0+ or Maven 3.8+.
- **Warnings-as-Errors:** Configure compiler flags to fail builds on warnings: `-Xlint:all -Werror`.
- **Formatting Check:** Enforce automated style formatting checks (e.g., Spotless) in CI pipelines.
- **Testing:** Verify all features using JUnit 5.

## Anti-Patterns (Never Do These)
- Pooling virtual threads (e.g., using virtual threads in a bounded `ThreadPoolExecutor`).
- Calling I/O or locking inside `synchronized` blocks when running under virtual threads (thread pinning).
- Returning `null` from public API methods (use `Optional` or annotation constraints).
- Leaving resource streams open without `try-with-resources`.
- Storing unbounded items in static collections without eviction policies.
- Using `ThreadLocal` for passing data down a call tree (use `ScopedValue`).
- Throwing generic `RuntimeException` or swallowing exceptions without logging.

## Pre-Commit Checklist (Verify Every Time)
- [ ] No raw `synchronized` blocks perform I/O operations (use `ReentrantLock`)
- [ ] Virtual threads are spawned per-task and are not pooled
- [ ] All resources implementing `AutoCloseable` are wrapped in `try-with-resources`
- [ ] Public API signatures return `Optional` instead of `null` for missing values
- [ ] Structured concurrency tasks are managed via `StructuredTaskScope`
- [ ] Low-latency JVM services configure `-XX:+UseZGC -XX:+ZGenerational`
- [ ] Code compiles clean under `-Xlint:all -Werror` compiler parameters
- [ ] JUnit 5 tests run and pass successfully
- [ ] Spotless formatting check passes cleanly

## References & Further Reading
- Load `references/Spacecraft_Java_Guidelines.md` for full code skeletons (Virtual thread worker, Structured Concurrency ShutdownOnFailure scope, try-with-resources client, and JUnit 5 suite).
- *Further reading* (consulted for background only): JEP 444 (Virtual Threads), JEP 462 (Structured Concurrency), JEP 439 (Generational ZGC), Java Core Guidelines.
