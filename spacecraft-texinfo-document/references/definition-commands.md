# Definition Commands — Documenting Software APIs

Definition commands are Texinfo's purpose-built machinery for documenting a public
API. Each produces a formatted header for an entity (function, type, variable,
method), auto-indexes it, and renders consistently across Info/HTML/PDF. This is
what makes Texinfo the right tool for a reference manual versus prose Markdown.

## Choosing the right command

The split is **untyped vs typed**. Dynamically-typed or shell/Lisp APIs use the
plain forms; statically-typed languages (Rust, Go, C, C++, Zig) use the `@deftype…`
forms so the return type and argument types render in the signature.

| Entity | Untyped | Typed (Rust/Go/C/Zig) |
|--------|---------|------------------------|
| Function / command | `@deffn cat name args` | `@deftypefn cat type name args` |
| Plain function | `@defun name args` | `@deftypefun type name args` |
| Macro | `@defmac name args` | — |
| Variable / option | `@defvr cat name` / `@defopt name` | `@deftypevr cat type name` / `@deftypevar type name` |
| Data type | `@deftp cat name attrs` | `@deftp cat name attrs` |
| Class/instance var | `@defcv cat class name` / `@defivar class name` | `@deftypecv cat class type name` / `@deftypeivar class type name` |
| Method | `@defmethod class name args` | `@deftypemethod class type name args` |
| Operation | `@defop cat class name args` | `@deftypeop cat class type name args` |

Each has an `x` variant (`@deffnx`, `@deftypefnx`, …) used to give a second
heading for the **same** description — e.g. two overloads or an alias that share
one explanation. All definition commands are written on **one line** (a lone `@`
at line end continues it); multiword arguments go in braces. Close with
`@end def…`.

## Anatomy

```
@deftypefn  {category}  {return-type}  name  arguments
   ^command   ^braces if multiword       ^      ^
description text…
@end deftypefn
```

The `category` prints as the entity's kind ("Function", "Macro", "Constant"). Brace
any multiword category or type: `{Special Form}`, `{Result<Shell, DetectError>}`.
The return type may be empty as `{}`.

## Rust (primary)

Map Rust constructs as follows. Use `@deftp` for `struct`/`enum`/`trait`,
`@deftypefn {Function}` for free functions, `@deftypeop {Method}` or
`@deftypemethod` for inherent/trait methods, `@deftypevr {Constant}` for
`const`/`static`.

```texinfo
@c --- an enum ---
@deftp {Type} Shell ash bash ion nu pwsh sh Unknown
The shells @code{detect} can identify.  @code{Unknown} is returned when no
shell can be determined.
@end deftp

@c --- a free function returning Result ---
@deftypefn {Function} {Result<Shell, DetectError>} detect ()
Detect the current process's shell.  Returns @code{Ok(Shell)}, or
@code{Err(DetectError)} on failure.
@end deftypefn

@c --- a method on a type (category, class, return type, name, args) ---
@deftypeop {Method} Shell {&'static str} name ()
Return the canonical lowercase name of the shell, e.g.@: @code{"nu"}.
@end deftypeop

@c --- a trait method, plus an alias sharing the description (x-variant) ---
@deftypefn  {Trait Method} {Result<(), Error>} write_all (@var{buf}: &[u8])
@deftypefnx {Trait Method} {Result<(), Error>} write_vectored (@var{bufs}: &[IoSlice])
Write the entire buffer.  @code{write_vectored} is the scatter-gather form.
@end deftypefn

@c --- a constant ---
@deftypevr {Constant} {&'static str} VERSION
The crate version string.
@end deftypevr
```

Conventions: wrap argument *names* in `@var{}` so they render as metavariables;
put types in `@code{}` within prose. Keep the signature faithful to the real
Rust — readers copy from it.

## GNU Guile (Scheme — untyped)

Scheme is dynamically typed, so use the plain forms. Procedures use `@deffn` with
category `{Scheme Procedure}`; variables/parameters use `@defvr`.

```texinfo
@deffn {Scheme Procedure} detect-shell
@deffnx {Scheme Procedure} detect-shell-from pid
Return a symbol naming the current shell.  @code{detect-shell-from} starts at
@var{pid}.
@end deffn

@defvr {Parameter} current-shell
A SRFI-39 parameter holding the active shell symbol.
@end defvr
```

## Go and C (typed)

```texinfo
@c Go: package-qualified name, Go type syntax
@deftypefn {Function} {(Shell, error)} Detect ()
Detect the current process's shell.
@end deftypefn

@c C: return type, name, fully-typed args
@deftypefn {Function} int shell_detect (shell_t *@var{out})
Write the detected shell into @var{out}; return 0 on success, -1 on error.
@end deftypefn

@deftypevr {Macro} int SHELL_DETECT_MAXDEPTH
Maximum parent-process walk depth.
@end deftypevr
```

## Indexing and presentation

- Definition commands **auto-index** into the function index (`fn`) — emit it with
  `@printindex fn` under a final `@unnumbered Function Index` node.
- To group entities, document related items under one node and use `@deffnx` for
  variants so the index stays clean.
- For a wholly separate API index, declare one with `@defcodeindex` and route
  entries with `@syncodeindex`.
- In prose around a definition, reference the entity with `@code{}` and its
  arguments with `@var{}` so cross-format rendering stays consistent.

## House notes

- A pure API reference starts from `assets/software-manual.texi` (already wired
  with `@deftp`/`@deftypefn`/`@deftypevr` and a function index).
- Keep one entity's description to its own definition block; do not narrate
  multiple unrelated functions inside one `@deffn`.
- Faithfully reproduce the real signature — the manual is a contract with the
  reader, and a wrong type is worse than no manual.
