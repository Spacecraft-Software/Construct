---
name: spacecraft-gleam-guidelines
description: Use for writing type-safe fault-tolerant highly-concurrent Gleam code on the BEAM following Spacecraft Software standards. Triggers on any request involving Gleam, .gleam files, gleam.toml, the gleam CLI (build, check, test, format, add, publish, export), gleam_stdlib, typed OTP — gleam_otp actors, static_supervisor, factory_supervisor, supervision child specs — gleam_erlang processes, Subjects, selectors, named processes, Result/Option error handling, use expressions, exhaustive case, opaque types, @external FFI, the JavaScript target, or gleeunit/qcheck/birdie testing. Trigger even when implicit, e.g. "write a typed actor", "port this GenServer to Gleam", "supervise this Gleam process", or "make this Gleam code faster". Do NOT trigger for Erlang (spacecraft-erlang-guidelines) or Elixir (spacecraft-elixir-guidelines) — Gleam's typed actor and supervision APIs differ sharply from raw OTP, and generic BEAM advice gets them wrong. By Mohamed Hammad and Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Gleam Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Gleam engineer at Spacecraft Software specializing in type-safe, fault-tolerant, massively-concurrent BEAM systems.** Always follow these rules when writing or reviewing Gleam code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Gleam layers a sound static type system on the BEAM's fault tolerance — two complementary safety nets. Model *expected* failures as `Result` values the compiler forces callers to handle; let *unexpected* faults crash the process and let its supervisor restart it. Never blur the two lanes.
- **Then Performance (Priority 2).** Concurrency is the default lever — cheap isolated BEAM processes, one scheduler per core, share-nothing message passing through typed `Subject`s. Avoid process bottlenecks; keep stateless operations synchronous.
- **Types are the design tool.** No nulls, no exceptions, exhaustive `case`. Make invalid states unrepresentable with custom types and `opaque` constructors instead of validating at every call site.
- **Immutable, functional core.** All data is immutable; transform with pure functions, pattern matching, and pipelines. Keep side effects at the process boundary.
- **Target discipline.** The Erlang target is the Spacecraft default for services — OTP, supervision, and `gleam_otp` exist only there. The JavaScript target has **no supervision**: a panic kills the whole runtime, so "let it crash" is Erlang-target advice only.
- **Current APIs only.** `gleam_otp` and `gleam_erlang` had breaking 1.0 redesigns (2025-06-12). Verify against hexdocs for the pinned version — most older tutorials (and model memory) show APIs that no longer exist.

## Memory Safety & Type Guarantees
- **Compile-Time Soundness:** Gleam's compiler enforces type-safe boundaries. It contains no `null` or `nil` values (use `Option` or `Result`), requires exhaustive matching on `case` patterns (no unhandled paths), and prevents mutable state bugs.
- **Process Heap Isolation:** On the BEAM, every process has its own private heap. Garbage collection runs per-process and does not block the entire VM ("stop-the-world"). A crash or memory leak in one process is isolated and does not affect the rest of the application.
- **FFI Protection:** FFI allows unsafe operations to escape compiler checks. Always wrap foreign calls in thin, safe wrappers, decode untyped data immediately using `gleam/dynamic/decode`, and convert Erlang/JS exceptions/throws into Gleam `Result`s.

## Concurrency vs. Performance Tradeoffs
- **When Concurrency Helps (Do Spawn):**
  - Stateful services that require serialized updates (e.g. database connections, user sessions).
  - Highly concurrent independent I/O tasks (e.g. parallel HTTP requests or parallel file operations).
  - Isolating failure domains so a crash in one connection does not take down others.
- **When Concurrency Hurts (Do NOT Spawn):**
  - Stateless calculations: Wrapping stateless logic in an actor serializes requests, introducing a CPU bottleneck and overhead from message passing.
  - Trivial tasks: The overhead of spawning a process and copying data exceeds execution time for small computations.
  - Large data transfer: BEAM copies message payloads between process heaps (except ref-counted binaries >64 bytes). For large terms, keep data process-local or use read-optimized ETS only if benchmarked.


## Mandatory Abstraction Choice
Always match the abstraction to the workload — and **prefer a plain module of pure functions when no state or isolation is needed** (do not wrap pure logic in a process):

- **Serialized stateful service**: an actor via the 1.x builder — `actor.new(state) |> actor.on_message(handler) |> actor.start`. The handler is `fn(state, message) -> actor.Next(state, message)`; return `actor.continue(new_state)`, `actor.stop()`, or `actor.stop_abnormal(reason)`. Keep handlers short — messages are processed one at a time.
- **Request/reply**: put a `reply_to: Subject(reply)` field in the message variant; clients use `process.call(subject, timeout, Constructor(args, _))` — the hole is where `call` injects its reply subject. `call` **panics** on timeout or callee death — that is deliberate "let it crash" (the old `try_call` was removed); pick timeouts consciously.
- **Lifecycle & recovery**: `gleam/otp/static_supervisor` — `supervisor.new(strategy)` (`OneForOne` / `RestForOne` / `OneForAll`) `|> supervisor.add(child_spec) |> supervisor.start`, with `restart_tolerance(intensity:, period:)` tuned. Every long-lived process descends from the root supervisor started in `main`. Compose trees by exposing a `supervised()` child spec (`supervision.worker` / `supervision.supervisor`, restart `Permanent` / `Transient` / `Temporary`) from each component module.
- **Dynamic children at runtime**: `gleam/otp/factory_supervisor` (gleam_otp ≥ 1.2.0) — a supervised factory over one child template, `factory.start_child(sup, argument)` per child. This replaces ad-hoc per-request spawning.
- **Discovery across restarts**: create `process.Name`s with `process.new_name(prefix)` **once at program start**, pass them down, register actors with `actor.named(name)`, reach them via `process.named_subject(name)`. Names are atoms — never create them dynamically (the atom table is finite and not GC'd; exhaustion kills the VM).
- **Protocol / state machine**: an ADT state inside an actor (`gleam_otp` has no `gen_statem` module) — one custom-type variant per protocol state, exhaustive `case` per event.
- **Fire-and-forget side task**: `process.spawn` (linked) under something supervised; `process.spawn_unlinked` only with a monitor and a written reason. There is **no `gleam/otp/task` module in 1.x** — it was removed.
- **Shared read-heavy data**: keep it in an actor and hand out copies; there is no first-party typed ETS. Raw ETS/`persistent_term` via Erlang FFI is an escape hatch that forfeits type safety — isolate it behind one module and document the trade-off (Standard §3.2).

Never spawn an unsupervised, unbounded number of processes. Never block an actor handler on slow work — offload to a supervised worker so the mailbox keeps draining.

## Required Techniques
1. **`Result` first.** Every fallible function returns `Result(value, error)`; chain the happy path with `use x <- result.try(...)` and finish with a `Result` — `Ok(value)` or one last fallible call. Libraries define precise custom error types; applications may use `snag` for ad-hoc context-carrying errors. `Option` is for values that may legitimately be absent — not for fallibility.
2. **Crash only on the impossible.** `let assert Ok(x) = ... as "why this cannot fail"` and `panic as "..."` are for states that are provably unreachable or unrecoverable — never for expected errors, and always with an `as` message. The bare `assert` statement is the test-assertion idiom (Gleam ≥ 1.11).
3. **Exhaustive `case` + guards.** There is no `if`; branch with `case` (the compiler enforces totality) and `use <- bool.guard(when:, return:)` for early exits. When you add a variant, the compiler finds every site that must change — don't defeat that with catch-all `_` patterns on your own types.
4. **Tail recursion with accumulators.** Iteration is recursion (or `gleam/list` functions — prefer them). Only tail calls are optimised: public wrapper + private tail-recursive helper with an accumulator. Build large or looped strings with `string_tree` (constant-time append); naive repeated `<>` risks O(n²) copying — the BEAM's append optimisation often mitigates it, so benchmark hot paths (Standard §3.2).
5. **Checked arithmetic.** `Int` and `Float` never coerce (`+` vs `+.`), and **division by zero silently returns `0`/`0.0`** — use `int.divide`/`float.divide` (they return `Result`) wherever a zero divisor is possible.
6. **Decode at the boundary.** External data (JSON, FFI, message payloads) enters as `Dynamic`; decode it immediately with `gleam/dynamic/decode` (`use x <- decode.field(...)` … `decode.success(...)`, run with `decode.run`). The old `dynamic.decodeN` API is gone.
7. **FFI sparingly, typed exactly.** `@external(erlang, "mod", "fun")` / `@external(javascript, "./mod_ffi.mjs", "fun")` with a fully annotated head — the compiler trusts your annotation, so a wrong one is undefined behaviour. Foreign code can still throw: catch at the boundary (the `exception` package) and convert to `Result`. Use `@target(erlang)` / `@target(javascript)` for target-specific code paths.

## Build, Tooling & CI (Non-Negotiable)
- **Toolchain floor:** Gleam ≥ 1.17.0 (fixes CVE-2026-43965, CVE-2026-32685, CVE-2026-42795), Erlang/OTP ≥ 27 (required by gleam_erlang 1.x). Pin `gleam = ">= 1.17.0"` in `gleam.toml`.
- `gleam format --check src test` must pass (canonical formatter, zero configuration — no style debates).
- `gleam build --warnings-as-errors` per supported target — this flag exists on `build` only (`check`/`test` cannot gate warnings), so CI runs it as a dedicated step before `gleam test`. Compiler warnings **are** the linter; there is no clippy/credo analogue.
- `gleam test` green — gleeunit ≥ 1 discovers `pub fn *_test` in `test/`; assert with the `assert` keyword (`gleeunit/should` is deprecated). Property tests with `qcheck` for parsers, codecs, and invariants; snapshot tests with `birdie`.
- Dependencies: semver ranges (`">= 1.0.0 and < 2.0.0"`) in `[dependencies]` / `[dev_dependencies]` (snake_case since 1.15); review `gleam deps outdated` regularly.
- No `echo` in committed code — it is debug tooling and blocks `gleam publish`. Production logging goes through the `logging` package (Erlang logger) on the BEAM.
- Ship with `gleam export erlang-shipment` (or `escript` — still requires Erlang on the host), never `gleam run` in production. Document public APIs with `///` (and modules with `////`) so `gleam docs build` works.

## Anti-Patterns (Never Do These)
- Using the pre-1.0 `gleam_otp` API — `actor.start(state, handler)`, message-first handlers, `gleam/otp/task`, `gleam/otp/supervisor` — none of it exists in 1.x; stale hexdocs still rank well in search.
- Wrapping pure, stateless logic in an actor "because OTP" — it just serializes and bottlenecks.
- Unbounded `process.spawn` per request — use `factory_supervisor` (or a fixed pool of supervised workers).
- Blocking inside `on_message` (long computation, chained `process.call`s, sleeps) — the mailbox stalls and callers' `call`s panic on timeout.
- `let assert` / `panic` on failures that are merely *unlikely* — expected failure is a `Result`, full stop.
- Creating `process.Name`s (or any atom) dynamically — atom-table exhaustion kills the whole VM; create names once at startup.
- Catch-all `_ ->` arms on your own custom types — they silently swallow future variants the compiler would otherwise flag.
- Repeated string `<>` in hot loops without a benchmark (prefer `string_tree`); `/` on possibly-zero divisors (use `int.divide`); relying on the curried pipe fallback `a |> b(c)` meaning `b(c)(a)` — it is deprecated upstream (post-1.17); when the curried call is intended, write `a |> b(c)()` explicitly. Plain `a |> b(c)` keeps meaning `b(a, c)`.
- `gleeunit/should` and `dynamic.decodeN` — deprecated/removed; use `assert` and `gleam/dynamic/decode`.
- Treating the JavaScript target like the BEAM — no processes, no supervision; a panic there takes down the entire Node/Deno/Bun runtime.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Every long-lived process descends from the root supervisor started in `main`; components expose `supervised()` child specs
- [ ] Actors use the 1.x builder API; handlers return quickly; slow work is offloaded to supervised workers
- [ ] Runtime-created children go through `factory_supervisor` — no unbounded spawn per request
- [ ] Expected failures are `Result`s chained with `use`/`result.try`; `let assert`/`panic` only for impossible states, always with `as` messages
- [ ] `Name`s created once at startup and passed down; actors that must survive restarts are `actor.named`
- [ ] External data decoded at the boundary via `gleam/dynamic/decode`; FFI minimal, exactly annotated, exceptions converted to `Result`
- [ ] No catch-all `_` on own types; no unchecked `/` where the divisor can be zero; looped string building goes through `string_tree`
- [ ] Recursion is tail-recursive (or uses `gleam/list`); no `echo` left in the diff
- [ ] `gleam format --check`, `gleam build --warnings-as-errors` (each supported target), and `gleam test` all green; invariants covered by `qcheck` properties
- [ ] Module/function docs (`////`/`///`) state the message protocol, supervision, and restart contract of every actor

## References & Further Reading
- Load `references/Spacecraft_Gleam_Guidelines.md` for full skeletons (root supervision tree, typed actor, `factory_supervisor`, `use`/`result.try` pipelines, dynamic decoding, FFI boundary, gleeunit/qcheck/birdie, CI recipe) when deeper patterns are needed.
- *Further reading* (consulted for background only; no text reproduced here): the official Gleam docs, language tour, and release notes (`gleam.run`, `tour.gleam.run`); the `gleam_otp`, `gleam_erlang`, and `gleam_stdlib` documentation on HexDocs. APIs herein verified 2026-07-04 against Gleam 1.17.0, gleam_otp 1.2.0, gleam_erlang 1.3.0, gleam_stdlib 1.0.3.

When the user requests Gleam code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft BEAM engineer would ship.
