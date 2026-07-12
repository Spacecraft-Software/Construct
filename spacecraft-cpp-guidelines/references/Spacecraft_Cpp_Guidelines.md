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

## 3. Concurrency: Safe Thread Management, Named Locks, scoped_lock, and Condition Predicates

Avoid raw `std::thread` to prevent crashes due to unjoined threads. Use C++20 `std::jthread` to automatically join upon scope exit, and always name your lock guards to prevent immediate release vulnerabilities.

```cpp
#include <iostream>
#include <vector>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <atomic>

class TelemetryWorker {
private:
    std::mutex mtx_;
    std::condition_variable cv_;
    std::vector<double> readings_;
    std::atomic<bool> status_active_{false};
    std::jthread worker_thread_;

    // Thread internal loop task
    void run_loop(std::stop_token stop_token) {
        while (!stop_token.stop_requested()) {
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            
            // Simulating sensor poll
            double mock_reading = 42.0;

            // Lock context safely using a NAMED RAII guard
            {
                // CRITICAL: Always name the lock variable (here: 'lock'). 
                // DO NOT write: std::lock_guard<std::mutex>(mtx_); which creates a temporary that is immediately destroyed!
                std::lock_guard<std::mutex> lock(mtx_);
                readings_.push_back(mock_reading);
            }
            cv_.notify_one();
        }
    }

public:
    TelemetryWorker() = default;
    
    // Non-copyable (Rule of Five)
    TelemetryWorker(const TelemetryWorker&) = delete;
    TelemetryWorker& operator=(const TelemetryWorker&) = delete;

    ~TelemetryWorker() {
        // jthread destructor automatically signals cancellation stop request and joins
    }

    void start() {
        status_active_.store(true);
        worker_thread_ = std::jthread(&TelemetryWorker::run_loop, this);
    }

    void stop() {
        status_active_.store(false);
        worker_thread_.request_stop();
    }

    // Safely await new reading using a condition variable predicate loop
    double await_next_reading() {
        std::unique_lock<std::mutex> lock(mtx_);
        
        // CRITICAL: Always pass a predicate lambda to cv.wait to prevent spurious wakeup bugs
        cv_.wait(lock, [this] { return !readings_.empty(); });
        
        double val = readings_.back();
        readings_.pop_back();
        return val;
    }
};

// Deadlock-free Multi-Mutex Locking
struct Account {
    std::mutex mtx_;
    double balance_ = 0.0;
};

void transfer_funds(Account& from, Account& to, double amount) {
    // CRITICAL: Use std::scoped_lock to acquire multiple mutexes atomically and deadlock-free
    std::scoped_lock lock(from.mtx_, to.mtx_);
    
    from.balance_ -= amount;
    to.balance_ += amount;
}
```

---

## 4. Lifecycle Management: Rule of Zero & Rule of Five

If a class does not manage resources directly, rely on the compiler-generated defaults (**Rule of Zero**). If a class manages resources manually, you must explicitly declare or delete all five special member functions (**Rule of Five**).

```cpp
#include <cstddef>
#include <memory>
#include <algorithm>
#include <stdexcept>

class SafeBufferManager {
private:
    std::unique_ptr<char[]> buffer_;
    std::size_t size_;

public:
    explicit SafeBufferManager(std::size_t size)
        : buffer_(std::make_unique<char[]>(size)), size_(size) {}

    // --- RULE OF FIVE IMPLEMENTATION ---

    // 1. Destructor
    ~SafeBufferManager() = default; // unique_ptr handles cleanup automatically

    // 2. Copy Constructor (Deep copy allocation)
    SafeBufferManager(const SafeBufferManager& other)
        : buffer_(std::make_unique<char[]>(other.size_)), size_(other.size_) {
        std::copy_n(other.buffer_.get(), size_, buffer_.get());
    }

    // 3. Copy Assignment Operator (Strong exception safety)
    SafeBufferManager& operator=(const SafeBufferManager& other) {
        if (this != &other) {
            auto temp_buffer = std::make_unique<char[]>(other.size_);
            std::copy_n(other.buffer_.get(), other.size_, temp_buffer.get());
            
            buffer_ = std::move(temp_buffer);
            size_ = other.size_;
        }
        return *this;
    }

    // 4. Move Constructor
    SafeBufferManager(SafeBufferManager&& other) noexcept
        : buffer_(std::move(other.buffer_)), size_(other.size_) {
        other.size_ = 0;
    }

    // 5. Move Assignment Operator
    SafeBufferManager& operator=(SafeBufferManager&& other) noexcept {
        if (this != &other) {
            buffer_ = std::move(other.buffer_);
            size_ = other.size_;
            other.size_ = 0;
        }
        return *this;
    }

    // Safe accessor
    char& at(std::size_t index) {
        if (index >= size_) {
            throw std::out_of_range("Buffer bounds overflow");
        }
        return buffer_[index];
    }
};
```

---

## 5. Type-Safe Interfaces: Scoped Enums, std::array, and Ownership

Do not use legacy C-style features. Enforce type safety at the API boundary.

```cpp
#include <array>
#include <string_view>
#include <iostream>

// CRITICAL: Prefer scoped enum class to prevent global scope contamination and implicit casting
enum class TelemetryChannel {
    velocity,
    altitude,
    temperature,
    pressure
};

// CRITICAL: Prefer std::array over raw C arrays to maintain bounds safety information
struct SensorCalibration {
    TelemetryChannel channel;
    std::array<double, 4> coefficients; // Safe bounds
};

// Use std::string_view for zero-overhead, read-only string references
void log_calibration(std::string_view sensor_name, const SensorCalibration& cal) {
    std::cout << "Sensor: " << sensor_name << ", coeffs: ";
    for (double val : cal.coefficients) {
        std::cout << val << " ";
    }
    std::cout << "\n";
}
```

---

## 6. Drop-In Memory Safety: Fil-C Compiler Setup

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

## 7. Testing: GoogleTest Assertions

Test concurrent code, condition variables, and bounds assertions using GoogleTest.

```cpp
#include <gtest/gtest.h>
#include "telemetry_worker.h"

TEST(TelemetryWorkerTest, CaptureReadingsSuccessfully) {
    TelemetryWorker worker;
    
    worker.start();
    std::this_thread::sleep_for(std::chrono::milliseconds(250));
    worker.stop();

    double latest_value = worker.await_next_reading();
    EXPECT_DOUBLE_EQ(latest_value, 42.0);
}
```

---

## 8. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Raw Pointer allocation** | Use-after-free, memory leaks | Use `std::make_unique` or `std::make_shared`. |
| **Raw `std::thread` destruction** | `std::terminate()` aborts runtime | Replace thread with C++20 `std::jthread`. |
| **Unprotected shared container** | Data races, memory corruptions | Synchronize accesses using `std::mutex` and `std::lock_guard`. |
| **Unnamed Lock Guards** | Data races on protected resources | Always name the lock variable: `std::lock_guard<std::mutex> lock(mtx);`. |
| **Spurious Wakeups** | Program proceeds with invalid data | Always wait on condition variables using a predicate lambda. |
| **Rule of Five Violation** | Double free, slice copies, memory leaks | If writing a destructor or copy/move operation, implement all 5. |
| **C-Style raw arrays** | Buffer overflows, decay to pointer | Use `std::array` or `std::vector` combined with hardening. |
| **Plain enums or macros** | Namespace clashes, implicit cast bugs | Use scoped `enum class` and compile-time `constexpr` values. |
| **Out-of-bounds indexing** | Security exploits, silent bugs | Enable `-fhardened` / `_LIBCPP_HARDENING_MODE_EXTENSIVE`. |
| **Recursive mutex locking** | Thread deadlocks | Redesign lock domains; use `std::scoped_lock` for multi-locking. |

---

## 9. Code Review Compliance Gate

Before merging C++ code, verify:
1. No raw `new`/`delete` operators or manual pointer arithmetic exists in production files.
2. Background threading utilizes `std::jthread` instead of raw `std::thread`.
3. Mutex locks are managed exclusively using *named* RAII class constructs.
4. Multi-mutex acquisition is handled atomically using `std::scoped_lock`.
5. All condition variable wait loops include a predicate checking lambda to prevent spurious wakeups.
6. Custom resource managers conform fully to the Rule of Zero or Rule of Five guidelines.
7. C-style arrays are banned in favor of `std::array` or `std::vector`.
8. Enums are declared as scoped `enum class` to prevent implicit casts.
9. GCC/Clang extensive hardening flags are configured in CMakeLists.txt.
10. Safe C++ blocks utilize `safe` keywords and `std2` library classes.
11. Compilations pass cleanly without diagnostic warnings under `-Werror`.
