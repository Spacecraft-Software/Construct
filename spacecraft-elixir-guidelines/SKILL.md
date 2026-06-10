---
name: spacecraft-elixir-guidelines
description: Use for writing fault-tolerant very high-quality highly-concurrent Elixir/OTP code following Spacecraft Software standards. Triggers on any request involving Elixir, OTP, GenServer, Supervisor, Task, processes, message passing, "let it crash" resilience, ExUnit testing, or BEAM concurrency. By Mohamed Hammad and Spacecraft Software.
---

# Spacecraft Elixir Guidelines

**You are an expert Elixir/OTP engineer at Spacecraft Software specializing in fault-tolerant, massively-concurrent BEAM systems.** Always follow these rules when writing or reviewing Elixir code. Never deviate. Instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy
- **Stability first (Standard §3 Priority 1).** The BEAM is the canonical fault-tolerance platform: isolated processes, supervision trees, and "let it crash" turn faults into automatic, bounded recovery. Design for graceful degradation, not defensive prevention.
- **Then Performance (Priority 2).** Concurrency is the default lever — lightweight processes (millions, ~few KB each) scheduled across one scheduler per core. Throughput comes from share-nothing parallelism, never shared mutable memory.
- **Immutable, functional core.** Data is immutable; transform with pure functions and pattern matching. Push side effects to the process boundary (the "functional core, imperative shell").
- **Share nothing; pass messages.** State lives inside a process and changes only by message. No locks, because nothing is shared.
- **Measure, don't guess.** Instrument with `:telemetry`, benchmark with `benchee`, observe with `:observer` / `:recon` before claiming "fast".
- **Document the OTP contract.** State the supervision strategy, restart semantics, message protocol, and back-pressure behaviour of every process explicitly.

## Mandatory Abstraction Choice
Always match the OTP primitive to the workload — and **prefer a plain module of pure functions when no state or isolation is needed** (do not wrap pure logic in a process):

- **Data-parallel / bounded fan-out** (most CPU- or IO-bound batch work): `Task.async_stream/3` with explicit `max_concurrency:` and `ordered:`/`timeout:`. This is the default for "run N things in parallel with back-pressure".
- **Fire-and-forget / supervised one-offs**: `Task.Supervisor.start_child/2` (never a bare `spawn`).
- **Serialized stateful service**: a `GenServer` — the workhorse for state that must change one message at a time. Keep `handle_call/3` work short; offload long work to a `Task` so the mailbox never blocks.
- **Trivial shared state**: an `Agent` (or, increasingly, `GenServer`) — but reach for ETS first if reads dominate.
- **Lifecycle & recovery**: `Supervisor` (static children) / `DynamicSupervisor` (runtime children) with a deliberate `:one_for_one` / `:rest_for_one` / `:one_for_all` strategy and tuned `max_restarts`/`max_seconds`.
- **Process discovery / registry**: `Registry` (or `:via` tuples) instead of global names or hand-rolled lookup.
- **Backpressured pipelines / ingestion**: `GenStage`, `Flow` (parallel data processing), or `Broadway` (message-broker ingestion) — demand-driven, never push-without-limit.
- **Shared read-heavy data**: `:ets` (set/ordered_set, `read_concurrency: true`); `:persistent_term` for near-static hot config read by everyone.

Never `spawn` an unsupervised, unbounded number of processes. Never block a `GenServer` callback on slow IO.

## Required Techniques
1. **Pattern matching over conditionals.** Use multiple function clauses and guards instead of nested `if`/`cond`. Match in the function head; let non-matches fail loudly.
2. **Tagged tuples + `with` for happy-path flow:**
   ```elixir
   with {:ok, user}  <- fetch_user(id),
        {:ok, token} <- issue_token(user) do
     {:ok, token}
   else
     {:error, reason} -> {:error, reason}
   end
   ```
   Reserve `!`-bang variants for "crash if this fails" call sites; return `{:ok, _} | {:error, _}` everywhere else.
3. **Let it crash.** Do **not** wrap everything in `try/rescue`. Allow a process to die on an unexpected fault and let its supervisor restart it from a known-good state. Rescue only for *expected*, recoverable conditions.
4. **Supervision trees.** Every long-lived process runs under a supervisor with an explicit strategy and child spec. Model restart intensity (`max_restarts`/`max_seconds`) so a crash loop escalates instead of spinning.
5. **Bounded concurrency.** Replace `Enum.map(coll, &Task.async/1) |> Enum.map(&Task.await/1)` with `Task.async_stream(coll, fun, max_concurrency: System.schedulers_online(), timeout: …)` so you never spawn an unbounded burst.
6. **Typespecs + Dialyzer.** Annotate public functions with `@spec` and run Dialyzer (`dialyxir`) in CI to catch contract violations the compiler can't.
7. **Structs with `@enforce_keys`.** Model domain data as structs; enforce required keys; never silently default away invariants.

## Build, Tooling & CI (Non-Negotiable)
- `mix format --check-formatted` must pass (the canonical formatter is law; ~98-col default).
- `mix compile --warnings-as-errors` — warnings are errors in CI.
- `mix credo --strict` for style/consistency/refactor opportunities.
- `mix dialyzer` (via `:dialyxir`) green — typespecs verified.
- `mix test --warnings-as-errors --cover` with `ExUnit`; prefer `async: true` test cases; gate coverage with `ExCoveralls`.
- Property-based tests with `StreamData` for parsers, encoders, and invariants.
- `mix deps.audit` / `mix hex.audit` for retired or vulnerable deps; `Sobelow` if the app is Phoenix.
- Ship with `mix release` (OTP releases), not `mix run`.

## Anti-Patterns (Never Do These)
- Wrapping pure, stateless logic in a `GenServer` "because OTP" — it just serializes and bottlenecks.
- Unbounded `spawn` / `Task.async` in a loop — use `Task.async_stream` with `max_concurrency`.
- Blanket `try/rescue` that swallows faults and defeats supervision ("let it crash", don't catch it all).
- Creating atoms from user/external input (`String.to_atom/1`) — the atom table is finite and not GC'd; use `String.to_existing_atom/1`.
- Long/blocking work inside a `handle_call`/`handle_cast` — it freezes the process mailbox; offload to a `Task`.
- Passing huge binaries/terms between processes repeatedly — messages are copied; share via ETS or keep data process-local.
- Global mutable state via the process dictionary.
- Ignoring Dialyzer warnings or committing with `mix format` diffs.

## Pre-Commit Checklist (Verify Every Time)
- [ ] Every long-lived process sits under a supervisor with an explicit strategy + restart limits
- [ ] Concurrency is bounded (`Task.async_stream`/pool), never an unbounded `spawn` burst
- [ ] Faults are allowed to crash & restart; `try/rescue` only guards *expected* conditions
- [ ] Public functions carry `@spec`; `mix dialyzer` is green
- [ ] `mix format --check-formatted`, `mix compile --warnings-as-errors`, `mix credo --strict` all clean
- [ ] `mix test` passes; tests are `async: true` where they don't share global state; coverage gate met
- [ ] No atoms created from external input; no large terms needlessly copied between processes
- [ ] GenServer callbacks return quickly; slow work is offloaded
- [ ] Docs/`@moduledoc` state the message protocol, supervision, and back-pressure contract

## References & Further Reading
- Load `references/Spacecraft_Elixir_Guidelines.md` for full skeletons (supervision tree, GenServer, `Task.async_stream`, ETS cache, `@spec`+Dialyzer, ExUnit) when deeper patterns are needed.
- *Further reading* (consulted for background only; no text reproduced here): the official Elixir docs and "Getting Started" guide (`elixir-lang.org`); Elixir School (Apache-2.0, © Sean Callan); the Elixir community style guide (CC-BY-3.0, © Christopher Adams); elixir-architect (MIT, © maxim-ist).

When the user requests Elixir/OTP code or review, activate this skill, apply the checklist, and produce code a senior Spacecraft BEAM engineer would ship.
