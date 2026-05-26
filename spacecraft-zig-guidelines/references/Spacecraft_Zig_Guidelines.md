# Spacecraft Zig Guidelines — Extended Reference

This document supplements the main SKILL.md with concrete code patterns, version-specific notes, and deeper explanations. Load this file when the user asks for "full guidelines", "examples", "complete document", or when you need more than the concise rules.

## Version Notes (Zig 0.11 – 0.16+)

- **Zig ≤ 0.15**: Primary primitives are `std.Thread`, `std.Thread.Pool`, `std.Thread.WaitGroup`, `std.atomic`.
- **Zig ≥ 0.16**: Prefer `std.Io.Threaded` + `std.Io.Group` for unified task spawning. The `Io` interface is dependency-injected like `Allocator`. `std.Thread.Pool` remains available but `Io` is the future direction for most code.
- Always guard version-specific code with `if (builtin.zig_version.major == 0 and builtin.zig_version.minor >= 16)` or use `comptime` detection.

## Concrete Code Patterns

### 1. High-Performance Data-Parallel Skeleton (Zig 0.16+ style)

```zig
const std = @import("std");

pub fn parallelProcess(io: std.Io, allocator: std.mem.Allocator, input: []const u64, output: []u64) !void {
    const num_threads = std.Thread.getCpuCount() orelse 8;
    const chunk_size = (input.len + num_threads - 1) / num_threads;

    var group: std.Io.Group = .init;
    defer group.cancel(io);

    for (0..num_threads) |i| {
        const start = i * chunk_size;
        const end = @min(start + chunk_size, input.len);
        if (start >= end) break;

        try group.concurrent(io, processChunk, .{ input[start..end], output[start..end] });
    }

    try group.await(io);
}

fn processChunk(chunk_in: []const u64, chunk_out: []u64) void {
    // Hot inner loop — can use @Vector(8, u64) + SIMD here
    for (chunk_in, 0..) |val, j| {
        chunk_out[j] = val * val + 42; // example transform
    }
}
```

### 2. Lock-Free Counter with Proper Ordering

```zig
const std = @import("std");

const AtomicCounter = struct {
    value: std.atomic.Value(u64) = .init(0),

    pub fn increment(self: *AtomicCounter) u64 {
        return self.value.fetchAdd(1, .monotonic);
    }

    pub fn get(self: *const AtomicCounter) u64 {
        return self.value.load(.monotonic);
    }
};
```

### 3. Cache-Line Padded Shared State

```zig
const std = @import("std");

const align_cache = 64;

const HotShared = extern struct {
    counter: std.atomic.Value(u64) align(align_cache),
    // padding is implicit due to align on field, or explicit:
    // _pad: [align_cache - @sizeOf(std.atomic.Value(u64))]u8 = undefined,
};
```

### 4. Per-Task Arena + Thread Pool (Safe & Fast)

```zig
fn workerTask(allocator: std.mem.Allocator, data: []u8) !void {
    var arena = std.heap.ArenaAllocator.init(allocator);
    defer arena.deinit();
    const task_alloc = arena.allocator();

    const temp = try task_alloc.alloc(u8, 1024);
    // ... use temp, freed automatically on arena.deinit()
}
```

### 5. Ownership Documentation Example

```zig
/// Spawns worker threads that take ownership of `owned_slice`.
/// Caller must not access `owned_slice` after this call returns.
/// Each worker will free its portion using the provided allocator.
fn spawnWorkers(pool: *std.Thread.Pool, owned_slice: []u8, allocator: std.mem.Allocator) !void {
    // ...
}
```

## Common Pitfalls & How to Avoid Them

- **ABA problem in lock-free queues**: Always use tagged pointers (store generation counter in upper bits of the pointer) or an epoch-based reclamation scheme.
- **False sharing on accumulators**: Never place two frequently-updated atomics in the same cache line unless they are intentionally coordinated.
- **Leaking across threads**: If a thread allocates and another frees, both must use the exact same `Allocator` instance. Prefer per-thread arenas.
- ** UB in ReleaseFast**: The only way to catch it is exhaustive testing in ReleaseSafe + miri-like tools (Zig has good integration with external sanitizers).

## Recommended External Study

- Zig language reference: "Atomics", "Threads", "Memory", "comptime".
- Source code: `lib/std/atomic.zig`, `lib/std/Thread/Pool.zig`, `lib/std/Io.zig`.
- Mental model: Translate every Rust `crossbeam` / `rayon` pattern into explicit Zig atomics + manual chunking.
- Benchmarking: Write a small harness that reports "scaling efficiency" as a first-class metric.

This extended reference is intentionally kept lean. When you need even more detail (full queue implementations, NUMA pinning, etc.), ask the user for clarification or load additional domain references.

— Spacecraft Software, 2026
