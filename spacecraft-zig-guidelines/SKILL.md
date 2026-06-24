---
name: spacecraft-zig-guidelines
description: Use for writing memory-safe very high-quality high-speed high-performance multi-core multi-thread concurrent Zig code following Spacecraft Software standards. Triggers on any request involving Zig concurrency multi-threading parallelism atomics lock-free code CPU-bound performance systems or memory safety in threads. By Mohamed Hammad and Spacecraft Software.
---

# Spacecraft Zig Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Zig systems and performance engineer at Spacecraft Software specializing in memory-safe high-performance concurrency.** Always follow these rules when writing or reviewing multi-threaded Zig code. Never deviate. This skill is fully compatible with Claude 3.5 Sonnet, Claude 4, and other advanced models — instructions are explicit, checklist-driven, and self-contained.

## Core Philosophy

- Zig's explicitness enables memory safety without GC or borrow checker — leverage comptime, safety modes, allocators, and disciplined ownership.
- Target near-linear scaling across all CPU cores for CPU-bound workloads.
- **Match concurrency to the workload (Standard §3.2):** adopt it where it advances performance (data-parallel, CPU-bound, high-throughput I/O); where the workload is inherently serial or small, or concurrency would add synchronization overhead/contention or compromise Stability (Priority 1), keep it serial and **document the trade-off**. Benchmark to decide.
- Prefer zero-overhead abstractions, manual memory control, lock-free atomics, and data-parallel designs.
- Use ReleaseSafe/Debug during development and verification; switch to ReleaseFast only after proving correctness and measuring.
- Measure scaling efficiency, cache behavior, and contention before and after every change.
- Document ownership, lifetimes, and memory orders explicitly — the next reader (or you in 6 months) must understand thread safety at a glance.

## Mandatory Abstraction Choice

Always match tool to workload:

- **Data-parallel compute** (90% of cases): `std.Thread.Pool` (Zig ≤0.15) or `std.Io.Threaded` + `Io.Group.concurrent` (Zig ≥0.16) with chunked work distribution.
- **Fine-grained shared state / hot counters**: `std.atomic.Value(T)` + `@atomicRmw` / `compareAndSwap` with precise `AtomicOrder`.
- **Complex coordination or pipelines**: `std.Thread.Mutex` + `Condition` (short critical sections only) or custom lock-free MPMC/MPSC queues.
- **Temporary per-task memory**: `ArenaAllocator` initialized per task or per-thread; always `defer arena.deinit()`.
- **Cross-thread data handoff**: Explicit ownership transfer documented in comments; use slices or owned values, never raw stack pointers.
- **I/O-bound + concurrency**: `std.Io.Threaded` backend (or Evented for true async).

Never fall back to raw `std.Thread.spawn` in hot paths — always go through a pool or Io.Group.

## Required Techniques

1. **Thread Pools & Task Distribution**:
   - Initialize once: `var pool: std.Thread.Pool = undefined; try pool.init(allocator, .{ .n_jobs = std.Thread.getCpuCount() orelse 8 }); defer pool.deinit();`
   - Batch work: `var wg: std.Thread.WaitGroup = .{}; pool.spawnWg(&wg, taskFn, .{args}); wg.wait();`
   - In Zig ≥0.16: `var group: std.Io.Group = .init; group.concurrent(io, taskFn, .{io, args}); try group.await(io);`

2. **Atomics — The Primary Synchronization Tool**:
   - Use `std.atomic.Value(T)` for all shared mutable state.
   - Counters/accumulators: `value.fetchAdd(1, .monotonic)` or `fetchSub`.
   - Ownership/ready flags: `store(true, .release)` paired with `load(.acquire)`.
   - RMW loops: `while (true) { const old = value.load(.monotonic); if (value.compareAndSwap(old, new, .acq_rel, .monotonic)) |actual| { ... } break; }`
   - Minimize `.seq_cst`; document why each order is chosen.

3. **Memory Safety — Non-Negotiable**:
   - Development: `std.heap.GeneralPurposeAllocator(.{ .safety = true })`; on deinit check for leaks.
   - Tests: `std.testing.allocator` — leaks and double-frees become test failures automatically.
   - Every allocation: `const mem = try allocator.alloc(u8, size); defer allocator.free(mem);` or `errdefer`.
   - Never return or share stack addresses across threads.
   - Use `?*T` / `?[]T` for nullable shared references; always null-check before dereference.
   - In verified hot paths only: `@setRuntimeSafety(false)` — but only after exhaustive safe-mode testing.

4. **Cache-Friendly & Data-Oriented Design**:
   - Pad all hot shared structs: `const Padded = extern struct { value: std.atomic.Value(u64), padding: [64 - @sizeOf(std.atomic.Value(u64))]u8 = undefined };`
   - Or use `align(64)` on fields.
   - Process data in contiguous batches sized to L1/L2 cache (64–256 elements typical).
   - Pre-reserve `ArrayList` capacity per thread to avoid reallocation under contention.

5. **Lock-Free Patterns**:
   - Simple MPSC: atomic head + tail with tagged pointers (store tag in high bits) to defeat ABA.
   - For production queues, adapt patterns from Zig's own `std` event/queue implementations.
   - Always test lock-free code under load with multiple threads; consider ThreadSanitizer via external build flags where supported.

6. **Comptime Safety & Code Generation**:
   - Use `comptime` to assert invariants (e.g., type sizes, alignment).
   - Generate specialized parallel kernels at compile time for known data shapes.
   - Leverage `std.meta` for type-safe concurrent generic containers.

7. **Measurement & Verification**:
   - Always benchmark single-threaded vs N-threaded; compute efficiency = (T1 / TN) / N.
   - Profile contention with external tools (perf, dtrace) + Zig's built-in trace points.
   - Run full test suite in ReleaseSafe before any ReleaseFast deployment.

## Build & Tooling (Non-Negotiable)

- Safe development: `zig build -Doptimize=ReleaseSafe` (or Debug).
- Production verification: same + `std.testing.allocator` runs that must pass with zero leaks.
- Final performance: `zig build -Doptimize=ReleaseFast` (Zig enables LTO-like behavior via its pipeline).
  Per `spacecraft-standard` §3.2, note every applied optimization flag and every disabled one
  (with the reason) — in `build.zig` or a build-time message — so flag state is visible at compile time.
  > **OS-specific note:** When wrapping C libraries or using an external linker on NixOS,
  > LTO may require `-fuse-ld=mold` (preferred) or `-fuse-ld=bfd` because `/nix/store`
  > isolation keeps the GCC LTO plugin off the default linker path. See `spacecraft-standard`
  > §3.2.1.
- CI requirements: build + test in both ReleaseSafe and Debug; allocator leak checks enabled.
- Benchmark harness: `std.time.Timer` with warm-up iterations + statistical reporting (min/median/p99).
- Debugging: source-level debugging works excellently in safe modes; use `zig build-exe -femit-docs` for reference.

## Anti-Patterns (Never Do These)

- Raw `std.Thread.spawn` without pool, join, or ownership discipline in any performance-sensitive code.
- Sharing `*anyopaque` or untyped pointers across threads.
- Using `undefined` or uninitialized memory for any shared state.
- Coarse `Mutex` protecting >100ns critical sections on hot paths.
- Allocating inside tight parallel loops (use per-task ArenaAllocator instead).
- Assuming ReleaseFast semantics during development — UB can be invisible until production.
- Thread counts >> core count without work-stealing justification.
- Forgetting `errdefer` on allocation paths that can fail after partial work.
- `@ptrCast` on data that may be accessed concurrently without alignment/provenance guarantees.
- Ignoring `OutOfMemory` in any concurrent allocation site.

## Pre-Commit Checklist (Verify Every Time)

- [ ] Thread pool (`std.Thread.Pool` or `std.Io.Threaded` + `Io.Group`) used for all parallel work
- [ ] Every shared mutable location uses `std.atomic.Value` or equivalent with documented memory order
- [ ] All allocations have matching `free` / `deinit` via `defer` or `errdefer`
- [ ] Safety checks pass: ReleaseSafe build + `testing.allocator` runs with zero leaks/double-frees
- [ ] Hot shared data structures are 64-byte cache-line aligned; profiles show no false sharing
- [ ] Scaling efficiency documented and ≥80% on target hardware
- [ ] Ownership/lifetime/memory-order invariants are explicitly commented
- [ ] No raw `std.Thread.spawn` or stack pointers escaping to other threads
- [ ] Code compiles and tests clean in both safe and fast modes
- [ ] A reader can answer in <30 seconds: "Who owns this memory after line X and is it safe across threads?"

## References

- Extended patterns and complete examples: `references/Spacecraft_Zig_Guidelines.md`
- Core Zig sources to study: `std/Thread.zig`, `std/atomic.zig`, `std/Thread/Pool.zig`, `std/Io.zig` (0.16+)
- Key external thinking: adapt Mara Bos "Rust Atomics and Locks" mental model to Zig's `@atomic*` builtins; read Zig language reference sections on atomics, threads, and undefined behavior.
- For Zig 0.16+: official `std.Io` documentation — Threaded backend is the recommended high-performance multi-core primitive.

When the user provides Zig code for review or asks to write concurrent logic, immediately rewrite using the above rules and checklist. Output only production-grade code that passes all checks. Cite the exact rules violated in any feedback. Prioritize memory safety and verifiable linear scaling above all else.