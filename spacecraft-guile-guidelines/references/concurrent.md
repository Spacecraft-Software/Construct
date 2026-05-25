# Concurrent Guile Reference

Concurrency models, modules, idioms, and pitfalls for Guile Scheme 3.x. Default to Fibers for I/O concurrency; use threads only for preemption or CPU parallelism.

## Contents
- Fibers & channels (CSP) — the default
- Composable operations (select / timeout)
- Worker pool example
- Pipeline example
- POSIX threads
- Mutexes & condition variables
- Futures (CPU parallelism)
- Promises (lazy, once-only)
- Parameters (thread/fiber-local state)
- Pitfalls

## Fibers & channels (CSP) — the default

Fibers (`guile-fibers`, a separate library) implement Communicating Sequential Processes: many cheap green threads that share nothing and coordinate by passing messages over channels. This eliminates most data-race classes.

```scheme
(use-modules (fibers)
             (fibers channels))

(run-fibers                                  ; single entry point
  (lambda ()
    (let ((ch (make-channel)))
      (spawn-fiber                           ; producer
        (lambda ()
          (let loop ((i 0))
            (when (< i 5)
              (put-message ch i)             ; blocks until a receiver is ready
              (loop (1+ i))))
          (put-message ch 'done)))
      (let loop ()                           ; consumer
        (let ((msg (get-message ch)))        ; blocks until a message arrives
          (unless (eq? msg 'done)
            (format #t "got ~a~%" msg)
            (loop)))))))
```

Rules:
- `run-fibers` starts the scheduler; spawn every fiber inside it. It returns when the thunk returns.
- Channels are **unbuffered rendezvous** by default: `put-message` blocks until a `get-message` is ready and vice versa. This gives natural backpressure.
- Inside a fiber, use `(use-modules (fibers timers))` `sleep` / fiber-aware I/O — never a blocking C call, which stalls the whole scheduler thread.
- Fibers can run across multiple OS threads if you pass `#:parallelism` to `run-fibers`.

## Composable operations (select / timeout)

Operations are first-class synchronizable events — Guile's answer to Go's `select`. Build them, combine with `choice-operation`, run with `perform-operation`.

```scheme
(use-modules (fibers) (fibers channels) (fibers operations) (fibers timers))

;; Wait on whichever happens first: a message, or a 5-second timeout.
(perform-operation
  (choice-operation
    (wrap-operation (get-operation ch)
                    (lambda (msg) (cons 'msg msg)))
    (wrap-operation (sleep-operation 5)
                    (lambda _ 'timeout))))
```

- `get-operation` / `put-operation` turn a channel action into a (not-yet-performed) event.
- `wrap-operation` post-processes the result.
- `choice-operation` races several events; the first ready one wins, others are abandoned.
- `sleep-operation` is a timeout event. Combine it with any channel op to add deadlines.

## Worker pool example

Fan work out to N workers over a job channel, collect results on a result channel.

```scheme
(use-modules (fibers) (fibers channels) (ice-9 match))

(define (worker-pool n jobs results)
  (for-each
    (lambda (id)
      (spawn-fiber
        (lambda ()
          (let loop ()
            (match (get-message jobs)
              ('done #t)                          ; sentinel: exit
              (job   (put-message results
                                  (cons id (* job job)))
                     (loop)))))))
    (iota n)))

(run-fibers
  (lambda ()
    (let ((jobs (make-channel)) (results (make-channel)))
      (worker-pool 4 jobs results)
      (spawn-fiber (lambda ()                      ; feed jobs
        (for-each (lambda (j) (put-message jobs j)) (iota 10))
        (for-each (lambda (_) (put-message jobs 'done)) (iota 4))))
      (let loop ((seen 0))                         ; gather 10 results
        (when (< seen 10)
          (match (get-message results)
            ((id . r) (format #t "worker ~a → ~a~%" id r)))
          (loop (1+ seen)))))))
```

The `'done` sentinel is the idiomatic way to signal channel completion (channels have no built-in close).

## Pipeline example

Chain stages, each a fiber connected by a channel — a streaming, backpressured pipeline.

```scheme
(use-modules (fibers) (fibers channels))

(define (stage in out f)
  (spawn-fiber
    (lambda ()
      (let loop ()
        (let ((x (get-message in)))
          (if (eq? x 'done)
              (put-message out 'done)
              (begin (put-message out (f x)) (loop))))))))

(run-fibers
  (lambda ()
    (let ((a (make-channel)) (b (make-channel)) (c (make-channel)))
      (stage a b (lambda (x) (* x x)))      ; square
      (stage b c (lambda (x) (+ x 1)))      ; increment
      (spawn-fiber (lambda ()
        (for-each (lambda (x) (put-message a x)) (iota 5))
        (put-message a 'done)))
      (let loop ()
        (let ((y (get-message c)))
          (unless (eq? y 'done)
            (format #t "~a~%" y) (loop)))))))
```

## POSIX threads

Use real threads for CPU-bound parallelism or when you must call blocking code without stalling other work. `(use-modules (ice-9 threads))`.

```scheme
(define t (call-with-new-thread
            (lambda () (expensive-compute))))
(join-thread t)                 ; → thread's return value

;; Parallel forms
(par-map f lst)                 ; map across threads, collect results
(parallel e1 e2 e3)             ; evaluate in parallel, return all values
(n-par-map 4 f lst)             ; bounded to 4 worker threads
```

## Mutexes & condition variables

Shared mutable state needs a mutex. Always use `with-mutex` (exception-safe) over manual `lock-mutex`/`unlock-mutex` (leaks the lock on error).

```scheme
(use-modules (ice-9 threads))

(define mu (make-mutex))
(define shared 0)

(define (bump!)
  (with-mutex mu
    (set! shared (+ shared 1))))    ; critical section: keep it tiny

;; Condition variables for signaling
(define cv (make-condition-variable))

;; Waiter
(with-mutex mu
  (let loop ()
    (unless (ready? shared)
      (wait-condition-variable cv mu)   ; atomically releases mu while waiting
      (loop))))

;; Signaler
(with-mutex mu
  (set! shared (make-ready shared))
  (signal-condition-variable cv))       ; or broadcast-condition-variable for all
```

Rules: never block (I/O, long compute) while holding a mutex; acquire multiple mutexes in a fixed global order to avoid deadlock; always re-check the condition in a loop after waking (spurious wakeups).

## Futures (CPU parallelism)

`(use-modules (ice-9 futures))` — lightweight parallel computation on a worker pool. Best for pure CPU-bound work.

```scheme
(define f (future (fib 35)))    ; starts computing, maybe on another core
(touch f)                       ; blocks until done, returns result

;; Parallel map via futures
(map touch (map (lambda (x) (future (heavy x))) inputs))
```

`future`/`touch` is simpler than manual threads when the work is pure and you just want it parallelized.

## Promises (lazy, once-only)

Built in — no module needed. `delay` defers, `force` evaluates once and memoizes.

```scheme
(define config (delay (read-and-parse "config.scm")))
(force config)    ; computed on first force, cached thereafter
(force config)    ; returns cached value, no recompute
```

Use for expensive deferred initialization that may not be needed, or to break initialization-order cycles.

## Parameters (thread/fiber-local state)

`make-parameter` creates a dynamically-scoped cell that is **per-thread and per-fiber** — the safe way to thread context (current user, db handle, request id) without globals or explicit argument-passing.

```scheme
(define current-request (make-parameter #f))

(parameterize ((current-request req))
  (handle))                      ; (current-request) returns req in this dynamic extent

;; Outside the parameterize (or in another fiber), it reverts to the default.
```

Because each fiber/thread sees its own binding, parameters avoid the races that plain globals would introduce under concurrency.

## Pitfalls

- **Blocking call inside a fiber** (raw `read`, sleep from `ice-9 threads`, blocking C FFI) stalls every fiber on that scheduler thread. Use fiber-aware I/O and `(fibers timers)` `sleep`.
- **Assuming cooperative = race-free.** State shared between fibers still races at every suspension point (every `put-message`/`get-message`/`sleep`). Pass messages instead of sharing.
- **Forgetting the completion sentinel.** Channels don't close; a consumer loop blocks forever waiting for more. Send an explicit `'done` (one per consumer for pools).
- **Mutex held across blocking/long work** → contention, possible deadlock. Shrink the critical section; compute outside the lock, then take the lock only to publish.
- **Inconsistent multi-mutex ordering** → deadlock. Define and document a global lock order.
- **Not re-checking conditions after `wait-condition-variable`** → acting on stale state due to spurious wakeups. Always loop on the predicate.
- **Using futures for I/O-bound work** → ties up the compute pool. Futures are for CPU-bound pure work; use fibers for I/O.
