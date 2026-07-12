# Spacecraft Gleam Guidelines — Full Reference

**Version:** 1.1
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the SKILL.md for cases requiring deeper OTP patterns,
supervision design, or testing detail. Load it when the user asks for "complete
guidelines", "supervision tree", "actor skeleton", or when reviewing a real
Gleam service. All examples are hand-written illustrations of public idioms,
verified 2026-07-04 against Gleam 1.17.0, gleam_otp 1.2.0, gleam_erlang 1.3.0,
and gleam_stdlib 1.0.3. `gleam_otp`/`gleam_erlang` broke compatibility at 1.0
(2025-06-12) — reject any pattern from pre-1.0 tutorials.

## Concurrency vs. Performance: When it Helps vs. When it Hurts

BEAM processes are extremely cheap, but they are not free. Concurrency is an architectural decision that must be guided by the workload shape:

1. **When Concurrency Helps (Spawn Processes):**
   - **State Isolation:** When you need a serialized service managing state that changes over time (e.g., an actor maintaining a pool connection or session state).
   - **I/O Parallelism:** Offloading concurrent I/O operations (HTTP queries, file system access).
   - **Failure Isolation:** Running independent tasks (like client connection handlers) in isolated processes so that a crash in one task does not crash other sessions.
2. **When Concurrency Hurts (Keep it Serial / Synchronous):**
   - **Stateless Calculations:** Wrapping pure algorithms (e.g., parsing, string transformations, math) in processes serializes requests. This forces caller processes to block on a single actor's mailbox, turning the actor into a serialized bottleneck.
   - **Trivial/Small Tasks:** The overhead of spawning a process and scheduling it is larger than executing simple synchronous code.
   - **Data Copying Overhead:** BEAM processes share nothing; sending a message copies the payload from the sender's heap to the receiver's heap. While ref-counted binaries >64 bytes are shared via a global heap, large nested terms (large dicts, custom records) suffer significant copying overhead. Keep large data structures in the process that processes them, or use a read-optimized ETS table (via thin FFI modules) only if benchmarking proves a performance gain.

## Memory Safety & Runtime Isolation

Gleam delivers memory safety at two levels: the compile-time type system and the BEAM runtime's process model.

1. **Compile-Time Safety:**
   - **No Nulls/Exceptions:** Gleam eliminates null pointer exceptions by using `Option(Type)` and `Result(Type, Error)` types.
   - **Exhaustive Matching:** The compiler guarantees that all pattern matches (`case` expressions) are total. You cannot compile a program with unhandled variants or missing cases.
   - **Immutability:** All data structures are immutable. This eliminates class-level bugs like data races on shared memory, pointer aliasing, or accidental mutation.
2. **Runtime Isolation (BEAM Heap Model):**
   - **Per-Process Heaps:** Unlike traditional VMs (JVM, V8) that use a single shared heap with a global garbage collector, the BEAM allocates a small private heap for each process.
   - **Non-Blocking GC:** Garbage collection runs per-process. A process only collects its own heap when idle or when its heap is full. This prevents "stop-the-world" pauses from impacting other processes.
   - **Crash Containment:** A process crash terminates only the offending process and its linked children. Memory is reclaimed instantly when the process dies, avoiding VM-wide leaks.
3. **Safe FFI Boundaries:**
   - Since FFI targets (Erlang/JavaScript) bypass Gleam's compiler safety checks, foreign functions must be isolated behind a thin wrapper module.
   - Always catch exceptions thrown in foreign code (using the `exception` package on Erlang or standard try-catch on JavaScript) and map them to Gleam `Result` types.
   - External raw terms must be validated at the boundary using decoders (`gleam/dynamic/decode`) before passing them to the rest of the application.


## 1. Program shape: `main` + root supervisor (the backbone)

Create process names up front, start one root supervisor, park the main
process. Every long-lived process descends from this tree.

```gleam
import gleam/erlang/process
import gleam/otp/static_supervisor as supervisor
// plus your own component modules: import cache / import ingest

pub fn main() -> Nil {
  // Names are atoms: create them ONCE here, pass them down. Never in a loop.
  let cache_name = process.new_name("cache")

  let assert Ok(_started) =
    supervisor.new(supervisor.OneForOne)
    // one_for_one  — restart only the crashed child (default choice)
    // rest_for_one — restart the crashed child and everything started after it
    // one_for_all  — restart all children together (tightly-coupled set)
    |> supervisor.restart_tolerance(intensity: 3, period: 5)
    |> supervisor.add(cache.supervised(cache_name))
    |> supervisor.add(ingest.supervised())
    |> supervisor.start
    as "the root supervision tree must start"

  process.sleep_forever()
}
```

`restart_tolerance` caps a crash loop: more than `intensity` restarts within
`period` seconds kills the supervisor and escalates upward, instead of spinning
a doomed child forever. Nest trees by adding a child supervisor's
`supervisor.supervised(builder)` spec to its parent.

## 2. Typed actor skeleton (serialized state)

The 1.x actor is built with a builder: `actor.new` → `actor.on_message` →
(optionally `actor.named`) → `actor.start`. The handler takes **state first,
message second** and returns `actor.Next`.

```gleam
import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/otp/actor
import gleam/otp/supervision

pub type Message {
  Put(key: String, value: String)
  Get(key: String, reply_to: Subject(Result(String, Nil)))
}

// ---- Client API (runs in the caller) ----

pub fn supervised(
  name: process.Name(Message),
) -> supervision.ChildSpecification(Subject(Message)) {
  supervision.worker(fn() { start(name) })
}

pub fn start(
  name: process.Name(Message),
) -> actor.StartResult(Subject(Message)) {
  actor.new(dict.new())
  |> actor.on_message(handle_message)
  |> actor.named(name)
  |> actor.start
}

pub fn put(cache: Subject(Message), key: String, value: String) -> Nil {
  process.send(cache, Put(key:, value:))
}

pub fn get(cache: Subject(Message), key: String) -> Result(String, Nil) {
  // `call` monitors the actor and PANICS on timeout or actor death —
  // deliberate "let it crash". Pick the timeout consciously.
  process.call(cache, 100, Get(key, _))
}

// ---- Server callback (runs in the actor process) ----

fn handle_message(
  store: Dict(String, String),
  message: Message,
) -> actor.Next(Dict(String, String), Message) {
  case message {
    Put(key, value) -> actor.continue(dict.insert(store, key, value))
    Get(key, reply_to) -> {
      process.send(reply_to, dict.get(store, key))
      actor.continue(store)
    }
  }
}
```

- `actor.start` returns `Result(actor.Started(data), actor.StartError)`;
  `Started` carries `pid` (for monitoring/exit control) and `data` — for
  `actor.new` builders that is the actor's typed `Subject`.
- Because the actor is `named`, callers can also reach it with
  `process.named_subject(name)` — that subject keeps working after a
  supervisor restart re-registers the name. This is how supervised services
  stay reachable without threading fresh subjects back up the tree.
- Handlers run one message at a time. Keep them short; offload slow work to a
  supervised worker so the mailbox never blocks.
- For setup that must run inside the actor process (opening a socket,
  subscribing), use `actor.new_with_initialiser(timeout, fn(subject) { ... })`
  and build the result with `actor.initialised(state)`, chaining
  `actor.returning(subject)` so the actor's `Subject` reaches the parent —
  `initialised` alone returns `Nil` data, which won't type-check against
  `actor.StartResult(Subject(Message))`. If you install a custom selector
  with `actor.selecting`, remember it *replaces* the default one, so re-add
  the actor's own subject.

## 3. Dynamic children: `factory_supervisor`

For children created at runtime (one per connection, job, or device), use the
factory supervisor (gleam_otp ≥ 1.2.0) — one child template, supervised
instances on demand:

```gleam
import gleam/otp/actor
import gleam/otp/factory_supervisor as factory
import gleam/otp/supervision
// plus your own child module: import probe

pub fn start_pool() -> actor.StartResult(factory.Supervisor(String, _)) {
  factory.worker_child(fn(device_id) { probe.start(device_id) })
  // Transient (restart only on abnormal exit) is the factory default,
  // made explicit here; use supervision.Permanent for always-on children.
  |> factory.restart_strategy(supervision.Transient)
  |> factory.start
}

// Later, on demand — each child is supervised from birth:
// let assert Ok(probe) = factory.start_child(pool.data, "thermal-04")
```

The factory is your back-pressure and recovery point: crashed children restart
per the configured `supervision.Restart` policy (`Transient` is the factory
default — restarted only after abnormal termination; set `Permanent` for
children that must always run), and there is never an unsupervised
orphan. Do **not** hand-roll this with bare `process.spawn` per request.

## 4. `Result` pipelines with `use` (multi-step happy paths)

```gleam
import gleam/result

pub type CheckoutError {
  CartNotFound
  CartEmpty
  PaymentFailed(code: Int)
}

pub fn checkout(
  cart_id: Int,
  payment: Payment,
) -> Result(Order, CheckoutError) {
  use cart <- result.try(fetch_cart(cart_id))
  use _ <- result.try(ensure_not_empty(cart))
  use charge <- result.try(
    charge_card(payment, cart.total)
    |> result.map_error(PaymentFailed),
  )
  create_order(cart, charge)
}
```

Any `Error` short-circuits and is returned as-is; the code reads top-to-bottom
like the happy path. Design error types deliberately: **libraries** define a
precise custom error union per module (as above) so callers can match on it;
**applications** may use `snag` for ad-hoc errors that accumulate human-readable
context. Convert between lanes at the edges with `result.map_error`,
`option.to_result`, and `result.replace_error`.

## 5. Decoding dynamic data at the boundary

Everything external — JSON, FFI returns, Erlang messages — arrives as
`Dynamic`. Decode it immediately into a typed value; never pass `Dynamic`
deeper into the program. The current API is `gleam/dynamic/decode`
(`dynamic.decode1..9` no longer exist):

```gleam
import gleam/dynamic/decode
import gleam/json

pub type Reading {
  Reading(sensor: String, kelvin: Float, seq: Int)
}

pub fn reading_decoder() -> decode.Decoder(Reading) {
  use sensor <- decode.field("sensor", decode.string)
  use kelvin <- decode.field("kelvin", decode.float)
  use seq <- decode.field("seq", decode.int)
  decode.success(Reading(sensor:, kelvin:, seq:))
}

pub fn parse(input: String) -> Result(Reading, json.DecodeError) {
  json.parse(from: input, using: reading_decoder())
}
```

Use `decode.optional_field` for absent-able keys, `decode.one_of` for variant
payloads, and `decode.then` for value-dependent decoding. Decoders compose —
build small ones per type and reuse them.

## 6. Recursion, accumulators & string building

Gleam has no loops; iteration is recursion — but reach for `gleam/list`
(`map`, `filter`, `fold`, `try_fold`…) before writing recursion by hand. When
you do recurse, only **tail** calls are optimised. The house pattern is a
public wrapper around a private tail-recursive helper with an accumulator:

```gleam
pub fn factorial(x: Int) -> Int {
  factorial_loop(x, 1)
}

fn factorial_loop(x: Int, accumulator: Int) -> Int {
  case x {
    0 | 1 -> accumulator
    _ -> factorial_loop(x - 1, accumulator * x)
  }
}
```

Non-tail recursion grows the stack — a crash risk on deep data, especially on
the JavaScript target. Lists are singly-linked: prepend `[x, ..rest]` is O(1),
append and `list.length` are O(n) — build backwards then `list.reverse`. Build
accumulated strings with `gleam/string_tree` (constant-time append) rather
than repeated `<>` inside a loop — naive concatenation risks O(n²) copying,
though the BEAM's binary-append optimisation often mitigates it, so benchmark
hot paths before hand-optimising (Standard §3.2). Remember the arithmetic
footguns: `Int` and
`Float` have separate operators (`+` vs `+.`), and `/` on zero silently yields
`0` — use `int.divide`/`float.divide` (→ `Result`) when the divisor may be zero.

## 7. FFI at the boundary

Reach for FFI only when no Gleam package exists. Annotate the head exactly —
the compiler trusts you — and keep foreign calls behind one module that
converts failures to `Result` before they escape:

```gleam
import exception
import gleam/dynamic.{type Dynamic}
import gleam/dynamic/decode
import gleam/result

pub type RegisterError {
  HardwareFault
  MalformedRegister
}

// A thin, private, exactly-typed foreign call…
@external(erlang, "telemetry_ffi", "read_register")
fn read_register_raw(address: Int) -> Dynamic

// …wrapped so exceptions and bad data become Results at the boundary.
pub fn read_register(address: Int) -> Result(Int, RegisterError) {
  use raw <- result.try(
    exception.rescue(fn() { read_register_raw(address) })
    |> result.replace_error(HardwareFault),
  )
  decode.run(raw, decode.int)
  |> result.replace_error(MalformedRegister)
}
```

Cross-target packages provide per-target implementations with
`@target(erlang)` / `@target(javascript)` (or per-target `@external` heads
with a Gleam fallback body). On the JavaScript target remember there is no
supervision: FFI code that throws will take down the whole runtime unless
caught at this boundary.

## 8. Testing: gleeunit + `assert`, qcheck, birdie

`gleam test` runs `test/<package>_test.gleam`'s `main`; gleeunit discovers
every `pub fn *_test`. Dev dependencies: `gleam add --dev gleeunit@1 qcheck
birdie`. Assert with the `assert` keyword — `gleeunit/should` is deprecated:

```gleam
import gleam/erlang/process
import gleeunit
// plus the module under test: import cache

pub fn main() -> Nil {
  gleeunit.main()
}

pub fn put_then_get_test() {
  let name = process.new_name("cache_test")
  let assert Ok(started) = cache.start(name)

  cache.put(started.data, "mode", "orbit")
  assert cache.get(started.data, "mode") == Ok("orbit")
  assert cache.get(started.data, "absent") == Error(Nil)
}
```

Property tests with `qcheck` (runs under gleeunit; shrinking included) for
parsers, codecs, and invariants:

```gleam
import qcheck

pub fn encode_decode_roundtrip_test() {
  use reading <- qcheck.given(reading_generator())
  assert parse(encode(reading)) == Ok(reading)
}
```

Snapshot tests with `birdie` for rendered output (review queue via
`gleam run -m birdie`). Test failure paths, not just happy paths — an
`Error(...)` return is part of the contract. For actor tests, start the actor
inside the test with a test-local `Name` so parallel tests never collide.

## 9. CI recipe

```yaml
- uses: erlef/setup-beam@v1
  with:
    otp-version: "27"
    gleam-version: "1.17.0"
    rebar3-version: "3"
- run: gleam deps download
- run: gleam format --check src test
- run: gleam build --warnings-as-errors            # the only place this flag exists
- run: gleam build --warnings-as-errors --target javascript   # if dual-target
- run: gleam test
```

`gleam check`/`gleam test` cannot fail on warnings, so the dedicated
`build --warnings-as-errors` step per supported target is mandatory — compiler
warnings are Gleam's linter. Ship with `gleam export erlang-shipment`
(self-contained directory, `./entrypoint.sh run`) or `gleam export escript`
(single file; target machine still needs Erlang installed).

## 10. Common pitfalls & fixes

| Pitfall                                  | Symptom                              | Fix                                                      |
|------------------------------------------|--------------------------------------|----------------------------------------------------------|
| Pre-1.0 gleam_otp API (`actor.start(state, fn)`, `gleam/otp/task`) | Doesn't compile on 1.x               | Builder API: `actor.new \|> on_message \|> start`; no task module |
| Pure logic inside an actor               | Serialized bottleneck                | Plain module of functions                                |
| Unbounded `process.spawn` per request    | Orphaned processes, no recovery      | `factory_supervisor` / fixed supervised pool             |
| Slow work in `on_message`                | Mailbox stalls; callers' `call` panics | Offload to a supervised worker                           |
| `let assert` on expected failure         | Crash instead of recoverable error   | Return `Result`; reserve `let assert` for impossible states |
| `process.new_name` in a loop / per restart | Atom-table exhaustion → VM death     | Create names once at startup; pass them down             |
| Catch-all `_` on own custom types        | New variants silently unhandled      | Match every variant; let the compiler find change sites  |
| Repeated `<>` in a loop                  | Worst-case O(n²) copying             | `string_tree`, convert once at the end; benchmark hot paths |
| `/` with a possibly-zero divisor         | Silent `0`, corrupted math           | `int.divide` / `float.divide` (return `Result`)          |
| `dynamic.decodeN` / `gleeunit/should`    | Removed / deprecated APIs            | `gleam/dynamic/decode`; the `assert` keyword             |
| `echo` left in committed code            | `gleam publish` refuses the package  | Remove it; use the `logging` package for real logging    |
| BEAM habits on the JavaScript target     | One panic kills the whole runtime    | No supervision on JS — catch at boundaries; `gleam_javascript` promises |

## 11. Code-review mandate

Any Gleam change must pass:
1. `gleam format --check src test`
2. `gleam build --warnings-as-errors` (every supported target)
3. `gleam test` (gleeunit + qcheck properties where invariants exist)
4. Manual review of supervision strategy, restart tolerance, message protocol,
   and the `Result`-vs-crash boundary of every actor

**This skill ensures every Gleam process written at Spacecraft Software is
supervised, type-safe, fault-tolerant, and scales across all BEAM schedulers.**
