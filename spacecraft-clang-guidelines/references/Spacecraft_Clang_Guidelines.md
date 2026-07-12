# Spacecraft C (Clang) Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Clang 18+ (with `-fbounds-safety`), GCC 12+, Fil-C Compiler

This document expands on the `SKILL.md` for C systems programming. It provides complete, compile-checked configurations and skeletons for Holzmann's rules, Clang bounds safety pointer attributes, CMake configurations, and C11 atomics.

---

## 1. Holzmann's Rules: Bounded Loops & Preallocated Arrays

Gerard J. Holzmann's NASA Power of 10 rules ban dynamic memory allocations after initialization and require all loops to have constant upper bounds that can be statically verified.

```c
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <assert.h>

#define MAX_TELEMETRY_SIZE 256
#define MAX_LOOP_LIMIT 1000

typedef struct {
    uint32_t sensor_id;
    float value;
} TelemetryPacket;

// Static pre-allocated pool (Rule 3: No dynamic allocation after initialization)
static TelemetryPacket telemetry_pool[MAX_TELEMETRY_SIZE];
static uint32_t pool_count = 0;

// Function length is short (Rule 4: fits on one page)
bool register_packet(uint32_t id, float val) {
    // Assertions verify preconditions (Rule 5: high assertion density)
    assert(val >= -100.0f);
    assert(val <= 100.0f);

    if (pool_count >= MAX_TELEMETRY_SIZE) {
        return false;
    }

    // Small variable scope (Rule 6)
    TelemetryPacket *packet = &telemetry_pool[pool_count];
    packet->sensor_id = id;
    packet->value = val;
    pool_count++;

    return true;
}

float calculate_average(void) {
    // Rule 7: check parameters/state
    if (pool_count == 0) {
        return 0.0f;
    }

    float sum = 0.0f;

    // Rule 2: Loop must have a statically verifiable upper bound (MAX_TELEMETRY_SIZE)
    for (uint32_t i = 0; i < MAX_TELEMETRY_SIZE; ++i) {
        if (i >= pool_count) {
            break; // loop terminated safely before max bound
        }
        sum += telemetry_pool[i].value;
    }

    return sum / (float)pool_count;
}
```

---

## 2. Clang Bounds Safety Pointer Attributes (`-fbounds-safety`)

Clang's `-fbounds-safety` extension enables compile-time and runtime bounds checks for pointers in C. Use annotations like `__counted_by` to link pointer variables to length inputs.

```c
#include <stdint.h>
#include <stddef.h>
#include <assert.h>

// __counted_by(count) ensures ptr dereferences are checked against count
void process_readings(const float *__counted_by(count) readings, size_t count) {
    // Validate parameters at entry
    if (readings == NULL || count == 0) {
        return;
    }

    for (size_t i = 0; i < count; ++i) {
        // Under -fbounds-safety, this index access is automatically bounds-checked.
        // Attempting to read index i >= count traps out-of-bounds and panics at runtime.
        float value = readings[i];
        assert(value >= 0.0f);
    }
}
```

---

## 3. CMakeLists.txt Hardened Configuration

Force warnings-as-errors diagnostics and configure sanitizers and bounds safety compiler flags.

```cmake
cmake_minimum_required(VERSION 3.20)
project(SpacecraftC C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Enforce strict compiler warnings
add_compile_options(-Wall -Wextra -Wpedantic -Werror)

# Compiler hardening modes
if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Enable Clang bounds safety extension
    add_compile_options(-fbounds-safety)
    
    # Configure AddressSanitizer and UndefinedBehaviorSanitizer for testing builds
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer)
        add_link_options(-fsanitize=address,undefined)
    endif()
elseif(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    # Enable GCC hardening assertions
    add_compile_definitions(_GLIBCXX_ASSERTIONS _FORTIFY_SOURCE=3)
    add_compile_options(-fhardened -fstack-protector-strong)
endif()

add_executable(telemetry_c_node main.c telemetry.c)
```

---

## 4. Concurrency: C11 Threads & Atomics

Manage concurrency using standard C11 threads (`<threads.h>`) and atomic operations (`<stdatomic.h>`) to avoid race conditions.

```c
#include <threads.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdio.h>

// Atomic variables prevent data races
static _Atomic bool running = true;
static _Atomic uint64_t packet_counter = 0;

int worker_task(void *arg) {
    (void)arg;

    while (atomic_load(&running)) {
        // Perform non-blocking I/O or polling...
        thrd_sleep(&(struct timespec){.tv_sec = 0, .tv_nsec = 50000000}, NULL);

        // Safe atomic increment
        atomic_fetch_add(&packet_counter, 1);
    }
    return 0;
}

int main(void) {
    thrd_t thread;

    // Launch worker thread
    if (thrd_create(&thread, worker_task, NULL) != thrd_success) {
        return 1;
    }

    // Run for a short duration
    thrd_sleep(&(struct timespec){.tv_sec = 1, .tv_nsec = 0}, NULL);

    // Stop worker thread cooperatively
    atomic_store(&running, false);

    // Join thread to prevent leak
    int res;
    thrd_join(thread, &res);

    printf("Processed %lu packets.\n", atomic_load(&packet_counter));
    return 0;
}
```

---

## 5. Legacy C Safety: Fil-C Compiler Setup

Compile legacy C source code with **Fil-C** to enforce dynamic bounds checks and block memory exploits.

### Compilation commands
```bash
# Configure Fil-C compiler paths
export CC=/usr/local/fil-c/bin/clang

# Build C target
$CC -O2 -std=c11 main.c -o telemetry_c_safe

# Executing the binary panics immediately on memory violations
./telemetry_c_safe
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Dynamic allocations** | Memory leaks, heap fragmentation | Replace dynamic allocations with static memory pools. |
| **Pointer arithmetic** | Out-of-bounds memory access | Wrap pointers with Clang `-fbounds-safety` attributes. |
| **Data races on indices** | Undefined behaviors, race conditions| Lock mutexes or use C11 `<stdatomic.h>` primitives. |
| **Unbounded loops** | Infinite loops, hang crashes | Enforce constant upper limits on loops. |
| **Recursion** | Stack overflow crashes | Convert recursive algorithms into iterative loops. |

---

## 7. Code Review Compliance Gate

Before merging C code, verify:
1. Dynamic heap actions are strictly forbidden after initialization.
2. Every loop defines a constant upper limit.
3. No recursive functions or jumping constructs are used.
4. Function inputs check for `NULL` and return values are verified.
5. Pointer arguments use `__counted_by` bounds safety attributes under Clang.
6. C11 `<stdatomic.h>` variable updates are used for thread-safe state variables.
7. Warnings compile cleanly with warnings-as-errors flag `-Werror`.
