<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Concurrent Chez Scheme (threads, by hand)

Chez gives you **OS threads sharing one precise-GC heap (no GIL)**, **mutexes**, and
**condition variables** — and nothing higher. There is no Fibers, no CSP scheduler, no
channel type, no `choice-operation`. You build the abstractions. This file gives you the
ones you'll reuse.

**Contents:** The model · Primitives · `with-lock` · Mailbox (unbounded) · Bounded channel
(backpressure) · Worker pool · Timeouts · CPU parallelism over immutable data ·
Thread-local context · Pitfalls.

---

## The model

- Build Chez `--threads`. Check at runtime with `(threaded?)`.
- `fork-thread` starts an OS thread running a thunk; it returns immediately. There is **no
  built-in join** — signal completion yourself (a condition, or a result mailbox).
- Threads share the heap, so two threads touching the same mutable object race. The cheap,
  correct default is **share immutable values read-only**; coordinate the rest through
  mutex-guarded state or, better, message-passing.
- The GC is precise and stop-the-world-ish; it does not impose Boehm-style conservative
  pauses, but allocation on hot paths still costs — keep hot loops allocation-light.

## Primitives

```scheme
(import (chezscheme))

(define m  (make-mutex))
(mutex-acquire m)          ; blocking
(mutex-acquire m #f)       ; non-blocking; #t if acquired, #f if not
(mutex-release m)

(define cv (make-condition))
(condition-wait cv m)      ; atomically release m + sleep until signalled, then re-acquire m
(condition-wait cv m 0.5)  ; …with a 0.5s timeout; #t if signalled, #f if timed out
(condition-signal cv)      ; wake one waiter
(condition-broadcast cv)   ; wake all waiters
```

`condition-wait` must be called holding `m`, and it can wake **spuriously** — always
re-check the predicate in a loop, never with a bare `if`.

## `with-lock` — exception-safe locking

Chez has no `with-mutex`. Own this macro and use it everywhere; never bare acquire/release.

```scheme
(define-syntax with-lock
  (syntax-rules ()
    ((_ m body ...)
     (dynamic-wind
       (lambda () (mutex-acquire m))
       (lambda () body ...)
       (lambda () (mutex-release m))))))    ; runs on normal return, exception, or escape
```

## Mailbox (unbounded queue)

The workhorse: one mutex, one condition, a FIFO. Producers `put!`, consumers `take!` and
block until a value exists.

```scheme
(define-record-type mailbox
  (fields (mutable head) (mutable tail) (immutable mtx) (immutable non-empty))
  (protocol (lambda (new) (lambda () (new '() '() (make-mutex) (make-condition))))))

(define (mailbox-put! mb v)
  (with-lock (mailbox-mtx mb)
    (let ((cell (cons v '())))
      (if (null? (mailbox-head mb))
          (begin (mailbox-head-set! mb cell) (mailbox-tail-set! mb cell))
          (begin (set-cdr! (mailbox-tail mb) cell) (mailbox-tail-set! mb cell)))
      (condition-signal (mailbox-non-empty mb)))))

(define (mailbox-take! mb)
  (with-lock (mailbox-mtx mb)
    (let loop ()
      (if (null? (mailbox-head mb))
          (begin (condition-wait (mailbox-non-empty mb) (mailbox-mtx mb)) (loop))  ; re-check
          (let ((v (car (mailbox-head mb))))
            (mailbox-head-set! mb (cdr (mailbox-head mb)))
            (when (null? (mailbox-head mb)) (mailbox-tail-set! mb '()))
            v)))))
```

(`set-cdr!` requires Chez's mutable pairs, available from `(chezscheme)`. Keep the mutation
confined inside the mailbox; callers only ever see `put!`/`take!`.)

## Bounded channel (backpressure)

When a fast producer must not outrun a slow consumer, bound the queue and add a
**not-full** condition. This is the backpressure analogue of a Guile bounded channel.

```scheme
;; sketch: same structure as mailbox, plus a capacity and a `non-full` condition.
;; put! waits on non-full while (>= count cap); take! waits on non-empty while (= count 0);
;; put! signals non-empty; take! signals non-full. Both re-check predicates in a loop.
```

Closing convention: carry a `closed?` flag; `take!` on an empty closed channel returns an
eof sentinel so consumers terminate instead of blocking forever. Always provide a way to
close, or wedged consumers will hang at shutdown.

## Worker pool

N threads draining a shared mailbox; submit a `'stop` sentinel per worker to drain cleanly.

```scheme
(define (start-pool n handle)
  (let ((jobs (make-mailbox)))
    (do ((i 0 (+ i 1))) ((= i n))
      (fork-thread
        (lambda ()
          (let loop ()
            (let ((job (mailbox-take! jobs)))
              (unless (eq? job 'stop)
                (guard (e (#t (log-error e)))   ; one bad job must not kill the worker
                  (handle job))
                (loop)))))))
    jobs))

(define pool (start-pool 4 process-request))
(mailbox-put! pool some-request)
;; shutdown: (do ((i 0 (+ i 1))) ((= i 4)) (mailbox-put! pool 'stop))
```

## Timeouts (there is no `select`)

Compose deadlines from `condition-wait`'s timeout; loop while not ready and not past the
deadline.

```scheme
(define (take-within mb seconds)
  (let ((deadline (+ (current-time-seconds) seconds)))
    (with-lock (mailbox-mtx mb)
      (let loop ()
        (cond ((not (null? (mailbox-head mb))) (dequeue! mb))
              ((>= (current-time-seconds) deadline) 'timeout)
              (else (condition-wait (mailbox-non-empty mb) (mailbox-mtx mb)
                                    (- deadline (current-time-seconds)))
                    (loop)))))))
```

To wait on several sources, give them a *shared* condition and signal it from each; the
waiter re-checks every source on wake. This is the manual stand-in for `choice-operation`.

## CPU parallelism over immutable data

Partition immutable input, `fork-thread` a worker per partition, collect results through a
mailbox. Nothing is locked because nothing shared is mutated.

```scheme
(define (pmap-chunks f chunks)
  (let ((results (make-mailbox)) (n (length chunks)))
    (for-each (lambda (c) (fork-thread (lambda () (mailbox-put! results (f c))))) chunks)
    (let loop ((i 0) (acc '()))
      (if (= i n) (reverse acc) (loop (+ i 1) (cons (mailbox-take! results) acc))))))
```

## Thread-local context

`make-thread-parameter` gives each thread its own value — the threaded analogue of a
parameter, and the right tool for per-connection / per-request context without globals.

```scheme
(define current-session (make-thread-parameter #f))
(fork-thread (lambda () (current-session sess) (serve)))
```

## Pitfalls

- **Bare `mutex-acquire`/`mutex-release`** → leaked lock on exception/escape. Use `with-lock`.
- **`condition-wait` under `if`, not a loop** → spurious wakeups slip through. Always loop and
  re-check the predicate.
- **Forgetting threads share the heap** — "it's cooperative" is a Guile-Fibers intuition that
  is false here; any shared mutable object races. Share immutable values; guard the rest.
- **Holding a lock across a blocking call or long compute** → contention/deadlock. Copy out
  what you need under the lock, release, then do the slow work.
- **Inconsistent lock ordering** across two mutexes → deadlock. Fix one global order.
- **No close/stop path** for a mailbox/channel → consumers block forever at shutdown. Always
  provide a sentinel or `closed?` flag.
- **Per-job exceptions killing a worker** → silent pool shrinkage. `guard` each job (see the
  pool above).
