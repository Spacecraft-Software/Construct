# Authoring a Texinfo Manual

How to write a correct, well-structured `.texi`. The focus here is the rules that
*prevent build failures* and produce clean output across all formats — the
mechanical command list is in `command-reference.md`; house rules are in
`house-style.md`.

## The mental model

A Texinfo manual is a **tree of nodes**. Each node is a navigable unit (in Info,
a "page"). The tree is expressed twice and the two must agree:

1. **Sectioning** — `@chapter`/`@section`/`@subsection`/… give the visual/logical
   hierarchy.
2. **Menus** — `@menu` blocks list each node's children, giving Info its
   navigation.

Modern `texi2any` computes `Next`/`Previous`/`Up` automatically, so you rarely
write pointers by hand. But the **node↔section pairing** and the **menu↔tree
agreement** are on you, and getting them wrong is the #1 cause of `makeinfo`
warnings.

## The cardinal rule: one node, one section

Pair every `@node` with the sectioning command for that unit, on the very next
content line:

```texinfo
@node Installation
@section Installation
```

- Chapters pair `@node` with `@chapter`; sections with `@section`; and so on.
- The `Top` node pairs with `@top` and lives inside `@ifnottex` (it is omitted
  from printed/DocBook output).
- Do **not** put two sectioning commands under one node, or a section with no
  node — both desync the tree.

## Node names

- Must be unique within the manual.
- Avoid the characters `, : . ( )` and `@`-commands in node names — they confuse
  the parser and cross-reference syntax. Keep them short and plain
  ("Getting Started", not "Getting Started: A Tour (v1.0)").
- Case matters for cross-references; match exactly.

## Menus mirror the tree

Every node that has children needs a `@menu` listing them; the master menu in the
`Top` node lists the chapters. A menu entry whose label equals the node name uses
the `Node::` form:

```texinfo
@menu
* Installation::    How to install.
* Usage::           Day-to-day operation.
* Index::           Complete index.
@end menu
```

If `makeinfo` warns `node 'X' is next for 'Y' in menu but not in sectioning` (or
vice-versa), the menu and the section tree disagree — fix one to match the other.
See `linting.md` for the full warning catalog.

## Cross-references

Three flavours, distinguished by how they read in a sentence:

- `@xref{Node}.` → "See Node." Use at the **start** of a sentence; **must** be
  followed by a punctuation mark (the `.` is required).
- `@pxref{Node}` → "see Node". Use **inside parentheses**: `(@pxref{Usage})`.
- `@ref{Node}` → bare "Node"; also must be followed by punctuation.

For external manuals add the manual arg: `@xref{Node,,,manual-name,Manual
Title}`. For web links use `@url{https://…}` / `@uref{https://…, link text}`.
The trailing punctuation requirement is real — omitting it triggers a warning.

## Markup discipline

Use semantic markup so every output format renders correctly:

| Intent | Command |
|--------|---------|
| Code, identifiers, expressions | `@code{Shell::detect}` |
| A command name | `@command{makeinfo}` |
| A CLI option | `@option{--no-split}` |
| A file or path | `@file{doc/gitway.texi}` |
| A literal keystroke | `@kbd{C-c C-c}` / `@key{RET}` |
| An environment variable | `@env{PATH}` |
| A metasyntactic variable | `@var{pid}` |
| Defining use of a term | `@dfn{node}` |
| Emphasis / strong | `@emph{}` / `@strong{}` |
| Email / URL | `@email{a@@b.org}` / `@url{}` |

Escape the specials: `@@` for `@`, `@{`/`@}` for braces. After a sentence-ending
capital letter (an abbreviation), use `@:` to suppress extra space (`e.g.@:`); the
opposite, `@.`/`@!`/`@?`, forces end-of-sentence spacing.

## Examples and transcripts

Use `@example` for shell transcripts and code (fixed-width, not filled). Give the
language as the first argument where it helps tooling: `@example sh`. Use
`@verbatim … @end verbatim` when the content contains `@`/`{`/`}` you don't want
interpreted (e.g. a raw config snippet):

```texinfo
@example sh
$ gitway clone git@@host:repo.git
@end example
```

Note `@@` even inside `@example` (it is filled-off but still `@`-parsed);
`@verbatim` is the escape hatch where nothing is interpreted.

## Lists and tables

- `@itemize @bullet` / `@enumerate` for bullet/numbered lists, `@item` per entry.
- `@table @code` for option/term description lists — the formatting command
  (`@code`, `@samp`, `@option`, `@var`) styles each first-column term:

  ```texinfo
  @table @option
  @item --no-split
  Produce a single output file instead of one per node.
  @item --css-include=@var{file}
  Embed @var{file} as CSS in HTML output.
  @end table
  ```
- `@ftable @deffn`/`@vtable` auto-add the term to the function/variable index.
- `@multitable @columnfractions .3 .7` for true grid tables; rows start with
  `@item`/`@headitem`, columns separated by `@tab`.

## Indices

Sprinkle index entries as you write — they cost nothing and make the manual
navigable: `@cindex` for concepts, `@findex` for functions (definition commands
add these automatically), `@vindex` variables, `@kindex` keys, `@pindex`
programs, `@tindex` types. Emit the index near the end with `@printindex cp`
(concepts), `@printindex fn` (functions), etc., each under its own unnumbered
node. Merge indices with `@syncodeindex fn cp` to combine function entries into
the concept index when a single index is cleaner.

## Structural checklist

- [ ] `Top` node inside `@ifnottex`, paired with `@top`, contains the master menu.
- [ ] Every other `@node` immediately followed by exactly one sectioning command.
- [ ] Every parent node has a `@menu` listing its children; menus match the tree.
- [ ] Cross-references followed by punctuation; `@pxref` only inside parentheses.
- [ ] Index nodes (`@printindex …`) present and listed in the master menu.
- [ ] Builds with `makeinfo --no-split` at zero warnings (`linting.md`).
