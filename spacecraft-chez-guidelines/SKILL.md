---
name: spacecraft-chez-guidelines
description: Write idiomatic, functional, safe, concurrent Chez Scheme following Spacecraft Software standards. Use whenever writing, reviewing, or debugging Chez Scheme — any mention of Chez, Petite Chez, R6RS libraries, `define-record-type`, `syntax-case`/`syntax-rules`, the `foreign-procedure`/`load-shared-object`/`define-ftype` FFI, `fork-thread`/`make-mutex`/`make-condition` concurrency, `optimize-level`, `compile-program`/whole-program optimization, boot files, the nanopass framework, or Akku. Trigger even when implicit, e.g. "build a worker pool in Chez", "bind this C library from Chez", "make this Chez program faster", or "is optimize-level 3 safe?". Do NOT trigger for GNU Guile (use `spacecraft-guile-guidelines` — its modules, Fibers, and FFI differ), nor Racket, Clojure, Common Lisp, or other Schemes/Lisps; Chez is R6RS-rooted with its own threads, FFI, and AOT compiler, so generic Scheme or Guile advice gets it wrong. Prefer this over generic Scheme advice for any Chez work.
license: GPL-3.0-or-later
metadata:
  maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
  website: https://Construct.SpacecraftSoftware.org/
---

# Chez Scheme: Functional, Safe & Concurrent Programming

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

Write Chez Scheme that is **functional by default**, **memory-safe by default**, and
**concurrent via hand-built message-passing over real threads**. This skill encodes the
decisions a Chez expert makes automatically so you don't fall back on generic Lisp habits
or — the more dangerous trap — **Guile habits**. Chez is R6RS-rooted, natively AOT-compiled
(Dybvig's nanopass backend, precise generational GC), and ships *no* CSP/Fibers layer and
*no* built-in `match`. Its module system, FFI, threads, and compiler are its own.

Assume **Chez Scheme 9.5+** with the **threaded** build (`--threads`) unless told otherwise.
Assume R6RS libraries are the unit of code and **Akku** is the package manager.

> **Not Guile.** If you catch yourself writing `(use-modules …)`, `(srfi srfi-1)`,
> `spawn-fiber`, `run-fibers`, `(ice-9 match)`, or `(ice-9 exceptions)`, stop — those are
> Guile. The Chez equivalents are `(import …)`, `(srfi :1)`, `fork-thread` + a mailbox you
> build, a `match` library you bring, and R6RS `guard`. When in doubt, read the references.

## Core stance

1. **Pure first.** Functions take values and return values; push side effects (I/O,
   mutation, time, randomness) to the edges. A function that both computes and mutates is
   two functions in a trenchcoat — split them. Chez's persistent-friendly values and precise
   GC make this cheap.
2. **`optimize-level 2` is the law; `optimize-level 3` is Chez's `unsafe`.** At level 3 the
   compiler omits type and bounds checks and a wrong assumption is undefined behaviour —
   memory corruption, not an exception. Treat level 3 exactly like Rust `unsafe`: never the
   global default, opt-in per-module, justified, reviewed, confined, and recorded in the
   Standard §3.1 memory-safety exemption. Reach speed with safe code, type hints, and
   allocation discipline first. (See `references/ffi-and-build.md`.)
3. **Concurrency is hand-built on threads — there is no Fibers.** Chez gives you OS threads
   sharing one precise-GC heap (no GIL), plus mutexes and condition variables, and *nothing*
   higher. Default to immutable data shared read-only across threads; when you need
   message-passing, build a small mailbox/channel on mutex + condition (the references give
   you the code). Do not reach for a CSP scheduler — Chez has none to reach for.
4. **Tail calls are guaranteed (R6RS).** Any recursion over unbounded data must be in tail
   position — a named `let` with an accumulator, or `fold-left` (iterative), never
   `fold-right` or hand recursion on big inputs.
5. **Hygiene only.** `syntax-rules` for pattern macros, `syntax-case` when you must bend
   hygiene deliberately. There is no `define-macro` to misuse here — keep it that way.
6. **R6RS libraries are the unit; Akku is the package manager.** One concern per library,
   explicit `export`/`import`, dependencies declared in `Akku.manifest` and pinned in the
   lockfile. SRFI-1, `match`, and SRFI-64 are Akku packages, not built-ins.

## Decision tree: which concurrency tool?

```
Many I/O tasks / fan-out?          → a thread pool draining a mailbox you build (mutex+condition)
CPU-bound parallel compute?        → fork-thread over partitions of IMMUTABLE data; collect via a results mailbox
Backpressure between stages?       → a bounded channel (mutex + two conditions: not-full / not-empty)
Wait with a deadline?              → condition-wait with a timeout, re-checking the predicate in a loop
Lazy, compute-once value?          → (delay …) / (force …)   [R6RS]
Per-thread context without globals? → (make-thread-parameter …)   [Chez]
```

There is no `choice-operation`, no `select` over channels, no green-thread scheduler. You
compose timeouts out of `condition-wait` deadlines. For the two-process Majestic client
(PRD #3b) this is exactly right: a modest main loop + worker threads + one mailbox, while the
heavy parallelism lives in the Rust engine across the socket.

## Workflow

Before writing Chez code:

1. **Confirm the build.** Threaded (`--threads`) if any concurrency is involved; note the
   target (native AOT image vs. script). For a shipped binary, plan whole-program
   optimization (`references/ffi-and-build.md`).
2. **Confirm the model.** Functional/sequential, or concurrent? If concurrent, run the
   decision tree — and remember you are *building* the abstraction, not importing it.
3. **Identify the effects** (I/O, mutation, FFI, time). Isolate them so the core stays pure
   and testable.
4. **Load the relevant reference.** Read the matching `references/` file for exact forms,
   library names, and pitfalls. Do **not** write Chez from memory of Guile — the module
   names, the FFI, and the concurrency primitives all differ.
5. **Pick the packages.** Most functional Chez pulls in `(srfi :1)` and a `match` library;
   tests pull in SRFI-64. Declare them in `Akku.manifest`.

## Reference files

Read the file that matches the task; each is concise and worth reading in full when relevant.

- **`references/functional.md`** — R6RS libraries & `import`, pure functions, R6RS list ops
  + SRFI-1 (Akku), tail-recursion patterns, a brought-in `match`, R6RS `define-record-type`,
  `syntax-rules`/`syntax-case` hygiene, R6RS conditions + `guard`, parameters, and Akku
  packaging. Read for any non-trivial functional Chez.
- **`references/concurrent.md`** — The no-Fibers reality; `fork-thread`, mutexes, condition
  variables; an exception-safe `with-lock` macro; building a mailbox and a bounded channel;
  a worker pool; a timeout pattern; CPU parallelism over immutable data; `make-thread-parameter`.
  Read for anything concurrent.
- **`references/ffi-and-build.md`** — The FFI (`load-shared-object`, `foreign-procedure`,
  `foreign-callable`, `define-ftype`, `foreign-alloc`/`lock-object`) with the boundary
  memory-safety discipline; the AOT compilation model (`compile-program`,
  `compile-whole-program`/wpo, boot files); and **`optimize-level` as the safety lever**.
  Read for any C binding (e.g. PRD #3b's TwinScrew) or any "make it fast / ship a binary" task.

## Non-negotiable idioms (quick reference)

Inline because they're constant. For anything beyond them, read the references.

**Tail-recursive accumulation** — never a non-tail loop over unbounded data:
```scheme
(define (sum xs)
  (let loop ((xs xs) (acc 0))
    (if (null? xs) acc
        (loop (cdr xs) (+ acc (car xs))))))   ; or: (fold-left + 0 xs)
```

**Exception-safe locking — own a `with-lock`; Chez has no `with-mutex`:**
```scheme
(define-syntax with-lock
  (syntax-rules ()
    ((_ m body ...)
     (dynamic-wind
       (lambda () (mutex-acquire m))
       (lambda () body ...)
       (lambda () (mutex-release m))))))      ; releases even on exception / escape
```
Bare `mutex-acquire`/`mutex-release` leaks the lock if the body raises or a continuation
escapes — never write them unguarded.

**Message-passing over shared mutation** — build the mailbox once, then pass values:
```scheme
;; full implementation in references/concurrent.md
(define mb (make-mailbox))
(fork-thread (lambda () (mailbox-put! mb (work))))
(consume (mailbox-take! mb))                  ; blocks on a condition until a value arrives
```

**R6RS records, immutable by default:**
```scheme
(define-record-type point
  (fields (immutable x) (immutable y)))
(point-x (make-point 3 4))                    ; => 3
```

**Per-thread context, not globals:**
```scheme
(define current-conn (make-thread-parameter #f))
(parameterize ((current-conn c)) (run-query …))
```

## Style rules

- Every source file opens with a Spacecraft SPDX header (`SPDX-FileCopyrightText`,
  `SPDX-License-Identifier: GPL-3.0-or-later`) and is REUSE-clean.
- Naming: `kebab-case`; predicates end in `?` (`prime?`); mutators/effectful procedures end
  in `!` (`set-car!`, `mailbox-put!`). Type/record constructors are `make-…`.
- Avoid `set!`. If you need mutable state, isolate it behind a small interface, a record
  field touched only through accessors, or a mailbox.
- Prefer named `let` loops over `do`. Prefer `fold-left`/`for-each` combinators over manual
  recursion when iterating.
- One R6RS library per concern; explicit exports; import `(rnrs)` and only the `(chezscheme)`
  pieces you use. SRFI imports use the R6RS colon form: `(import (srfi :1))`.
- Comments: `;` inline, `;;` block, `;;;` section header, `;;;;` file header.
- Timestamps and dates are ISO 8601 / UTC (Steelbore Standard).

## Common pitfalls to actively avoid

- **Reaching for Guile** — `(use-modules …)`, `(srfi srfi-1)`, `spawn-fiber`, `run-fibers`,
  `(ice-9 match)`, `(ice-9 exceptions)`, `with-mutex`. None exist in Chez. Use `(import …)`,
  `(srfi :1)`, threads + a mailbox, a `match` library, R6RS `guard`, and your own `with-lock`.
- **`(optimize-level 3)` globally** — silently disables safety checks; a single wrong type or
  out-of-bounds index becomes memory corruption. The top safety pitfall; gate it like `unsafe`.
- **Bare `mutex-acquire`/`mutex-release`** without `dynamic-wind` → leaked lock on any
  exception or `call/cc` escape → deadlock.
- **`condition-wait` without a predicate loop** → spurious-wakeup bugs. Always
  `(let loop () (unless (ready?) (condition-wait cv m) (loop)))`.
- **Non-tail recursion (or `fold-right`) on big data** → stack overflow. Use `fold-left` / a
  tail loop.
- **Assuming a rich stdlib** — there is no built-in `match`; bring one. (`(chezscheme)` does
  provide `format`, `printf`, and pretty-printing.)
- **Handing a Scheme object's address to C without `lock-object`** → the precise GC may move
  or collect it. Lock for the call's duration, then unlock; free what you `foreign-alloc`.
- **Holding a lock across a blocking call or long compute** → contention/deadlock. Keep
  critical sections tiny; never block while locked.

*— Built by Spacecraft Software —*
