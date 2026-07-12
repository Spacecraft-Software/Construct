# Spacecraft C++ Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Circle Compiler (Safe C++), Clang 18+, GCC 13+, Fil-C Compiler

This document expands on the `SKILL.md` for C++ systems programming. It provides complete, compile-checked configurations and skeletons for Safe C++ syntax, hardened CMake setups, `std::jthread` concurrency, and testing.

---

## 1. Safe C++ (safecpp.org) Borrow Checker Syntax

Safe C++ extends the C++ parser to support compile-time borrow checking and safe execution contexts. Structure code inside `safe` blocks to enforce compile-time safety invariants.

```cpp
// Compilation requires Circle Compiler with Safe C++ enabled.
#include <std2/vector.h>
#include <std2/string.h>
#include <iostream>

public struct TelemetryPacket {
    std2::string id;
    double value;
};

// Declaring a safe function restricts unsafe features inside its block
public safe double calculate_average_value(const std2::vector<TelemetryPacket>& packets) {
    if (packets.empty()) {
        return 0.0;
    }

    double sum = 0.0;
    // The borrow checker tracks lifetimes and prevents collection mutation during iteration
    for (const auto& packet : packets) {
        sum += packet.value;
    }
    return sum / packets.size();
}

public int main() {
    safe {
        std2::vector<TelemetryPacket> list;
        list.push_back(TelemetryPacket{ .id = "sensor-101", .value = 88.5 });
        list.push_back(TelemetryPacket{ .id = "sensor-102", .value = 92.1 });
        
        double avg = calculate_average_value(list);
        std::cout << "Average: " << avg << "\n";
    } // safe context ends
    return 0;
}
```

---

## 2. Hardened Compiler Flags (CMakeLists.txt)

Configure your project to build with runtime safety assertions enabled in GCC and Clang. Trapping out-of-bound errors reduces performance by less than 1% but prevents memory corruptions.

```cmake
cmake_minimum_required(VERSION 3.20)
project(SpacecraftTelemetry CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Force warnings as errors
add_compile_options(-Wall -Wextra -Wpedantic -Werror)

# Apply runtime hardening flags based on compiler type
if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    add_compile_definitions(
        # Enable extensive LIBCPP hardening mode (traps out-of-bounds in std::vector, std::span)
        _LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_EXTENSIVE
    )
    # Enable stack smashing protection and stack layout hardening
    add_compile_options(-fstack-protector-strong -fsanitize=safe-stack)
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
    add_compile_definitions(
        # Enable bounds assertions in libstdc++ containers
        _GLIBCXX_ASSERTIONS
        # Enable source fortification to check string functions
        _FORTIFY_SOURCE=3
    )
    # Enable compiler-directed system hardening options
    add_compile_options(-fhardened -fstack-protector-strong -fcf-protection)
endif()

add_executable(telemetry_node main.cpp telemetry_worker.cpp)
```

---

## 3. Concurrency: Safe Thread Management via `std::jthread`

Avoid raw `std::thread` to prevent crashes due to unjoined threads. Use C++20 `std::jthread` to automatically join upon scope exit and handle cooperative thread cancellation.

```cpp
#include <iostream>
#include <vector>
#include <thread>
#include <mutex>
#include <chrono>
#include <atomic>

class TelemetryWorker {
private:
    std::mutex mtx_;
    std::vector<double> readings_;
    std::atomic<bool> status_active_{false};
    std::jthread worker_thread_;

    // Thread internal loop task
    void run_loop(std::stop_token stop_token) {
        while (!stop_token.stop_requested()) {
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            
            // Simulating sensor poll
            double mock_reading = 42.0;

            // Lock context safely using RAII guard
            {
                std::lock_guard<std::mutex> lock(mtx_);
                readings_.push_back(mock_reading);
            }
        }
    }

public:
    TelemetryWorker() = default;
    
    // Non-copyable
    TelemetryWorker(const TelemetryWorker&) = delete;
    TelemetryWorker& operator=(const TelemetryWorker&) = delete;

    ~TelemetryWorker() {
        // jthread destructor automatically signals cancellation stop request and joins
    }

    void start() {
        status_active_.store(true);
        // jthread takes a function accepting a std::stop_token automatically
        worker_thread_ = std::jthread(&TelemetryWorker::run_loop, this);
    }

    void stop() {
        status_active_.store(false);
        // Request thread interruption cooperatively
        worker_thread_.request_stop();
    }

    std::vector<double> get_readings() {
        std::lock_guard<std::mutex> lock(mtx_);
        return readings_;
    }
};
```

---

## 4. Drop-In Memory Safety: Fil-C Compiler Setup

To run legacy C++ code safely, use the **Fil-C** compiler toolchain. Fil-C replaces raw pointers with capabilities (InvisiCaps) and implements a concurrent garbage collector to block all spatial and temporal bugs.

### Compilation commands
```bash
# Set Fil-C clang compiler paths
export CC=/usr/local/fil-c/bin/clang
export CXX=/usr/local/fil-c/bin/clang++

# Build source code
$CXX -O3 -std=c++20 main.cpp -o telemetry_node_safe

# Running the binary will automatically panic and abort on out-of-bounds access
./telemetry_node_safe
```

---

## 5. Testing: GoogleTest Assertions

Test concurrent code and bounds assertions using GoogleTest.

```cpp
#include <gtest/gtest.h>
#include "telemetry_worker.h"

TEST(TelemetryWorkerTest, CaptureReadingsSuccessfully) {
    TelemetryWorker worker;
    
    worker.start();
    std::this_thread::sleep_for(std::chrono::milliseconds(250));
    worker.stop();

    auto readings = worker.get_readings();
    
    // Verify that data was appended by background jthread
    EXPECT_FALSE(readings.empty());
    for (double val : readings) {
        EXPECT_DOUBLE_EQ(val, 42.0);
    }
}
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Raw Pointer allocation** | Use-after-free, memory leaks | Use `std::make_unique` or `std::make_shared`. |
| **Raw `std::thread` destruction** | `std::terminate()` aborts runtime | Replace thread with C++20 `std::jthread`. |
| **Unprotected shared container** | Data races, memory corruptions | Synchronize accesses using `std::mutex` and `std::lock_guard`. |
| **Out-of-bounds indexing** | Security exploits, silent bugs | Enable `-fhardened` / `_LIBCPP_HARDENING_MODE_EXTENSIVE`. |
| **Recursive mutex locking** | Thread deadlocks | Redesign lock domains; avoid calling locked functions from within locks. |

---

## 7. Code Review Compliance Gate

Before merging C++ code, verify:
1. No raw `new`/`delete` operators or manual pointer arithmetic exists in production files.
2. Background threading utilizes `std::jthread` instead of raw `std::thread`.
3. Mutex locks are managed exclusively using RAII class constructs (`std::lock_guard`).
4. GCC/Clang extensive hardening flags are configured in CMakeLists.txt.
5. Safe C++ blocks utilize `safe` keywords and `std2` library classes.
6. Compilations pass cleanly without diagnostic warnings under `-Werror`.
