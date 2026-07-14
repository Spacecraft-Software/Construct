# Spacecraft Lua Guidelines — Extended Reference

This document supplements the main [SKILL.md](../SKILL.md) with concrete code patterns, version-specific notes, and deeper explanations. Load this file when the user asks for "full guidelines", "examples", "complete document", or when you need more than the concise rules.

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

---

## Version Notes (LuaJIT / 5.1 vs. Lua 5.4+)

- **LuaJIT / Lua 5.1:** 
  - Lacks native `<const>` and `<close>` attributes.
  - Rely on JIT-friendly patterns: cache global library functions locally, avoid metatable/OOP dynamic overrides in performance-critical loops, and use `table.new` for pre-allocation if the FFI/table size is known.
  - LuaJIT FFI (`require("ffi")`) provides high-speed C interaction; keep FFI allocations outside hot loops.
- **Lua 5.4+:**
  - Leverage `<const>` to declare read-only variables.
  - Leverage `<close>` and the `__close` metamethod for deterministic RAII resource cleanup (such as file handles, socket descriptors, and lock release).
  - Generational garbage collection is available (`collectgarbage("incremental")` vs. `collectgarbage("generational")`). Generational is generally better for workloads creating many short-lived objects.

---

## Concrete Code Patterns

### 1. High-Performance Module Skeleton

Always cache external functions and global standard library functions. Return a local table containing all exported values.

```lua
-- telemetry.lua
-- Module for handling spacecraft telemetry packets.

local M = {}

-- Cache standard library functions for performance (avoids global table lookups)
local format = string.format
local insert = table.insert
local concat = table.concat
local type = type

-- Local constants (Use upper case)
local MAX_PACKET_SIZE <const> = 1024
local PROTOCOL_VERSION <const> = "v2.0"

---@class TelemetryPacket
---@field id integer
---@field payload string
---@field timestamp integer

--- Parses raw byte string into a telemetry packet structure.
---@param raw_data string
---@return TelemetryPacket? packet, string? error
function M.parse(raw_data)
    if #raw_data > MAX_PACKET_SIZE then
        return nil, "Packet size exceeds maximum limit"
    end
    -- Parsing logic here
    local packet = {
        id = 1,
        payload = raw_data,
        timestamp = os.time(),
    }
    return packet
end

return M
```

---

### 2. Metatable-based OOP with Resource Safety (`__close`)

This pattern combines class structure, inheritance/instantiation, and the Lua 5.4 `<close>` RAII mechanism.

```lua
---@class DeviceConnection
---@field host string
---@field port integer
---@field socket table?
local DeviceConnection = {}
DeviceConnection.__index = DeviceConnection

--- Creates a new device connection instance.
---@param host string
---@param port integer
---@return DeviceConnection
function DeviceConnection.new(host, port)
    local self = setmetatable({
        host = host,
        port = port,
        socket = nil,
    }, DeviceConnection)
    return self
end

--- Simulates opening a network resource.
---@return boolean ok, string? error
function DeviceConnection:connect()
    self.socket = { status = "connected" } -- Placeholder connection
    return true
end

--- Closes the socket resource.
function DeviceConnection:close()
    if self.socket then
        self.socket = nil
        print(format("Closed connection to %s:%d", self.host, self.port))
    end
end

-- Define __close metamethod for Lua 5.4 deterministic RAII
function DeviceConnection:__close(err)
    if err then
        print(format("Closing connection to %s:%d due to error: %s", self.host, self.port, tostring(err)))
    else
        print(format("Closing connection to %s:%d normally", self.host, self.port))
    end
    self:close()
end

-- Usage Example (Lua 5.4):
-- do
--     local conn <close> = DeviceConnection.new("127.0.0.1", 5005)
--     conn:connect()
--     -- Do work... connection will close automatically here, even if errors occur
-- end
```

---

### 3. Pre-allocated Table Pattern

Dynamic table resizing (doubling capacity on reallocation) is expensive. Pre-allocate table sizes in performance-critical code blocks.

```lua
-- LuaJIT table pre-allocation
local has_table_new, table_new = pcall(require, "table.new")

--- Pre-allocates and initializes a sequence table of a given size.
---@param size integer
---@return table
local function create_preallocated_array(size)
    local t
    if has_table_new then
        t = table_new(size, 0) -- pre-allocate size slots for array-part, 0 for hash-part
    else
        t = {}
        -- Standard Lua fallback: hint allocation by filling table constructor or using dummy loop
        -- Note: loop pre-allocation is only effective in some VM implementations
    end
    return t
end

--- Generates a batch of sequence telemetry ids.
---@param count integer
---@return integer[]
local function generate_ids(count)
    local ids = create_preallocated_array(count)
    for i = 1, count do
        ids[i] = i
    end
    return ids
end
```

---

### 4. Coroutine Worker Loop (Cooperative Multitasking)

A modular cooperative task queue that manages asynchronous tasks using coroutines.

```lua
---@class TaskQueue
---@field tasks thread[]
local TaskQueue = {}
TaskQueue.__index = TaskQueue

--- Creates a new task queue.
---@return TaskQueue
function TaskQueue.new()
    return setmetatable({ tasks = {} }, TaskQueue)
end

--- Adds a new cooperative worker task.
---@param fn fun()
function TaskQueue:spawn(fn)
    local co = coroutine.create(fn)
    table.insert(self.tasks, co)
end

--- Runs one step of all active cooperative tasks.
--- Returns true if any tasks are still running.
---@return boolean
function TaskQueue:step()
    local active_tasks = {}
    local has_work = false

    for i = 1, #self.tasks do
        local co = self.tasks[i]
        if coroutine.status(co) ~= "dead" then
            local ok, err = coroutine.resume(co)
            if not ok then
                print(format("Task failed: %s", tostring(err)))
            else
                table.insert(active_tasks, co)
                has_work = true
            end
        end
    end

    self.tasks = active_tasks
    return has_work
end

-- Example Task implementation:
-- queue:spawn(function()
--     print("Task 1 - Step A")
--     coroutine.yield()
--     print("Task 1 - Step B")
-- end)
```

---

### 5. Error Boundary Envelope (xpcall wrapping)

Robust exception wrapping logic to capture debugging information cleanly.

```lua
--- Wraps a dynamic calculation and guarantees no uncaught crash.
---@param operation fun(...): any
---@vararg any arguments to pass to the operation
---@return boolean success, any result_or_error
local function run_safely(operation, ...)
    -- xpcall catches the error and invokes the handler (second arg) 
    -- before the call stack is unwound, preserving diagnostic context.
    local success, result_or_err = xpcall(operation, debug.traceback, ...)
    return success, result_or_err
end
```

---

## Common Pitfalls & How to Avoid Them

### 1. Global Scope Leaks
* **Pitfall:** Forgetting the `local` keyword creates global variables, polluting `_G` and causing hard-to-debug side-effects across unrelated modules.
* **Fix:** Use static analysis tools like `selene` or `luacheck` inside CI/CD to block global assignment checks. Use `local` exclusively.

### 2. Holes in Array Tables
* **Pitfall:** Setting an index to `nil` in the middle of a list (e.g., `t = {10, 20, nil, 40}`) invalidates `#t` (the length operator) and makes `ipairs(t)` terminate early at the `nil` element.
* **Fix:** Use a custom sentinel value (e.g., a unique string `"null"` or empty table `{}`) to represent empty slots, or perform table compaction.

### 3. Loop Concatenation Memory Inflation
* **Pitfall:** Repeatedly concatenating strings in a loop using `..` (e.g., `str = str .. val`) allocates new string segments each step, generating severe garbage collection pressure.
* **Fix:** Insert elements into an array table and join them using `table.concat`:
  ```lua
  local buffer = {}
  for i = 1, 1000 do
      insert(buffer, "packet")
  end
  local result = concat(buffer, "\n")
  ```

### 4. Metatable Complexity / Magic Abuse
* **Pitfall:** Relying extensively on deep metatable chains, `__index` functions that load dynamic files, and magic getters/setters. This harms readability, causes silent runtime failures, and breaks static analyzer typings.
* **Fix:** Keep metatable inheritance shallow (prefer composition over inheritance). Restrict metamethod functions to OOP instantiation, custom operators, and RAII cleanup (`__close`).
