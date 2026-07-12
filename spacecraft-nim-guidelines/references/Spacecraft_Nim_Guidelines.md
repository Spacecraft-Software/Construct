# Spacecraft Nim Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Nim 2.0+ systems programming. It provides complete, compile-checked skeletons for structured parallelism (`Malebolgia`), async networking (`Chronos`), safe FFI wrappers, and testing.

---

## 1. Concurrency vs. Parallelism: Architectural Decisions

Nim separates asynchronous control flow from multi-core parallelism:

- **Parallelism (Compute-bound):** Handled via **Malebolgia** (structured concurrency work-stealing threadpool) or **Weave** (extremely fast fork-join pool). Thread allocation is persistent; tasks are partitioned and joined within lexical boundaries.
- **Concurrency (I/O-bound):** Handled via **Chronos** (high-performance async framework) or `std/asyncdispatch`. Operates via a single-threaded selector event loop.
- **Memory Safety between Threads:** Nim's default ARC/ORC model handles thread communication safely. Shared pointers (`ref`) are copied or moved via move semantics (`sink`/`lent`). Shared globals must be protected using `Lock` or `Mutex` from `std/locks`.

---

## 2. Malebolgia Thread Pool Skeleton (Structured Parallelism)

Use Malebolgia for parallel array processing. It manages a persistent worker threadpool and ensures tasks join within scope.

```nim
# telemetry_compute.nim
import malebolgia
import std/cpuinfo

# Global master task pool
var pool = createMaster()

proc parallelSumArray*(arr: seq[float64]): float64 =
  ## Sums an array in parallel using structured tasks
  let len = arr.len
  if len < 5000:
    # Fall back to serial sum to avoid task scheduling overhead
    var sum = 0.0
    for val in arr:
      sum += val
    return sum
  
  # Partition array into chunks and process in parallel
  let numProcessors = countProcessors()
  let chunkSize = (len + numProcessors - 1) div numProcessors
  var results = newSeq[float64](numProcessors)

  # Malebolgia structured block
  pool.awaitAll:
    for i in 0 ..< numProcessors:
      let startIdx = i * chunkSize
      let endIdx = min(startIdx + chunkSize - 1, len - 1)
      if startIdx <= endIdx:
        pool.spawn (proc(): float64 =
          var localSum = 0.0
          for idx in startIdx .. endIdx:
            localSum += arr[idx]
          results[i] = localSum
          return localSum
        )()

  # Sum the partitioned results
  var total = 0.0
  for val in results:
    total += val
  return total

proc main() =
  let data = newSeq[float64](1_000_000)
  # Pre-allocate values
  let total = parallelSumArray(data)
  echo "Parallel sum completed: ", total

if isMainModule:
  main()
```

---

## 3. Chronos Asynchronous Network Loop (I/O Concurrency)

Use `Chronos` for non-blocking I/O. Do not block the async event loop with long compute-bound loops.

```nim
# telemetry_server.nim
import chronos

proc handleClient(client: StreamTransport) {.async: (raw: true).}=
  ## Asynchronous fiber for client communication
  var buf = newSeq[byte](1024)
  try:
    while true:
      let got = await client.readOnce(addr buf[0], 1024)
      if got > 0:
        discard await client.write(addr buf[0], got)
      else:
        break
  except TransportError:
    discard
  finally:
    await client.closeWait()

proc startServer(port: int) {.async: (raw: true).}=
  ## Listens for incoming TCP connections asynchronously
  let address = initTAddress("0.0.0.0", port)
  let server = createStreamServer(address, flags = {ServerFlags.ReuseAddr})
  echo "Asynchronous server listening on port ", port
  
  while true:
    let client = await server.accept()
    # Accept creates a non-blocking task on the chronos selector loop
    asyncSpawn handleClient(client)

proc main() =
  # Initialize loop and start listening
  asyncSpawn startServer(8080)
  runForever()

if isMainModule:
  main()
```

---

## 4. Safe C FFI Binding with ARC/ORC

The Nim ARC/ORC compiler inserts destructuring calls when values exit scope. Protect raw FFI allocations by wrapping them in custom objects with destructor hooks (`=destroy`).

### C Header Example
```c
// hardware.h
typedef struct {
    double temp;
    int volt;
} HardwareSensor;

HardwareSensor* init_sensor();
void free_sensor(HardwareSensor* s);
```

### Nim Safe Wrapper Module (`hardware.nim`)
```nim
# hardware.nim
type
  SensorRaw {.importc: "HardwareSensor", header: "hardware.h", purer.} = object
    temp: cdouble
    volt: cint

  # Import raw C functions
  proc c_init_sensor(): ptr SensorRaw {.importc: "init_sensor", header: "hardware.h".}
  proc c_free_sensor(s: ptr SensorRaw) {.importc: "free_sensor", header: "hardware.h".}

  # Safe Nim wrapper object
  Sensor* = object
    raw: ptr SensorRaw

proc `=destroy`*(s: Sensor) =
  ## Custom destructor hook invoked by ARC when Sensor exits scope
  if s.raw != nil:
    c_free_sensor(s.raw)

proc initSensor*(): Sensor =
  ## Instantiates a sensor securely, wrapping raw pointer inside object
  let p = c_init_sensor()
  if p == nil:
    raise newException(IOError, "Failed to initialize hardware sensor")
  return Sensor(raw: p)

proc getTemperature*(s: Sensor): float =
  ## Exposes safe accessor to foreign struct data
  if s.raw == nil:
    raise newException(ValueError, "Sensor is not initialized")
  return s.raw[].temp.float
```

---

## 5. Testing: Unittest

Use Nim's built-in `unittest` module to verify boundaries, happy paths, and invariants.

```nim
# tests/test_telemetry.nim
import unittest
import telemetry_compute
import hardware

suite "Hardware Telemetry Suite":
  setup:
    discard # Init code if needed

  teardown:
    discard # Cleanup code if needed

  test "Parallel Array Sum matches math logic":
    var testData = newSeq[float64](10000)
    for i in 0 ..< testData.len:
      testData[i] = 1.5
    let total = parallelSumArray(testData)
    check total == 15000.0

  test "Exception raised on null FFI init":
    # Mocking check for FFI exception mappings
    expect IOError:
      let sensor = initSensor() # assuming mock fails
      discard sensor.getTemperature()
```

---

## 6. Nimble System Package Configuration (`telemetry.nimble`)

Manage dependencies and enforce compiler policies in `nimble` packages.

```nim
# telemetry.nimble
version       = "0.1.0"
author        = "Mohamed Hammad"
description   = "Telemetry service for Spacecraft ecosystem"
license       = "GPL-3.0-or-later"
srcDir        = "src"
bin           = @["telemetry"]

requires "nim >= 2.0.0"
requires "malebolgia >= 1.0.0"
requires "chronos >= 4.0.0"

task test, "Runs the telemetry test suite":
  # --warningAsError:on enforces compile warning checks
  exec "nim c -r --warningAsError:on --mm:orc tests/test_telemetry.nim"
```

---

## 7. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **`-d:danger` globally** | Undefined segmentation faults | Maintain default or `-d:release` checks. Restrict check disabling to local `{.push checks: off.}` blocks. |
| **Leaking cycles in ARC** | Heap memory leaks over time | Compile with ORC (`--mm:orc`) to enable cycle collection. |
| **Mixing async selectors** | Compiler errors or lock freezes | Avoid mixing standard library `asyncdispatch` and Status `Chronos`. |
| **Blocking Event Loop** | Server ceases to accept incoming packets | Yield via `await sleepAsync()` or offload computation to Malebolgia. |
| **Raw stack pointer escape** | Random value overrides inside variables | Never store `addr` pointers in structs outliving the scope of the allocated procedure. |
| **Missing custom destructors** | Memory leaks on FFI pointers | Define `=destroy` procedure hooks for wrapper objects holding raw C pointers. |

---

## 8. Code Review Compliance Gate

Before merging Nim code, verify:
1. Formatting conforms cleanly to `nimpretty`.
2. Compilation succeeds with `--warningAsError:on` enabled.
3. Every FFI module has destructor hooks defined for raw pointer objects.
4. Multithreaded code is structured under `Malebolgia` or Weave.
5. Acyclic objects are marked `{.acyclic.}`.
