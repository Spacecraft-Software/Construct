# Spacecraft Golang Guidelines — Full Reference

**Version:** 1.0  
**Author:** Mohamed Hammad & Spacecraft Software  
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the SKILL.md for cases requiring deeper patterns, advanced tooling, or NUMA/multi-socket optimizations. Load this file when the user asks for "complete guidelines", "advanced concurrency", "custom scheduler", or when reviewing large-scale systems.

## 1. Structured Concurrency Patterns

### Worker Pool with Semaphore + errgroup (Recommended Default)
```go
import (
    "context"
    "golang.org/x/sync/errgroup"
    "golang.org/x/sync/semaphore"
    "runtime"
)

func ProcessItems(ctx context.Context, items []Item) error {
    const maxWorkers = 32 // or runtime.NumCPU() * 2 for I/O mix
    sem := semaphore.NewWeighted(int64(maxWorkers))
    g, ctx := errgroup.WithContext(ctx)

    for _, item := range items {
        if err := sem.Acquire(ctx, 1); err != nil {
            return err
        }
        g.Go(func() error {
            defer sem.Release(1)
            return processItem(ctx, item)
        })
    }
    return g.Wait()
}
```

### Pipeline with Context Cancellation
Each stage must:
- Receive from `in`, send to `out`
- Check `ctx.Done()` or `select` with done channel
- Close `out` exactly once after all work

Use `sync.Once` for close if multiple potential closers.

## 2. Memory Safety & Escape Analysis

- Run `go build -gcflags=-m=2 2>&1 | grep escape` to audit hot paths.
- Hot data should stay on stack or be pooled.
- Use `unsafe` **only** for FFI or zero-copy with extreme justification; wrap in `//go:nosplit` and document why race detector cannot catch issues.
- Always test with `-race` even in benchmarks (use `testing.B` with `b.RunParallel`).

## 3. Advanced Profiling

- CPU: `go tool pprof -focus=processItem -cum cpu.out`
- Goroutine leaks: `go tool pprof http://localhost:6060/debug/pprof/goroutine?debug=2`
- Contention: `go test -bench=. -mutexprofile=mutex.out`
- Trace: `go tool trace -http=:8080 trace.out` (look for "GC" and "syscall" stalls)

## 4. Go Version Specifics (2026+)

- Go 1.22+: `slices` package, `maps`, better inlining.
- Go 1.23+: Improved scheduler, `unique` package for interned values (reduces allocations).
- Always pin to latest stable in CI; test against `gotip` for scheduler changes.

## 5. NUMA & Large Core Counts

On >32 cores or multi-socket:
- Use `runtime.GOMAXPROCS(runtime.NumCPU())` explicitly.
- Pin goroutines if needed with `github.com/uber-go/automaxprocs` or custom `GOMAXPROCS` logic.
- Shard data structures by CPU (e.g., per-core counters with padding).
- Avoid global locks; use per-shard `RWMutex` or lock-free.

## 6. Testing Concurrent Code

- Use ` goleak` (go.uber.org/goleak) in TestMain to detect leaks.
- `testing/synctest` (Go 1.25+) for deterministic time-based tests.
- Property-based testing with `rapid` or `gopter` for concurrent invariants.
- Stress tests: run `-count=100 -race` in CI for critical packages.

## 7. Common Pitfalls & Fixes

| Pitfall                        | Symptom                     | Fix                                      |
|--------------------------------|-----------------------------|------------------------------------------|
| Goroutine leak                 | Growing goroutine count     | Always use errgroup or explicit Done() + context |
| Channel send on closed         | Panic                       | Close only from owner; use sync.Once    |
| Long critical section          | High mutex wait time        | Shrink lock scope; use atomic or channel |
| Unbounded work                 | OOM or high latency         | Semaphore or worker pool with backpressure |
| Ignoring ctx in goroutine      | Work continues after cancel | Pass ctx and check `ctx.Err()` early    |

## 8. Recommended External Packages (Spacecraft Approved)

- `golang.org/x/sync/errgroup` + `semaphore`
- `github.com/sourcegraph/conc` (structured concurrency)
- `go.uber.org/goleak` (leak detection)
- `github.com/uber-go/automaxprocs` (auto GOMAXPROCS from cgroup)
- `github.com/pkg/profile` (one-liner profiling)

## 9. Code Review Mandate

Any concurrent change must pass:
1. `go test -race -count=5 ./...`
2. `golangci-lint run --enable-all`
3. Manual review of ownership and cancellation contracts
4. Benchmark comparison showing no regression

**This skill ensures every line of concurrent Go written at Spacecraft Software is production-ready, memory-safe, and scales linearly.**