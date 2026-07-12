# Spacecraft Python Guidelines — Full Reference

**Version:** 1.0
**Date:** 2026-07-13
**Author:** Mohamed Hammad & Spacecraft Software
**Compatibility:** Claude 3.5+, Claude 4, Grok, and all advanced reasoning models

This document expands on the `SKILL.md` for Python systems programming. It provides complete, compile-checked configurations and skeletons for ProcessPoolExecutor parallelism, asyncio execution, Pydantic v2 validations, slots optimizations, and testing.

---

## 1. Concurrency: ProcessPoolExecutor & Asyncio run_in_executor

Do not block the single-threaded `asyncio` event loop. Offload CPU-heavy calculations to a process pool (bypassing the GIL) and synchronous blocking operations to thread executors.

```python
import asyncio
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
import time
from typing import List

# Helper CPU-bound math calculation
def compute_factorial_sync(n: int) -> int:
    if n < 0:
        raise ValueError("Must be non-negative")
    res = 1
    for i in range(2, n + 1):
        res *= i
    return res

# Helper blocking I/O operation
def blocking_file_read(filepath: str) -> str:
    # Simulates blocking OS operation
    time.sleep(0.1)
    with open(filepath, "r", encoding="utf-8") as f:
        return f.read()

class TelemetryManager:
    def __init__(self, max_processes: int = 4, max_threads: int = 8) -> None:
        self.process_pool = ProcessPoolExecutor(max_workers=max_processes)
        self.thread_pool = ThreadPoolExecutor(max_workers=max_threads)

    async def shutdown(self) -> None:
        self.process_pool.shutdown(wait=True)
        self.thread_pool.shutdown(wait=True)

    async def calculate_batch_factorials(self, numbers: List[int]) -> List[int]:
        """Runs GIL-bound CPU math inside a ProcessPoolExecutor concurrently."""
        loop = asyncio.get_running_loop()
        tasks = [
            loop.run_in_executor(self.process_pool, compute_factorial_sync, num)
            for num in numbers
        ]
        results: List[int] = await asyncio.gather(*tasks)
        return results

    async def read_files_async(self, filepaths: List[str]) -> List[str]:
        """Runs blocking system operations inside ThreadPoolExecutor to prevent event loop lag."""
        loop = asyncio.get_running_loop()
        tasks = [
            loop.run_in_executor(self.thread_pool, blocking_file_read, path)
            for path in filepaths
        ]
        results: List[str] = await asyncio.gather(*tasks)
        return results
```

---

## 2. Typing & Pydantic v2 Ingress Validation

Validate and parse raw dynamic data immediately at the entrance boundaries using Pydantic (v2) models. Use strict typing annotations for mypy.

```python
from datetime import datetime
import json
from uuid import UUID
from pydantic import BaseModel, ConfigDict, Field, ValidationError
from typing import Union, Dict, Any

class TelemetryPacket(BaseModel):
    # Configure model to be immutable and forbid extra parameters
    model_config = ConfigDict(
        frozen=True,
        extra="forbid"
    )

    packet_id: UUID = Field(alias="packetId")
    value: float = Field(ge=0.0) # validation: must be >= 0.0
    timestamp: datetime

# Functional result mappings
class ValidationSuccess:
    __slots__ = ("data",)
    def __init__(self, data: TelemetryPacket) -> None:
        self.data = data

class ValidationFailure:
    __slots__ = ("error",)
    def __init__(self, error: str) -> None:
        self.error = error

ValidationResult = Union[ValidationSuccess, ValidationFailure]

def parse_incoming_payload(json_str: str) -> ValidationResult:
    try:
        # Pydantic v2 direct JSON parsing (faster than json.loads)
        packet = TelemetryPacket.model_validate_json(json_str)
        return ValidationSuccess(packet)
    except ValidationError as e:
        return ValidationFailure(str(e))
```

---

## 3. Slots Attribute Memory Optimization

For classes that hold data and are instantiated frequently (e.g. millions of packet structures), define `__slots__` to prevent dynamic `__dict__` generation, lowering memory usage.

```python
from uuid import UUID
from typing import Tuple

class CompactReading:
    # Explicit slots reduce memory layout size and speed up variable lookup
    __slots__ = ("sensor_id", "measurements")

    def __init__(self, sensor_id: UUID, measurements: Tuple[float, ...]) -> None:
        self.sensor_id: UUID = sensor_id
        self.measurements: Tuple[float, ...] = measurements

    def average(self) -> float:
        if not self.measurements:
            return 0.0
        return sum(self.measurements) / len(self.measurements)
```

---

## 4. Testing: pytest & pytest-asyncio

Test concurrent routines and validations using `pytest` and `pytest-asyncio`. Ensure warnings are treated as errors.

```python
# tests/test_telemetry.py
import pytest
from uuid import uuid4
from datetime import datetime, timezone
from telemetry_manager import TelemetryManager
from telemetry_schemas import parse_incoming_payload, ValidationSuccess, ValidationFailure

@pytest.fixture
def manager() -> TelemetryManager:
    return TelemetryManager(max_processes=2, max_threads=2)

@pytest.mark.asyncio
async def test_factorial_calculations(manager: TelemetryManager) -> None:
    numbers = [5, 6, 7]
    results = await manager.calculate_batch_factorials(numbers)
    assert results == [120, 720, 5040]

def test_pydantic_valid_parsing() -> None:
    uuid_str = str(uuid4())
    payload = f'{{"packetId": "{uuid_str}", "value": 45.2, "timestamp": "2026-07-12T20:00:00Z"}}'
    
    result = parse_incoming_payload(payload)
    
    assert isinstance(result, ValidationSuccess)
    assert result.data.value == 45.2
    assert result.data.timestamp.tzinfo == timezone.utc

def test_pydantic_invalid_parsing() -> None:
    # Value violates constraint (must be >= 0.0)
    payload = '{"packetId": "invalid-uuid", "value": -10.0, "timestamp": "2026-07-12T20:00:00Z"}'
    
    result = parse_incoming_payload(payload)
    
    assert isinstance(result, ValidationFailure)
```

---

## 5. Tooling Configuration (`pyproject.toml`)

Standard configuration mapping for `Ruff` rules, `Mypy` strict settings, and `Pytest` warnings.

```toml
[tool.poetry]
name = "telemetry-service"
version = "0.1.0"
description = "Spacecraft Telemetry Service in Python"
authors = ["Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>"]
license = "GPL-3.0-or-later"

[tool.poetry.dependencies]
python = "^3.12"
pydantic = "^2.5.0"
pytest = "^7.4.0"
pytest-asyncio = "^0.21.0"
ruff = "^0.1.0"
mypy = "^1.6.0"

[tool.mypy]
strict = true
disallow_any_generics = true
warn_unused_ignores = true
plugins = ["pydantic.mypy"]

[tool.pydantic-mypy]
init_forbid_extra = true
init_typed = true
warn_required_dynamic_aliases = true

[tool.ruff]
target-version = "py312"
line-length = 100

[tool.ruff.lint]
select = [
    "E", "F", "W",    # Pycodestyle & Pyflakes
    "B",              # Flake8-bugbear
    "I",              # Isort
    "RUF",            # Ruff-specific rules
    "ANN",            # Type annotations
    "ASYNC",          # Asyncio lints
    "TCH",            # Type checking blocks
    "UP"              # Pyupgrade
]

[tool.ruff.format]
quote-style = "double"
indent-style = "space"

[tool.pytest.ini_options]
minversion = "7.0"
addopts = "-ra -q"
testpaths = ["tests"]
filterwarnings = [
    "error", # Treat all warnings as compilation errors
]
```

---

## 6. Common Pitfalls & Troubleshooting

| Pitfall | Symptom | Corrective Action |
| :--- | :--- | :--- |
| **Synchronous sleep in async** | Event loop blocks, other jobs halt | Replace `time.sleep` with `await asyncio.sleep`. |
| **Synchronous filesystem in async** | Event loop thread freezes | Offload execution using `loop.run_in_executor(None, sync_func)`. |
| **Threading for math operations** | High CPU context switches, slow speed | Replace threading with `ProcessPoolExecutor`. |
| **Loose typing annotations** | Mypy fails in strict mode | Annotate variables and parameters explicitly. |
| **Dynamic properties on dataclasses**| Memory growth on long processes | Declare class variables inside `__slots__` tuple. |
| **Mocking Pydantic dynamic fields** | Mypy failures on class initializers | Configure Pydantic Mypy plugin with `init_typed = true`. |

---

## 7. Code Review Compliance Gate

Before merging Python code, verify:
1. Static typing assertions pass under strict Mypy checks (no untyped parameters).
2. Asynchronous loops do not execute synchronous blocking operations.
3. Heavy numerical calculations run in separate processes (`ProcessPoolExecutor`).
4. High frequency data structures define `__slots__` tuples.
5. Ingress parsing checks are wrapped in Pydantic models configured with `frozen=True`.
6. Ruff lints pass cleanly and pytest treats warnings as error configurations.
