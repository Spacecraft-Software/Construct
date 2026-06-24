# GNU Standards in GNU Guile (Scheme)

Guile is special: GNU §3.1 names it **the preferred extension language** and "the path that
will lead to overall consistency of the GNU system." When the task is to make a C/C++ program
extensible, or to write a GNU application in a high-level language, Guile is the sanctioned
choice — prefer it over embedding Python or Perl. Several GNU facilities (gettext, getopt-long)
exist *natively* in Guile, so the mapping is the cleanest of the five languages.

This file covers only the **GNU-package layer**. For idiomatic functional and concurrent
Scheme craft (fibers, channels, SRFIs, hygienic macros), follow standard Guile practice —
the Guile Reference Manual, the relevant SRFIs, and R6RS/R7RS conventions.

## Formatting and naming

- Follow standard Scheme indentation (the style Emacs `scheme-mode` produces). GNU's
  column-1-brace C rules are C-specific and do not apply; §5.1 sanctions following the
  language's own style.
- Scheme names are **hyphenated-lower-case** (`ignore-space-change?`, `make-foo`), not
  underscored — this is the Scheme convention and overrides §5.4's underscore rule, which is
  about C. The *spirit* of §5.4 carries over: English, descriptive, not terse; predicates end
  in `?`, mutators in `!`.
- Organize code into `define-module` modules with a package-named namespace, e.g.
  `(define-module (gnu pkg core))`. This is Guile's analogue of GNU's library name-prefix rule.

## Robustness, realized in Guile

- **No arbitrary limits**: lists, vectors, and hash tables grow dynamically — never impose a
  fixed bound on input.
- **Check every error**: handle failures with `guard` or `with-exception-handler` rather than
  letting them escape silently; return meaningful values or raise typed conditions. This is
  the Scheme form of GNU's "check every system call."
- **Error messages** follow the GNU grammar (`conventions.md` §1), written to
  `(current-error-port)` in `program: message` form, including the underlying error text:

  ```scheme
  (format (current-error-port) "~a: cannot open ~a: ~a~%"
          %program-name path (strerror errno))
  ```

  `%program-name` is a constant the package defines, not taken from the invocation.
- **Impossible conditions**: signal an error via `error` or `assert` with an explanatory
  message — the Guile form of GNU's `abort ()`-with-a-comment.
- **UTF-8**: set ports to UTF-8 and use string ports for text processing; Guile strings are
  Unicode, so no characters are silently dropped.

## Command-line interface

- Use the native **`(ice-9 getopt-long)`** module for POSIX short options plus GNU long
  options — this is Guile's built-in `getopt_long` equivalent.
- Implement `--version` and `--help` to the contract in `conventions.md` §§2–3: print to the
  current output port and exit successfully. The first `--version` line is `GNU pkg X.Y`
  followed by the copyright/license/no-warranty block; the `--help` footer carries the
  bug-report/home-page/help lines.
- Ordinary arguments are input files; specify output with `-o`/`--output`.

## Internationalization

- Use Guile's native gettext support: `(use-modules (ice-9 i18n))`, bind the text domain to
  the package name with `textdomain`, and wrap translatable strings — the `(gettext "...")`
  and `(ngettext singular plural n)` procedures map directly onto `conventions.md` §8.
  Translate complete sentences.

## License and headers

Every `.scm` file carries a GPL-3.0-or-later header:

```scheme
;;; pkg --- <one-line description>      -*- mode: scheme; coding: utf-8 -*-
;;;
;;; SPDX-License-Identifier: GPL-3.0-or-later
;;; Copyright (C) <year> <holder>.
;;;
;;; This program is free software: you can redistribute it and/or modify it under
;;; the terms of the GNU General Public License as published by the Free Software
;;; Foundation, either version 3 of the License, or (at your option) any later version.
```

For **networked software** (`AGPL-3.0-or-later`; see `SKILL.md`), use the same header
with the AGPL identifier and name: `SPDX-License-Identifier: AGPL-3.0-or-later`, and
"the terms of the GNU Affero General Public License" in place of "GNU General Public
License". `COPYING` holds the full license text (GPL or AGPL).

## Build

Guile packages conventionally use Autotools (`configure.ac` + `Makefile.am`), which already
gives you GNU's directory variables and standard targets for free, plus compilation of `.scm`
to `.go` bytecode via `guild compile`. If you hand-write the Makefile, implement the standard
targets and directory variables from `conventions.md` §11, installing modules under
`$(datarootdir)/guile/site/<effective-version>/`. Documentation is a Texinfo manual.
