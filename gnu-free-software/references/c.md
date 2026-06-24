# GNU Standards in C

C is GNU's default language for compiled, high-speed code, and the Coding Standards were
written around it — so for C the rules are literal, not adapted. This file covers the
C-specific mechanics; the language-agnostic contract (errors, `--version`/`--help`, i18n,
release process) is in `conventions.md`.

## Standard and extensions

- ISO **C99** features are fine to use. **Never** use trigraphs (a C89 misfeature removed
  only in C23).
- Use standard interfaces where possible, but GNU C extensions are welcome when they make
  the program more maintainable, powerful, or clear — gate them behind an `--ansi`/`--posix`/
  `--compatible` option (or `POSIXLY_CORRECT`) only if they could break real programs.
- Don't redeclare system functions yourself; that invites conflicts. Define `_GNU_SOURCE`
  when compiling.
- Prefer `if (HAS_FOO) … else …` over `#ifdef HAS_FOO` when the condition is known at build
  time — the compiler checks all paths. For function-like macros, introduce a `HAS_xxx` 0/1
  macro to make this work.

## Formatting (this is GNU's canonical style)

- Keep source lines to **79 characters** or less.
- The open brace starting a **function body** goes in **column 1**, and the **function name**
  starts in **column 1** (tools find defuns this way):

  ```c
  static char *
  concat (char *s1, char *s2)
  {
    ...
  }
  ```

- Braces inside a function are **not** in column 1. `struct`/`enum` braces go in column 1
  (unless the whole thing fits on one line).
- Put a **space before open-parenthesis** and **after each comma**: `foo (a, b)`.
- When splitting an expression, split **before** an operator, not after; add extra parentheses
  so indentation reflects nesting (and so Emacs preserves it).
- Always brace a nested `if-else`; write `else if` on one line or brace the inner `if`.
- Format `do … while` with the `while` on its own line after the closing brace.
- Use formfeed (Ctrl-L) on its own line to page the file at logical boundaries (not mid-function).

This is also the default style of `indent` 1.2+. GNU treats formatting as a *recommendation*,
not a requirement — but within one program, be consistent, and when editing an existing
program follow its style.

## Commenting

- English; a complete sentence with the first word capitalized (but never recapitalize a
  lowercase identifier that opens a sentence — reword instead). **Two spaces** after a
  sentence-ending period (so Emacs sentence commands work).
- Every program's `main` file opens with a one-line "what it is for" comment; every source
  file opens with its name and purpose. Comment each function (what it does, its arguments and
  their meaningful values, the return value) and each static variable. Speak about argument
  *values* in upper case ("the inode number NODE_NUM").
- Every non-trivial `#endif`/`#else` carries a comment naming the condition and its sense:
  `#endif /* not foo */`.

## Clean constructs and naming

- Explicitly declare all types; declarations of external/forward functions go near the file
  top or in a header, never inside a function. Use `enum`, not `#define`, for integer
  constants (the debugger knows enums).
- One distinct local variable per purpose with a meaningful name; minimal scope; don't shadow
  globals (`-Wshadow`). One variable per declaration line. Don't combine a struct tag
  declaration with variable/typedef declarations. Avoid assignments inside `if` conditions
  (inside `while` is fine).
- Names are **English**, descriptive (not terse), **underscore_separated**, lower case —
  reserve upper case for macros, enum constants, and uniform name-prefixes. Name option flags
  after the option's *meaning*, not its letter, with a comment giving both:
  `int ignore_space_change_flag; /* Ignore changes in horizontal whitespace (-b). */`
- A library reserves a name prefix (>2 chars) for all external symbols; undocumented entry
  points start with `_` + prefix. Make library functions reentrant.

## Portability

- Handle CPU differences (byte order, alignment). GNU does **not** support 16-bit `int`. Use
  `unsigned char` for raw byte I/O (`getchar` into an `int`, store through an `unsigned char`).
  Avoid casting pointers to integers. Remember `off_t` may be wider than `long`.
- Use **Gnulib** for portable implementations of standard/enhanced interfaces and for
  `xmalloc`/`xrealloc`-style checked allocation; integrate it via Autoconf/Automake so
  `configure` fills in missing pieces. GNU/Linux is the primary target; other Unix-likes are
  nice-to-have, non-Unix systems are not an obligation.

## Robustness, realized in C

- Check every system call and `malloc`/`realloc` return (check `realloc` even when shrinking).
  `malloc` failure is fatal in a noninteractive program; in an interactive one, abort the
  command back to the command loop. After `free`, treat the block as altered — fetch what you
  need first.
- Decode arguments with **`getopt_long`** (POSIX short options plus GNU long options).
- On impossible conditions, `abort ()` with an explanatory comment rather than a message.
- Create temp files in `$TMPDIR` (or `/tmp`) safely:
  `open (name, O_WRONLY | O_CREAT | O_EXCL, 0600)` or Gnulib `mkstemps`.
- Internationalize with `gettext`/`ngettext` (see `conventions.md` §8); a translatable string
  is `gettext ("...")`.

## Build and license

- Ship `configure` + a GNU-conformant Makefile with the standard targets and directory
  variables (`conventions.md` §11). Compile with `-g` by default.
- GPL-3.0-or-later header in every file; `COPYING` holds the GPL. For **networked
  software**, use `AGPL-3.0-or-later` instead (the GNU Affero GPL — swap the SPDX
  identifier and the license name in the header). Texinfo manual is the canonical
  documentation.
