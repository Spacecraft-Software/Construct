<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-->

# Safe-Concurrent Clojure

Reference for concurrent Clojure. The unifying principle: **separate identity from state**. Values are immutable; reference types move an identity from one immutable value to the next under precisely defined semantics. Choose the reference type by *how* the state changes, and most data races become unrepresentable.

Platform note: atoms and `core.async` work on JVM, ClojureScript, and Babashka. **Refs/STM, agents, `future`, `pmap`, reducers `fold`, and real threads are JVM-only.** ClojureScript is single-threaded (one event loop) — `core.async` gives you cooperative concurrency there, not parallelism.

## Contents

1. Choosing a reference type
2. Atoms — uncoordinated, synchronous
3. Refs + STM — coordinated, synchronous
4. Agents — uncoordinated, asynchronous
5. core.async — CSP channels & `go` blocks
6. Futures, promises, delays
7. Parallelism: `pmap`, reducers `fold`
8. `volatile!`, transients, dynamic vars
9. Worked example: bounded worker pool
10. Worked example: pipeline with backpressure
11. Pitfall catalogue

---

## 1. Choosing a reference type

| Need | Type | Coordinated? | Sync? |
|------|------|--------------|-------|
| One independent cell | **atom** | no | yes |
| Several cells, atomic together | **ref** (STM) | yes | yes |
| Independent cell, async, effects OK | **agent** | no | no |
| Streaming / flow / backpressure | **core.async** | — | either |
| One-shot async result | **future** / **promise** | no | async then deref |
| Per-call context | **dynamic var** | — | — |

Start with an atom for shared state; escalate to refs only when an invariant spans more than one cell. Use channels for communication and flow rather than rolling your own queues.

## 2. Atoms — uncoordinated, synchronous

The workhorse for a single piece of shared state. Backed by compare-and-swap (CAS): `swap!` applies your function to the current value and retries the whole function if another thread won the race.

```clojure
(def state (atom {:count 0 :items []}))

(swap! state update :count inc)
(swap! state update :items conj :x)
(reset! state {:count 0 :items []})          ; unconditional set
(compare-and-set! state @state new-val)      ; explicit CAS
(swap-vals! state update :count inc)         ; => [old new]
@state                                       ; deref, never blocks
```

**The rule:** the function passed to `swap!` must be **pure**. Under contention it is re-invoked until its CAS wins, so a side effect inside it fires an unpredictable number of times. Do effects *after*:

```clojure
(let [new-state (swap! state update :count inc)]
  (when (zero? (mod (:count new-state) 100))
    (flush-metrics! new-state)))             ; effect outside the retry loop
```

Watches and validators react to or guard changes:

```clojure
(set-validator! state map?)                  ; reject illegal new values
(add-watch state :audit
  (fn [_key _ref old new]                    ; runs after a successful change
    (when (not= (:count old) (:count new))
      (tap> [:count-changed new]))))
```

Watches/validators must also be cheap and tolerate being called often.

## 3. Refs + STM — coordinated, synchronous

When two or more cells must change as one atomic unit, wrap refs in a `dosync` transaction. The canonical case is a transfer that must never lose or duplicate money.

```clojure
(def account-a (ref 100))
(def account-b (ref 0))

(defn transfer! [from to amount]
  (dosync
    (when (< @from amount)
      (throw (ex-info "Insufficient funds" {:have @from :want amount})))
    (alter from - amount)
    (alter to   + amount)))
```

Transactions are atomic, consistent, and isolated; they retry automatically on write conflict. Therefore:

- **No side effects inside `dosync`** — they replay on every retry. To fire an effect exactly once *on commit*, `send` it to an agent from inside the transaction; the dispatch is held until the transaction commits.
- Prefer `alter` (read-modify-write) over `ref-set` (blind set).
- Use `commute` instead of `alter` for **commutative** updates (counters, set insertion) to cut retries — it relaxes ordering, so only use it when order truly doesn't matter.
- Use `ensure` to protect a ref you read but don't write when that read must stay consistent, preventing write skew.

```clojure
(dosync
  (commute total + amount)        ; commutative: fewer conflicts
  (ensure exchange-rate)          ; read must remain stable for this txn
  (alter ledger conj entry))
```

## 4. Agents — uncoordinated, asynchronous

An agent owns independent state mutated by actions that run **asynchronously** on a thread pool; the caller returns immediately. Actions for one agent are serialized, so the agent's own state is safe — and because each action runs exactly once, **side effects are allowed** here.

```clojure
(def log-agent (agent [] :error-mode :continue
                         :error-handler (fn [a ex] (tap> [:agent-error ex]))))

(send    log-agent conj entry)        ; CPU-bound action → bounded pool
(send-off log-agent write-to-disk!)   ; blocking/IO action → expandable pool
(await log-agent)                     ; block until queued actions finish (never inside an action)
@log-agent                            ; current value
```

Pick the right dispatch: `send` uses a fixed pool sized for CPU work; `send-off` an expandable pool for blocking I/O. Blocking work sent via `send` starves CPU-bound actions. On failure, an agent enters a failed state and drops further sends unless you set `:error-mode`/`:error-handler`; recover with `restart-agent`. Agents resemble actors (à la BEAM) minus mailboxes and supervision.

## 5. core.async — CSP channels & `go` blocks

For streaming, decoupling producers from consumers, fan-in/fan-out, timeouts, and backpressure, communicate over channels (CSP, in the spirit of Go). Inside `go` blocks use the **parking** ops `<!`/`>!`; on real threads use the **blocking** ops `<!!`/`>!!`.

```clojure
(require '[clojure.core.async :as a])

(let [ch (a/chan 16)]                     ; bounded buffer = backpressure
  (a/go (a/>! ch (work)) (a/close! ch))   ; producer (parks if full)
  (a/go-loop []                           ; consumer
    (when-let [v (a/<! ch)]               ; nil when closed
      (handle v)
      (recur))))
```

Select across channels and add timeouts with `alts!`:

```clojure
(a/go
  (let [[v port] (a/alts! [data-ch (a/timeout 1000)])]
    (if (= port data-ch) (process v) (handle-timeout))))
```

`pipeline` parallelizes a transducer between two channels:

```clojure
(a/pipeline 8 out (map expensive-pure-fn) in)        ; CPU work, n parallel
(a/pipeline-blocking 8 out (map blocking-io) in)      ; blocking work variant
```

**Never block inside a `go` block.** The `go` machinery runs on a small fixed thread pool; a blocking call (`<!!`, JDBC, `Thread/sleep`, file I/O) ties up a dispatch thread and can deadlock everything. For blocking work use `a/thread` (returns a channel), a `future`, `send-off`, or `pipeline-blocking`. Choose buffers deliberately (`a/chan n`, `a/dropping-buffer`, `a/sliding-buffer`) and always `close!` so consumers terminate.

## 6. Futures, promises, delays

```clojure
(def f (future (expensive-computation)))   ; runs on another thread now
@f                                          ; blocks until done
(deref f 1000 :timed-out)                   ; bounded wait with default
(realized? f)                               ; done yet?

(def p (promise))
(future (deliver p (compute)))              ; some thread delivers
@p                                          ; another blocks until delivered

(def d (delay (compute-once)))              ; lazy, memoized
@d                                          ; computes on first deref, caches
```

`future` for fire-and-get-later async work, `promise` for one-shot handoff between threads, `delay` for compute-once-on-demand. (All JVM-only.)

## 7. Parallelism: `pmap`, reducers `fold`

For CPU-bound, coarse-grained work over a collection:

```clojure
(pmap expensive-pure-fn coll)               ; lazy, parallel map; pure fns only
```

`pmap` only pays off when each item is substantial work; for fine-grained items its coordination overhead loses to plain `map`. For large in-memory vectors/maps, reducers do a parallel fork-join fold:

```clojure
(require '[clojure.core.reducers :as r])
(r/fold + (r/map (fn [x] (* x x)) (r/filter even? (vec (range 1000000)))))
```

Reducers shine on contiguous, already-realized structures; they don't help on lazy seqs or small data.

## 8. `volatile!`, transients, dynamic vars

**`volatile!`** — a fast mutable cell with visibility across threads but **no atomicity**. Use only in thread-confined hot loops (e.g., stateful transducers), never for shared coordination.

```clojure
(let [v (volatile! 0)] (vswap! v inc) (vreset! v 10) @v)
```

**Transients** — single-thread mutable build of a persistent structure, then frozen (see functional.md §1). Thread-confined only.

**Dynamic vars** — per-thread context without globals. Declare `^:dynamic`, rebind with `binding` (the binding is visible only on the current thread and its `bound-fn` descendants).

```clojure
(def ^:dynamic *db* nil)
(binding [*db* conn] (run-query *db* q))     ; thread-local override
```

## 9. Worked example: bounded worker pool

Fixed set of workers draining a job channel, results onto another, with backpressure from bounded buffers.

```clojure
(require '[clojure.core.async :as a])

(defn worker-pool
  "Run `n` workers applying pure `f` to items from `in`, results to `out`."
  [n f in out]
  (dotimes [_ n]
    (a/go-loop []
      (when-let [job (a/<! in)]
        (a/>! out (f job))
        (recur))))
  out)

(let [in  (a/chan 64)
      out (a/chan 64)]
  (worker-pool 8 process-job in out)
  (a/go (doseq [j jobs] (a/>! in j)) (a/close! in))   ; feed, then close
  (a/go-loop [acc []]                                  ; collect
    (if-let [r (a/<! out)] (recur (conj acc r)) (deliver result acc))))
```

If `f` blocks, swap the `go-loop` workers for `a/thread` workers (and `<!!`/`>!!`), or use `a/pipeline-blocking`.

## 10. Worked example: pipeline with backpressure

Compose stages as transducers over channels; bounded buffers mean a slow stage naturally throttles upstream.

```clojure
(let [raw    (a/chan 32)
      parsed (a/chan 32 (map parse))           ; transducer on the channel
      valid  (a/chan 32 (filter valid?))]
  (a/pipe raw parsed)
  (a/pipe parsed valid)
  (a/go-loop [] (when-let [x (a/<! valid)] (sink! x) (recur)))
  (a/go (doseq [line (read-lines)] (a/>! raw line)) (a/close! raw)))
```

## 11. Pitfall catalogue

- **Effects inside `swap!`/`alter`/`commute`** → replayed on retry. Compute purely; effect afterward, or via an agent.
- **Effects inside `dosync`** → replayed on transaction retry. `send` to an agent from inside the txn for exactly-once-on-commit.
- **Blocking inside `go`** → starves the dispatch pool, risks deadlock. Use parking ops in `go`; push blocking work to `a/thread`/`future`/`send-off`/`pipeline-blocking`.
- **Atom for a multi-cell invariant** → read/update race. Put the invariant in one atom value, or use refs+STM.
- **`send` for blocking I/O** → starves the CPU pool. Use `send-off`.
- **Unbounded channels** → memory blowup, no backpressure. Choose a buffer; `close!` to end consumers.
- **Ignoring agent error state** → silent drop of sends. Set `:error-mode`/`:error-handler`; `restart-agent`.
- **`pmap` on fine-grained items** → coordination overhead beats the gain. Use plain `map` or batch the work.
- **Holding a lazy seq head across threads / a long computation** → unbounded retention. Force eagerly where bounded.
- **Sharing a `volatile!` or transient across threads** → not safe; both are thread-confined by contract.
- **Reaching for JVM-only constructs on CLJS/Babashka** → agents/STM/`future`/`pmap`/threads are absent. Use atoms + `core.async`.

*— Built by Spacecraft Software —*
