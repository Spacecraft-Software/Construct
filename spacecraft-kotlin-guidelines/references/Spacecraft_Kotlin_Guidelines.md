# Spacecraft Kotlin Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-12
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Kotlin systems programming. It provides complete, compile-checked configurations and skeletons for coroutines dispatcher injection, `supervisorScope` isolation, Arrow `Either` error handling, Exposed ORM transactions, and unit testing.

---

## 1. Concurrency: Injected Dispatchers & Structured Scopes

To facilitate deterministic unit testing, avoid hardcoding `Dispatchers.IO` or `Dispatchers.Default` inside classes. Inject a dispatcher wrapper or pass the coroutine context via constructors.

### Dispatcher Provider & Injected Processor
```kotlin
package org.spacecraft.telemetry

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.InputStream
import java.util.UUID

public interface DispatcherProvider {
    public fun io(): CoroutineDispatcher
    public fun default(): CoroutineDispatcher
    public fun main(): CoroutineDispatcher
}

public class DefaultDispatcherProvider : DispatcherProvider {
    override fun io(): CoroutineDispatcher = Dispatchers.IO
    override fun default(): CoroutineDispatcher = Dispatchers.Default
    override fun main(): CoroutineDispatcher = Dispatchers.Main
}

public data class TelemetryData(val id: UUID, val value: Double)

public class TelemetryFileLoader(
    private val dispatchers: DispatcherProvider
) {
    public suspend fun parseTelemetryFile(stream: InputStream): List<TelemetryData> =
        withContext(dispatchers.io()) {
            // Processing blocking stream read on IO dispatcher
            stream.bufferedReader().useLines { lines ->
                // Switch to default CPU dispatcher for parsing strings
                withContext(dispatchers.default()) {
                    lines.asSequence()
                        .mapNotNull { line ->
                            try {
                                val parts = line.split(",")
                                TelemetryData(
                                    id = UUID.fromString(parts.getOrNull(0)),
                                    value = parts.getOrNull(1)?.toDouble() ?: 0.0
                                )
                            } catch (e: Exception) {
                                null
                            }
                        }
                        .toList()
                }
            }
        }
}
```

---

## 2. Structured Concurrency Sibling Failure Isolation (`supervisorScope`)

Use `supervisorScope` when spawning multiple independent child jobs. A failure in one child does not cancel siblings or propagate to the parent scope.

```kotlin
package org.spacecraft.telemetry

import kotlinx.coroutines.async
import kotlinx.coroutines.supervisorScope
import java.net.URL

public class TelemetryBatchDownloader(
    private val loader: TelemetryFileLoader,
    private val dispatchers: DispatcherProvider
) {
    public suspend fun downloadBatch(urls: List<URL>): List<TelemetryData> =
        supervisorScope {
            val deferreds = urls.map { url ->
                async(dispatchers.io()) {
                    try {
                        url.openStream().use { stream ->
                            loader.parseTelemetryFile(stream)
                        }
                    } catch (e: Exception) {
                        // Failures are logged and contained inside this child job
                        emptyList()
                    }
                }
            }
            // Await all and flatten the results
            deferreds.flatMap { it.await() }
        }
}
```

---

## 3. Arrow Either Error Modeling & Exposed ORM Transactions

Manage database operations safely inside transaction boundaries. Model runtime exceptions using Arrow `Either` to avoid throwing exceptions out of business domains.

```kotlin
package org.spacecraft.telemetry

import arrow.core.Either
import arrow.core.left
import arrow.core.right
import org.jetbrains.exposed.sql.Table
import org.jetbrains.exposed.sql.insert
import org.jetbrains.exposed.sql.transactions.transaction
import java.util.UUID

public object TelemetryTable : Table("telemetry") {
    val id = uuid("id")
    val value = double("value")
    override val primaryKey = PrimaryKey(id)
}

public sealed class DatabaseFailure {
    public data class ConnectionError(val details: String) : DatabaseFailure()
    public data class ValidationError(val details: String) : DatabaseFailure()
}

public class TelemetryRepository {
    
    public fun saveTelemetry(data: TelemetryData): Either<DatabaseFailure, Unit> {
        if (data.value < 0.0) {
            return DatabaseFailure.ValidationError("Telemetry value must be positive").left()
        }

        return try {
            transaction {
                TelemetryTable.insert {
                    it[id] = data.id
                    it[value] = data.value
                }
            }
            Unit.right()
        } catch (e: Exception) {
            DatabaseFailure.ConnectionError(e.message ?: "Unknown database error").left()
        }
    }
}
```

---

## 4. Testing: coroutines-test & MockK

Test concurrent classes using `StandardTestDispatcher` and `runTest` from the `kotlinx-coroutines-test` library. Mock dependencies using `MockK`.

```kotlin
package org.spacecraft.telemetry

import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestCoroutineScheduler
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import java.io.ByteArrayInputStream
import java.util.UUID

@OptIn(ExperimentalCoroutinesApi::class)
class TelemetryFileLoaderTest {

    private val scheduler = TestCoroutineScheduler()
    private val testDispatcher = StandardTestDispatcher(scheduler)
    
    private val testDispatchers = object : DispatcherProvider {
        override fun io() = testDispatcher
        override fun default() = testDispatcher
        override fun main() = testDispatcher
    }

    @Test
    fun `test file parsing with valid values`() = runTest(testDispatcher) {
        val id = UUID.randomUUID()
        val mockData = "$id,42.5\n"
        val inputStream = ByteArrayInputStream(mockData.toByteArray())
        
        val loader = TelemetryFileLoader(testDispatchers)
        val result = loader.parseTelemetryFile(inputStream)
        
        assertEquals(1, result.size)
        assertEquals(id, result[0].id)
        assertEquals(42.5, result[0].value)
    }
}
```

---

## 5. Gradle Kotlin DSL Build Configuration (`build.gradle.kts`)

Configure strict compiler checks and treat warnings as errors in your `build.gradle.kts`.

```kotlin
plugins {
    kotlin("jvm") version "1.9.20"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    implementation("io.arrow-kt:arrow-core:1.2.1")
    implementation("org.jetbrains.exposed:exposed-core:0.44.0")
    
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3")
    testImplementation("io.mockk:mockk:1.13.8")
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    kotlinOptions {
        // Enforce all warnings as errors in compilation
        allWarningsAsErrors = true
        jvmTarget = "17"
        freeCompilerArgs = freeCompilerArgs + listOf(
            "-opt-in=kotlinx.coroutines.ExperimentalCoroutinesApi",
            "-opt-in=kotlin.RequiresOptIn"
        )
    }
}

tasks.test {
    useJUnitPlatform()
}
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **`!!` operators** | `NullPointerException` crashes at runtime | Use `?.` optional chaining, `?:` fallback, or smart casts. |
| **Hardcoded Dispatchers** | Unit tests are non-deterministic or slow | Inject the `DispatcherProvider` interface. |
| **Blocking threads inside Default** | Thread pool starvation / lag | Move blocking calls (Stream reading, DB) to `Dispatchers.IO`. |
| **Ignoring coroutine cancellation** | Leaking memory or orphaned jobs | Check `ensureActive()` inside heavy loops or use suspending calls. |
| **Swallowing `CancellationException`** | Parent cancellation fails to stop child | Re-throw `CancellationException` inside catch blocks. |
| **Direct DB updates in UI loops** | Main thread blocks, UI freezes | Wrap Exposed ORM inside transactional, suspended functions on `Dispatchers.IO`. |

---

## 7. Code Review Compliance Gate

Before merging Kotlin code, verify:
1. Android-specific UI and device features are delegated to `@android-skills`.
2. No force-unwraps (`!!`) exist in any production path.
3. Class dispatchers are injected via constructor arguments (no hardcoded `Dispatchers.IO`).
4. Coroutine loops verify cancellation using suspending calls or `ensureActive()`.
5. Exposed database operations run inside explicit `transaction` blocks on `Dispatchers.IO`.
6. Arrow `Either` or similar functional structures model errors (no raw exceptions thrown).
7. Kotlin compiler configurations inside `build.gradle.kts` enforce `allWarningsAsErrors = true`.
