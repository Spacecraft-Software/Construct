---
name: spacecraft-clojure-guidelines
description: Write idiomatic, functional, safe-by-construction concurrent Clojure. Use whenever the user is writing, reviewing, refactoring, or debugging Clojure — any .clj, .cljs, .cljc, .bb, or .edn file, or any mention of Clojure, ClojureScript, or Babashka. Trigger on Clojure concurrency (atoms, refs/STM, dosync, agents, core.async, channels, futures, pmap, reducers), functional patterns (immutability, persistent data structures, threading macros, destructuring, reduce/transduce/transducers, loop/recur, lazy seqs), and abstraction features (defrecord, defprotocol, defmulti, clojure.spec, ex-info). Also trigger on implicit requests like 'make this Clojure thread-safe', 'write a core.async pipeline', 'port this to transducers', or 'fix this STM transaction'. Do NOT trigger for other Lisps — Common Lisp, Scheme, or GNU Guile (use spacecraft-guile-guidelines) — or plain Java/Kotlin; Clojure's reference types and idioms differ and generic Lisp or FP advice gets them wrong. Prefer this over generic FP advice for Clojure.
license: GPL-3.0-or-later
metadata:
  maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
  website: https://Construct.SpacecraftSoftware.org/
---

# Clojure: Functional & Safe-Concurrent Programming

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

Write Clojure that is **functional by default** and **safe-concurrent by construction**. This skill encodes the decisions an experienced Clojurist makes automatically so you don't fall back on generic Lisp or imperative habits that miss Clojure's specific model and pitfalls.

Assume **Clojure 1.12+ on the JVM** unless told otherwise. The same core ideas apply to **ClojureScript** (single-threaded event loop — `core.async` works, but JVM-only threads/STM/agents do not; see notes inline) and to **Babashka** (`.bb` scripting — fast-start subset, `core.async` and atoms available, no agents/STM). `.cljc` is for code shared across platforms via reader conditionals. Jank (native Clojure) is emerging; treat it as JVM-semantics-compatible unless told otherwise.

## The central idea: identity vs. state

Clojure's safety story is *structural*, not disciplinary. **Values are immutable; a reference points to a succession of immutable values over time.** Readers never see a torn or half-updated value, so concurrent reads need no locks and no defensive copying, and whole classes of data races simply cannot be expressed. This is the same goal you pursue in Rust — eliminating data races at the design level — reached by a different mechanism: persistent immutable data plus a small set of well-defined reference types, instead of ownership and borrowing. Internalize this and most "how do I make it thread-safe" questions answer themselves: you don't guard the data, you choose the right reference type for *how* it changes.

## Core stance

1. **Immutability is the substrate.** Default to persistent maps, vectors, and sets. Never reach for a mutable container to "go faster" until a profiler says so — and then reach for `transient`/`volatile!` in a thread-confined scope, not shared mutation.
2. **Pure first; effects at the edges.** A function that both computes a value *and* performs I/O or mutation is two functions wearing a trenchcoat — split them. This matters doubly under concurrency: the functions you hand to `swap!`/`alter`/`commute` are **retried**, so any side effect inside them runs an unpredictable number of times.
3. **Right reference type for the job.** Choose along two axes — *coordinated vs. uncoordinated* (must several things change together atomically?) and *synchronous vs. asynchronous* (does the caller wait?). Picking an atom where you needed a transaction is the single most common real concurrency bug. Run the decision tree below.
4. **Data-oriented design.** Model the domain as plain data (maps/vectors/sets keyed with namespaced keywords), then write functions over that data. Validate shape at boundaries with `clojure.spec`. Reach for `defprotocol`/`defrecord`/`defmulti` only when you genuinely need polymorphism or a performance-critical type — not as a reflex from OO.
5. **`recur` and seq combinators over hand recursion.** The JVM has no general tail-call optimization; only `recur` (and `trampoline` for mutual recursion) guarantee a constant stack. For everything else prefer `reduce`/`transduce`/`map`/`filter` to manual loops — they're clearer and harder to get wrong.

## Decision tree: which concurrency primitive?

```
One independent piece of state, synchronous update?      → atom         (swap! / reset! / compare-and-set!)
Several pieces that must change together atomically?     → refs + STM   (dosync / alter / commute / ensure)
Independent state, async update, side effects are fine?  → agent        (send / send-off)         [JVM only]
Streaming, pipelines, backpressure, fan-in/fan-out?      → core.async   (chan / go / <! >! / alts! / pipeline)
One-shot async result you'll deref later?                → future + @   or promise               [future = JVM]
Parallel map over a collection (coarse-grained work)?    → pmap / future / reducers fold          [JVM only]
Per-thread/per-call dynamic context without globals?     → dynamic var + binding (^:dynamic)
Fast, thread-confined mutable cell (no coordination)?    → volatile! / transient
```

When in doubt for **shared state**, start with an `atom` and only escalate to refs+STM when an invariant spans more than one cell. When in doubt for **communication and flow**, reach for `core.async` rather than building your own queues. On **ClojureScript/Babashka**, prefer atoms and `core.async`; agents, STM, `future`, and `pmap` are JVM-only.

## Workflow

Before writing Clojure:

1. **Decide the shape of the data first.** What maps/vectors flow through the system, keyed by what? Data-oriented design means the schema comes before the functions.
2. **Sequential or concurrent?** If concurrent, run the decision tree and name the reference type(s) explicitly before coding.
3. **Inventory the effects.** List every side effect (I/O, mutation, randomness, time, logging). Plan to isolate them at the edges so the core stays pure and testable — and so nothing side-effecting ends up inside a retried `swap!` or `dosync`.
4. **Load the matching reference file.** Read the relevant file in `references/` for exact APIs, idioms, and gotchas. Don't write Clojure concurrency from memory of generic FP — the retry semantics and the JVM-only constraints are easy to get subtly wrong.

## Reference files

Read the file that matches the task. Both are concise; read them in full when relevant.

- **`references/functional.md`** — Immutability and persistent data structures, pure functions, threading macros (`->` `->>` `as->` `some->` `cond->`), destructuring, `reduce`/`transduce` and transducers, `loop`/`recur` and the JVM no-TCO rule, lazy sequences and their traps, `defrecord`/`defprotocol`/`defmulti` polymorphism, `clojure.spec` validation, and error handling with `ex-info`/`ex-data`. Read this for any non-trivial functional Clojure.
- **`references/concurrent.md`** — The safe-concurrency core: atoms, refs + STM, agents, `core.async` (CSP), `future`/`promise`, `pmap`/reducers `fold`, `volatile!`/`transient`, and dynamic vars. Includes worker-pool and pipeline examples and the full list of concurrency pitfalls (effects in retried CAS loops and transactions, blocking inside `go`, lazy-seq head retention). Read this for anything concurrent.

## Non-negotiable idioms (quick reference)

Common enough to inline. For anything beyond these, read the reference files.

**Threading for data pipelines** — read top-to-bottom instead of inside-out. `->` threads as the *first* arg (objects/maps), `->>` as the *last* arg (sequences):
```clojure
(->> users (filter :active?) (map :email) (into #{}))     ; seq pipeline
(-> request :params (get "id") parse-long)                ; map drill-down
```

**Atom with a pure update fn** — the fn may be retried under contention, so keep it side-effect-free:
```clojure
(def state (atom {:count 0}))
(swap! state update :count inc)        ; pure: safe to retry
;; side effect AFTER the swap, never inside it
(when (= 100 (:count @state)) (notify!))
```

**Coordinated change across cells** — anything touching more than one ref goes in one `dosync`:
```clojure
(dosync
  (alter from - amount)
  (alter to   + amount))               ; both commit atomically, or neither
```

**core.async skeleton** — communicate over channels; never block inside a `go`:
```clojure
(require '[clojure.core.async :as a])
(let [ch (a/chan 16)]                  ; bounded buffer = real backpressure
  (a/go (a/>! ch (work)))
  (a/go (handle (a/<! ch))))
```

**Destructuring at the binding site** — pull fields out declaratively rather than repeated `get`:
```clojure
(defn greet [{:keys [first-name last-name] :or {last-name ""}}]
  (str "Hi " first-name " " last-name))
```

**Transducers** — compose transformations once, reuse across `into`/`sequence`/`transduce`/channels, with no intermediate collections:
```clojure
(def clean (comp (filter :active?) (map :email) (distinct)))
(into #{} clean users)
(a/chan 16 clean)                      ; same xform, applied to a channel
```

## Style rules

- **Naming.** `kebab-case` for everything. Predicates end in `?` (`even?`, `active?`). Side-effecting / unsafe / mutating fns end in `!` (`swap!`, `reset!`, `delete-file!`). Dynamic vars wear "earmuffs" (`*out*`, `*db*`). Use **namespaced keywords** (`::user/id`) for map keys that cross a boundary.
- **One job per function.** Small, composable, named for what they return. Push branching into `cond`/`case`/`condp` or multimethods rather than deep `if` nests.
- **Validate at the edges.** Use `:pre`/`:post` conditions or `clojure.spec` (`s/fdef` + `instrument` in dev/test) at system boundaries; trust pure interior functions.
- **`ns` form order**, per Stuart Sierra's "how to ns": `:refer-clojure`, then `:require` (alias everything — `[clojure.string :as str]`, avoid bare `:refer :all`), then `:import`. Keep requires sorted.
- **Formatting.** The recommended house formatter is **Standard Clojure Style** (`standard-clj`) — zero-config, `gofmt`-style, pretty-prints and sorts `ns` forms. Run `standard-clj check` in CI and `standard-clj fix` locally. (It's ISC-licensed tooling — fine to depend on, do not vendor its source into GPL projects.)
- **Comments.** `;` trailing/inline, `;;` block within a form, `;;;` section header, `;;;;` file header. Disable a form with the `#_` reader macro or wrap exploratory code in `(comment ...)`. Use `tap>` (with a tap like Portal/Reveal) for non-intrusive debugging over scattered `println`.

## Common pitfalls to actively avoid

- **Side effects inside `swap!`/`alter`/`commute`** → they run multiple times when the CAS/transaction retries under contention. Compute purely inside; perform the effect after the update returns, or hand it to an agent.
- **Side effects inside `dosync`** → the whole transaction replays on conflict, repeating the effect. The idiomatic escape hatch: `send` it to an agent from inside the transaction — agent actions dispatched during a transaction are *deferred until commit* and fire exactly once.
- **Blocking inside a `go` block** (`<!!`, JDBC calls, `Thread/sleep`) → starves the small fixed `go` dispatch pool and can deadlock the whole system. Use parking `<!`/`>!` inside `go`; for real blocking work use `a/thread`, a `future`, or `send-off`.
- **Using an atom for a multi-key invariant** (read A, then update B based on it) → races between the read and the write. Either put the whole invariant inside *one* atom's value and update it in a single `swap!`, or use refs + STM.
- **Holding the head of a lazy seq** → forces and retains the entire realized sequence in memory. And lazy seqs are the wrong tool for side effects (they realize unpredictably) — use `doseq`/`run!` for effects, `doall`/`dorun` to force, `mapv`/`into` for eager transforms.
- **`send` vs `send-off` mixup** → blocking work on `send` (bounded CPU pool) starves CPU-bound actions; route blocking I/O through `send-off` (expandable pool).
- **Unbounded `core.async` channels** → unbounded memory and no backpressure. Choose a buffer deliberately (fixed, `dropping-buffer`, `sliding-buffer`) and always `close!` so consumers terminate.
- **Ignoring agent failure state** → an uncaught exception in an action puts the agent in a failed state and silently drops further sends. Set `:error-handler`/`:error-mode`; recover with `restart-agent`.
- **Reflection on hot paths** → silent performance cliffs. Set `(set! *warn-on-reflection* true)` and add type hints (`^long`, `^String`) where warned.
- **Reaching for ClojureScript/Babashka with JVM-only constructs** → agents, refs/STM, `future`, `pmap`, and real threads don't exist there. Use atoms and `core.async`.

*— Built by Spacecraft Software —*
