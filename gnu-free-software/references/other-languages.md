# GNU Standards in Rust, Go, and Python (brief)

The GNU Coding Standards predate the prominence of these languages and **do not
mention Rust or Go**; Python is named in §3.1 only as acceptable when peak
efficiency is not required. This file applies GNU's **principles** to all three and
is honest about the gap: **GNU has not officially blessed Rust or Go**, and for a
GNU *extension language* specifically, **Guile remains the endorsed choice** (see
`SKILL.md` and `guile.md`). C is the canonical case (`c.md`); these three are a
thin conformance layer over each language's own established style.

One rule spans all three: GNU §5.1 treats formatting as a *recommendation* and
§4.1 treats outside standards as suggestions, so **deferring to the language's
canonical formatter is GNU-consistent, not a violation**. Use the formatter; don't
impose GNU's C-era brace/indent rules on a language that has its own.

The language-agnostic contract — error grammar, the `--version`/`--help` output,
long options, i18n, Texinfo docs, ChangeLog/NEWS, the release process — is in
`conventions.md` and applies unchanged. Below are only the per-language specifics.

---

## Shared robustness mapping

| GNU goal | Rust | Go | Python |
|---|---|---|---|
| No arbitrary limits | `Vec`/`String`/`HashMap`, never fixed buffers | slices/maps, never fixed arrays | lists/dicts/generators; stream large inputs |
| Check every error | return `Result<T,E>`, propagate `?`; no `unwrap`/`expect` in library code | `if err != nil {…}` after every call; never discard with `_`; wrap with `%w` | exceptions; catch what you handle, propagate the rest; never bare `except:` |
| Allocation failure | aborts by default (≈ fatal); use `try_reserve` to recover; catch at an interactive loop | panics (≈ fatal); `recover` at an interactive loop | `MemoryError`; fatal noninteractively, catch at a command loop |
| Impossible condition → abort | `panic!`/`unreachable!`/`debug_assert!` + comment | `panic` + comment (reserve for invariants, not ordinary errors) | `assert`/`raise RuntimeError` + comment |
| Full byte/char range | `String`/`str` are UTF-8; `OsStr`/`Vec<u8>` for non-UTF-8 paths/data | UTF-8 source/strings; `unicode/utf8` and `[]byte` for non-UTF-8 | `str` is Unicode; `encoding="utf-8"`; `bytes` for binary |

For all three: write error messages in the GNU grammar (`conventions.md` §1) to
**stderr** in `program: message` form, including the OS error text, where the
program name is a **constant**, never derived from the invocation (`argv[0]` /
`os.Args[0]` / `sys.argv[0]`). Don't use an error *count* as the exit status; exit
with a small fixed code.

```rust
eprintln!("{}: cannot open {}: {}", PROG, path.display(), err);   // PROG = env!("CARGO_PKG_NAME") or a const
```
```go
fmt.Fprintf(os.Stderr, "%s: cannot open %s: %v\n", prog, path, err)  // prog is a package constant
```
```python
print(f"{PROG}: cannot open {path}: {err.strerror}", file=sys.stderr)  # PROG is a module constant
```

---

## Per-language specifics

### Rust

- **Formatter / naming:** `rustfmt`. Rust's `snake_case` + `SCREAMING_SNAKE_CASE`
  already match GNU's naming intent; keep names English and descriptive.
- **CLI:** `clap` (derive API) for POSIX short options plus GNU `--long` options,
  intermixed with arguments. clap's default `--version`/`--help` is **not**
  GNU-exact — customize it to the contract in `conventions.md` §§2–3 (first line
  `GNU pkg X.Y`, then the copyright/license/no-warranty block to stdout, success
  exit; bug-report/home-page/help footer on `--help`).
- **i18n:** `gettext-rs` (bindings to GNU gettext), sharing the `.po`/`.mo`
  toolchain and `localedir`.
- **Conditional compilation:** prefer runtime `if cfg!(feature = "foo")` over
  `#[cfg(...)]` when both paths should be type-checked (the analogue of GNU's
  "prefer `if (HAS_FOO)` over `#ifdef`").

### Go

- **Formatter / naming:** `gofmt` (mandatory). Go's exported `MixedCaps` /
  `camelCase` has no underscores — **follow Go's convention**; the §5.4 carry-over
  is only the spirit (English, descriptive, not terse).
- **CLI:** the stdlib `flag` package is **not POSIX-compatible** (single-dash long
  flags, no `--`). Use a getopt_long-style library (e.g. `spf13/pflag`) for POSIX
  short options plus GNU `--long` options. Set a custom usage function to meet the
  `--version`/`--help` contract.
- **i18n:** `golang.org/x/text` (the `message` package / `gotext` tooling); bind
  the catalog to the package name and use its plural support.

### Python

- **Formatter / naming:** `black` + PEP 8. Python's `snake_case` + `UPPER_CASE`
  already match GNU §5.4's intent.
- **CLI:** `argparse` maps directly onto `getopt_long` (POSIX short + GNU `--long`,
  intermixed, automatic `--help`). Add `add_argument("--version",
  action="version", ...)` but set its text to the GNU contract; put the
  bug-report/home-page/help footer in the parser's `epilog`.
- **i18n:** the standard-library `gettext` module — the closest one-to-one match
  to GNU's model of any of the three. `gettext.textdomain("pkg")`,
  `_ = gettext.gettext`, wrap strings as `_("...")`, `ngettext` for plurals.

---

## License headers

Every source file carries a `GPL-3.0-or-later` header; an SPDX line may accompany
it (never replace it). Comment syntax differs, text does not:

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
// <one-line description of the file's purpose>
//
// Copyright (C) <year> <holder>.
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
```

Use `//` for Rust/Go and `#` for Python; the wording is identical. `COPYING` holds
the full GPL. For an author-held (non-FSF) project, put the author's name in the
`Copyright` line (see `SKILL.md`, "Copyright and assignment").

For **networked software**, license `AGPL-3.0-or-later` instead: use
`SPDX-License-Identifier: AGPL-3.0-or-later` and replace "GNU General Public License"
with "GNU Affero General Public License" in the header text — wording otherwise
identical, comment syntax unchanged (`COPYING` then holds the AGPL). See `SKILL.md`
for when the AGPL applies.

---

## Build: a GNU layer over the native tool

A GNU package is defined partly by its release interface, so wrap the native
toolchain in a GNU-conformant Makefile implementing the standard targets and
directory variables (`conventions.md` §11) rather than expecting users to know
Cargo / `go build` / `pip`. The manifest (`Cargo.toml`, `go.mod`, `pyproject.toml`)
declares `license = "GPL-3.0-or-later"` and stays the build source of truth; the
Makefile is the GNU-facing entry point.

```makefile
prefix      = /usr/local
exec_prefix = $(prefix)
bindir      = $(exec_prefix)/bin

# Rust:   BUILD = cargo build --release ;  CHECK = cargo test ;  artifact = target/release/pkg
# Go:     BUILD = go build -o pkg ./... ;  CHECK = go test ./... ;  artifact = pkg
# Python: BUILD = python3 -m build      ;  CHECK = python3 -m pytest

all:
	$(BUILD)

check:
	$(CHECK)

install: all installdirs
	$(NORMAL_INSTALL)
	$(INSTALL_PROGRAM) <artifact> $(DESTDIR)$(bindir)/pkg

installdirs:
	mkdir -p $(DESTDIR)$(bindir)

clean:
	$(CLEAN)

.PHONY: all check install installdirs clean
```

Documentation is a **Texinfo** manual in every case; `rustdoc` / `go doc` /
docstrings are supplementary API reference, never the primary manual.
