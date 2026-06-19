<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Chez FFI, AOT Compilation & the Safety Lever

The three Chez-specific topics where generic Scheme advice is useless and Guile advice is
wrong. Read the relevant section.

**Contents:** `optimize-level` (the safety lever — read first) · FFI: loading & calling C ·
FFI: ftypes & memory · FFI: callbacks & the GC · Boundary memory-safety discipline · AOT
compilation & whole-program optimization · Other performance levers.

---

## `optimize-level` — the safety lever (read this first)

Chez's `optimize-level` parameter (0–3) controls how much the compiler trusts the code.

- **Levels 0–2 are *safe***: full type checks, bounds checks, and argument checks. A bad
  index or a type error raises a condition. **`(optimize-level 2)` is the Spacecraft default**
  — safe *and* well-optimized.
- **Level 3 is *unsafe***: the compiler **omits** type and bounds checks and assumes your
  code is correct. A wrong assumption is **undefined behaviour** — silent memory corruption,
  not an exception. Level 3 is Chez's `unsafe`.

**Rule (Steelbore §3.1, memory-safety priority):** treat `(optimize-level 3)` exactly as you
treat Rust `unsafe`:

- Never set it globally or as a project default.
- Opt in narrowly — a single reviewed module or a specific `compile-file` invocation — and
  only after profiling proves a safe version cannot meet the budget.
- Justify it in writing and record it in the project's memory-safety exemption, exactly like
  an enumerated FFI surface.
- Re-establish safety at the boundary: validate inputs in safe code *before* they reach a
  level-3 region.

Reach for speed in this order: idiomatic safe code → type hints + allocation discipline →
whole-program optimization (below) → *then*, if still short, a confined level-3 region. Most
hot paths never need level 3; the precise GC and native backend are already fast.

## FFI: loading & calling C

```scheme
(import (chezscheme))

(load-shared-object "librms_ipc.so")          ; or a full path; "libc.so.6" for libc, etc.

;; (foreign-procedure conventions* "c_name" (arg-type ...) return-type)
(define ts-connect
  (foreign-procedure "ts_connect" (string) iptr))      ; char* -> handle (intptr)
(define ts-send
  (foreign-procedure "ts_send" (iptr u8* size_t) int)) ; handle, buf, len -> status
```

Common types: `int`, `unsigned`, `integer-8/16/32/64`, `unsigned-8/…`, `size_t`,
`iptr`/`uptr` (pointer-sized signed/unsigned), `float`, `double`, `boolean`, `void`,
`string` (UTF-8 marshalled), `void*` and `u8*` (raw addresses / bytevector data), and
`(* ftype)` for typed pointers (below). Use `__collect_safe` on a `foreign-procedure` that
may block, so the collector can run while the C call is in flight.

## FFI: ftypes & raw memory

`define-ftype` describes C layouts; `make-ftype-pointer`, `ftype-ref`, `ftype-set!` read and
write them; `foreign-alloc`/`foreign-free` manage off-heap memory you own.

```scheme
(define-ftype winsize
  (struct (rows unsigned-16) (cols unsigned-16) (xpix unsigned-16) (ypix unsigned-16)))

(define ws (make-ftype-pointer winsize (foreign-alloc (ftype-sizeof winsize))))
(ftype-set! winsize (rows) ws 40)
(ftype-set! winsize (cols) ws 120)
(some-ioctl fd ws)
(foreign-free (ftype-pointer-address ws))      ; free exactly what you alloc'd
```

## FFI: callbacks & the GC (the subtle part)

`foreign-callable` exposes a Scheme procedure to C. Because Chez's GC is **precise and
moving**, any Scheme object whose address you hand to C must be **locked** for the duration,
or the collector may move or reclaim it under C's feet.

```scheme
(define cb (foreign-callable (lambda (x) (handle x)) (int) void))
(lock-object cb)                               ; pin it before C can call it
(define cb-addr (foreign-callable-entry-point cb))
;; … register cb-addr with the C library, run …
(unlock-object cb)                             ; only after C will never call it again
```

The same applies to a bytevector whose data pointer you pass to C and that C retains beyond
the call: `lock-object` it, pass `(#%$object-address bv …)` per the manual, unlock when done.
For call-and-return (C doesn't retain the pointer) the marshalling handles it — locking is
for addresses C keeps.

## Boundary memory-safety discipline

The FFI is the one place Scheme's safety guarantees stop. Confine and guard it:

- **One narrow module per C library** (e.g. `(majestic term backend vterm)`), exporting safe
  Scheme procedures only — callers never see a `foreign-procedure` or a raw address.
- **Validate everything in safe Scheme before the call**: lengths, ranges, non-null,
  encoding. Never pass a Scheme-controlled length to C unchecked.
- **Own the lifetime**: `foreign-free` what you `foreign-alloc`; `unlock-object` what you
  `lock-object`; pair them with `dynamic-wind` so they run on escape.
- **Fuzz the wrapper** in CI with malformed inputs (truncated frames, adversarial bytes).
- **Enumerate the surface** in the memory-safety exemption — each linked library and any
  `optimize-level 3` region is a named, justified entry.

This is exactly the posture PRD #3b's TwinScrew binding takes: the unchanged `librms_ipc.so`
C ABI is reached through one validating `foreign-procedure` module, adding no new `unsafe`
beyond the audited, fuzzed seam.

## AOT compilation & whole-program optimization

Chez compiles to native code ahead of time — no bytecode VM, no JIT warmup.

```scheme
(compile-file "src/foo.ss")                    ; -> foo.so (native, fast-loading)
(compile-library "lib/bar.sls")                ; compile one R6RS library
```

For a shipped binary, use **whole-program optimization (wpo)** to inline and dead-strip
across libraries into one image:

```scheme
(generate-wpo-files #t)                        ; emit .wpo alongside .so during compile
(compile-whole-program "app.wpo" "app.boot")   ; fuse into a single optimized boot image
```

Distribute as a boot file run by the Chez kernel (`scheme --boot app.boot`) or, for a single
self-contained executable, embed the boot image with the kernel per the Chez manual
(`make-boot-file`). `compile-imported-libraries` can be set so a top-level program compiles
its dependency graph automatically. Keep `optimize-level` at 2 through all of this — wpo gets
most of the speed *without* surrendering safety.

## Other performance levers (use sparingly)

- **Type hints / `fx`-ops**: use fixnum operations (`fx+`, `fxvector-ref`) and `flonum` ops
  (`fl+`) on numeric hot paths so the compiler skips generic dispatch — safe at level 2.
- **Profile-guided optimization**: `(compile-profile #t)`, run a representative load, dump
  with `(profile-dump-data …)`, recompile using the profile.
- **GC tuning**: `(collect-trip-bytes)` and `collect-request-handler` adjust collection
  cadence; measure before touching.
- **`cp0`**: the source optimizer runs at level ≥1; `(cp0-effort-limit)` bounds inlining
  effort if compile time matters.

Order of reach, always: safe idioms → fx/fl hints + allocation discipline → wpo → profiling →
(last, confined, justified) a level-3 region.
