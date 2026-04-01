# Rust-Guidelines — AI Skill

> **Trigger:** Apply this skill whenever writing, reviewing, or refactoring Rust code.
> **Source:** Microsoft Pragmatic Rust Guidelines + community best practices.

---

## 1. Error Handling

### Libraries — Use Canonical Error Structs
- Errors are situation-specific **structs**, not catch-all enums.
- Include a `Backtrace`, an optional upstream cause, and `is_xxx()` helpers.
- Keep inner `ErrorKind` enum `pub(crate)` — expose via helper methods only.
- Implement `Debug`, `Display`, and `std::error::Error`.
- Use separate error types per domain (`DownloadError`, `ParseError`), not one global enum.

```rust
use std::backtrace::Backtrace;

#[derive(Debug)]
pub struct ConfigError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind { Io(std::io::Error), Parse }

impl ConfigError {
    pub(crate) fn io(e: std::io::Error) -> Self {
        Self { kind: ErrorKind::Io(e), backtrace: Backtrace::capture() }
    }
    pub fn is_io(&self) -> bool { matches!(self.kind, ErrorKind::Io(_)) }
}
```

### Applications — Use `anyhow` / `eyre`
- Apps (and app-only crates) may use `anyhow::Result` or `eyre::Result`.
- Pick one and use it consistently; do not mix multiple error crates.
- Libraries consumed by multiple crates must **not** use `anyhow`/`eyre`.

---

## 2. Safety & Soundness

### `unsafe` Rules
- `unsafe` may **only** mark functions where misuse risks **undefined behavior (UB)**.
  Do **not** use `unsafe` to mean "dangerous" (e.g., `delete_database()`).
- Valid reasons: novel abstractions, benchmarked performance, FFI/platform calls.
- Every `unsafe` block needs a `// SAFETY:` comment explaining correctness.
- Never bypass `Send`/`Sync` bounds or lifetimes via ad-hoc `transmute`.
- All unsafe code must pass **Miri** and follow the
  [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/).

### Soundness — No Exceptions
- A safe function that can cause UB under **any** safe calling pattern is **unsound**.
- Unsound code is **never** acceptable. Expose `unsafe fn` instead and document invariants.
- Soundness boundaries align with **module** boundaries — internal safe helpers may rely
  on guarantees upheld elsewhere in the same module.

---

## 3. Panics & Error Philosophy

- **Panics mean "stop the program"** — they are not exceptions.
- Do **not** use panics to communicate errors upstream or handle recoverable conditions.
- **Do** panic on detected programming bugs (invariant violations, impossible states).
- Prefer **"correct by construction"**: use types to make bad states unrepresentable.
- Valid panic uses: `expect("invariant")`, const-context `unwrap`, poisoned locks.

---

## 4. Naming & Style

- **No weasel words** in type names: avoid `Service`, `Manager`, `Factory`.
  Use `Bookings` not `BookingService`; `FooBuilder` not `FooFactory`.
- **First doc sentence** ≤ 15 words, on one line — it becomes the module summary.
- Follow [Rust API Guidelines naming](https://rust-lang.github.io/api-guidelines/checklist.html):
  `as_`, `to_`, `into_` conversions; getter names without `get_` prefix.
- Prefer **regular functions** over associated functions for non-instance logic.
- Magic values → named constants with comments explaining *why* and *side effects*.

---

## 5. Documentation

### Canonical Sections (in order)
```rust
/// Summary sentence < 15 words.
///
/// Extended documentation.
///
/// # Examples
/// # Errors       (if returns Result)
/// # Panics       (if may panic)
/// # Safety       (if unsafe)
/// # Abort        (if may abort)
pub fn foo() {}
```

- Do **not** create `# Parameters` tables — describe params inline in prose.
- Every public module needs `//!` docs; every public item needs `///` docs.
- Mark re-exports with `#[doc(inline)]` (except std/3rd-party types).

---

## 6. Type Design

### Common Traits — Eagerly Implement
`Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Default`.
Add `Display` when the type is meant to be read.

### Public Types
- All public types **must** implement `Debug`.
- Sensitive types: custom `Debug` impl that redacts secrets + a unit test proving it.
- Types should be `Send` unless instantaneous/never held across `.await`.
- Assert `Send` at compile time: `const _: () = assert_send::<MyType>();`

### Strong Types
- Use `PathBuf`/`Path` for OS paths, not `String`/`&str`.
- Use `impl RangeBounds<T>` instead of `(low, high)` pairs.
- Use newtypes (`C-NEWTYPE`) to avoid primitive obsession.

### Construction
- `Foo::new()` for simple construction (even if `Default` is also impl'd).
- ≤ 2 optional params → `with_x()` methods. ≥ 3 → **Builder** pattern.
- Builders: `Foo::builder() -> FooBuilder`, chainable setters, `.build()` final.
- ≥ 4 constructor params → group into semantic helper types (cascaded init).

---

## 7. API Design for Libraries

### Accept Flexible Input
```rust
fn print(x: impl AsRef<str>) {}
fn read_file(x: impl AsRef<Path>) {}
fn send(x: impl AsRef<[u8]>) {}
fn parse_data(r: impl std::io::Read) {} // "sans-io"
```

### Don't Leak External Types
- Prefer `std` types in public APIs.
- Sibling crates in an umbrella may share types freely.
- Behind feature flags, leaking types like `serde::Serialize` is OK.

### Don't Glob Re-Export
```rust
// Bad
pub use foo::*;
// Good
pub use foo::{A, B, C};
```

### Avoid Wrappers in APIs
- Hide `Arc<T>`, `Rc<T>`, `Box<T>`, `RefCell<T>` behind clean APIs.
- Accept `&T`, `&mut T`, or `T` at API boundaries.

### Prefer Concrete Types > Generics > `dyn Trait`
- Use enum dispatch + mocking over trait objects when possible.
- If `dyn Trait` is needed, wrap it in a newtype to preserve flexibility.

### Services are `Clone` (via `Arc<Inner>`)
```rust
#[derive(Clone)]
pub struct Service { inner: Arc<ServiceInner> }
```

### Essential Functionality is Inherent
- Core methods live in `impl Foo {}`, not behind trait impls.
- Trait impls forward to inherent methods.

### Features are Additive
- No `no-std` features — use `std` feature instead.
- Adding a feature must never disable or modify existing public items.

### Escape Hatches
- Types wrapping native handles provide `unsafe from_native` / `into_native`.

---

## 8. Crate Organization

- **Split aggressively**: if a submodule can stand alone, extract it to a crate.
- Crates are for independent units; features are for opt-in extras.
- Re-join via umbrella crates only when needed (proc macros, runtime bundles).
- Libraries must build on all Tier 1 platforms with only `cargo` + `rustc`.
- `-sys` crates: embed sources, use `cc` crate, pre-generate `bindgen` glue.

---

## 9. Performance

- **Identify hot paths early**, create benchmarks (criterion/divan), profile regularly.
- Enable debug symbols for benchmarks: `[profile.bench] debug = 1`.
- Optimize for **throughput** (items/CPU-cycle), avoid empty spin-waits.
- Batch work, exploit CPU caches, partition independently per thread.
- Long-running async tasks: insert `yield_now().await` every ~10–100 μs of CPU work.
- Use **mimalloc** as the global allocator in applications for ~15–25% gains.

```rust
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

### Common Performance Pitfalls
- Frequent re-allocations (cloned/growing/`format!`-assembled strings).
- Short-lived allocations → consider bump allocators.
- Repeated re-hashing of equal data.
- Default hasher where collision resistance is unnecessary.

---

## 10. Logging & Telemetry

- Use **structured logging** with message templates — avoid `format!()` in log calls.
- **Name events** with dot-notation: `<component>.<operation>.<state>`.
- Follow **OpenTelemetry semantic conventions** for attribute names.
- **Redact sensitive data** (emails, tokens, PII) — consider `data_privacy` crate.

```rust
event!(
    name: "file.open.success",
    Level::INFO,
    file.path = path.display(),
    "file opened: {{file.path}}",
);
```

---

## 11. Testing & Mocking

- I/O and syscalls must be **mockable** (clocks, network, entropy, filesystem).
- Use enum dispatch: `Native` vs `Mocked(MockCtrl)` behind `#[cfg(feature = "test-util")]`.
- Test utilities (mocking, sensitive data inspection, safety overrides) → **feature-gated**.
- Return mock controllers as tuples: `fn new_mocked() -> (Self, MockCtrl)`.
- Ensure good test coverage of observable behavior to enable safe AI-assisted refactoring.

---

## 12. Static Verification & Linting

### Recommended Compiler Lints (`Cargo.toml`)
```toml
[lints.rust]
ambiguous_negative_literals = "warn"
missing_debug_implementations = "warn"
redundant_imports = "warn"
redundant_lifetimes = "warn"
trivial_numeric_casts = "warn"
unsafe_op_in_unsafe_fn = "warn"
unused_lifetimes = "warn"
```

### Recommended Clippy Lints
```toml
[lints.clippy]
cargo = { level = "warn", priority = -1 }
complexity = { level = "warn", priority = -1 }
correctness = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
style = { level = "warn", priority = -1 }
suspicious = { level = "warn", priority = -1 }
# Restriction lints (selected):
allow_attributes_without_reason = "warn"
clone_on_ref_ptr = "warn"
map_err_ignore = "warn"
string_to_string = "warn"
undocumented_unsafe_blocks = "warn"
unused_result_ok = "warn"
```

### Use `#[expect]` Not `#[allow]`
```rust
#[expect(clippy::unused_async, reason = "API fixed, will use I/O later")]
pub async fn ping() {}
```

### Toolchain
- `rustfmt` for formatting, `cargo-audit` for CVEs, `cargo-hack` for feature combos,
  `cargo-udeps` for unused deps, `miri` for unsafe validation.

---

## 13. FFI

- Each DLL has its own statics, type layouts, and `TypeId` sets.
- Only share **portable** data across DLL boundaries: `#[repr(C)]`, no statics interaction,
  no `TypeId` dependency, no pointers to non-portable data.
- Transferring `String`, `Vec`, `Box`, `tokio` types, or `#[repr(Rust)]` structs
  across DLL boundaries causes UB.
- Prefer established interop libraries over hand-rolled `unsafe` FFI.

---

## 14. Avoid Statics in Libraries

- Statics risk silent duplication across crate versions (especially `0.x`).
- If correctness depends on a single shared value, don't use `static`.
- Statics purely for caching/performance are acceptable.
- Prefer dependency injection or builder-injected state.

---

## 15. AI-Friendly Coding

- **Idiomatic Rust** = better AI comprehension. Follow upstream guidelines.
- **Thorough docs + examples** help AI agents generate correct code.
- **Strong types** compensate for AI's lack of deep understanding.
- **Testable APIs** let AI iterate quickly with feedback loops.
- **Good test coverage** enables safe AI-driven refactoring.

---

## Quick Reference Checklist

- [ ] Error types are structs with `Backtrace`, `Debug`, `Display`, `Error`
- [ ] No unsound code — ever
- [ ] `unsafe` only for UB-relevant code, with `// SAFETY:` comments
- [ ] Panics only for programming bugs, not control flow
- [ ] All public types implement `Debug`; sensitive types redact secrets
- [ ] First doc sentence ≤ 15 words
- [ ] `#[doc(inline)]` on `pub use` re-exports
- [ ] No glob re-exports (`pub use foo::*`)
- [ ] `impl AsRef<T>` for flexible function inputs
- [ ] Builder pattern for ≥ 3 optional construction params
- [ ] Features are additive; test utils behind `test-util` feature
- [ ] Clippy pedantic + restriction lints enabled
- [ ] `#[expect(...)]` over `#[allow(...)]`
- [ ] Hot paths benchmarked and profiled
- [ ] Structured logging with named events and OTel conventions
- [ ] I/O is mockable; no ad-hoc file/network access in libraries
- [ ] Types are `Send` unless there's a specific reason not to be
- [ ] Services use `Clone` via `Arc<Inner>` pattern
- [ ] mimalloc set as global allocator in applications
