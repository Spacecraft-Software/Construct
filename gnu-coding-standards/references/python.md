# GNU Standards in Python

GNU §3.1 lists Python among the languages that are acceptable when peak efficiency is not
required — but notes that for a GNU *extension language* specifically, Guile is preferred for
overall consistency of the system. Where Python aligns unusually well is i18n: the standard
library ships a **`gettext`** module, the closest one-to-one match to GNU's translation model
of any of the five languages, and **`argparse`** maps cleanly onto `getopt_long`.

## Formatting and naming

- Format with **black** (and follow **PEP 8**). This overrides GNU's C-era brace/indent rules;
  §5.1 treats GNU formatting as a recommendation and §4.1 treats outside standards as
  suggestions, so following PEP 8 is GNU-consistent.
- Python's `snake_case` for functions/variables and `UPPER_CASE` for constants already match
  GNU §5.4's intent. Keep names English, descriptive, not terse — that carries over directly.
- GNU's "prefer `if (HAS_FOO)` over `#ifdef`" maps naturally: Python has no preprocessor, so
  use ordinary runtime `if`/feature checks.

## Robustness, realized in Python

- **No arbitrary limits**: lists, dicts, and generators grow dynamically; stream large inputs
  line by line rather than imposing a size bound (`conventions.md` §6).
- **Check every error**: use exceptions — catch what you can handle, let the rest propagate;
  never bury failures behind a bare `except:` that swallows them. This is Python's form of
  GNU's "check every system call."
- **Error messages** follow the GNU grammar (`conventions.md` §1), written to stderr in
  `program: message` form, including the OS error text from the exception:

  ```python
  print(f"{PROG}: cannot open {path}: {err.strerror}", file=sys.stderr)
  ```

  `PROG` is a module constant, not derived from `sys.argv[0]`.
- **Allocation failure** surfaces as `MemoryError`; let it be fatal in a noninteractive
  program, or catch it at an interactive command loop to return to the prompt.
- **Impossible conditions → abort**: `raise AssertionError`/`assert` (or `raise RuntimeError`)
  with an explanatory message — the Python form of GNU's `abort ()`-with-a-comment.
- **UTF-8 / NUL**: `str` is Unicode; open text files with `encoding="utf-8"` and use `bytes`
  for binary data so nothing is silently dropped.
- Don't use an error *count* as the exit code (`conventions.md` §1); call `sys.exit` with a
  small fixed status.

## Command-line interface

- Use **`argparse`**, which maps directly onto `getopt_long`: POSIX short options plus GNU
  `--long` options, intermixed with arguments, with an automatic `--help`. Add
  `add_argument("--version", action="version", ...)` but set its text to the GNU contract.
- Implement `--version` to the contract in `conventions.md` §§2–3: print `GNU pkg X.Y` plus
  the copyright/license/no-warranty block to stdout and exit successfully. Add the
  bug-report/home-page/help footer to `--help` via the parser's epilog.
- Ordinary arguments are input files; specify output with `-o`/`--output`.

## Internationalization

- Use the standard-library **`gettext`** module — bind the text domain to the package name
  (`gettext.textdomain("pkg")`, `_ = gettext.gettext`), wrap translatable strings as `_(
  "...")`, and use `ngettext` for plurals. Translate complete sentences, not fragments
  (`conventions.md` §8). This shares GNU's `.po`/`.mo` toolchain and `localedir` directly.

## License and headers

Every `.py` file carries a GPL-3.0-or-later header:

```python
# SPDX-License-Identifier: GPL-3.0-or-later
# <one-line description of the file's purpose>
#
# Copyright (C) <year> <holder>.
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later version.
```

`COPYING` holds the GPL.

## Build: a GNU layer over packaging

Keep `pyproject.toml` (`license = "GPL-3.0-or-later"`) as the build manifest, and wrap it in a
GNU-conformant Makefile implementing the standard targets and directory variables
(`conventions.md` §11):

```makefile
prefix      = /usr/local
exec_prefix = $(prefix)
bindir      = $(exec_prefix)/bin
PYTHON      = python3

all:
	$(PYTHON) -m build

check:
	$(PYTHON) -m pytest

install: all
	$(NORMAL_INSTALL)
	$(PYTHON) -m pip install --prefix=$(DESTDIR)$(prefix) .

clean:
	rm -rf build dist *.egg-info

.PHONY: all check install clean
```

Documentation is a Texinfo manual; docstrings are supplementary API reference, not the primary
manual.
