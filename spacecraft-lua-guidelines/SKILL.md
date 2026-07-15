---
name: spacecraft-lua-guidelines
description: Expert guidelines for writing memory-safe, high-performance, and concurrent Lua code targeting Lua 5.1/LuaJIT and Lua 5.4+. Triggers on any request involving Lua script files (.lua), LuaJIT, Neovim configuration, metatable design, tables, to-be-closed (close) variables, constant (const) attributes, error handling (pcall/xpcall), LuaCATS annotations, or StyLua formatting. By Spacecraft Software.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
---

# Spacecraft Lua Guidelines

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

**You are an expert Lua systems engineer at Spacecraft Software specializing in memory-safe, high-performance, and concurrent systems targeting Lua 5.1/LuaJIT and Lua 5.4+.** Always follow these rules when writing, reviewing, or refactoring Lua code. Never deviate.

---

## Core Philosophy

- **Stability First (Standard §3 Priority 1).** Lua is a dynamic language. Type safety and early error detection are crucial. Use LuaCATS (Lua Comment and Type System) annotations for static type-checking and IDE support. Guard FFI boundaries strictly.
- **Performance (Priority 2).** Lua can be extremely fast (especially with LuaJIT). Write JIT-friendly code: declare local variables by default, cache global functions in local scopes, avoid mixed array/hash table designs, and minimize table re-allocations in hot loops.
- **Modern Syntax Adaptability.** Support both legacy Lua 5.1/LuaJIT environments (e.g., Neovim, embedded targets) and modern Lua 5.4+ engines. Detect the execution context and leverage Lua 5.4 features like `<const>` and `<close>` when available.
- **Zero Global Pollution.** Every module must return a local table, and all variables/functions must be local. The global namespace `_G` must remain pristine.

---

## Mandatory Syntax & Layout Conventions

### 1. Spacing and Operators
- **Mandatory Single Space:** Place exactly one space before and after the assignment operator `=`, concatenation operator `..`, arithmetic/comparison operators, and table separators (commas/semicolons).
  - **Correct:** `local sum = a + b`, `local path = dir .. "/" .. file`, `{ x = 1, y = 2 }`
  - **Incorrect:** `local sum=a+b`, `local path = dir.."/"..file`, `{x=1,y=2}`
- **Trailing Commas:** Always include trailing commas in multi-line tables to keep git diffs clean.
  - **Correct:**
    ```lua
    local config = {
        host = "127.0.0.1",
        port = 8080,
    }
    ```
- **Block Formatting:** Use StyLua style formatting: 4 spaces for indentation (no tabs), matching `end` alignments, and space inside table curly braces `{ }`.

### 2. Variables & Attributes
- **Always Local:** Declare every variable using `local`. If a variable is constant, name it in `UPPER_SNAKE_CASE` or use the Lua 5.4 `<const>` attribute where target-compatible.
  - **Correct (Lua 5.4):** `local MAX_RETRIES <const> = 5`
  - **Correct (LuaJIT/5.1):** `local MAX_RETRIES = 5`
- **To-Be-Closed Resource Safety:** For resource management (files, network connections, memory descriptors), use the Lua 5.4 `<close>` attribute. It ensures `__close` is called deterministically when going out of scope, even on errors.
  - **Example:** `local file <close> = io.open("data.txt", "r")`

---

## Memory Safety & Table Design

- **Avoid Table Resizing:** Table growth triggers re-allocation and re-hashing. If the final size of a table is known, pre-allocate it.
  - In LuaJIT, use `require("table.new")(narr, nrec)`.
  - In standard Lua, initialize with dummy values or build the array directly in one table constructor.
- **Array-Hash Separation:** Do not mix array indices (integer keys starting at 1) and hash keys (string keys) in the same table. This causes mixed table layouts that prevent JIT compilation.
- **No Table Holes:** Never insert `nil` values into the middle of an array table. This creates "holes," breaking length operations (`#`) and `ipairs`. Use a placeholder sentinel or filter the table.
- **Avoid Key Deletion in Hot Loops:** Deleting keys by setting `t[key] = nil` can degrade table performance and cause resizing. Prefer creating a new table or reuse pre-allocated slots.

---

## Concurrency & Coroutines

- **Cooperative Multitasking:** Use Lua coroutines (`coroutine.create`, `coroutine.resume`, `coroutine.yield`) to implement concurrent workflows without OS thread overhead.
- **Non-Blocking I/O:** Never run blocking OS-level functions inside cooperative event loops (like Neovim or openresty). Use asynchronous/event-driven APIs or yield during long operations.
- **Clean Coroutine Exit:** In Lua 5.4, call `coroutine.close(co)` to release pending resources and trigger `__close` metamethods inside dead or suspended coroutines.
- **No Shared State Concurrency:** Lua lacks built-in multithreading with shared states. When scaling across threads, run isolated Lua states (`lua_State`) communicating via lock-free message channels (e.g., in Guile Scheme or Rust host runtimes).

---

## Error Handling & Debugging

- **Use xpcall for Stack Traces:** For functions that can fail, do not use bare `pcall`. Use `xpcall` with `debug.traceback` to preserve stack trace information for the error handler.
  ```lua
  local ok, err = xpcall(dangerous_fn, debug.traceback, arg1, arg2)
  ```
- **Fail Gracefully:** Return `nil, error_message` for expected failures (e.g., I/O issues, network timeouts) instead of raising exceptions with `error()`. Reserved `error()` for programmer contract violations (e.g., invalid argument types).

---

## Static Type Safety (LuaCATS)

Always document all public functions, modules, and structures using LuaCATS annotations to ensure static analysis checks:
```lua
---@class TelemetrySender
---@field host string
---@field port integer
local TelemetrySender = {}

---@param host string
---@param port integer
---@return TelemetrySender
function TelemetrySender.new(host, port)
    ---@type TelemetrySender
    local self = setmetatable({}, { __index = TelemetrySender })
    self.host = host
    self.port = port
    return self
end
```

---

## Build, Tooling & CI (Non-Negotiable)

- **Linter & Formatter:** Format code using `stylua`. Check code syntax using `selene` or `luacheck`.
- **Static Analysis:** Require clean runs of the Lua Language Server (`lua-language-server`) diagnostic suite. Enable strict type checking.

---

## Anti-Patterns to Avoid

- ❌ **Global Variable Leaks:** Assigning value to a variable without the `local` keyword (e.g., `x = 10` inside a function).
- ❌ **Mixed-Type Arrays:** Creating tables with integer and string keys combined, which degrades V8/JIT memory optimization.
- ❌ **String Concatenation in Loops:** Using `str = str .. new_val` inside loops. Use `table.insert` and `table.concat` to build strings.
- ❌ **Ignoring `pcall` Errors:** Silently discarding error messages returned from `pcall` / `xpcall`.

---

## Pre-Commit Checklist (Verify Every Time)

- [ ] All variables and functions are declared as `local` (no global leakage).
- [ ] Operators and assignments have exactly one space on both sides.
- [ ] Multiline tables have trailing commas on each field.
- [ ] No `nil` holes exist in array-like tables.
- [ ] String concatenation in loops uses `table.concat` instead of `..`.
- [ ] LuaCATS annotations are present on all functions, classes, and parameter definitions.
- [ ] Error-prone function calls are wrapped in `xpcall(fn, debug.traceback)`.
- [ ] Resources (files, descriptors) are closed using `<close>` (Lua 5.4) or explicit `defer`/`finally` patterns.
- [ ] Code passes formatting checks with `stylua` and static checks with `selene`/`luacheck`.

---

## References & Further Reading

- Load `references/Spacecraft_Lua_Guidelines.md` for full code skeletons (metatable-based OOP, JIT table pre-allocation, coroutine task loops, LuaCATS annotations, error boundary envelopes) when deeper patterns are needed.
- *Further Reading:* Roberto Ierusalimschy's *Programming in Lua (5th Edition)*, *Lua Programming Gems (Lua Performance Tips)*, LuaJIT Performance Guide, and the LuaCATS specification.

When the user requests Lua code, configuration, or reviews, apply this checklist to ensure production-grade Spacecraft code.
