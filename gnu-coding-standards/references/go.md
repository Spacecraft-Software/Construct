# GNU Standards in Go

The GNU Coding Standards do not mention Go. This file applies GNU's **principles** to it and
is honest about the gaps: GNU has not officially blessed Go, and for a GNU *extension language*
Guile remains the endorsed choice (see `SKILL.md`). Go's strongest alignment with GNU is its
**explicit error-return model**, which maps almost one-to-one onto GNU's "check every error
return."

## Formatting and naming

- Format with **gofmt** — it is mandatory and non-negotiable in Go, and it overrides GNU's
  C-era formatting rules. §5.1 treats GNU formatting as a recommendation and §4.1 treats
  outside standards as suggestions, so deferring to gofmt is GNU-consistent, not a violation.
- Go's exported-identifier convention is `MixedCaps`/`camelCase` with **no underscores** — this
  conflicts with §5.4's underscore rule, which is C-specific. **Follow Go's convention**; the
  carry-over from §5.4 is the spirit: English, descriptive, not terse. (Don't fight the
  language to satisfy a C rule.)
- GNU's "prefer `if (HAS_FOO)` over `#ifdef`" maps to preferring a runtime `if` over Go build
  tags (`//go:build`) when the code can compile both ways and you want both paths checked.

## Robustness, realized in Go

- **No arbitrary limits**: use slices and maps, never fixed-size arrays that truncate input.
- **Check every error return**: the idiomatic `if err != nil { ... }` after every fallible
  call *is* GNU's rule, enforced by the language. Never discard an error with `_` unless you
  genuinely mean to ignore it. Wrap with `%w` to preserve the underlying OS error text.
- **Error messages** follow the GNU grammar (`conventions.md` §1), written to stderr in
  `program: message` form, including the OS error:

  ```go
  fmt.Fprintf(os.Stderr, "%s: cannot open %s: %v\n", prog, path, err)
  ```

  `prog` is a constant the package defines, not `os.Args[0]`.
- **Allocation failure** is rare in Go and panics; that satisfies GNU's "fatal in a
  noninteractive program." In an interactive command loop, `recover` at the loop boundary to
  return to the prompt.
- **Impossible conditions → abort**: `panic` with an explanatory message — the Go form of
  GNU's `abort ()`-with-a-comment. Reserve it for genuine invariants, not ordinary errors
  (which return `error`).
- **UTF-8 / NUL**: Go source and strings are UTF-8 by convention; use the `unicode/utf8`
  package and `[]byte` for data that may not be valid UTF-8 so nothing is silently dropped.
- Don't use an error *count* as the exit code (`conventions.md` §1); call `os.Exit` with a
  small fixed status.

## Command-line interface

- The standard-library **`flag`** package is **not POSIX-compatible** — it uses single-dash
  long flags (`-verbose`) and does not support `--`. For GNU compliance use a getopt_long-style
  library (e.g. `spf13/pflag`) so you get POSIX short options *and* GNU `--long` options,
  intermixed with arguments and stopping at `--`.
- Implement `--version` and `--help` to the contract in `conventions.md` §§2–3 by setting a
  custom usage function: `--version` prints `GNU pkg X.Y` plus the copyright/license/no-warranty
  block to stdout and exits successfully; `--help` carries the bug-report/home-page/help footer.
- Ordinary arguments are input files; specify output with `-o`/`--output`.

## Internationalization

- Use **`golang.org/x/text`** (the `message` package with a message catalog, or the `gotext`
  tooling) to integrate translation. Bind the catalog to the package name, wrap translatable
  strings, and use the plural-form support rather than appending `"s"`; translate complete
  sentences (`conventions.md` §8).

## License and headers

Every `.go` file carries a GPL-3.0-or-later header:

```go
// SPDX-License-Identifier: GPL-3.0-or-later
// <one-line description of the file's purpose>
//
// Copyright (C) <year> <holder>.
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later version.

package pkg
```

`COPYING` holds the GPL.

## Build: a GNU layer over `go build`

Wrap the Go toolchain in a GNU-conformant Makefile implementing the standard targets and
directory variables (`conventions.md` §11):

```makefile
prefix      = /usr/local
exec_prefix = $(prefix)
bindir      = $(exec_prefix)/bin
GO          = go

all:
	$(GO) build -o pkg ./...

check:
	$(GO) test ./...

install: all installdirs
	$(NORMAL_INSTALL)
	$(INSTALL_PROGRAM) pkg $(DESTDIR)$(bindir)/pkg

installdirs:
	mkdir -p $(DESTDIR)$(bindir)

clean:
	$(GO) clean
	rm -f pkg

.PHONY: all check install installdirs clean
```

Keep `go.mod` as the build manifest; the Makefile is the GNU-facing entry point. Documentation
is a Texinfo manual, with `go doc` comments as supplementary API reference.
