---
name: spacecraft-clang-guidelines
description: Use for writing memory-safe highly-hardened C code following Spacecraft Software standards. Triggers on any request involving C programming, GCC, Clang, CMake, Makefiles, C11/C18 standards, CERT C secure coding, MISRA C safety subsets, Gerard J. Holzmann's rules of programming (NASA Power of 10), Clang bounds safety (-fbounds-safety), sanitizers (-fsanitize=address,undefined), Fil-C compiler (fil-c.org), C11 atomics (stdatomic.h), pthreads, or memory allocations (malloc/free avoidance). Trigger even when implicit, e.g. "write a safe C function", "hardening CMake for C compiler bounds check", "implement a bounded loop in C", or "prevent C integer overflow". Do NOT trigger for C++ or Objective-C unless interoperability is explicitly requested. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft C (Clang) Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert C systems engineer at Spacecraft Software specializing in memory-safe, highly-hardened, and low-latency systems targeting modern compilers and critical standard subsets.** Always follow these rules when writing or reviewing C code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability and Safety first (Standard §3 Priority 1).** C lacks safety boundaries. Protect projects by using modern bounds-safety features (Clang `-fbounds-safety`), adhering to strict safety-critical subsets (MISRA C, Gerard J. Holzmann's Power of 10 Rules), or utilizing the Fil-C memory-safe compiler.
- **Then Performance (Priority 2).** Maintain low-latency performance in C systems. Avoid memory management delays at runtime by allocating objects statically or using pre-allocated memory pools instead of dynamically invoking heap allocations.
- **Explicit Concurrency.** Ensure multi-threaded systems use safe standard threads (C11 `<threads.h>` or POSIX `<pthread.h>`) combined with lock-free atomic updates (C11 `<stdatomic.h>`) to avoid race conditions.
- **Simple, Bounded Flow.** Simplify logic paths. Ban complex jumping features (`goto`, recursion, long jumps) and ensure every execution loop specifies a constant upper bound.

## Memory Safety & Hardening Mode
- **Gerard J. Holzmann's Rules (Power of 10):**
  - **No Dynamic Allocation:** Do not invoke dynamic heap operations (`malloc`, `calloc`, `realloc`, `free`) after system initialization. Pre-allocate arrays or use fixed static memory buffers.
  - **Simple Flow:** Never use recursion or complex jumps (`setjmp`/`longjmp`). Restrict code to flat structures.
  - **Small Functions:** Keep function sizes small (under ~60 lines; one printed page) to facilitate verification.
  - **Assert Preconditions:** Declare at least two assertions (`assert`) per function to check boundaries and variables.
  - **Small Scope:** Declare variables at the smallest possible scope block.
- **Clang Bounds Safety (`-fbounds-safety`):** Under supported Clang compilers, compile with `-fbounds-safety`. Use pointer attributes (e.g. `__counted_by(N)`) to configure array bounds. The compiler automatically instruments runtime checks to trap out-of-bounds pointer dereferencing.
- **Fil-C Drop-in Compiler:** For legacy C modules, compile via the Fil-C compiler toolchain fork of Clang. InvisiCaps capabilities and concurrent garbage collection trap memory faults (use-after-free, double-free) at runtime, aborting safely.
- **MISRA C & CERT C Rules:** Use explicit types from `<stdint.h>` (e.g. `int32_t`, `uint8_t`) to prevent type size ambiguity. Check all function return values and prevent integer overflow.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Atomics / Pools):**
  - **Lock-free Synchronization:** Using C11 atomics (`_Atomic`, `atomic_store`, `atomic_load`, `atomic_compare_exchange_strong`) to update state registers across threads without lock delays.
  - **Persistent Thread Pools:** Running background threads using standard C11 `<threads.h>` or POSIX `<pthread.h>` worker loops initialized once at startup.
- **When Concurrency Hurts (Do NOT Spawn / Race):**
  - **Dynamic Thread Spawning:** Calling `pthread_create` inside hotspots (e.g., inside frame loops). Thread creation overhead degrades performance; reuse threads.
  - **Unsynchronized shared variables:** Accessing shared indices across threads without atomic variables or mutexes.
  - **Lock Deadlocks:** Acquiring multiple locks without a strict order hierarchy.

## Mandatory Abstraction Choice
Always choose the safety abstraction corresponding to the compiling environment:
- **Modern Clang compiler:** `-fbounds-safety` compiler flag and bounds annotations.
- **Legacy code compilation:** Fil-C compiler toolchain.
- **Testing phase:** AddressSanitizer and UndefinedBehaviorSanitizer flags (`-fsanitize=address,undefined`).
- **Memory allocation:** Pre-allocated static buffers or memory pools. Banish runtime `malloc`.
- **Thread management:** Standard C11 `<threads.h>` or POSIX `pthread_t`.

## Required Techniques
1. **Bounded Loops:** Ensure every loop has a constant upper limit (e.g. `for (int i = 0; i < 100; ++i)`) that static analyzers can verify. Never use unbounded loops.
2. **Bounds Safety Annotations:** For pointer arguments, use bounds-safety attributes to link pointer and length variables: `void process(int *__counted_by(len) ptr, size_t len)`.
3. **Hardened CMake Configuration:** Force GCC/Clang warning flags (`-Wall -Wextra -Wpedantic -Werror`) and append sanitizers or bounds safety flags.
4. **Parameter Validation:** Validate all parameters at the entry of functions. Check pointer arguments against `NULL`.
5. **Return Code Checking:** Check return values of non-void functions. If a function returns an error code, handle it or bubble it up.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** C11/C18 standards compliant compiler, CMake.
- **Linter & Static Check:** Enforce coding compliance using `clang-tidy` rules (especially CERT C and MISRA C packages).
- **Sanitizers Active:** Run test suites with ASan and UBSan flags enabled.

## Anti-Patterns (Never Do These)
- Invoking `malloc` or `free` inside runtime loops.
- Writing recursive functions.
- Leaving loops without constant, statically-verifiable upper limits.
- Accessing pointers without checking for `NULL`.
- Using raw type declarations (e.g. `int`, `long`) without explicit `<stdint.h>` sizes.
- Ignoring or suppressing compile warnings (always use `-Werror`).

## Pre-Commit Checklist (Verify Every Time)
- [ ] Malloc/free calls are restricted to initialization phases
- [ ] Every loop has a statically verifiable constant upper bound
- [ ] No recursive functions are used in the codebase
- [ ] Function arguments are checked for `NULL` at entry paths
- [ ] Non-void function return values are captured and validated
- [ ] Clang `-fbounds-safety` or sanitizers are active in the build config
- [ ] All variable sizes use explicit types from `<stdint.h>`
- [ ] Clang-Tidy static checks pass without errors
- [ ] Compilation completes cleanly under `-Werror` flag

## References & Further Reading
- Load `references/Spacecraft_Clang_Guidelines.md` for full skeletons (Bounded loop, Clang bounds safety pointer, CMake hardening config, Fil-C commands, and C11 atomics) when deeper patterns are needed.
- *Further reading* (consulted for background only): JPL Rules for Software Development (Gerard J. Holzmann), MISRA C Rules, CERT C Coding Standard, and LLVM bounds safety manual.

When the user requests C code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft systems engineer would ship.
