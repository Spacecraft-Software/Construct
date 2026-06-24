# GNU Universal Conventions

Language-agnostic mechanics that every GNU program must get right, regardless of
implementation language. Read this before the per-language file. Where a rule is
inherently C-specific (e.g. brace placement), it lives in `c.md`, not here.

## Contents

1. Error message format
2. The `--version` contract
3. The `--help` contract
4. Command-line options and the long-option table
5. Interfaces generally
6. Memory and file usage
7. Character set and quoting
8. Internationalization (gettext)
9. Documenting programs (Texinfo, NEWS, terminology)
10. Change logs
11. The release process (configure, Makefile, directory variables, targets)

---

## 1. Error message format

Line and column numbers start at **1**. Messages from noninteractive programs follow a
colon-delimited grammar so tools (and Emacs) can parse them.

From a compiler-like tool with a source location:

```
sourcefile:lineno: message
sourcefile:lineno:column: message
sourcefile:line1.column1-line2.column2: message
```

From another noninteractive program:

```
program:sourcefile:lineno: message      (when a source file is relevant)
program: message                        (when no source file is relevant)
```

When the message follows a program and/or file name, it does **not** begin with a capital
letter (the sentence conceptually started at the line beginning) and does **not** end with
a period. Always include the system error text (the `strerror`/`errno` equivalent), the
file name if any, and the program name: `myprog: cannot open foo.c: No such file or
directory` — never a bare `stat failed`.

**Interactive** programs (reading commands from a terminal) omit the program name from
error messages — the prompt or screen layout shows which program is running. Interactive
messages and usage messages **start with a capital letter** but still **do not end with a
period**.

## 2. The `--version` contract

`--version` prints name/version/origin/legal status to **stdout** and exits **successfully**;
once seen, it overrides all other options and the program does not do its normal job.

The first line is machine-parseable: the version number proper starts after the last space,
and the program name is a **constant string** (never computed from `argv[0]`). The canonical
template:

```
GNU hello 2.3
Copyright (C) 2007 Free Software Foundation, Inc.
License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.
```

- Prefix the program name with `GNU ` **only for an official GNU Project package**;
  other free software uses just the program name (e.g. `tally 1.0`, not `GNU tally`).
  Don't claim the GNU name otherwise.
- A subsidiary program names its package in parentheses: `emacsserver (GNU Emacs) 19.30`.
- Use the standard license abbreviation with `vN+` for "or later" (e.g. `GPLv3+`).
- Write the word **Copyright** in English (and use the `©` symbol only if the charset
  supports it); the notice need only list the most recent year of changes.
- Mention separately-distributed library versions only if they genuinely matter for
  debugging — don't list every dependency "for completeness."
- Optionally end with a list of major authors as credit.

## 3. The `--help` contract

`--help` prints brief invocation documentation to **stdout** and exits **successfully**,
overriding other options. Near the end, include:

```
Report bugs to: mailing-address
pkg home page: <https://www.gnu.org/software/pkg/>
General help using GNU software: <https://www.gnu.org/gethelp/>
```

All programs support both `--version` and `--help`. CGI programs accept them as
command-line options and also via `PATH_INFO`.

## 4. Command-line options and the long-option table

Follow POSIX option syntax, and **also** define long-named (`--verbose`) equivalents for
single-letter options — the GNU way to be both POSIX-compatible and user-friendly. Permit
options intermixed with arguments (a GNU extension over strict POSIX), stopping at `--`.

Ordinary arguments should normally be **input files only**; specify output files with an
option, preferably `-o` / `--output`. Even where an output filename is accepted positionally
for compatibility, also provide the option form.

For cross-program consistency, reuse the spellings from GNU's long-option table rather than
inventing new ones. The most load-bearing conventions:

- `--verbose`, `--quiet`/`--silent` (each a synonym of the other), `--help`, `--version`
- `--output` (output file), `--force` (`-f`), `--recursive`, `--interactive` (`-i`)
- `--all` (`-a`), `--directory`, `--exclude-from`, `--no-warn`, `--dry-run`/`--just-print`

If a program honors `POSIXLY_CORRECT`, it suppresses extensions that conflict with POSIX.

## 5. Interfaces generally

- **A program's behavior must not depend on the name it was invoked under.** If `foo` is a
  link to `ls`, it behaves like `ls`. Select alternate behaviors with a runtime option or a
  compile switch, or build two binaries.
- **Don't make behavior depend on whether stdout/stdin is a terminal or a pipe.** Device
  independence is a design principle. Narrow, documented exceptions exist (refuse to write
  binary to a TTY; make `-f` override), and a few programs must behave like their Unix
  counterparts (`ls`, `sh`) — for those, GNU sometimes ships a device-independent alternate
  (`dir` vs `ls`).
- A program that needs to find its own executable or data files starts from `argv[0]`, but
  because `argv[0]` is only a convention, it must also offer environment variables to
  specify those locations explicitly. Don't give setuid privilege to programs that search
  heuristically for their files.
- If a program offers a GUI, also provide a command-line interface so the same jobs can be
  scripted, and make the GUI work with assistive technology.

## 6. Memory and file usage

- If a program uses only a few megabytes, don't contort it to save memory — reading a whole
  small input into memory is fine. But tools that can process arbitrarily large inputs
  (`cat`, `tail`) must not impose a memory-bound limit on input size; line-oriented tools
  keep only a line in memory.
- Don't free memory just before exit merely to silence a leak checker, and don't complicate
  code to suppress analyzer false alarms.
- Programs must work when `/usr` and `/etc` are read-only. Internal files (logs, locks,
  backups) go in `localstatedir`, not under `/usr` or `/etc`. Honor `TMPDIR` for temp files.

## 7. Character set and quoting

- Prefer plain **ASCII** in source comments and text documents unless the application domain
  needs otherwise. If you need non-ASCII, stick to one encoding per file; **UTF-8** is the
  best choice. Contributor names in change logs may use non-ASCII.
- In the C locale, user-facing quotes should be ASCII `"` or `'` for both open and close.
  Locale-specific quotes in non-C locales are acceptable (typically via gettext). If output
  may be parsed by another program, provide an option for reliable quoting (cf. GNU `ls`
  `--quoting-style`).

## 8. Internationalization (gettext)

Wrap user-visible strings so they can be translated, using the GNU gettext model. The
package declares a text-domain name equal to the package name.

- Wrap each translatable string: the equivalent of `gettext("Processing file '%s'...")`.
- **Translate complete sentences, not fragments.** Don't assemble a sentence from
  conditionally-chosen words — gender and word order differ across languages. Use two full
  alternative strings instead of inserting `"disk"` vs `"floppy disk"` into one frame.
- **Use `ngettext` for plurals** rather than appending `"s"` — plural rules vary (Polish has
  three forms): the equivalent of `ngettext("%d file processed", "%d files processed", n)`.

The per-language file names the concrete binding (C `gettext`, Python `gettext`, Guile
`(ice-9 i18n)`, Rust `gettext-rs`, Go `golang.org/x/text`).

## 9. Documenting programs

- **Texinfo is the canonical GNU documentation format.** Every package should have a Texinfo
  manual serving as *both* tutorial and reference. Man pages are **secondary** — optional,
  and if present they should note that the Texinfo manual is authoritative. Don't make a man
  page the primary documentation; `help2man` can generate one from `--help` output.
- **Structure documentation by the user's concepts and questions, not by the program's
  implementation.** Cover a coherent topic (e.g. one manual for "comparison of files" over
  `diff`/`diff3`/`cmp`), document every option with examples, but organize by subtopic — say
  what jobs each feature is good for, not just what it does.
- Each documented program needs an `Invoking program` (or `program Invocation`) node.
- Provide an **Index** of functions, variables, options, and concepts.
- Keep a **NEWS** file of user-visible changes, newest first, never discarding old entries.
- **Terminology rules:** write "file name" (two words), never "pathname" (reserve "path" for
  search paths). Use "invalid", not "illegal", for bad input ("illegal" is for law). Don't
  write `()` after a function name to mean "this is a function" — `foo ()` is a *call*.
  Prefer the **active voice** and **present tense** ("The function returns…", not "…will be
  returned").
- Manuals more than a few pages long use the **GFDL**; credit the human writers as authors,
  and thank (but don't list as author) any sponsoring company.

## 10. Change logs

Whether kept as `ChangeLog` files or derived from a VCS, change logs must let someone doing
software forensics answer: what changed in this file/function, was it renamed or moved, what
was deleted, and *why*.

- Start each entry with a **header line**: a single complete sentence summarizing the change
  (VCS tools like `git log --oneline` treat it specially).
- Follow with a free-text description of the overall change and its rationale — but the
  *why* of code behavior belongs in code comments; the change log explains why code was
  deleted or moved.
- List the changed entities by file: `* path/file (function_name): what changed.` Name
  functions/variables **in full** — never abbreviate or combine (`{insert,jump-to}-register`
  defeats search). Break long name lists by closing with `)` and reopening with `(`.
- **Conditional changes** use square brackets for the condition:
  `* xterm.c [SOLARIS2]: Include <string.h>.` The changed part of a function uses angle
  brackets: `* sh-script.el (sh-while-getopts) <sh>: Handle empty option string.`
- Simple cases: a short header line can stand alone ("Doc fix"); "All callers changed" avoids
  per-caller entries; one mechanical change across many files is described once.
- When committing someone else's work, attribute them as the author (e.g. `git commit
  --author=`), not in the entry text.

## 11. The release process

### configure

A distribution ships a `configure` shell script (commonly via Autoconf/Automake, but any
implementation is fine) that records the configuration so it affects compilation. It accepts
`--srcdir`, the standard directory options (below), `--build`/`--host`/`--target` for
cross-compilation, and any `--enable-FEATURE` / `--with-PACKAGE` / `VAR=value` argument — a
configure script must accept every `--enable-`/`--with-` option even if it ignores it, so an
entire GNU tree can be configured with one option set. `--enable-` is only for whether to
build part of the program, never to substitute one behavior for another.

### Makefile conventions

- Begin with `SHELL = /bin/sh`; clear and set `.SUFFIXES` explicitly.
- Write rules for POSIX `sh`, not `csh`, `ksh`, or `bash` extensions. Restrict directly-used
  utilities to the portable set (`awk cat cmp cp diff echo expr false grep install-info ln
  ls mkdir mv printf pwd rm rmdir sed sleep sort tar test touch tr true`). Avoid `mkdir -p`
  in rules. Run compilers/tools through make variables (`$(CC)`, `$(INSTALL)`, …).
- Distinguish `./` (build dir) from `$(srcdir)/` (source dir) so VPATH builds work; use `$<`
  / `$@` for single-dependency rules and explicit `$(srcdir)/` for multi-dependency rules.
- Each program variable gets a `…FLAGS` companion (`CFLAGS`, `BISONFLAGS`). Put `CFLAGS` last
  in the compile command so the user can override; never put mandatory flags in `CFLAGS`.
- Make build and install targets work under parallel `make`.

> **OS-specific note:** Compiler and linker optimization flags are not universally
> portable, so don't bake them into `CFLAGS` (above) and adapt them per target. On
> store-isolated distributions — GNU Guix (`/gnu/store`) or Nix (`/nix/store`) — `-flto`
> (Link Time Optimization) may require pointing the linker to the LTO plugin explicitly,
> e.g. `-fuse-ld=mold` (preferred) or `-fuse-ld=bfd` (fallback), because store isolation
> keeps the plugin off the default linker path. The general rule: optimization and linker
> flags are platform-specific and must be adapted to the build host, never assumed.

### Directory variables

Name every install directory by variable so installers can relocate with `make prefix=/usr
install` or `configure --prefix=/usr`. Don't guess system-appropriate values; use these
defaults so all GNU packages behave identically:

| Variable | Default | Holds |
|---|---|---|
| `prefix` | `/usr/local` | base for the others |
| `exec_prefix` | `$(prefix)` | base for machine-specific dirs |
| `bindir` | `$(exec_prefix)/bin` | user-runnable programs |
| `sbindir` | `$(exec_prefix)/sbin` | sysadmin programs |
| `libexecdir` | `$(exec_prefix)/libexec` | programs run by other programs |
| `libdir` | `$(exec_prefix)/lib` | object files and libraries |
| `datarootdir` | `$(prefix)/share` | read-only arch-independent data root |
| `datadir` | `$(datarootdir)` | this program's read-only data |
| `sysconfdir` | `$(prefix)/etc` | per-machine config (ASCII text) |
| `sharedstatedir` | `$(prefix)/com` | arch-independent state programs modify |
| `localstatedir` | `$(prefix)/var` | per-machine state programs modify |
| `runstatedir` | `$(localstatedir)/run` | transient state (PID files) |
| `includedir` | `$(prefix)/include` | headers for GCC |
| `oldincludedir` | `/usr/include` | headers for non-GCC compilers |
| `docdir` | `$(datarootdir)/doc/PKG` | documentation (non-Info) |
| `infodir` | `$(datarootdir)/info` | Info files |
| `htmldir`/`pdfdir`/`psdir` | `$(docdir)` | per-format docs |
| `mandir` | `$(datarootdir)/man` | man pages (`man1dir` = `$(mandir)/man1`, etc.) |
| `localedir` | `$(datarootdir)/locale` | gettext message catalogs |
| `srcdir` | (set by configure) | the sources being compiled |

### Standard targets

Provide: `all` (default; compile with `-g`, don't rebuild docs), `install` (and
`install-html`/`install-pdf`/`install-ps`), `install-strip`, `uninstall`, `clean`,
`distclean`, `mostlyclean`, `maintainer-clean`, `TAGS`, `info`, `html`/`pdf`/`ps` (always
exist, may be no-ops, never dependencies of `all`), `dist`, `check` (self-tests, runnable
without installing), `installcheck`, `installdirs`.

- Support **`DESTDIR`** in `install*`/`uninstall*` only: prepend it to every target path
  (`$(DESTDIR)$(bindir)/foo`) for staged installs. Never set `DESTDIR` in the Makefile, and
  it must not change program behavior.
- Don't strip executables in `install` (keep debug symbols; disk is cheap) — that's what
  `install-strip` is for. Install Info with `$(INSTALL_DATA)` then run `install-info`.
- Classify install commands into **normal / pre-installation / post-installation** using the
  `$(NORMAL_INSTALL)`, `$(PRE_INSTALL)`, `$(POST_INSTALL)` category lines (e.g. `install-info`
  is a post-install command because it edits the shared Info `dir`).

### Making releases

- Identify each release by `major.minor` version. Package `Foo 69.96` as `foo-69.96.tar.gz`
  unpacking into `foo-69.96/`.
- Building/installing must never modify any distributed file. Classify files as source (human
  written) vs non-source (generated); ship generated files (Autoconf/Automake/Bison/Texinfo
  output) only when up to date and machine-independent.
- Ship a `README` (name, version, description, pointer to `INSTALL`, layout notes, pointer to
  the copying conditions), `INSTALL`, and `COPYING` (GPL) / `COPYING.LESSER` (LGPL).
- Make files world-readable, directories `755`. Include **no symbolic links** and no two
  names for one file in different directories (some filesystems can't unpack that). Keep file
  names unique under MS-DOS 8.3 truncation if practical.
