---
name: spacecraft-typescript-guidelines
description: Use for writing type-safe highly-concurrent optimized TypeScript code targeting TypeScript 7.0+ (Go-based native compiler). Triggers on any request involving TypeScript, tsc, tsconfig.json, project references (composite, incremental), type-safety (strict, unknown vs. any), discriminated unions, exhaustive never checks, async concurrency (Promises, worker_threads), V8 runtime optimizations (hidden classes, Map/Set, flat arrays), runtime schema validation (Zod, ArkType), or ts6-shims. Trigger even when implicit, e.g. "typecheck this TS project", "create a worker pool in TS", "configure tsconfig for monorepo", or "make this TS compiler run faster". Do NOT trigger for standard JavaScript (unless type-safety is requested) or other languages. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft TypeScript Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert TypeScript systems engineer at Spacecraft Software specializing in type-safe, high-performance, and concurrent systems targeting TypeScript 7.0+ (the native Go-based compiler).** Always follow these rules when writing or reviewing TypeScript code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** TypeScript compile-time checks are your primary guarantee. Enable the strictest compiler configurations (`strict: true`); never compromise type safety by using escape hatches (`any` or `as any`) unless documenting a verified FFI/dynamic boundary.
- **Then Performance (Priority 2).** TypeScript compiles to JavaScript executed on the V8 engine (Node.js/Bun/Deno/Browser). Write V8-friendly code by keeping object shapes stable (hidden classes), avoiding GC churn on hot loops, and choosing optimized collections (`Map`/`Set`) over raw objects.
- **Leverage the TypeScript 7.0 Go Compiler.** Capitalize on the 10x speedup of the native Go-based `tsc` by structuring projects with Project References to maximize incremental caching and multithreaded compilation.
- **Defensive boundary validation.** Since TypeScript type parameters disappear at runtime, validate all incoming external data (JSON API payloads, process boundaries) using schema parsers like `Zod` or `ArkType`.

## Memory Safety & Type Safety
- **Strict Configuration:** Ensure your `tsconfig.json` contains:
  ` "strict": true, "noImplicitOverride": true, "exactOptionalPropertyTypes": true, "noUncheckedIndexedAccess": true`
  This prevents null dereferencing, index out-of-bounds omissions, and dynamic object corruption.
- **Avoid Escape Hatches:** Never use `any`. Use `unknown` for dynamic values, forcing developers to narrow types using type guards (`typeof`, `instanceof`, or custom predicates) before accessing properties.
- **Exhaustive Control Flow:** Use discriminated unions and exhaustive `switch`/`case` or `if`/`else` control chains, asserting completeness via the `never` type at the default/fallback branches.
- **No Non-Null Assertions:** Avoid the non-null assertion operator (`!`). Refactor code to check for `undefined` or supply fallback default values.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Async / Multi-thread):**
  - **Asynchronous I/O:** Using `async`/`await` and non-blocking APIs to parallelize network, file, and database operations.
  - **Concurrency Joining:** Resolving independent asynchronous tasks concurrently using `Promise.all` or `Promise.allSettled` instead of sequential `await` calls.
  - **CPU-bound Parallelism:** Spawning heavy computations (parsing, encryption, data conversions) onto separate threads via Node.js `worker_threads` (or Web Workers in browsers) to keep the main event loop responsive.
- **When Concurrency Hurts (Do NOT Spawn / Block):**
  - **Blocking the Event Loop:** Running synchronous heavy calculations on the main thread, freezing the single-threaded event loop and halting client connection handlers.
  - **Unbounded Thread Spawning:** Creating raw worker threads on demand (high CPU/memory overhead). Use a persistent worker threadpool like `Piscina`.
  - **Structured Clone Overhead:** Sending massive nested objects to worker threads. Data is serialized and copied; prefer `SharedArrayBuffer` for flat zero-copy buffers.

## Mandatory Abstraction Choice
Always choose the concurrency model corresponding to the workload:
- **Compute-heavy workload:** A persistent threadpool using `Piscina` to queue and execute tasks on background workers.
- **Asynchronous I/O workload:** Event-loop-friendly async/await. Yield using microtasks where computations are unavoidable.
- **Data Collections:** Use `Map` and `Set` for dynamic key-value lookups rather than raw object literals.
- **TypeScript 7.0+ Monorepo Compilation:** Use ASDF/Quicklisp equivalents in package managers (npm workspaces, pnpm) combined with Project References (`composite: true`, `incremental: true`, `declarationMap: true`).
- **Tooling Compatibility Shim:** For tools relying on programmatic compiler APIs (Webpack/ESLint plugins), utilize `@typescript/typescript6` compatibility shims alongside the standard `tsc` CLI.

## Required Techniques
1. **Hidden Class Preservation:** Initialize all object fields in constructor functions. Never dynamically add (`obj.newProp = x`) or delete (`delete obj.prop`) properties on hot objects, as this forces V8 to de-optimize the object's hidden class.
2. **Pre-allocate Arrays:** When array sizes are known beforehand, initialize flat arrays using `new Array(size)` or TypedArrays (e.g. `Float64Array`) to prevent V8 from resizing arrays dynamically in memory.
3. **Zod Boundary Parsing:** Decode untrusted data using `.parse()` or `.safeParse()` immediately at network or FFI entrances.
4. **Tail Call Loop Optimization:** JavaScript engines have varying support for tail-call optimizations. Prefer explicit `while` or `for` loops for performance-critical deep iterations to avoid stack size failures.
5. **Incremental Building:** Enable `--incremental` and `--tsBuildInfoFile` in `tsconfig.json` to leverage Go compiler caching.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** TypeScript ≥ 7.0.0, Node.js ≥ 20.0.0 (or Bun/Deno equivalents).
- **Warnings-as-Errors:** Compile with `noEmitOnError: true` to prevent emitting broken JS outputs when compiler errors occur.
- **Linting & Formatting:** ESLint for checking syntax patterns; Prettier for format checks.
- **Testing:** Vitest or Jest for unit testing; property testing via `fast-check` to assert type invariants.

## Anti-Patterns (Never Do These)
- Using `any` or `as any` to mute compiler warnings.
- Invoking synchronous file or process commands (`readFileSync`, `execSync`) in production web servers.
- Spawning worker threads on-demand for short-lived task requests.
- Modifying object properties dynamically in hot loops, causing V8 de-optimizations.
- Leaving `!` (non-null assertions) unchecked on API response mappings.
- Programmatic Webpack/ESLint builds pointing to TS 7.0 programmatic compiler API (use TS 6 compatibility package).

## Pre-Commit Checklist (Verify Every Time)
- [ ] `strict: true` and all strict compiler flags are configured in `tsconfig.json`
- [ ] No `any` type annotations or non-null assertions (`!`) left in source code
- [ ] Union checks are exhaustive and validated with the `never` type assert
- [ ] Incoming dynamic structures are validated at the boundaries using Zod/ArkType
- [ ] Hot loops preserve V8 object shapes (no dynamic property additions/deletions)
- [ ] CPU-heavy blocks are offloaded to Piscina threadpools; I/O uses async/await
- [ ] Project references are set up (`composite: true`, `incremental: true`) in monorepos
- [ ] TypeScript compiles cleanly with zero errors (`noEmitOnError: true`)
- [ ] Unit and property-based (`fast-check`) test suites are green

## References & Further Reading
- Load `references/Spacecraft_TypeScript_Guidelines.md` for full skeletons (monorepo configurations, Piscina threadpool worker, Zod parser boundary, never-exhaustiveness check, Vitest suite) when deeper patterns are needed.
- *Further reading* (consulted for background only): Microsoft TypeScript 7.0 Release Notes, Project References Guide, V8 Hidden Classes documentation, and Piscina API docs.

When the user requests TypeScript code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
