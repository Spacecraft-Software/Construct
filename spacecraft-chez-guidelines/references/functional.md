<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Functional Chez Scheme (R6RS)

Pure, value-oriented Chez. Read top to bottom or jump by section.

**Contents:** Libraries & imports · Pure functions & immutability · List processing
(R6RS + SRFI-1) · Tail recursion · Pattern matching (`match` — bring it) · Records
(`define-record-type`) · Macros (`syntax-rules`/`syntax-case`) · Conditions & `guard` ·
Parameters · Akku packaging.

---

## Libraries & imports

Code lives in **R6RS libraries**. One concern per library; explicit exports; import `(rnrs)`
plus only the `(chezscheme)` pieces you actually use.

```scheme
;; SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
;; SPDX-License-Identifier: GPL-3.0-or-later
(library (majestic stratum rope)
  (export make-rope rope-insert rope-length rope->string)
  (import (rnrs)              ; R6RS base: define, lambda, let, cond, list ops, …
          (only (chezscheme) fxvector make-fxvector))  ; Chez extras, named explicitly
  ;; … definitions …
  )
```

A runnable entry point is a **top-level program**, not a library:

```scheme
#!r6rs
(import (rnrs) (majestic stratum rope))
(display (rope->string (make-rope "hello")))
(newline)
```

- SRFI imports use the **R6RS colon form**: `(import (srfi :1))`, `(import (srfi :43))`.
  (Guile's `(srfi srfi-1)` is wrong here.)
- Chez-specific bindings come from `(chezscheme)`. Import the whole thing only in scripts;
  in libraries, `(only (chezscheme) …)` the names you use so the surface stays auditable.

## Pure functions & immutability

Default to functions that read inputs and return new values. Chez's lists, vectors-as-values,
and records (below) make this idiomatic; the precise generational GC makes short-lived
intermediate values cheap. Keep effects at the edges (see Conditions for failure, and
`concurrent.md` for state held in mailboxes rather than variables).

```scheme
;; pure
(define (normalize v) (let ((s (apply + v))) (map (lambda (x) (/ x s)) v)))
;; effect at the edge
(define (emit! port v) (for-each (lambda (x) (put-string port (number->string x))) v))
```

## List processing — R6RS base + SRFI-1

R6RS already gives you a lot from `(rnrs)` / `(rnrs lists)`:
`map`, `for-each`, `filter`, `partition`, `find`, `assoc`/`assp`, `member`/`memp`,
`fold-left`, `fold-right`, `for-all`, `exists`, `remove`/`remp`, `cons*`, `list-sort`.

For the richer combinators, pull in **SRFI-1** (Akku: `(srfi :1)`):
`fold`, `reduce`, `unfold`, `take`/`drop`, `span`/`break`, `iota`, `filter-map`,
`append-map`, `delete-duplicates`, `count`, `last`, `list-index`, `zip`/`unzip`.

```scheme
(import (rnrs) (srfi :1))
(fold-left + 0 (iota 100))                 ; iterative (tail) — preferred on big lists
(filter-map (lambda (x) (and (odd? x) (* x x))) xs)
(delete-duplicates (append-map neighbours nodes) =)
```

Prefer a combinator to a hand-rolled loop; prefer `fold-left` (tail/iterative) to
`fold-right` (builds stack) on large inputs.

## Tail recursion

Chez guarantees proper tail calls (R6RS). Recursion over unbounded data must be in tail
position — a named `let` with an accumulator:

```scheme
(define (reverse* xs)
  (let loop ((xs xs) (acc '()))
    (if (null? xs) acc (loop (cdr xs) (cons (car xs) acc)))))
```

Non-tail recursion (or `fold-right`) on a long list overflows the control stack. When you
need the right-fold *shape* without the stack, reverse-and-`fold-left`.

## Pattern matching — bring a `match`

Chez has **no built-in `match`**. Install one via Akku (e.g. the portable `match` /
`(matchable)` package, or the `match` bundled with the nanopass framework) and import it.

```scheme
(import (rnrs) (matchable))
(define (eval-expr e)
  (match e
    (('lit n)        n)
    (('add a b)      (+ (eval-expr a) (eval-expr b)))
    (('neg a)        (- (eval-expr a)))
    ((? symbol? s)   (lookup s))
    (_               (error 'eval-expr "bad expression" e))))
```

Until a `match` is in scope, destructure with `car`/`cdr`/`cond` or R6RS `case` — but for
anything beyond trivial shapes, add the dependency; it pays for itself immediately.

## Records — R6RS `define-record-type`

Records are the standard product type. **Fields are immutable unless declared `mutable`** —
keep them immutable and return new records to change state.

```scheme
(define-record-type buffer
  (fields (immutable text)        ; -> buffer-text
          (immutable cursor)      ; -> buffer-cursor
          (mutable   dirty?)))    ; -> buffer-dirty?  /  buffer-dirty?-set!

(define b (make-buffer "hi" 0 #f))
(buffer-text b)                                  ; "hi"
;; "change" by constructing a new value:
(define (move-cursor b n) (make-buffer (buffer-text b) n (buffer-dirty? b)))
```

Chez also has the older `define-structure`; prefer R6RS `define-record-type` for
portability and explicit mutability. Use opaque records (no exported constructor) to hide
representation behind accessor procedures.

## Macros — hygiene only

**`syntax-rules`** for pattern macros:

```scheme
(define-syntax swap!
  (syntax-rules ()
    ((_ a b) (let ((tmp a)) (set! a b) (set! b tmp)))))
```

**`syntax-case`** when you must compute identifiers or inspect syntax — still hygienic;
break hygiene only deliberately with `datum->syntax`:

```scheme
(define-syntax define-getter
  (lambda (stx)
    (syntax-case stx ()
      ((_ name field)
       (with-syntax ((getter (datum->syntax #'name
                               (string->symbol
                                 (string-append "get-"
                                   (symbol->string (syntax->datum #'field)))))))
         #'(define (getter r) (field r)))))))
```

There is no `define-macro` in Chez to misuse — do not emulate one. Macros that need a fender
(guard expression) put it between the pattern and the template in `syntax-case`.

## Conditions & error handling — R6RS `guard`

Errors are R6RS **conditions**, raised with `raise`/`raise-continuable` and caught with
`guard`. Build informative conditions with `who`/`message`/`irritants`.

```scheme
(import (rnrs) (rnrs conditions))

(define (parse-port-number s)
  (let ((n (string->number s)))
    (unless (and n (exact? n) (<= 0 n 65535))
      (assertion-violation 'parse-port-number "not a valid port" s))
    n))

(guard (e ((assertion-violation? e)
           (log-error (condition-who e) (condition-message e) (condition-irritants e))
           #f)
          ((i/o-error? e) (reconnect) 'retry)
          (else (raise e)))                 ; re-raise what you don't handle
  (parse-port-number user-input))
```

Define your own condition types when a domain needs them:

```scheme
(define-condition-type &stale-tag &error
  make-stale-tag-error stale-tag-error?
  (tag stale-tag-error-tag))
```

Guidance: raise specific conditions; catch narrowly and re-raise the rest with `(raise e)`;
never swallow `else` silently. Keep `guard` at effect boundaries, not sprinkled through pure
code.

## Parameters

`make-parameter` / `parameterize` for dynamically-scoped configuration (R6RS). For
per-thread context in a threaded build, use Chez's **`make-thread-parameter`** (see
`concurrent.md`).

```scheme
(define current-log (make-parameter (current-error-port)))
(parameterize ((current-log my-port)) (run …))
```

## Akku packaging

Declare dependencies in `Akku.manifest`; `akku install` resolves and writes `Akku.lock`
(commit both). Typical functional deps: `(srfi :1)`, a `match` package, `(srfi :64)` for
tests. Keep the dependency set small and GPL-compatible (Steelbore §10 gate); every addition
is a deliberate choice recorded in the manifest.
