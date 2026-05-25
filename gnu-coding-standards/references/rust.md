# GNU Standards in Rust

The GNU Coding Standards do not mention Rust. This file applies GNU's **principles** to it
faithfully and is honest about the gaps: GNU has not officially blessed Rust, and for a GNU
*extension language* specifically, Guile remains the endorsed choice (see `SKILL.md`). What
Rust offers is that several GNU robustness goals — no arbitrary limits, no silent error
swallowing, correct UTF-8 handling — fall out of the type system almost for free.

**Defer language craft to the `rust-guidelines` skill.** This file covers only the
GNU-package layer on top of idiomatic Rust.

## Formatting and naming

- Format with **rustfmt**. GNU's column-1-brace C style does not apply — §5.1 of the Standards
  explicitly treats formatting as a recommendation and says to follow the program's own style,
  and §4.1 treats outside standards as suggestions. rustfmt *is* the canonical Rust style.
- Rust's default `snake_case` for functions/variables and `SCREAMING_SNAKE_CASE` for constants
  already match GNU's "lower case, underscores, upper case for constants" naming intent. Keep
  names English and descriptive, not terse — that part of §5.4 carries over directly.
- GNU's "prefer `if (HAS_FOO)` over `#ifdef`" maps to preferring runtime `if cfg!(feature =
  "foo")` over `#[cfg(feature = "foo")]` when the code compiles either way and you want both
  paths type-checked.

## Robustness, realized in Rust

- **No arbitrary limits**: use `Vec`/`String`/`HashMap`, never fixed-size buffers that
  truncate. This is already idiomatic.
- **Check every error return**: return `Result<T, E>` and propagate with `?`. In **library**
  code never `unwrap()`/`expect()` on fallible operations — that is the Rust form of GNU's
  "check every system call." Reserve `unwrap`/`expect` for genuinely-impossible invariants.
- **Error messages** still follow the GNU grammar (`conventions.md` §1). Write them to stderr
  in the `program: message` form and include the OS error text — `std::io::Error` already
  carries it:

  ```rust
  eprintln!("{}: cannot open {}: {}", PROG, path.display(), err);
  ```

  `PROG` is a constant (`env!("CARGO_PKG_NAME")` or a `const`), never derived from `argv[0]`.
- **Allocation failure**: Rust aborts the process on allocation failure by default, which
  matches GNU's "fatal in a noninteractive program." For an interactive command loop, catch
  fallible work at the loop boundary and return to the prompt rather than aborting; use
  `try_reserve` where you need to recover.
- **Impossible conditions → abort**: use `panic!`, `unreachable!`, or `debug_assert!` with an
  explanatory message — the Rust form of GNU's `abort ()`-with-a-comment.
- **UTF-8 / NUL**: `String`/`str` are UTF-8 by construction; use `OsStr`/`Vec<u8>` for paths
  and data that may not be UTF-8 so nothing is silently dropped.
- Don't use an error *count* as the process exit code (`conventions.md` §1).

## Command-line interface

- Use **clap** (derive API) for POSIX short options plus GNU long options, intermixed with
  arguments. clap does not produce GNU-exact `--version`/`--help` output by default, so
  customize it to match the contract in `conventions.md` §§2–3: the first `--version` line is
  `GNU pkg X.Y`, followed by the copyright/license/no-warranty block, printed to stdout with a
  success exit. Set the bug-report/home-page/help footer on `--help`.
- Ordinary arguments are input files; specify output with `-o`/`--output`.

## Internationalization

- Use **gettext-rs** (bindings to GNU gettext) so the package shares the gettext toolchain and
  `localedir`. Wrap translatable strings (`gettext("...")`); use the `ngettext` binding for
  plurals; translate complete sentences, not fragments (`conventions.md` §8).

## License and headers

Every `.rs` file carries a GPL-3.0-or-later header; an SPDX line may accompany it:

```rust
// SPDX-License-Identifier: GPL-3.0-or-later
// <one-line description of the file's purpose>
//
// Copyright (C) <year> <holder>.
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.
```

`COPYING` holds the GPL.

## Build: a GNU layer over Cargo

A GNU package is defined partly by its release interface, so wrap Cargo in a GNU-conformant
Makefile that implements the standard targets and directory variables (`conventions.md` §11)
rather than expecting users to know Cargo:

```makefile
prefix      = /usr/local
exec_prefix = $(prefix)
bindir      = $(exec_prefix)/bin
CARGO       = cargo

all:
	$(CARGO) build --release

check:
	$(CARGO) test

install: all installdirs
	$(NORMAL_INSTALL)
	$(INSTALL_PROGRAM) target/release/pkg $(DESTDIR)$(bindir)/pkg

installdirs:
	mkdir -p $(DESTDIR)$(bindir)

clean:
	$(CARGO) clean

.PHONY: all check install installdirs clean
```

Keep `Cargo.toml` (`license = "GPL-3.0-or-later"`) as the build manifest; the Makefile is the
GNU-facing entry point. Documentation is still a Texinfo manual, with rustdoc as supplementary
API reference, not the primary manual.
