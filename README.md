<!--
  README for the Spacecraft Software `Construct` repository.
  Audience: humans browsing on GitHub, and LLM agents loading these skills.
  Maintenance: keep the skill list in §2 aligned with the top-level directories.
-->

# Spacecraft Software Construct

A collection of **Claude / LLM agent skills** used across the Spacecraft Software
ecosystem. Each top-level directory is a self-contained skill — a `SKILL.md`
that the agent loads on demand, plus optional `references/` files consulted
only when a deeper lookup is warranted.

These skills encode conventions, tool preferences, brand rules, and compliance
requirements so agents produce Spacecraft Software-consistent output without needing
the rules re-attached to every prompt.

<!-- §1 — Who this is for -->
## Audience

- **LLM agents** (Claude Code, Gemini CLI, Copilot CLI, Codex, etc.) loading
  skills from `~/.claude/skills/`, `~/.gemini/skills/`, `~/.codex/skills/`.
- **Humans** reviewing, extending, or auditing Spacecraft Software's conventions.

<!-- §2 — Skill catalogue: keep alphabetical, one line per skill -->
## Skills in this repository

| Skill | Purpose |
|-------|---------|
| [`gnu-coding-standards`](gnu-coding-standards/) | Applies the GNU Coding Standards to C, Rust, GNU Guile, Go, and Python — error-message grammar, CLI contract, i18n, build conventions, and free-software philosophy. |
| [`gnu-free-software`](gnu-free-software/) | Self-sufficient, distributable skill to produce and enforce **free software** in the FSF/GNU tradition — free license + `COPYING`, GNU Coding Standards (`--version`/`--help`, Texinfo, ChangeLog) across C/Guile/Rust/Go/Python, the JavaScript Trap, and GNU vocabulary. FSF-default; stands alone (no other skill required). |
| [`spacecraft-guile-guidelines`](spacecraft-guile-guidelines/) | Write idiomatic, functional, concurrent GNU Guile (Guile Scheme 3.x) — fibers/CSP, SRFI-1, tail calls, hygienic macros. |
| [`microsoft-rust-guidelines`](microsoft-rust-guidelines/) | Enforces Microsoft Pragmatic Rust Guidelines before any `.rs` edit. |
| [`spacecraft-accessibility`](spacecraft-accessibility/) | Implements and audits Standard §18 accessibility across CLI, TUI, and GUI — the `--accessible` / `SPACECRAFT_A11Y` activation contract, always-on status tags, TUI linear (non-redraw) mode, screen-reader bridges (AccessKit, `GtkAccessible`, Flutter `Semantics`, `QAccessible`), the `steelbore-high-contrast` / `steelbore-mono` variants, and a run-it-don't-read-it audit checklist (WCAG 2.2 AA, EN 301 549 clause 11). |
| [`spacecraft-ada-guidelines`](spacecraft-ada-guidelines/) | Provably-correct safety-critical Ada/SPARK guidance — the `gnatprove` assurance ladder (Stone→Platinum), design-by-contract (`Pre`/`Post`/`Global`/`Depends`), strong typing, SPARK ownership, Ravenscar/Jorvik tasking, the AdaCore Safe & Secure 41-rule catalogue (adapted, in `references/`), and the Alire/GNAT toolchain. |
| [`spacecraft-agentic-cli`](spacecraft-agentic-cli/) | Agent-facing UX layer for Spacecraft Software CLIs — pairs with `spacecraft-cli-standard`. Presence-based agent-env detection (harnesses set descriptive strings, not `=1`), run-to-verify auditing, and no-clobber scaffolding on the user's own machine. |
| [`spacecraft-brand-guidelines`](spacecraft-brand-guidelines/) | Applies Spacecraft Software's official colors and typography to artifacts. |
| [`spacecraft-carbon-guidelines`](spacecraft-carbon-guidelines/) | Type-safe highly-interoperable Carbon guidance — introducers (`let`/`var`), null safety (`Optional(T)`), bidirectional C++ interoperability (`import Cpp`), safety build profiles (debug/hardened), and C++ thread/atomic integrations. |
| [`spacecraft-chez-guidelines`](spacecraft-chez-guidelines/) | Functional, safe, concurrent Chez Scheme guidance — R6RS libraries + Akku, pure-first design, `optimize-level` as the safety lever (level 3 = `unsafe`), hand-built mailboxes/channels over real threads (no Fibers), the FFI + AOT/whole-program compilation, and Guile-habit guardrails. |
| [`spacecraft-clang-guidelines`](spacecraft-clang-guidelines/) | Memory-safe highly-hardened C guidance — NASA Power of 10 Rules (no runtime heap allocation, bounded loops, small functions, high assertion density), MISRA C safety subsets, CERT C secure coding rules, Clang `-fbounds-safety` compiler extensions, and C11 atomics. |
| [`spacecraft-cli-preference`](spacecraft-cli-preference/) | Modern CLI substitutions: `eza` for `ls`, `rg` for `grep`, `gitway` for Git SSH, etc. Conditional on the local host — substitute only when the tool is installed, fall back to the legacy tool with a note when it isn't, never launch a TUI in the agent's TTY-less shell, and get consent before anything that mutates or deletes. |
| [`spacecraft-cli-standard`](spacecraft-cli-standard/) | Enforces the Spacecraft Software Dual-Mode Self-Documenting CLI Standard (v1.0.0) on every CLI. Presence-based agent-env detection (harnesses set descriptive strings, not `=1`), run-to-verify compliance, and no-clobber scaffolding on the user's own machine. |
| [`spacecraft-cli-shell`](spacecraft-cli-shell/) | Syntax-compliance guard for Nushell / Ion / POSIX / Bash commands. Measures the host instead of guessing it, and targets whichever shell will actually execute the command — agent-run, handed to the user, or written to a file. |
| [`spacecraft-clojure-guidelines`](spacecraft-clojure-guidelines/) | Functional, safe-concurrent Clojure guidance — immutable-first design, reference-type decision tree (atoms / refs+STM / agents / core.async), transducers, lazy-seq discipline, ClojureScript and Babashka platform notes, and `standard-clj` formatting. |
| [`spacecraft-commonlisp-guidelines`](spacecraft-commonlisp-guidelines/) | Type-safe highly-concurrent Common Lisp guidance (targeting SBCL) — Bordeaux-Threads and `lparallel` pools, compare-and-swap (CAS) atomics, dynamic scope thread-local let-bindings, compile-time type declarations, safe FFI memory hygiene via `cffi:with-foreign-object`, and SBCL compiler optimization flags. |
| [`spacecraft-cpp-guidelines`](spacecraft-cpp-guidelines/) | Memory-safe highly-hardened C++ guidance — Safe C++ compile-time borrow checking (`safe` context blocks and `std2` library), compiler hardening flags for bounds-trapping assertions in Clang & GCC (`-fhardened`, `_LIBCPP_HARDENING_MODE_EXTENSIVE`), Fil-C runtime safety compilation, and C++20 `std::jthread` concurrent loops. |
| [`spacecraft-dartflutter-guidelines`](spacecraft-dartflutter-guidelines/) | Type-safe highly-concurrent Dart & Flutter guidance — Sound Null Safety (avoiding `!`), multithreaded isolate task loops (`Isolate.run`), event-loop concurrency, Flutter rendering checks (`const` caching and `RepaintBoundary`), widget testing, and explicit controller disposal. |
| [`spacecraft-document-format`](spacecraft-document-format/) | Texinfo-first document authoring: `.texi` is canonical for prose (one source → Info/text, HTML, PDF, and GFM; Standard §8), ODF (`.odt`/`.ods`/`.odp`) is secondary for prose and primary for spreadsheets/presentations, MS Office (`.docx`/`.xlsx`/`.pptx`) is the last-resort fallback; GFM Markdown companion paired with every binary deliverable; PDF always an export; CC-BY-SA-4.0 document license; Void Navy + Standard §11 palette. |
| [`spacecraft-elixir-guidelines`](spacecraft-elixir-guidelines/) | Fault-tolerant concurrent Elixir/OTP guidance — supervision trees, `GenServer`/`Task.async_stream`, "let it crash" resilience, share-nothing message passing, pattern matching & `with` flow, ExUnit/StreamData testing, and `mix format`/Credo/Dialyzer gates. |
| [`spacecraft-erlang-guidelines`](spacecraft-erlang-guidelines/) | Fault-tolerant concurrent Erlang/OTP guidance — `gen_server`/`gen_statem`/`supervisor` behaviours, restart strategies, links & monitors, "let it crash", ETS/Mnesia state, `-spec` + Dialyzer, and the rebar3 (eunit/Common Test/xref/dialyzer) toolchain. |
| [`spacecraft-gleam-guidelines`](spacecraft-gleam-guidelines/) | Type-safe fault-tolerant Gleam-on-the-BEAM guidance — gleam_otp 1.x typed actors and static/factory supervision, `Subject` message passing, `Result`/`use` error flow, exhaustive `case`, `@external` FFI discipline, gleeunit/qcheck/birdie testing, and the `gleam format --check` / `build --warnings-as-errors` gates. |
| [`spacecraft-golang-guidelines`](spacecraft-golang-guidelines/) | High-performance concurrent Go guidance — goroutines, channels, errgroup, context cancellation, atomics, `sync.Pool`, pprof / race-detector workflow, and memory-safe parallelism patterns. |
| [`spacecraft-java-guidelines`](spacecraft-java-guidelines/) | Type-safe concurrent Java guidance — Virtual Threads (Project Loom), structured concurrency (`StructuredTaskScope`), avoiding thread pinning, garbage collection tuning (Generational ZGC), try-with-resources resource safety, and immutable records. |
| [`spacecraft-kotlin-guidelines`](spacecraft-kotlin-guidelines/) | Type-safe highly-concurrent Kotlin guidance — structured concurrency, `supervisorScope` failure isolation, injected dispatchers (`IO`/`Default`/`Main`), null safety (avoiding `!!`), Exposed ORM transaction boundaries, and functional `Either` error mappers. |
| [`spacecraft-lua-guidelines`](spacecraft-lua-guidelines/) | Expert guidelines for writing memory-safe, high-performance, and concurrent Lua code (targeting Lua 5.1/LuaJIT and Lua 5.4+) — local variables, table performance, metatable-based OOP, coroutine event loops, and LuaCATS. |
| [`spacecraft-markdown-document`](spacecraft-markdown-document/) | Produces well-formed GFM documents conforming to the GitHub Flavored Markdown spec and Spacecraft Software house style. Slash-command only: `/spacecraft-markdown-document`. |
| [`spacecraft-missing-pkg`](spacecraft-missing-pkg/) | Provides missing software on the user's own machine — ephemeral first (`guix shell`, `nix run`, `npx`/`uvx`), a committed project env when a repo needs it repeatedly, a proposed declarative config edit when it should persist, and consent-gated imperative installs only as a last resort. Never `sudo`, never system-distro managers. |
| [`spacecraft-nickel-guidelines`](spacecraft-nickel-guidelines/) | Expert guidelines for writing high-quality, correct, maintainable, and type-safe Nickel configuration code — contracts, merging, priorities, and import options. |
| [`spacecraft-nim-guidelines`](spacecraft-nim-guidelines/) | Type-safe highly-concurrent Nim guidance (targeting Nim 2.0+) — ARC/ORC deterministic memory management, move semantics (`sink`/`lent`), structured parallelism via `Malebolgia`, asynchronous networking with `Chronos`, safe FFI custom destructors (`=destroy`), and warnings-as-errors compiler configuration. |
| [`spacecraft-nix-guidelines`](spacecraft-nix-guidelines/) | Expert guidelines for writing high-performance, clean, reproducible, and type-safe Nix language expressions — formatting standard (RFC 166), recursive attributes avoidance, module options design, and flake inputs consolidation. |
| [`spacecraft-nu-guidelines`](spacecraft-nu-guidelines/) | Expert guidelines for writing high-performance, clean, and type-safe Nushell (Nu) script code — structured data pipelines, variables and mutability, custom commands, environment management, and error handling. |
| [`spacecraft-ocamel-guidelines`](spacecraft-ocamel-guidelines/) | Type-safe highly-concurrent OCaml guidance — direct-style I/O fibers via `Eio` and Domainslib task parallelism pools, Saturn lock-free structures, Software Transactional Memory with Kcas, C FFI memory safety (`CAMLparam`/`CAMLlocal`/`CAMLreturn`), and warning-as-errors compilation flags. |
| [`spacecraft-python-guidelines`](spacecraft-python-guidelines/) | Type-safe highly-concurrent Python guidance (targeting Python 3.12+) — strict static typing (`mypy`), boundary validation (`Pydantic v2`), non-blocking asynchronous event loops (`asyncio`), multiprocess CPU scaling (`ProcessPoolExecutor`), memory-optimized slots classes, and Ruff linting rules. |
| [`spacecraft-rust-guidelines`](spacecraft-rust-guidelines/) | High-performance concurrent Rust guidance — concurrency model selection, lock-free synchronisation, memory layout, tooling gates, and unsafe hygiene — plus a distilled idiom layer (`references/idioms.md`, adapted from Apollo's Rust Best Practices, MIT) covering borrowing, clippy discipline, testing, dispatch, and type-state. |
| [`spacecraft-standard-constitution`](spacecraft-standard-constitution/) | Authoritative compliance reference (The Steelbore Standard). |
| [`spacecraft-swift-guidelines`](spacecraft-swift-guidelines/) | Type-safe highly-concurrent Swift guidance (targeting Swift 6.2+) — Swift 6.2 concurrency, explicit `@concurrent` background offloading, isolated conformances, `@MainActor` isolated ViewModels, Swift Testing `@Suite` and `@Test` parameterized checks, and ARC reference cycle safety. |
| [`spacecraft-texinfo-document`](spacecraft-texinfo-document/) | How-to layer for authoring, building, linting, and converting GNU Texinfo — the canonical Spacecraft prose format (one `.texi` → Info/HTML/PDF/DocBook/text/EPUB); house-style header/licensing, node/menu discipline, `@def*` API docs, the `texi2any`/`texi2pdf` toolchain, and HTML/PDF brand theming. |
| [`spacecraft-theme-factory`](spacecraft-theme-factory/) | Generates Spacecraft Software-compliant themes for IDEs and terminals. |
| [`spacecraft-typescript-guidelines`](spacecraft-typescript-guidelines/) | Type-safe highly-concurrent TypeScript guidance (targeting TypeScript 7.0+) — Go native compiler optimizations, Project References (`composite`/`incremental`), strict type checking, non-blocking asynchronous event loops, CPU-parallel worker pools (`Piscina`), V8 engine tuning (hidden classes), and Zod data validation boundaries. |
| [`spacecraft-zig-guidelines`](spacecraft-zig-guidelines/) | Memory-safe high-performance concurrent Zig guidance — `std.Thread.Pool` / `std.Io.Threaded`, atomics, allocator discipline, comptime safety, and CPU-bound scaling patterns. |

<!-- §3 — Layout convention -->
## Directory layout

Every skill follows the same shape:

```
<skill-name>/
├── SKILL.md           # Frontmatter + the agent-facing instructions
├── LICENSE.md         # Skill license (Standard §4.1.1: skills are GPL-3.0-or-later; third-party-derived skills keep their upstream license)
├── CREDITS.md         # Required when the skill builds on third-party work (Standard §15.3)
└── references/        # Optional; consulted only when depth is needed
    ├── <topic>.md
    └── ATTRIBUTION.md # Optional deeper credit file for adapted references
```

Skills are also distributed as `<skill-name>.skill` bundles (zipped) at the
repository root for drop-in installation.

<!-- §4 — Installation -->
## Installation

### Direct clone

Clone into any of the supported agent skill directories:

```sh
# Claude Code
git clone git@github.com:Spacecraft-Software/Construct.git ~/.claude/skills

# Gemini CLI
git clone git@github.com:Spacecraft-Software/Construct.git ~/.gemini/skills

# Codex
git clone git@github.com:Spacecraft-Software/Construct.git ~/.codex/skills

# Grok
git clone git@github.com:Spacecraft-Software/Construct.git ~/.grok/skills
```

The SSH remote is configured to work with
[Gitway](https://github.com/Spacecraft-Software/Gitway), Spacecraft Software's pinned-host-key
SSH transport for Git.

### Nix flake (Home Manager)

The repository is a Nix flake. Add it as an input and import the Home Manager
module:

```nix
{
  inputs.construct.url = "github:Spacecraft-Software/Construct";

  outputs = { self, nixpkgs, home-manager, construct, ... }: {
    homeConfigurations."you" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        construct.homeManagerModules.default
        {
          spacecraft.construct.enable = true;       # cross-platform skills
          spacecraft.construct.enableGrok = true;   # Grok skills
        }
      ];
    };
  };
}
```

With `enable`, all cross-platform skills are installed to `~/.agents/skills/`
and every known agent harness's skill path
(`~/.claude/skills`, `~/.gemini/skills`, `~/.codex/skills`, `~/.ai/skills`,
`~/.agent/skills`) becomes a directory symlink to that canonical location.
Add more paths via `spacecraft.construct.agentPaths`.

With `enableGrok`, Grok-specific skills (from [`grok-skills/`](grok-skills/))
install to `~/.grok/skills/` — kept separate because Grok's bundle format is
flat (no enclosing skill-name directory) and is not interchangeable with the
Claude/Gemini/Codex layout.

To pick up the latest commit, run `nix flake update construct` in the consumer
flake and rebuild.

Individual skills are also exposed as packages — e.g.
`nix build github:Spacecraft-Software/Construct#spacecraft-standard-constitution` produces
a `result/` directory with that skill's contents.

### Grok skills

Skills authored for the Grok agent platform live in
[`grok-skills/`](grok-skills/) with their own catalogue. The bundle layout
differs (flat zip root); see the section's README for details.

### Android skills

Google's official **[Android skills](https://github.com/android/skills)** are
vendored verbatim (Apache-2.0, © Google LLC) under
[`android-skills/`](android-skills/) with their own catalogue. They use the same
open-standard `SKILL.md` format, so `spacecraft.construct.enableAndroid = true`
merges them into the canonical `~/.agents/skills/` tree alongside the
cross-platform skills. They are not first-party skills, so they are kept out of
the §2 catalogue above and ship no `.zip`/`.skill` bundles (Google's `android`
CLI installs them from upstream). See the section's README for provenance and
the upstream→vendored path mapping.

<!-- §5 — Contributing / standards -->
## Standards

All skills in this repository are expected to conform to
[The Steelbore Standard](spacecraft-standard-constitution/) — including:

- **§4** SPDX/REUSE-compliant licensing (§4.3): two-tag headers / `REUSE.toml`, a
  `LICENSES/` directory, `reuse lint`-clean. Skills are GPL-3.0-or-later (§4.1.1);
  third-party-derived skills preserve their upstream license (§4.2).
- **§14** ISO 8601 dates throughout.
- Functional naming (no codenames for skill IDs).

<!-- §6 — License -->
## License

This repository follows the [REUSE specification](https://reuse.software) — see
`REUSE.toml` and the `LICENSES/` directory; run `reuse lint` to verify. Per Standard
§4.1.1, skills are **GPL-3.0-or-later** by default; third-party-derived skills keep
their upstream license (e.g. `microsoft-rust-guidelines` is MIT, `gnu-coding-standards`
is GFDL-1.3-or-later). The published Standard *document* is CC-BY-SA-4.0, but the
`spacecraft-standard-constitution` *skill* here is GPL-3.0-or-later.

---

*─── Forged in Spacecraft Software ───*
