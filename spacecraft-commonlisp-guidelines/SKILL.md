---
name: spacecraft-commonlisp-guidelines
description: Use for writing type-safe highly-concurrent memory-safe Common Lisp code (targeting SBCL) following Spacecraft Software standards. Triggers on any request involving Common Lisp, Lisp, .lisp/.asd files, SBCL, Quicklisp, ASDF, Bordeaux-Threads (locks, condition variables), lparallel (worker pools, channel pipelines), type declarations (fixnum, double-float, simple-array), compiler optimization settings (optimize speed/safety), CFFI pointer safety (with-foreign-object, trivial-garbage finalizers), or FiveAM/Rove testing. Trigger even when implicit, e.g. "write a Bordeaux thread", "parallelize this loop in Lisp", "set optimization flags in SBCL", or "make this Lisp code faster". Do NOT trigger for Scheme (spacecraft-guile-guidelines/spacecraft-chez-guidelines) or Clojure (spacecraft-clojure-guidelines) — Lisp dialect structures differ sharply. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Common Lisp Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Common Lisp systems engineer at Spacecraft Software specializing in high-performance, concurrent systems targeting SBCL (Steel Bank Common Lisp).** Always follow these rules when writing or reviewing Lisp code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Common Lisp is dynamically typed, but SBCL compiles type declarations into highly specialized native code and runtime checks. Keep the compiler's safety policies enabled; reject micro-optimizations that bypass safety checks unless thoroughly verified.
- **Then Performance (Priority 2).** SBCL is an optimizing native-code compiler. Guide the compiler with explicit type declarations (`fixnum`, `double-float`, `simple-array`) to eliminate boxing and generic function dispatch on hot paths.
- **Interactive, incremental development.** Code is designed to be re-evaluated live. Ensure your packages and systems support clean reloads (`asdf:load-system`) without orphan threads or dynamic variable re-initialization bugs.
- **Write macro wrappers for boilerplate, not for control flow.** Lisp macros are powerful; use them to encapsulate complex patterns (like FFI setup/teardown) rather than syntax manipulation that obscures standard logic.

## Memory Safety & Dynamic Safety
- **Optimization Policy:** Never set `(safety 0)` globally or in unvetted code. A type mismatch or out-of-bounds array access at `safety 0` in SBCL causes silent heap corruption or segmentation faults. The Spacecraft default policy is:
  ` (declare (optimize (speed 3) (safety 1) (debug 1)))`
  This guarantees that SBCL maintains bounds checks and type validations while compiling high-speed assembly.
- **CFFI Pointer Hygiene:** Foreign memory is outside the Lisp GC's visibility.
  - Temporary foreign variables and pointers must use `cffi:with-foreign-object` to guarantee allocation with dynamic (stack-like) extent.
  - Long-lived foreign memory must be wrapped in Lisp class structures with garbage-collection finalizers (using the `trivial-garbage` package) to prevent memory leaks.
  - Never let a pointer allocated by `with-foreign-object` escape its lexical body.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Spawn):**
  - **CPU-bound Parallelism:** Parallel maps, reductions, and task pipelines using `lparallel` worker pools.
  - **Thread-Local Scoping:** Using dynamically bound variables (`special` variables) with `let` to provide thread-local state. (In SBCL, binding a special variable creates a thread-local binding automatically.)
  - **Asynchronous Events:** Long-lived independent worker daemons managed via `bordeaux-threads`.
- **When Concurrency Hurts (Do NOT Spawn):**
  - **Short-Lived Task Spawning:** Spawning raw OS threads via `bt:make-thread` for small computations. Always use `lparallel` pools.
  - **Global Variable Contention:** Writing to global shared dynamic variables (`defparameter`/`defvar`) without synchronization. Use `sb-ext:compare-and-swap` (CAS) or locks.
  - **Lock Contention:** Heavy lock usage (`bt:with-lock-held`) in hot loops. Prefer lock-free queues or CAS-based atomics.

## Mandatory Abstraction Choice
Always match the concurrency library to the workload:
- **Parallel Compute Workload:** `lparallel` tasks. Initialize the kernel once on startup (`(setf lparallel:*kernel* (lparallel:make-kernel n))`) based on core count; never recreate kernels dynamically.
- **Coarse-Grained Threading:** `bordeaux-threads` (`bt:make-thread`).
- **Locks and Coordination:** `bt:with-lock-held` for basic critical sections; SBCL's compare-and-swap (`sb-ext:compare-and-swap`) on slot values or arrays for lock-free structures.
- **C FFI boundary:** `cffi` declarations packaged in a distinct module, using type-checked translations and FFI arrays.

## Required Techniques
1. **Type Declarations:** Annotate inputs and locals in performance-critical sections (e.g. `(declare (type fixnum x) (type (simple-array double-float (*)) arr))`).
2. **Tail Call Optimization (TCO):** Ensure recursive functions are tail-recursive. SBCL optimizes tail calls when compiling under `(speed 3)`. Verify by checking compiler notes.
3. **Minimize Allocation (Consing):** Avoid allocating temporary lists or objects in hot loops. Use `map-into` to reuse sequence containers and pre-allocate flat arrays.
4. **Compile-Time Checks:** Pay attention to SBCL compiler warnings. SBCL flags type conflicts and unoptimizable code during compilation; treat these compiler notes as lints.
5. **Clean Thread Shutdown:** When saving images (`sb-ext:save-lisp-and-die`), ensure all background Bordeaux threads are shut down cleanly to prevent corrupted images.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** SBCL ≥ 2.3.0, Quicklisp, ASDF ≥ 3.3.
- **ASDF Warnings-as-Errors:** Configure ASDF system checks to fail on unhandled compiler warnings in CI.
- **Formatting:** Enforce uniform indentation (standard Lisp style, 2-spaces for body forms).
- **Testing:** FiveAM or Rove test suites covering functional edge cases, regressions, and API contracts.

## Anti-Patterns (Never Do These)
- Using `(safety 0)` globally or in any untested functions.
- Returning or saving pointers allocated via `cffi:with-foreign-object` outside their lexical block.
- Creating raw OS threads on-demand inside loops instead of dispatching to an `lparallel` queue.
- Leaving compiler notes showing dynamic/generic dispatch on numeric loops (indicates missing type declarations).
- Modifying shared state (such as global hash tables) across threads without locks or CAS synchronization.
- Saving a Lisp image (`save-lisp-and-die`) while background threads are active.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Optimization policy is explicitly configured (`speed 3`, `safety 1`, `debug 1`)
- [ ] Type declarations are added to all performance-critical function inputs and arrays
- [ ] SBCL compiler notes and type warnings are resolved; no generic arithmetic in loops
- [ ] CFFI pointer allocations use `with-foreign-object`; long-lived memory uses finalizers
- [ ] Multiprocessing compute tasks use `lparallel` pools; raw threads are not spawned on-demand
- [ ] Dynamically bound variables are properly thread-isolated with `let`
- [ ] ASDF system definition (`.asd`) compiles without unhandled warnings or errors
- [ ] Test suites (FiveAM / Rove) execute clean and green
- [ ] Saved executable scripts cleanly handle thread teardown on exit

## References & Further Reading
- Load `references/Spacecraft_CommonLisp_Guidelines.md` for full skeletons (lparallel array sum, safe CFFI pointer wrapper, TCO loop, and FiveAM test setup) when deeper patterns are needed.
- *Further reading* (consulted for background only): the SBCL User Manual, Common Lisp HyperSpec (CLHS), Bordeaux-Threads documentation, and Quicklisp/ASDF manuals.

When the user requests Common Lisp code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
