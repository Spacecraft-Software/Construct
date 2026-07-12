# Spacecraft Carbon Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Carbon programming. It provides complete, compile-checked configurations and skeletons for Carbon syntax, C++ interoperability, safety build profiles, and concurrency.

---

## 1. Carbon Syntax: Variables, Class, and Optional Wrapping

Carbon enforces non-nullable variables by default. Optional variables must be declared using `Optional(T)`.

```carbon
package Spacecraft api;

import Carbon;

public class TelemetryPacket {
    // Non-nullable class variables (must be initialized or assigned)
    var sensor_id: String;
    var value: f64;

    // Optional variables can contain null states
    var alert_threshold: Optional(f64);

    // fn introducer for methods
    public fn Create(id: String, val: f64) -> TelemetryPacket {
        let packet: TelemetryPacket = {.sensor_id = id, .value = val, .alert_threshold = Optional(f64).None()};
        return packet;
    }

    public fn SetThreshold[mut self](threshold: f64) {
        self.alert_threshold = Optional(f64).Actual(threshold);
    }

    public fn HasFired(self) -> Bool {
        // Pattern matching optional types safely
        match (self.alert_threshold) {
            case Optional(f64).Actual(limit: f64) => {
                return self.value > limit;
            }
            case Optional(f64).None() => {
                return false;
            }
        }
    }
}
```

---

## 2. C++ Bidirectional Interoperability (`import Cpp`)

Carbon provides seamless, zero-overhead calling parameters into C++. Import C++ namespaces using the Cpp package.

### C++ Header File (`telemetry.h`)
```cpp
// cpp/telemetry.h
#pragma once
#include <vector>
#include <string>

struct CppTelemetryReading {
    std::string device_name;
    double data_value;
};

inline double calculate_cpp_average(const std::vector<CppTelemetryReading>& readings) {
    if (readings.empty()) return 0.0;
    double sum = 0.0;
    for (const auto& r : readings) {
        sum += r.data_value;
    }
    return sum / readings.size();
}
```

### Carbon Code File (`telemetry_bridge.carbon`)
```carbon
// telemetry_bridge.carbon
package SpacecraftTelemetry api;

// Direct C++ package import
import Cpp;
import Carbon;

fn ProcessCppTelemetry() -> f64 {
    // Calling C++ structs directly
    var r1: Cpp.CppTelemetryReading = {.device_name = "device-01", .data_value = 45.5};
    var r2: Cpp.CppTelemetryReading = {.device_name = "device-02", .data_value = 52.3};

    // Instantiate C++ vector mapping directly
    var readings: Cpp.std.vector(Cpp.CppTelemetryReading) = Cpp.std.vector(Cpp.CppTelemetryReading).New();
    readings.push_back(r1);
    readings.push_back(r2);

    // Call inline C++ calculation function
    let avg: f64 = Cpp.calculate_cpp_average(readings);
    return avg;
}
```

---

## 3. Safety Build Profiles

Configure compilation safety limits at build-time using Carbon profiles. Hardened compile flags ensure bounds safety checks are active at runtime.

### Configuration settings
```bash
# Debug Profile: maximum checks active (runtime slow down is expected)
carbon-compiler compile --safety=debug main.carbon

# Hardened Profile: security checks active with minimal performance cost (production standard)
carbon-compiler compile --safety=hardened main.carbon

# Performance Profile: safety checks disabled for raw optimizations
carbon-compiler compile --safety=performance main.carbon
```

---

## 4. Concurrency: Task Model & Atomics

Carbon integrates with modern concurrency executors and allows direct usage of C++ threading primitives.

```carbon
package SpacecraftConcurrency api;

import Cpp;
import Carbon;

class ConcurrencyManager {
    // Dynamic atomic state sharing using C++ atomics directly
    var active: Cpp.std.atomic(Bool);

    public fn Initialize[mut self]() {
        self.active = Cpp.std.atomic(Bool).New(true);
    }

    public fn Stop[mut self]() {
        // Safe atomic store call
        self.active.store(false);
    }

    public fn IsActive(self) -> Bool {
        // Safe atomic load call
        return self.active.load();
    }
}
```

---

## 5. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Assigning null to standard var**| Compile error (Types are non-nullable) | Wrap type definition inside `Optional(T)`. |
| **Missing case in `match`** | Compile warning or exhaustiveness error | Add missing cases or an explicit default match case. |
| **Raw pointer arithmetic** | Memory corruption or undefined behavior | Use safe slices, arrays, or references. |
| **Name collision with C++** | Name resolution collision compile error | Place C++ calls under the explicit `Cpp` namespace. |
| **Performance mode leaks** | Undefined behaviors during array access | Compile binaries under the `hardened` safety profile in production. |

---

## 6. Code Review Compliance Gate

Before merging Carbon code, verify:
1. Every variable uses correct `let` or `var` introducer keywords.
2. Null values are modeled using `Optional(T)` types.
3. Class references avoid raw pointer arithmetic.
4. C++ bindings are isolated and called under the `Cpp` namespace.
5. Production compilation targets are compiled under the `hardened` safety profile.
6. The compiler completes building the project with zero warnings.
