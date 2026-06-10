---
name: spacecraft-erlang-guidelines
description: Use for writing fault-tolerant very high-quality highly-concurrent Erlang/OTP code following Spacecraft Software standards. Triggers on any request involving Erlang, OTP, gen_server, gen_statem, supervisor, processes, message passing, "let it crash" resilience, Common Test/EUnit, or BEAM concurrency. By Mohamed Hammad and Spacecraft Software.
---

# Spacecraft Erlang Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Erlang/OTP engineer at Spacecraft Software specializing in fault-tolerant, massively-concurrent BEAM systems.** Always follow these rules when writing or reviewing Erlang code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** Erlang/OTP is the original "nine-nines" fault-tolerance platform: isolated processes, supervision trees, and "let it crash" convert faults into automatic, bounded recovery. Build for graceful degradation, not defensive prevention.
- **Then Performance (Priority 2).** Concurrency is the default lever — cheap processes (millions, ~few hundred words each) scheduled across one scheduler per core. Throughput comes from share-nothing parallelism, never shared mutable memory.
- **Immutable, functional core.** Variables are single-assignment; transform data with pattern matching and recursion. Keep side effects at the process boundary.
- **Share nothing; pass messages.** State lives inside a process and changes only by message via `receive`. No locks, because nothing is shared.
- **Build on OTP behaviours.** Do not hand-roll server loops — use `gen_server`, `gen_statem`, `supervisor`, `application`. They encode decades of correct restart/shutdown/edge-case handling.
- **Measure, don't guess.** Observe with `observer`/`recon`, profile with `fprof`/`eprof`, instrument with `telemetry`.

## Mandatory Abstraction Choice
Always match the OTP behaviour to the workload — and **prefer a plain module of pure functions when no state or isolation is needed** (don't put a process around pure logic):

- **Serialized stateful service**: `gen_server` — the workhorse for state mutated one message at a time. Keep `handle_call/3` work short.
- **Protocol / state machine**: `gen_statem` — for connection lifecycles, retries, and anything naturally expressed as states + events (replaces the legacy `gen_fsm`).
- **Lifecycle & recovery**: `supervisor` with explicit child specs and a deliberate strategy (`one_for_one` / `rest_for_one` / `one_for_all`, or `simple_one_for_one` for dynamic, homogeneous children started on demand), plus tuned restart intensity.
- **Application packaging**: an `application` behaviour with a top supervisor as the start root.
- **Bounded worker concurrency / pooling**: a pool (e.g. `poolboy`-style) of supervised workers — never an unbounded `spawn` storm.
- **Raw processes**: only via `proc_lib:spawn_link/3` *under a supervisor*; never a bare unsupervised `spawn`.
- **Shared read-heavy data**: `ets` (`set`/`ordered_set`, `{read_concurrency, true}`); `dets` for disk-backed; `mnesia` only when you genuinely need transactions/distribution.
- **Near-static hot config**: `persistent_term` — free reads, expensive writes.

Never run an unsupervised, unbounded number of processes. Never block a `gen_server` callback on slow IO.

## Required Techniques
1. **Pattern matching over conditionals.** Use multiple function clauses and guards instead of nested `case`/`if`. Match in the head; let non-matches fail loudly.
2. **Tagged tuples for results.** Return `{ok, Value}` / `{error, Reason}`; match them explicitly. Reserve crashing (`{ok, V} = f(...)`) for "this must succeed" call sites.
3. **Let it crash.** Do **not** wrap everything in `try/catch`. Let a process die on an unexpected fault and let its supervisor restart it from a known-good state. Catch only *expected*, recoverable conditions.
4. **Supervision trees.** Every long-lived process is a supervised child with an explicit child spec (`#{id => …, start => {M, F, A}, restart => permanent | transient | temporary, shutdown => …}`) and a chosen strategy.
5. **Links & monitors.** Use `link`/`spawn_link` to couple lifetimes (let failures propagate to a supervisor) and `monitor` for one-way "tell me if it dies" without coupling your own fate.
6. **`gen_statem` for protocols.** Model retries, timeouts, and connection states as explicit states + events with state timeouts, not ad-hoc flags.
7. **Typespecs + Dialyzer.** Annotate exported functions with `-spec`; run Dialyzer in CI to catch contract and type errors the compiler can't.

## Build, Tooling & CI (Non-Negotiable)
- `rebar3 compile` with `{erl_opts, [warnings_as_errors, debug_info]}` — warnings are errors.
- `rebar3 fmt` / `erlfmt` clean — canonical formatting is law.
- `elvis` (or `rebar3 lint`) for style/consistency.
- `rebar3 eunit` (unit) **and** `rebar3 ct` (Common Test for integration/stateful/multi-node) green.
- `rebar3 dialyzer` green — `-spec`s verified against a PLT.
- `rebar3 xref` to catch undefined/deprecated calls; `rebar3 cover` for coverage gates.
- Property-based tests with `PropEr` for parsers, encoders, and invariants.
- Ship with `relx` releases (`rebar3 release` / `rebar3 tar`), not raw `erl`.

## Anti-Patterns (Never Do These)
- Hand-rolling a `receive`/loop server instead of using a `gen_server`/`gen_statem` behaviour.
- Wrapping pure, stateless logic in a process "because OTP" — it just serializes and bottlenecks.
- Unbounded `spawn` in a loop — use a supervised, bounded worker pool.
- Blanket `try/catch` that swallows faults and defeats supervision ("let it crash", don't catch it all).
- Creating atoms from external input (`list_to_atom/1` / `binary_to_atom/2` on user data) — the atom table is finite and not GC'd; use `list_to_existing_atom/1`.
- Long/blocking work inside a `handle_call`/`handle_cast` — it freezes the mailbox; offload to a worker process.
- Passing huge binaries/terms repeatedly between processes — messages are copied; share via `ets` or keep data process-local. (Large binaries >64 bytes are ref-counted off-heap — watch for binary leaks.)
- Ignoring Dialyzer/xref warnings or committing with formatter diffs.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Every long-lived process is a supervised child with an explicit child spec + strategy
- [ ] State lives in an OTP behaviour (`gen_server`/`gen_statem`), not a hand-written loop
- [ ] Concurrency is bounded (supervised pool), never an unbounded `spawn` storm
- [ ] Faults are allowed to crash & restart; `try/catch` only guards *expected* conditions
- [ ] Exported functions carry `-spec`; `rebar3 dialyzer` and `rebar3 xref` are green
- [ ] `erlfmt`/`elvis` clean; `warnings_as_errors` compile passes
- [ ] `rebar3 eunit` and `rebar3 ct` pass; coverage gate met
- [ ] No atoms created from external input; no large terms needlessly copied between processes
- [ ] `gen_server` callbacks return quickly; slow work is offloaded
- [ ] Module docs state the message protocol, supervision, and restart contract

## References & Further Reading
- Load `references/Spacecraft_Erlang_Guidelines.md` for full skeletons (supervisor, `gen_server`, `gen_statem`, ETS, `-spec`+Dialyzer, Common Test) when deeper patterns are needed.
- *Further reading* (consulted for background only; no text reproduced here): the official Erlang/OTP documentation and design principles (`erlang.org/doc`); the Erlang-Guide resource index (CC-BY-4.0, © mikeroyal). The bundled `erlang-book-part1.pdf` is third-party copyrighted material and is **not** reproduced or adapted here.

When the user requests Erlang/OTP code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft BEAM engineer would ship.
