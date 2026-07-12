# Spacecraft Swift Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Swift 6.2+ systems programming. It provides complete, compile-checked configurations and skeletons for background execution offloading, SwiftUI ViewModel isolation, Swift Testing with zipped arguments, and optional safety checks.

---

## 1. Swift 6.2 `@concurrent` Background Execution (SE-0461)

Swift 6.2 defaults to staying on the caller's actor for nonisolated async functions. To run computations in the background, explicitly annotate functions with `@concurrent` to hop to the cooperative concurrent pool.

```swift
import Foundation

public struct TelemetryPacket: Sendable, Codable {
    public let id: UUID
    public let value: Double
    public let timestamp: Date
}

/// A service to process telemetry.
public actor TelemetryProcessor {
    private var processedPackets: [TelemetryPacket] = []

    public init() {}

    /// Main entry point isolated to actor.
    public func processPackets(_ rawData: [Data]) async throws {
        // Explicitly offload data parsing to a background concurrent task.
        let parsed = try await parseTelemetryConcurrent(rawData)
        self.processedPackets.append(contentsOf: parsed)
    }

    /// Explicitly offloaded background parsing function.
    /// Runs on the global concurrent threadpool via the @concurrent pragma.
    @concurrent
    private func parseTelemetryConcurrent(_ data: [Data]) async throws -> [TelemetryPacket] {
        try await withThrowingTaskGroup(of: TelemetryPacket.self) { group in
            for rawBytes in data {
                group.addTask {
                    // Check task cancellation before parsing
                    try Task.checkCancellation()
                    let decoder = JSONDecoder()
                    decoder.dateDecodingStrategy = .iso8601
                    return try decoder.decode(TelemetryPacket.self, from: rawBytes)
                }
            }

            var results: [TelemetryPacket] = []
            for try await packet in group {
                results.append(packet)
            }
            return results
        }
    }
}
```

---

## 2. MainActor ViewModel & SwiftUI Isolation

Isolate SwiftUI state updates to the `@MainActor`. Offload heavy I/O tasks to background contexts to prevent UI thread starvation.

```swift
import SwiftUI

@MainActor
public final class TelemetryViewModel: ObservableObject {
    @Published public private(set) var packets: [TelemetryPacket] = []
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let processor = TelemetryProcessor()

    public init() {}

    public func loadPackets(from urls: [URL]) async {
        self.isLoading = true
        self.errorMessage = nil
        
        defer { self.isLoading = false }

        do {
            var rawData: [Data] = []
            // Network fetching is non-blocking async
            for url in urls {
                let (data, _) = try await URLSession.shared.data(from: url)
                rawData.append(data)
            }

            // Processing occurs on background actor, then hops back to MainActor
            try await processor.processPackets(rawData)
        } catch {
            self.errorMessage = error.localizedDescription
        }
    }
}

struct TelemetryView: View {
    @StateObject private var viewModel = TelemetryViewModel()

    var body: some View {
        NavigationStack {
            List(viewModel.packets, id: \.id) { packet in
                HStack {
                    Text(packet.id.uuidString.prefix(8))
                    Spacer()
                    Text(String(format: "%.2f", packet.value))
                }
            }
            .navigationTitle("Telemetry Log")
            .overlay {
                if viewModel.isLoading {
                    ProgressView()
                }
            }
            .task {
                let mockURLs = [URL(string: "https://api.spacecraft.org/t1")!]
                await viewModel.loadPackets(from: mockURLs)
            }
        }
    }
}
```

---

## 3. Zipped Parameterized Swift Testing (SE-0470)

Use the Swift Testing framework for testing. Parameterize tests using `arguments` tuples. Use `zip()` to align inputs rather than performing combinations.

```swift
// Tests/TelemetryTests.swift
import Testing
import Foundation
@testable import TelemetryService

@Suite("Telemetry Service Tests")
struct TelemetryTests {
    
    // Zipped parameterized test case
    @Test("Validate Parsing Outputs", arguments: zip(
        [
            "{\"id\":\"00000000-0000-0000-0000-000000000000\",\"value\":42.5,\"timestamp\":\"2026-07-12T20:00:00Z\"}",
            "{\"id\":\"11111111-1111-1111-1111-111111111111\",\"value\":-10.0,\"timestamp\":\"2026-07-12T21:00:00Z\"}"
        ],
        [42.5, -10.0]
    ))
    func testParsing(jsonString: String, expectedValue: Double) throws {
        guard let data = jsonString.data(using: .utf8) else {
            Issue.record("Failed to convert mock json to UTF8 data")
            return
        }

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        
        let packet = try decoder.decode(TelemetryPacket.self, from: data)
        
        #expect(packet.value == expectedValue)
        #expect(packet.timestamp != nil)
    }

    @Test("Assert invalid JSON throws error")
    func testParsingFailure() {
        let invalidData = "invalid-json-data".data(using: .utf8)!
        let decoder = JSONDecoder()
        
        #expect(throws: DecodingError.self) {
            try decoder.decode(TelemetryPacket.self, from: invalidData)
        }
    }
}
```

---

## 4. Retain Cycles & ARC Memory Hygiene

To prevent leaks in async closures, use weak capturing. Avoid force-unwraps (`!`) to prevent runtime crashes.

```swift
import Foundation

public final class DeviceMonitor {
    private var updateHandler: ((Double) -> Void)?
    private var lastValue: Double = 0.0

    public init() {}

    public func configureHandler() {
        // Use weak self to prevent retain cycles
        self.updateHandler = { [weak self] val in
            guard let self = self else { return }
            self.lastValue = val
            print("Received value: \(val)")
        }
    }

    public func processSample(value: Double?) {
        // Safe unwrapping instead of force-unwrapping (value!)
        guard let unwrappedValue = value else {
            print("Received null telemetry reading")
            return
        }
        self.updateHandler?(unwrappedValue)
    }
}
```

---

## 5. Swift Package Manager Configuration (`Package.swift`)

Configure strict concurrency checking and warnings as errors in `Package.swift`.

```swift
// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "TelemetryService",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "TelemetryService", targets: ["TelemetryService"])
    ],
    dependencies: [],
    targets: [
        .target(
            name: "TelemetryService",
            dependencies: [],
            swiftSettings: [
                // Enforce strict concurrency checks (warnings treated as errors in Swift 6)
                .enableUpcomingFeature("StrictConcurrency"),
                .enableUpcomingFeature("NonisolatedNonsendingByDefault"),
                .enableUpcomingFeature("InferIsolatedConformances"),
                .unsafeFlags(["-warnings-as-errors"])
            ]
        ),
        .testTarget(
            name: "TelemetryServiceTests",
            dependencies: ["TelemetryService"],
            swiftSettings: [
                .enableUpcomingFeature("StrictConcurrency"),
                .unsafeFlags(["-warnings-as-errors"])
            ]
        )
    ]
)
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Force-unwrapping (`!`) optionals** | Thread crashes with fatal error | Use `guard let` or provide defaults using `??`. |
| **Retain cycles in async closures** | Memory leaks (objects are never released) | Capture references weakly: `[weak self]`. |
| **MainActor blockages** | SwiftUI interfaces freeze or frame drops | Annotate heavy functions with `@concurrent` or process on a separate background `actor`. |
| **Unchecked Sendable escapes** | Undefined thread data races | Avoid `@unchecked Sendable`. Use locks only if wrapping FFI structures, verified under Thread Sanitizer. |
| **Cartesian parameterized tests** | Tests run too many combinations | Use `zip()` inside `@Test(arguments: zip(...))` to align arguments. |

---

## 7. Code Review Compliance Gate

Before merging Swift code, verify:
1. Compilation passes with strict concurrency checks and `-warnings-as-errors` active.
2. No force-unwraps (`!`) or force-casts (`as!`) exist in any production path.
3. Every asynchronous closure capturing `self` uses the `[weak self]` capture list.
4. ViewModels update state variables on the `@MainActor`.
5. Unit tests run on the modern `Testing` framework (not `XCTest`).
