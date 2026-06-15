<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
-->

# Functional Clojure

Reference for non-trivial functional Clojure. Assumes Clojure 1.12+ on the JVM; notes mark anything platform-specific.

## Contents

1. Immutability & persistent data structures
2. Pure functions & function building
3. Threading macros
4. Destructuring
5. `reduce`, `transduce` & transducers
6. Recursion: `loop`/`recur`, the no-TCO rule, `trampoline`
7. Lazy sequences & their traps
8. Polymorphism: protocols, records, multimethods
9. Validation with `clojure.spec`
10. Error handling with `ex-info`

---

## 1. Immutability & persistent data structures

Maps, vectors, sets, and lists are immutable and persistent: "modifying" one returns a new structure that shares structure with the old, so it's cheap (O(log32 n)) and the old value stays valid for any reader. This is the foundation of Clojure's concurrency safety — share freely, no copying, no locks for reads.

```clojure
(def v [1 2 3])
(conj v 4)        ; => [1 2 3 4]   ;; v is still [1 2 3]
(assoc {:a 1} :b 2)               ; => {:a 1 :b 2}
(update {:n 1} :n inc)            ; => {:n 2}
(assoc-in {} [:a :b] 1)          ; => {:a {:b 1}}
(update-in {:a {:b 1}} [:a :b] inc)
(get-in {:a {:b 1}} [:a :b])     ; => 1
```

Prefer the data literals (`{}` `[]` `#{}`) and these operators over building structures imperatively. For the rare hot path where you build a large structure in one local scope, use **transients** (§5 of concurrent.md covers `volatile!`; transients are the single-thread-build counterpart):

```clojure
(persistent!
  (reduce conj! (transient []) (range 1000000)))   ; thread-confined, then frozen
```

Never leak a transient across threads or read it after `persistent!`.

## 2. Pure functions & function building

Keep functions referentially transparent: same inputs → same outputs, no observable effects. Build behavior by composing small functions.

```clojure
(def pipeline (comp inc #(* % 2)))   ; right-to-left: (inc (* x 2))
(pipeline 3)                         ; => 7

(partial + 10)                       ; => fn that adds 10
(map (partial * 2) [1 2 3])          ; => (2 4 6)

(complement even?)                   ; => odd?-ish predicate
(every-pred pos? even?)              ; => true only if both hold
(some-fn :a :b)                      ; => first truthy of (:a m) (:b m)

(fnil + 0)                           ; => + with nil arg replaced by 0
(update m :count (fnil inc 0))       ; safe increment when key absent
```

## 3. Threading macros

Threading turns nested calls into a top-to-bottom pipeline. Choose by where the value goes:

- `->` (thread-first): inserts the value as the **first** argument. Use for maps/objects.
- `->>` (thread-last): inserts as the **last** argument. Use for sequences.
- `as->`: bind to a name when arg position varies.
- `some->` / `some->>`: short-circuit to `nil` if any step yields `nil`.
- `cond->` / `cond->>`: conditionally apply steps.

```clojure
(-> {:a {:b 5}} :a :b inc)                     ; => 6
(->> (range 10) (filter even?) (map #(* % %))  ; => [0 4 16 36 64]
     (into []))
(as-> 5 $ (+ $ 1) (* 2 $) (- $ 3))             ; => 9
(some-> req :session :user :id)                ; nil-safe drill-down
(cond-> {} verbose? (assoc :log true)
            dry-run? (assoc :commit false))
```

## 4. Destructuring

Pull pieces out at the binding site instead of repeated accessors. Works in `let`, `fn` params, `defn`, `for`, etc.

```clojure
;; sequential
(let [[a b & rest] [1 2 3 4]] [a b rest])      ; => [1 2 (3 4)]

;; associative
(let [{:keys [x y] :or {y 0} :as point} {:x 3}]
  [x y point])                                  ; => [3 0 {:x 3}]

;; namespaced keys
(let [{:user/keys [id name]} {:user/id 1 :user/name "Ada"}]
  [id name])

;; nested
(let [{[first-item] :items} {:items [:a :b]}] first-item)  ; => :a
```

Destructure in function signatures to document the expected shape:

```clojure
(defn make-order [{:keys [items customer] :or {items []}}] ...)
```

## 5. `reduce`, `transduce` & transducers

`reduce` is the universal fold. Reach for it before hand-rolling accumulation.

```clojure
(reduce + 0 [1 2 3 4])                          ; => 10
(reduce (fn [acc x] (assoc acc x (* x x))) {} (range 4))  ; => {0 0 1 1 2 4 3 9}
(reduced acc)                                   ; early-terminate a reduce
```

**Transducers** are composable, collection-independent transformation recipes. A transducer made from `map`/`filter`/`take`/etc. (called *without* a collection) can be reused across `into`, `sequence`, `transduce`, `eduction`, and `core.async` channels — and they fuse, so there are **no intermediate sequences**.

```clojure
(def xf (comp (filter odd?) (map #(* % %)) (take 3)))
(into [] xf (range 100))         ; => [1 9 25]      eager, into a vector
(sequence xf (range 100))        ; => (1 9 25)      lazy
(transduce xf + 0 (range 100))   ; => 35            fold with a reducing fn
(eduction xf (range 100))        ; deferred, re-iterable view
```

Note transducer composition reads **left-to-right** (unlike `comp` of plain fns), because each transducer wraps the next reducing step.

For CPU-bound parallel folds over large vectors/maps, see reducers `fold` in concurrent.md.

## 6. Recursion: `loop`/`recur`, the no-TCO rule, `trampoline`

The JVM does not optimize general tail calls. A self-call in tail position must use `recur`, which reuses the stack frame; otherwise large inputs overflow the stack.

```clojure
;; explicit accumulator, constant stack
(defn sum [coll]
  (loop [c coll, acc 0]
    (if (seq c)
      (recur (rest c) (+ acc (first c)))
      acc)))

;; recur also rebinds defn params directly
(defn count-down [n]
  (when (pos? n)
    (println n)
    (recur (dec n))))
```

`recur` only works in tail position and only to the nearest `loop`/`fn`. For **mutual** recursion that must not grow the stack, return thunks and drive them with `trampoline`:

```clojure
(defn evn? [n] (if (zero? n) true  #(odd?  (dec n))))
(defn odd? [n] (if (zero? n) false #(evn? (dec n))))
(trampoline evn? 1000000)
```

In practice, prefer `reduce`/`map`/`filter`/`into` to manual recursion whenever the shape allows.

## 7. Lazy sequences & their traps

Many core seq fns (`map`, `filter`, `take`, `iterate`, `range` without args) are lazy: nothing is computed until consumed. This enables infinite sequences and pipelines, but has two classic hazards.

**Head retention.** If you hold a reference to the head of a lazy seq while walking it, the whole realized sequence is retained in memory. Let the head go out of scope, or use eager forms.

**Effects in lazy seqs fire unpredictably.** Side effects placed in a `map`/`for` body may run zero, partial, or multiple times depending on realization. Rules:

```clojure
;; transform → eager when you need the whole thing:
(mapv f coll)   (into [] (map f) coll)   (doall (map f coll))

;; side effects → never via lazy map; use these:
(run! println coll)          ; effects, returns nil
(doseq [x coll] (do-stuff x)) ; effects with multiple bindings/loops
(dorun (map f coll))         ; force a lazy seq for effects, discard
```

`for` is a lazy list comprehension (not a loop); use `doseq` when you want effects.

## 8. Polymorphism: protocols, records, multimethods

Default to plain maps. Add these only when you need dispatch or a performance-critical type.

**Protocols + records** — fast, type-based polymorphism (like interfaces):

```clojure
(defprotocol Shape
  (area [s])
  (perimeter [s]))

(defrecord Circle [r]
  Shape
  (area [_] (* Math/PI r r))
  (perimeter [_] (* 2 Math/PI r)))

(area (->Circle 2))     ; => 12.566...
```

Records are maps with a fixed key set and a concrete type; they keep map semantics (`assoc`, `:keys`, etc.) but dispatch protocols fast. Use `deftype` only for low-level mutable/host-interop types.

**Multimethods** — open, value-based dispatch on an arbitrary function of the args:

```clojure
(defmulti handle :event/type)                  ; dispatch on a map key
(defmethod handle :click [e] ...)
(defmethod handle :scroll [e] ...)
(defmethod handle :default [e] ...)
```

Multimethods win when dispatch depends on data (not type) or must stay open to extension; protocols win on speed and single-type dispatch.

## 9. Validation with `clojure.spec`

Describe data shapes as specs, validate at boundaries, and get generated test data and clear failure explanations.

```clojure
(require '[clojure.spec.alpha :as s])

(s/def :user/id   pos-int?)
(s/def :user/name (s/and string? (complement clojure.string/blank?)))
(s/def :user/user (s/keys :req [:user/id :user/name]))

(s/valid? :user/user {:user/id 1 :user/name "Ada"})   ; => true
(s/explain-str :user/user {:user/id 0})               ; human-readable failure
(s/conform (s/cat :op keyword? :args (s/* any?)) [:add 1 2])

;; instrument function args in dev/test:
(s/fdef charge :args (s/cat :amount pos-int?) :ret map?)
```

Keep spec at the edges (incoming requests, config, public fns); don't litter pure interior code with it.

## 10. Error handling with `ex-info`

Carry structured context on exceptions rather than parsing message strings.

```clojure
(throw (ex-info "Payment declined"
                {:type :payment/declined :order-id 42 :amount 1999}))

(try
  (charge order)
  (catch clojure.lang.ExceptionInfo e
    (let [{:keys [type order-id]} (ex-data e)]
      (log/warn (ex-message e) type order-id)
      (retry-or-fail type)))
  (finally (release-resources!)))
```

Use `with-open` for anything `Closeable` so it's released even on throw:

```clojure
(with-open [r (io/reader path)]
  (doall (line-seq r)))            ; force inside the scope; never return a lazy seq over a closed reader
```

Note the head-retention/lazy interaction above: forcing with `doall` inside `with-open` is required, or the reader closes before the seq is realized.

*— Built by Spacecraft Software —*
