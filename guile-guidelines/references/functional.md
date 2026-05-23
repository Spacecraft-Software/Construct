# Functional Guile Reference

Idioms, modules, and pitfalls for writing pure, functional Guile Scheme 3.x.

## Contents
- Pure functions & scoping
- List processing (SRFI-1)
- Tail recursion patterns
- Pattern matching (ice-9 match)
- Records (SRFI-9)
- Partial application & composition (SRFI-26)
- Macros (syntax-rules / syntax-case)
- Modules
- Error handling
- Useful SRFIs

## Pure functions & scoping

Keep functions referentially transparent: same inputs → same outputs, no observable effects. Use `let`/`let*`/`letrec` for local bindings instead of internal `define` when expressing a computation.

```scheme
(define (price-with-tax base rate)
  (let* ((tax   (* base rate))
         (total (+ base tax)))
    total))
```

`let` binds in parallel, `let*` sequentially (later bindings see earlier ones), `letrec` for mutually recursive local definitions.

## List processing (SRFI-1)

Always `(use-modules (srfi srfi-1))` for real list work — Guile's core list ops are a subset.

```scheme
(map (lambda (x) (* x x)) '(1 2 3))   ; → (1 4 9)
(filter odd? '(1 2 3 4 5))            ; → (1 3 5)
(fold + 0 '(1 2 3))                   ; → 6   ; (fold kons knil lst): (kons elem acc)
(fold-right cons '() '(1 2 3))        ; → (1 2 3)
(reduce + 0 '(1 2 3))                 ; → 6   ; like fold but seeds with first element
(filter-map (lambda (x) (and (odd? x) (* x x))) '(1 2 3 4))  ; → (1 9)
(append-map (lambda (x) (list x x)) '(1 2))                  ; → (1 1 2 2)
(partition odd? '(1 2 3 4))           ; → (1 3) and (2 4) as two values
(iota 5)                              ; → (0 1 2 3 4)
(iota 5 1)                            ; → (1 2 3 4 5)
(iota 5 0 2)                          ; → (0 2 4 6 8)
(take '(1 2 3 4 5) 2)                 ; → (1 2)
(last '(1 2 3))                       ; → 3
(delete-duplicates '(1 1 2 3 3))      ; → (1 2 3)
(count even? '(1 2 3 4))              ; → 2
```

`fold` argument order is `(fold proc init lst)` where `proc` receives `(element accumulator)` — element first, accumulator second. This is the reverse of some other Schemes; getting it backwards is a classic bug.

Build lists with `unfold` when generating rather than consuming:
```scheme
(unfold (lambda (x) (> x 5))   ; stop?
        (lambda (x) (* x x))   ; map
        (lambda (x) (+ x 1))   ; next seed
        1)                     ; → (1 4 9 16 25)
```

## Tail recursion patterns

Guile guarantees proper tail calls, so a tail-recursive loop runs in constant stack. Non-tail recursion over unbounded input overflows the stack.

Named `let` is the canonical loop:
```scheme
(define (range-sum n)
  (let loop ((i 0) (acc 0))
    (if (> i n) acc
        (loop (+ i 1) (+ acc i)))))
```

Accumulator-reverse pattern for building a list (cons in reverse, then reverse once):
```scheme
(define (squares lst)
  (let loop ((l lst) (acc '()))
    (if (null? l) (reverse acc)
        (loop (cdr l) (cons (* (car l) (car l)) acc)))))
```
In practice prefer `(map (lambda (x) (* x x)) lst)` — only hand-roll when the combinator doesn't fit.

For lazy/infinite sequences use streams from `(srfi srfi-41)` rather than eager lists.

## Pattern matching (ice-9 match)

`(use-modules (ice-9 match))` — far cleaner than nested `car`/`cdr`/`cond`.

```scheme
(define (eval-expr e)
  (match e
    ((? number? n)        n)
    (('+ a b)             (+ (eval-expr a) (eval-expr b)))
    (('* a b)             (* (eval-expr a) (eval-expr b)))
    ((op . _)             (error "unknown op" op))))

(match lst
  (()              'empty)
  ((x)             (list 'one x))
  ((x y)           (list 'two x y))
  ((head . tail)   (list 'more head tail)))

;; Guards and named sub-patterns
(match pt
  ((x y) (= x y))         ; bind and use
  (_ #f))
```

`match-lambda` and `match-let` are available for matching in argument and binding position.

## Records (SRFI-9)

Prefer typed records over raw pairs/alists for structured data.

```scheme
(use-modules (srfi srfi-9))

(define-record-type <point>
  (make-point x y)        ; constructor
  point?                  ; predicate
  (x point-x set-point-x!)   ; field with getter and (optional) setter
  (y point-y))               ; immutable field — no setter

(define p (make-point 3 4))
(point-x p)               ; → 3
```

Omit the setter to make a field immutable (the functional default). Records print opaquely; add a custom printer with `(srfi srfi-9 gnu)`'s `set-record-type-printer!` if needed.

## Partial application & composition (SRFI-26)

```scheme
(use-modules (srfi srfi-26))   ; cut / cute

(map (cut * 2 <>) '(1 2 3))            ; → (2 4 6)
(map (cut list 'x <> 'y) '(1 2))       ; → ((x 1 y) (x 2 y))
(filter (cut > <> 3) '(1 2 3 4 5))     ; → (4 5)
```

`cut` re-evaluates its non-slot args each call; `cute` evaluates them once at construction. Use `cute` when an argument is an expensive constant.

Composition (define once, reuse):
```scheme
(define ((compose2 f g) x) (f (g x)))
(define inc-then-double (compose2 (cut * 2 <>) 1+))
(inc-then-double 5)                    ; → 12
```

## Macros (syntax-rules / syntax-case)

Default to hygienic `syntax-rules`:
```scheme
(define-syntax swap!
  (syntax-rules ()
    ((_ a b)
     (let ((tmp a)) (set! a b) (set! b tmp)))))

(define-syntax my-when
  (syntax-rules ()
    ((_ test body ...)
     (if test (begin body ...)))))
```

Use `syntax-case` (a procedural macro with hygiene control) when you must inspect or generate identifiers, or deliberately introduce a binding:
```scheme
(define-syntax my-or
  (lambda (stx)
    (syntax-case stx ()
      ((_)        #'#f)
      ((_ e)      #'e)
      ((_ e1 e2 ...) #'(let ((t e1)) (if t t (my-or e2 ...)))))))
```

Avoid `define-macro` (Guile supports it for compatibility, but it's unhygienic and captures variables). Reserve it for quick scripts only.

## Modules

```scheme
(define-module (myapp core)
  #:use-module (srfi srfi-1)
  #:use-module (srfi srfi-26)
  #:use-module (ice-9 match)
  #:export (parse transform render))

;; Selective import to avoid clashes
(use-modules ((srfi srfi-1) #:select (fold filter-map partition)))

;; Rename on import
(use-modules ((mylib) #:renamer (symbol-prefix-proc 'lib:)))
```

Compile modules with `guild compile file.scm` for speed; Guile 3 also JITs hot code at runtime.

## Error handling

Guile 3 uses `(ice-9 exceptions)` plus the portable `guard` form.

```scheme
(use-modules (ice-9 exceptions))

;; Raise a structured exception
(raise-exception
  (make-exception
    (make-error)
    (make-exception-with-message "bad input")
    (make-exception-with-irritants (list value))))

;; Handle with guard (SRFI-34 style, built in)
(guard (exn ((error? exn)
             (format (current-error-port) "error: ~a~%"
                     (exception-message exn))
             #f))
  (risky-operation))

;; with-exception-handler for non-unwinding handlers
(with-exception-handler
  (lambda (exn) (cleanup) (raise-exception exn))
  (lambda () (do-work))
  #:unwind? #t)
```

Prefer returning values (e.g. `(values 'ok result)` / `(values 'error reason)`) over exceptions for expected failure paths; reserve exceptions for genuinely exceptional conditions. `dynamic-wind` guarantees cleanup runs even on non-local exit:
```scheme
(dynamic-wind
  (lambda () (acquire))
  (lambda () (use))
  (lambda () (release)))
```

## Useful SRFIs

| SRFI | Module | Purpose |
|------|--------|---------|
| 1  | `(srfi srfi-1)`  | List library (map/fold/filter/iota/partition) |
| 9  | `(srfi srfi-9)`  | Records |
| 11 | `(srfi srfi-11)` | `let-values` for multiple return values |
| 13 | `(srfi srfi-13)` | String library |
| 26 | `(srfi srfi-26)` | `cut`/`cute` partial application |
| 41 | `(srfi srfi-41)` | Streams (lazy sequences) |
| 43 | `(srfi srfi-43)` | Vector library |
| 64 | `(srfi srfi-64)` | Testing framework |
| 171| `(srfi srfi-171)`| Transducers (composable, allocation-free pipelines) |
