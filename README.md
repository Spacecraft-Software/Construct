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
| [`spacecraft-guile-guidelines`](spacecraft-guile-guidelines/) | Write idiomatic, functional, concurrent GNU Guile (Guile Scheme 3.x) — fibers/CSP, SRFI-1, tail calls, hygienic macros. |
| [`microsoft-rust-guidelines`](microsoft-rust-guidelines/) | Enforces Microsoft Pragmatic Rust Guidelines before any `.rs` edit. |
| [`spacecraft-agentic-cli`](spacecraft-agentic-cli/) | Agent-facing UX layer for Spacecraft Software CLIs — pairs with `spacecraft-cli-standard`. |
| [`spacecraft-brand-guidelines`](spacecraft-brand-guidelines/) | Applies Spacecraft Software's official colors and typography to artifacts. |
| [`spacecraft-chez-guidelines`](spacecraft-chez-guidelines/) | Functional, safe, concurrent Chez Scheme guidance — R6RS libraries + Akku, pure-first design, `optimize-level` as the safety lever (level 3 = `unsafe`), hand-built mailboxes/channels over real threads (no Fibers), the FFI + AOT/whole-program compilation, and Guile-habit guardrails. |
| [`spacecraft-cli-preference`](spacecraft-cli-preference/) | Modern CLI substitutions: `eza` for `ls`, `rg` for `grep`, `gitway` for Git SSH, etc. |
| [`spacecraft-cli-standard`](spacecraft-cli-standard/) | Enforces the Spacecraft Software Dual-Mode Self-Documenting CLI Standard (SFRS v1.0.0) on every CLI. |
| [`spacecraft-cli-shell`](spacecraft-cli-shell/) | Syntax-compliance guard for Nushell / Ion / POSIX / Bash commands. |
| [`spacecraft-clojure-guidelines`](spacecraft-clojure-guidelines/) | Functional, safe-concurrent Clojure guidance — immutable-first design, reference-type decision tree (atoms / refs+STM / agents / core.async), transducers, lazy-seq discipline, ClojureScript and Babashka platform notes, and `standard-clj` formatting. |
| [`spacecraft-document-format`](spacecraft-document-format/) | Texinfo-first document authoring: `.texi` is canonical for prose (one source → Info/text, HTML, PDF, and GFM; Standard §8), ODF (`.odt`/`.ods`/`.odp`) is secondary for prose and primary for spreadsheets/presentations, MS Office (`.docx`/`.xlsx`/`.pptx`) is the last-resort fallback; GFM Markdown companion paired with every binary deliverable; PDF always an export; CC-BY-SA-4.0 document license; Void Navy + Standard §11 palette. |
| [`spacecraft-elixir-guidelines`](spacecraft-elixir-guidelines/) | Fault-tolerant concurrent Elixir/OTP guidance — supervision trees, `GenServer`/`Task.async_stream`, "let it crash" resilience, share-nothing message passing, pattern matching & `with` flow, ExUnit/StreamData testing, and `mix format`/Credo/Dialyzer gates. |
| [`spacecraft-erlang-guidelines`](spacecraft-erlang-guidelines/) | Fault-tolerant concurrent Erlang/OTP guidance — `gen_server`/`gen_statem`/`supervisor` behaviours, restart strategies, links & monitors, "let it crash", ETS/Mnesia state, `-spec` + Dialyzer, and the rebar3 (eunit/Common Test/xref/dialyzer) toolchain. |
| [`spacecraft-golang-guidelines`](spacecraft-golang-guidelines/) | High-performance concurrent Go guidance — goroutines, channels, errgroup, context cancellation, atomics, `sync.Pool`, pprof / race-detector workflow, and memory-safe parallelism patterns. |
| [`spacecraft-markdown-document`](spacecraft-markdown-document/) | Produces well-formed GFM documents conforming to the GitHub Flavored Markdown spec and Spacecraft Software house style. Slash-command only: `/spacecraft-markdown-document`. |
| [`spacecraft-missing-pkg`](spacecraft-missing-pkg/) | Handles missing-package situations in the Spacecraft Software workflow. |
| [`spacecraft-rust-guidelines`](spacecraft-rust-guidelines/) | High-performance concurrent Rust guidance — concurrency model selection, lock-free synchronisation, memory layout, tooling gates, and unsafe hygiene — plus a distilled idiom layer (`references/idioms.md`, adapted from Apollo's Rust Best Practices, MIT) covering borrowing, clippy discipline, testing, dispatch, and type-state. |
| [`spacecraft-standard`](spacecraft-standard/) | Authoritative compliance reference (The Spacecraft Software Standard v1.2). |
| [`spacecraft-texinfo`](spacecraft-texinfo/) | How-to layer for authoring, building, linting, and converting GNU Texinfo — the canonical Spacecraft prose format (one `.texi` → Info/HTML/PDF/DocBook/text/EPUB); house-style header/licensing, node/menu discipline, `@def*` API docs, the `texi2any`/`texi2pdf` toolchain, and HTML/PDF brand theming. |
| [`spacecraft-theme-factory`](spacecraft-theme-factory/) | Generates Spacecraft Software-compliant themes for IDEs and terminals. |
| [`spacecraft-zig-guidelines`](spacecraft-zig-guidelines/) | Memory-safe high-performance concurrent Zig guidance — `std.Thread.Pool` / `std.Io.Threaded`, atomics, allocator discipline, comptime safety, and CPU-bound scaling patterns. |

<!-- §3 — Layout convention -->
## Directory layout

Every skill follows the same shape:

```
<skill-name>/
├── SKILL.md           # Frontmatter + the agent-facing instructions
├── LICENSE.md         # Skill license (Standard §4.1.1: skills are GPL-3.0-or-later; third-party-derived skills keep their upstream license)
├── CREDITS.md         # Required when the skill builds on third-party work (Standard §13.3)
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
`nix build github:Spacecraft-Software/Construct#spacecraft-standard` produces
a `result/` directory with that skill's contents.

### Grok skills

Skills authored for the Grok agent platform live in
[`grok-skills/`](grok-skills/) with their own catalogue. The bundle layout
differs (flat zip root); see the section's README for details.

<!-- §5 — Contributing / standards -->
## Standards

All skills in this repository are expected to conform to
[The Spacecraft Software Standard](spacecraft-standard/) — including:

- **§4** SPDX/REUSE-compliant licensing (§4.3): two-tag headers / `REUSE.toml`, a
  `LICENSES/` directory, `reuse lint`-clean. Skills are GPL-3.0-or-later (§4.1.1);
  third-party-derived skills preserve their upstream license (§4.2).
- **§11** ISO 8601 dates throughout.
- Functional naming (no codenames for skill IDs).

<!-- §6 — License -->
## License

This repository follows the [REUSE specification](https://reuse.software) — see
`REUSE.toml` and the `LICENSES/` directory; run `reuse lint` to verify. Per Standard
§4.1.1, skills are **GPL-3.0-or-later** by default; third-party-derived skills keep
their upstream license (e.g. `microsoft-rust-guidelines` is MIT, `gnu-coding-standards`
is GFDL-1.3-or-later). The published Standard *document* is CC-BY-SA-4.0, but the
`spacecraft-standard` *skill* here is GPL-3.0-or-later.

---

*─── Forged in Spacecraft Software ───*
