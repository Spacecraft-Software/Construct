# Linting & Fixing Texinfo

`makeinfo` is the linter. A manual is done only when it builds at **zero errors,
zero warnings**. This guide is the workflow plus a catalog of the common
diagnostics and their fixes.

## The validation loop

```sh
# Strictest, fastest gate — Info build, output discarded:
makeinfo --no-split FILE.texi -o /dev/null
```

Silence = pass. Anything on stderr is a real problem; fix it rather than
suppressing it. Escalate detail with `--verbose`; bound noise with
`--error-limit=N`. **Do not reach for `--no-validate` or `@novalidate`** to make
warnings disappear — they hide structural breakage that surfaces later as broken
navigation. The two legitimate uses of `@novalidate` are single-node drafts and
deliberately incomplete partial builds.

Also run REUSE on the source: `reuse lint` must pass, which means the inline
`@c SPDX-FileCopyrightText`/`@c SPDX-License-Identifier` header (full address) is
present and well-formed (`house-style.md` §1).

## Diagnostic catalog

### Node / menu desync (most common)

> `warning: node 'X' is next for 'Y' in menu but not in sectioning`
> `warning: node 'Y' is up for 'X' in sectioning but not in menu`
> `warning: unreferenced node 'Z'`

**Cause:** the menu tree and the section tree disagree. **Fix:** make the `@menu`
entries exactly match the child `@node`/sectioning sequence. Every node reachable
in sectioning must appear in its parent's menu, and vice-versa. An `unreferenced
node` is a node missing from any menu — add it to the right `@menu`.

### Missing or mispaired section

> `warning: '@node X' is not followed by a section command`

**Cause:** a `@node` with no `@chapter`/`@section`/… on the next content line (or
two sectioning commands under one node). **Fix:** restore the one-node↔one-section
pairing.

### Bad cross-reference

> `warning: @xref to 'X', which is not a node`
> `warning: @ref/@xref should be followed by a comma or period`

**Cause:** the target node name is misspelled/missing, or the reference is not
followed by punctuation. **Fix:** correct the node name (case-sensitive); add the
trailing `.` after `@xref{…}`/`@ref{…}`; ensure `@pxref` is only used inside
parentheses.

### Unmatched block

> `error: '@end table' expected 'example'` (or similar)
> `error: unmatched '@end'`

**Cause:** a block command (`@example`, `@table`, `@quotation`, `@menu`,
`@deffn`, …) opened but not closed, closed with the wrong name, or nested
incorrectly. **Fix:** match each `@foo` with its `@end foo`; check nesting order
(close inner before outer).

### Brace / special-character errors

> `error: misplaced @}` / `error: @command missing closing brace`

**Cause:** an unescaped literal `@`, `{`, or `}`, or an unbalanced brace in a
command argument. **Fix:** escape literals as `@@`, `@{`, `@}`; balance braces.
Inside `@email{}`/`@url{}` the `@` in an address still needs `@@`. Use `@verbatim`
for blocks where you cannot escape everything.

> **Confusing-message trap:** an unescaped `@` in prose that happens to spell a
> block command produces a misleading error far from the real cause. A bare
> `me@example.com` in text parses as the `@example` block command and reports
> `warning: @example should only appear at the beginning of a line` plus a
> cascading `@node seen before @end example` lines later. **Fix:** escape it
> (`me@@example.com`) or wrap it (`@email{me@@example.com}`). When an `@example`/
> `@node`/`@end` error points at a line that looks innocent, scan the *preceding*
> prose for a stray `@word`.

### Header / setup problems

> `warning: must specify a title with @settitle`
> `warning: @documentencoding … unrecognized`

**Fix:** ensure the header block has `@setfilename`, `@settitle`, and a valid
`@documentencoding UTF-8`. Confirm `\input texinfo` is the literal first line and
`@bye` is the literal last.

### Duplicate node

> `error: node 'X' previously defined`

**Fix:** node names must be unique; rename one.

## A repair recipe

When handed a broken `.texi`:

1. **Build and read the first error.** `makeinfo --no-split FILE.texi -o /dev/null`.
   Texinfo reports line numbers; fix top-down — early structural errors often
   cascade into spurious later ones.
2. **Fix block matching first** (`@end` mismatches), then **node↔section
   pairing**, then **menu↔tree agreement**, then **cross-references**, then
   **brace/escape** issues. This order clears the most cascading errors first.
3. **Rebuild after each class of fix** rather than batching — you want to watch
   the warning count fall to zero.
4. **House pass:** confirm the inline SPDX header, `@copying`/`@insertcopying`,
   `@documentencoding UTF-8`, ISO dates (no `@today{}`), and attribution are
   present (`house-style.md`).
5. **Final gate:** the Info build is silent **and** `reuse lint` passes.

## Don't silence — fix

Every warning corresponds to a navigation or rendering defect a reader will hit.
Suppressing with `--no-validate` ships the defect. The only acceptable end state
is a genuinely clean build.
