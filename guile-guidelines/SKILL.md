---
name: guile-functional-concurrency
description: Write idiomatic, functional, concurrent GNU Guile (Guile Scheme 3.x) code. Use this skill whenever the user is writing, reviewing, or debugging Guile Scheme — any .scm file, any mention of Guile, Scheme, fibers, Guile channels, SRFIs, syntax-rules/syntax-case macros, Guile-specific APIs (spawn-fiber, run-fibers, make-channel, put-message, get-message, choice-operation), or functional Scheme patterns. Trigger even when the request is implicit, e.g. "write a concurrent worker pool in Guile", "make this Scheme tail-recursive", "set up a CSP pipeline", or "port this to fibers." Covers functional style (pure functions, proper tail calls, SRFI-1 list processing, records, hygienic macros) AND concurrency (Fibers/CSP, Guile channels, POSIX threads, futures, promises, parameters). Do NOT trigger for Common Lisp, Python, Rust, or other languages — only Guile Scheme. Prefer this skill over generic Scheme/Lisp advice — Guile has specific modules, idioms, and pitfalls that generic advice gets wrong.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# GNU Guile: Functional & Concurrent Programming

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

Write Guile Scheme that is **functional by default** and **concurrent via message-passing**. This skill encodes the decisions a Guile expert makes automatically so you don't fall back on generic Lisp habits that miss Guile-specific modules and pitfalls.

Assume **Guile 3.x** unless told otherwise (JIT compiler, `(ice-9 match)`, `(ice-9 exceptions)`, `define-syntax` hygiene). For concurrency, assume **guile-fibers** is available or installable.

## Core stance

1. **Pure first.** Functions take values and return values. Push side effects (I/O, mutation, logging) to the edges of the program. A function that both computes and mutates is two functions wearing a trenchcoat — split them.
2. **Message-passing over shared state.** When work needs to happen concurrently, default to Fibers + channels (CSP). Reach for threads + mutexes only when you genuinely need preemption or CPU parallelism, and keep critical sections tiny.
3. **Tail calls are not optional.** Guile guarantees proper tail calls. Any recursion over a list or stream must be in tail position (named `let` with an accumulator), or it will blow the stack on large inputs.
4. **Hygiene over `define-macro`.** Use `syntax-rules` for pattern macros and `syntax-case` when you need to break hygiene deliberately. Avoid the unhygienic `define-macro` except in throwaway scripts.

## Decision tree: which concurrency model?

```
I/O-bound concurrency (network, pipes, many tasks)? → Fibers + channels
CPU-bound parallel compute?                          → futures (ice-9 futures) or threads
Need true preemption / blocking C calls?             → POSIX threads (ice-9 threads)
Composable select / timeout across events?           → fiber operations (choice-operation)
Lazy, compute-once value?                            → promises (delay / force)
Per-task context without globals?                    → parameters (make-parameter + parameterize)
```

When in doubt for I/O work, choose Fibers. It scales to many lightweight tasks and avoids almost all data-race bugs because state is not shared.

## Workflow

Before writing Guile code:

1. **Confirm the model.** Functional/sequential, or concurrent? If concurrent, run the decision tree above.
2. **Identify the effects.** List every side effect (I/O, mutation, randomness, time). Plan to isolate them so the core stays pure and testable.
3. **Load the relevant reference.** Read the matching file in `references/` for exact module names, idioms, and pitfalls. Do not write Guile from memory of generic Scheme — the module names and gotchas differ.
4. **Pick the SRFIs.** Most functional Guile pulls in SRFI-1 (lists) and often SRFI-26 (`cut`). See the functional reference.

## Reference files

Read the file that matches the task. Both are concise and worth reading in full when relevant.

- **`references/functional.md`** — Pure functions, SRFI-1 list processing, `fold`/`unfold`, tail-recursion patterns, `(ice-9 match)` pattern matching, SRFI-9 records, `cut`/`cute` partial application, function composition, hygienic macros (`syntax-rules`/`syntax-case`), modules, and error handling with `(ice-9 exceptions)` + `guard`. Read this for any non-trivial functional Guile.
- **`references/concurrent.md`** — Fibers (`spawn-fiber`, `run-fibers`), channels (`make-channel`, `put-message`, `get-message`), composable operations (`choice-operation`, `select`, timeouts), POSIX threads + mutexes/condition variables, futures, promises, and thread-local state via parameters. Includes a worker-pool and a pipeline example. Read this for anything concurrent.

## Non-negotiable idioms (quick reference)

These are common enough to inline. For anything beyond them, read the reference files.

**Tail-recursive accumulation** — never build a non-tail recursive loop over unbounded data:
```scheme
(define (sum lst)
  (let loop ((l lst) (acc 0))
    (if (null? l) acc
        (loop (cdr l) (+ acc (car l))))))
```

**Functional iteration over manual recursion** — prefer the combinator:
```scheme
(use-modules (srfi srfi-1))
(fold + 0 (iota 100))                ; not a hand-rolled loop
(filter-map (lambda (x) (and (odd? x) (* x x))) lst)
```

**Channels over shared mutation** — concurrent state lives in messages, not variables:
```scheme
(use-modules (fibers) (fibers channels))
(run-fibers
  (lambda ()
    (let ((ch (make-channel)))
      (spawn-fiber (lambda () (put-message ch (work))))
      (consume (get-message ch)))))
```

**`with-mutex`, never manual lock/unlock** — it's exception-safe; manual locking leaks on error:
```scheme
(use-modules (ice-9 threads))
(define mu (make-mutex))
(with-mutex mu (set! shared (update shared)))
```

**`parameterize` for per-task context** — parameters are thread/fiber-local, unlike globals:
```scheme
(define current-db (make-parameter #f))
(parameterize ((current-db conn)) (run-query ...))
```

## Style rules

- Naming: `kebab-case`; predicates end in `?` (`prime?`); mutating procedures end in `!` (`set-car!`, `vector-fill!`).
- Avoid `set!`. If you need mutable state, isolate it behind a small interface or move it into a channel/parameter.
- Prefer named `let` loops over `do`. `do` is legal but reads as un-idiomatic in modern Guile.
- Keep `run-fibers` as the single entry point for fiber programs; spawn all fibers inside it.
- Inside fibers, use fiber-aware sleep/I/O — a blocking C call stalls the whole scheduler.
- Catch exceptions per-fiber so one failure doesn't take down the scheduler.
- Comments: `;` inline, `;;` block, `;;;` section header, `;;;;` file header. `pk` ("peek") is the idiomatic debug print and returns its last argument so you can wrap expressions.

## Common pitfalls to actively avoid

- **Non-tail recursion on big lists** → stack overflow. Always check recursion is in tail position.
- **Blocking inside a fiber** with a non-fiber-aware call → stalls every other fiber. Use the fibers I/O facilities or move blocking work to a thread.
- **Sharing mutable state between fibers** assuming it's "safe because cooperative" → still races at every suspension point. Pass messages.
- **`define-macro`** in library code → breaks hygiene, captures variables. Use `syntax-rules`.
- **Holding a mutex across a blocking call or a long computation** → contention and deadlock. Keep critical sections minimal and never block while holding a lock.
- **Acquiring multiple mutexes in inconsistent order** → deadlock. Fix a global lock ordering.
- **Reaching for generic Scheme module names** (e.g. assuming R6RS library paths) → Guile uses `(srfi srfi-N)`, `(ice-9 ...)`, and `(fibers ...)`. Check the references.

*— Built by Spacecraft Software —*
