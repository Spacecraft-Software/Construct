# Spacecraft Java Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** JDK 21+, OpenJDK 21+, Generational ZGC

This document expands on the `SKILL.md` for Java systems programming. It provides complete, compile-checked configurations and skeletons for Virtual Threads, Structured Concurrency, resource management, and unit testing.

---

## 1. Virtual Threads (Project Loom) Executor Flow

Virtual threads are lightweight, user-mode threads scheduled by the JVM on carrier platform threads. Never pool virtual threads. Create them on-demand per task.

```java
package org.spacecraftsoftware.telemetry;

import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;

public class VirtualThreadProcessor {

    public record TelemetryTask(String sensorId) implements Callable<Double> {
        @Override
        public Double call() throws Exception {
            // Simulate blocking I/O network poll
            Thread.sleep(50); 
            return Math.random() * 100.0;
        }
    }

    public List<Double> fetchAllTelemetry(List<String> sensorIds) throws InterruptedException {
        // CRITICAL: Banish thread pools for virtual threads. Spawn on-demand.
        // Use try-with-resources to automatically close the executor (which awaits task completion).
        try (var executor = Executors.newVirtualThreadPerTaskExecutor()) {
            List<TelemetryTask> tasks = sensorIds.stream()
                .map(TelemetryTask::new)
                .toList();

            List<Future<Double>> futures = executor.invokeAll(tasks);

            return futures.stream()
                .map(f -> {
                    try {
                        return f.get();
                    } catch (InterruptedException | ExecutionException e) {
                        Thread.currentThread().interrupt();
                        throw new RuntimeException("Telemetry fetch failed", e);
                    }
                })
                .toList();
        }
    }
}
```

---

## 2. Preventing Virtual Thread Pinning with ReentrantLock

Avoid using `synchronized` blocks or methods when executing blocking I/O inside virtual threads. If a virtual thread blocks inside `synchronized`, it pins its carrier platform thread, causing carrier thread starvation. Replace `synchronized` with `ReentrantLock`.

```java
package org.spacecraftsoftware.telemetry;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.locks.ReentrantLock;

public class PinFreeTelemetryLogger {

    private final ReentrantLock lock = new ReentrantLock();
    private final List<String> logs = new ArrayList<>();

    public void logEvent(String event) {
        // CRITICAL: Replace synchronized(this) { ... } with explicit ReentrantLock
        // to prevent pinning the carrier thread during downstream blocking calls.
        lock.lock();
        try {
            logs.add(event);
            // Simulate blocking write (disk write / network flush)
            writeToDisk(event);
        } finally {
            lock.unlock(); // Always release inside finally block
        }
    }

    private void writeToDisk(String data) {
        try {
            // Simulate blocking disk I/O
            Thread.sleep(10);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    public List<String> getLogs() {
        lock.lock();
        try {
            return List.copyOf(logs);
        } finally {
            lock.unlock();
        }
    }
}
```

---

## 3. Structured Concurrency (JEP 462+)

Structured Concurrency treats groups of concurrent tasks as a single unit of work, improving reliability and cancellation tracking.

```java
package org.spacecraftsoftware.telemetry;

import java.util.concurrent.StructuredTaskScope;
import java.util.concurrent.StructuredTaskScope.Subtask;
import java.time.Instant;

public class ResilientTaskScheduler {

    public record ServiceResponse(String data, Instant timestamp) {}

    public ServiceResponse queryResilientData() throws Exception {
        // Use StructuredTaskScope to coordinate subtasks. ShutdownOnFailure shuts down
        // all subtasks immediately if any single subtask fails (fail-fast behavior).
        try (var scope = new StructuredTaskScope.ShutdownOnFailure()) {
            
            // Fork subtask queries
            Subtask<String> dbSubtask = scope.fork(() -> queryDatabase());
            Subtask<Instant> timeSubtask = scope.fork(() -> fetchNetworkTime());

            scope.join();           // Join both subtasks
            scope.throwIfFailed();  // Propagate exceptions if any subtask failed

            // Safe retrieval of results
            return new ServiceResponse(dbSubtask.get(), timeSubtask.get());
        }
    }

    private String queryDatabase() throws Exception {
        Thread.sleep(30);
        return "Spacecraft Coordinates Active";
    }

    private Instant fetchNetworkTime() throws Exception {
        Thread.sleep(20);
        return Instant.now();
    }
}
```

---

## 4. Resource Safety: try-with-resources

Always wrap resource declarations implementing `AutoCloseable` in `try-with-resources` statements to prevent heap memory leak traps.

```java
package org.spacecraftsoftware.telemetry;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.nio.file.Path;

public class TelemetryFileReader {

    public String readFirstTelemetryLine(Path filePath) throws IOException {
        // CRITICAL: Banish resource leaks by ensuring try-with-resources handles autoclosing
        try (var fileReader = new FileReader(filePath.toFile());
             var bufferedReader = new BufferedReader(fileReader)) {
            
            String line = bufferedReader.readLine();
            return line != null ? line : "";
        }
    }
}
```

---

## 5. Sequenced Collections and Record Constants

Java 21 introduces Sequenced Collections, standardizing collection retrieval order. Combine with record classes for clean, type-safe data access.

```java
package org.spacecraftsoftware.telemetry;

import java.util.LinkedHashSet;
import java.util.SequencedSet;

public class SequenceManager {

    public record SensorReading(int sensorId, double value) {}

    public void processSequencedReadings() {
        SequencedSet<SensorReading> readings = new LinkedHashSet<>();
        readings.add(new SensorReading(1, 12.3));
        readings.add(new SensorReading(2, 45.6));
        readings.add(new SensorReading(3, 78.9));

        // Retrieve first and last elements cleanly
        SensorReading first = readings.getFirst();
        SensorReading last = readings.getLast();

        System.out.println("First reading: " + first.value());
        System.out.println("Last reading: " + last.value());

        // Traverse in reverse order
        for (SensorReading r : readings.reversed()) {
            System.out.println("Reversed: " + r.sensorId());
        }
    }
}
```

---

## 6. Testing: JUnit 5 Integration

Write structured test validations using JUnit 5 assertions.

```java
package org.spacecraftsoftware.telemetry;

import org.junit.jupiter.api.Test;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class TelemetryWorkerTest {

    @Test
    void testVirtualThreadExecution() throws Exception {
        VirtualThreadProcessor processor = new VirtualThreadProcessor();
        List<String> sensors = List.of("GPS_01", "GYRO_02", "TEMP_03");
        
        List<Double> results = processor.fetchAllTelemetry(sensors);
        
        assertNotNull(results);
        assertEquals(3, results.size());
        for (Double reading : results) {
            assertTrue(reading >= 0.0 && reading <= 100.0);
        }
    }

    @Test
    void testTelemetryFileReaderLeaks() throws IOException {
        Path tempFile = Files.createTempFile("telemetry_mock", ".log");
        Files.writeString(tempFile, "SYSTEM_OK\nDETAILS_MOCK");

        TelemetryFileReader reader = new TelemetryFileReader();
        String result = reader.readFirstTelemetryLine(tempFile);

        assertEquals("SYSTEM_OK", result);
        Files.delete(tempFile);
    }
}
```
