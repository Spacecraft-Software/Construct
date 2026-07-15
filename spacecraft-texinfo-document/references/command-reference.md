# Texinfo Command Reference

A categorized map of Texinfo `@`-commands for quick lookup. Distilled from the
GNU Texinfo reference card and manual (v7.x). For full prose explanations of any
command, consult the GNU Texinfo manual; for *how to apply them in house style*,
see the sibling references.

## Contents

1. [Beginning & ending a file](#1-beginning--ending-a-file)
2. [Document metadata & title](#2-document-metadata--title)
3. [Nodes](#3-nodes)
4. [Chapter structuring](#4-chapter-structuring)
5. [Menus](#5-menus)
6. [Cross-references](#6-cross-references)
7. [Marking text](#7-marking-text)
8. [Quotations & displays](#8-quotations--displays)
9. [Lists & tables](#9-lists--tables)
10. [Indices](#10-indices)
11. [Definition commands](#11-definition-commands) (see also `definition-commands.md`)
12. [Special characters, glyphs & breaks](#12-special-characters-glyphs--breaks)
13. [Conditionals & new commands](#13-conditionals--new-commands)
14. [Includes & printed-output formatting](#14-includes--printed-output-formatting)

---

## 1. Beginning & ending a file

| Command | Purpose |
|---------|---------|
| `\input texinfo` | Mandatory first line. |
| `@setfilename name.info` | Name the Info output file. |
| `@settitle title` | Set the document title (page headers). |
| `@documentencoding UTF-8` | Declare input/output encoding (house: always UTF-8). |
| `@documentlanguage ll[_cc]` | Set document language, e.g. `en`. |
| `@bye` | End of source; everything after is ignored. |
| `@c text` / `@comment text` | Comment to end of line. |
| `@c %**start of header` / `@c %**end of header` | Delimit the header block. |

## 2. Document metadata & title

| Command | Purpose |
|---------|---------|
| `@copying … @end copying` | Declare copying/licence permissions once. |
| `@insertcopying` | Insert the `@copying` text (use on title page + Top node). |
| `@dircategory cat` / `@direntry … @end direntry` | Info directory menu entry. |
| `@titlepage … @end titlepage` | Title-page block (printed output). |
| `@title`, `@subtitle`, `@author` | Title-page fields. |
| `@titlefont{text}` | Print text in a larger font. |
| `@center line` | Center a line (titles). |
| `@page` | Force a new page (printed). |
| `@vskip 0pt plus 1filll` | Spring spacing to push copyright to page bottom. |
| `@contents` | Full table of contents. |
| `@shortcontents` / `@summarycontents` | Chapter-level TOC (print/HTML only). |

## 3. Nodes

| Command | Purpose |
|---------|---------|
| `@node name` | Begin a node (one per sectioning command). |
| `@top title` | Mark the topmost node (immediately after `@node Top`). |
| `@anchor{name}` | Define a cross-reference target at the current spot. |
| `@nodedescription text` | Description for menu references to this node. |
| `@novalidate` | Suppress node-reference validation (avoid; masks real errors). |

Modern Texinfo auto-computes `Next`/`Previous`/`Up` pointers, so a bare
`@node Name` line is enough — do not hand-write pointers unless overriding.

## 4. Chapter structuring

Numbered, in contents: `@chapter`, `@section`, `@subsection`, `@subsubsection`.
Unnumbered, in contents: `@unnumbered`, `@unnumberedsec`, `@unnumberedsubsec`,
`@unnumberedsubsubsec`. Lettered appendices: `@appendix`, `@appendixsec`,
`@appendixsubsec`, `@appendixsubsubsec`. Headings (not in contents, no node):
`@chapheading`, `@majorheading`, `@heading`, `@subheading`, `@subsubheading`.
Grouping/shifting: `@part title`, `@raisesections`, `@lowersections`.

## 5. Menus

| Command | Purpose |
|---------|---------|
| `@menu … @end menu` | A menu of child nodes (drives Info/structure). |
| `* Node::  desc` | Menu entry pointing to a node of the same name. |
| `* Label: Node.  desc` | Menu entry with a distinct label. |
| `@detailmenu … @end detailmenu` | Optional detailed listing in a master menu. |

## 6. Cross-references

| Command | Purpose |
|---------|---------|
| `@xref{node,…}` | Reference starting "See …" (sentence start; follow with `.`). |
| `@pxref{node,…}` | "see …" inside parentheses. |
| `@ref{node,…}` | Bare reference, no "See"/"see"; follow with punctuation. |
| `@link{node,[label],[manual]}` | Plain link, no visible markup (no-op in Info). |
| `@url{url[,text[,repl]]}` / `@uref{…}` | External hyperlink. |
| `@cite{title}` | Name a work with no Info file (no hyperlink). |
| `@xrefautomaticsectiontitle on\|off` | Use section title vs node name in refs. |

Full args: `@xref{node, [entry], [node-title], [info-file], [manual]}`.

## 7. Marking text

**Regular-text markup:** `@abbr{…}`, `@acronym{…}`, `@dfn{term}`, `@emph{}`,
`@strong{}`, `@sub{}`, `@sup{}`, `@var{metavar}`.

**Literal/code markup:** `@code{expr}`, `@command{name}`, `@env{var}`,
`@file{name}`, `@kbd{input}`, `@key{KEY}`, `@option{--flag}`, `@samp{text}`,
`@verb{Xliteral textX}`, `@indicateurl{url}`, `@email{addr[,text]}`.

**Other:** `@sc{text}` (small caps), `@t{text}` (typewriter), `@math{expr}`,
`@minus{}`, `@geq{}`, `@leq{}`.

## 8. Quotations & displays

| Command | Purpose |
|---------|---------|
| `@quotation [leader]` / `@smallquotation` | Indented quote (used for licence text). |
| `@indentedblock` / `@smallindentedblock` | Left-indent block. |
| `@example [lang]` / `@smallexample` | Fixed-width, unfilled example (give language). |
| `@verbatim … @end verbatim` | Output exactly as-is, no `@`-processing. |
| `@lisp` / `@smalllisp` | Like `@example`, for Lisp. |
| `@display` / `@format` / `@flushleft` / `@flushright` | Unfilled text-font displays. |
| `@cartouche` | Box around the enclosed text (printed). |

## 9. Lists & tables

| Command | Purpose |
|---------|---------|
| `@itemize mark` | Unordered list (`@bullet`, `@minus`, …); `@item` per entry. |
| `@enumerate [start]` | Numbered/lettered list; `@item` per entry. |
| `@table fmt` | Two-column description list; `@item` first-column text. |
| `@ftable fmt` / `@vtable fmt` | Like `@table`, auto-index into fn/var index. |
| `@multitable cols` | Multi-column table; `@item`/`@headitem` rows, `@tab` separates. |
| `@item` / `@itemx` | List/table entry (`@itemx` = no extra space above). |
| `@headitem` / `@headitemfont{}` | Heading row (multitable). |
| `@asis` | Pass-through formatting-command for `@table`. |

## 10. Indices

Predefined: `@cindex` (concept), `@findex` (function), `@vindex` (variable),
`@kindex` (key), `@pindex` (program), `@tindex` (type). Print with
`@printindex cp\|fn\|vr\|ky\|pg\|tp`. Management: `@defindex newidx`,
`@defcodeindex newidx`, `@syncodeindex from to`, `@synindex from to`,
`@subentry`, `@sortas{key}`, `@seealso{entry}`, `@seeentry{entry}`.

## 11. Definition commands

Function-like: `@deffn[x]`, `@defun[x]`, `@defmac[x]`, `@defspec[x]`,
`@deftypefn[x]`, `@deftypefun[x]`. Variable-like: `@defvr[x]`, `@defvar[x]`,
`@defopt[x]`, `@deftypevr[x]`, `@deftypevar[x]`. Types: `@deftp[x]`.
Object-oriented: `@defcv`, `@defivar`, `@defmethod`, `@defop`, `@deftypecv`,
`@deftypeivar`, `@deftypemethod`, `@deftypeop`. **See `definition-commands.md`
for full syntax, language mappings, and worked examples — this is the core of API
documentation.**

## 12. Special characters, glyphs & breaks

**Literal specials:** `@@` (`@`), `@{` `@}` (braces), `@backslashchar{}`,
`@ampchar{}`, `@hashchar{}`, `@comma{}`.

**Symbols/logos:** `@copyright{}`, `@registeredsymbol{}`, `@bullet{}`,
`@dots{}`, `@enddots{}`, `@euro{}`, `@pounds{}`, `@textdegree{}`, `@TeX{}`,
`@LaTeX{}`, `@today{}` *(avoid — non-ISO date, see house-style.md §5)*,
`@U{XXXX}` (Unicode hex). Accents: `@'`, `@`` `, `@"`, `@~`, `@^`, `@,` etc.

**Code-example glyphs:** `@result{}` (⇒), `@expansion{}` (↦), `@equiv{}` (≡),
`@error{}`, `@print{}` (⊣), `@point{}` (⋆).

**Breaks/space:** `@*` (force break), `@/` (allow break), `@-` (soft hyphen),
`@w{text}` (no break within), `@tie{}` (non-breaking space), `@sp n`,
`@page`, `@need mils`, `@group … @end group`.

## 13. Conditionals & new commands

**Conditionals:** `@ifinfo`/`@ifnotinfo`, `@ifhtml`/`@ifnothtml`,
`@iftex`/`@ifnottex`, `@ifplaintext`, `@ifdocbook`, `@ifset var`/`@ifclear var`,
`@inlineraw`, `@inlinefmt`. Flags/values: `@set var [val]`, `@clear var`,
`@value{var}`, `@ifcommanddefined`. Raw: `@html … @end html`, `@tex … @end tex`.

**Defining commands:** `@macro name {params} … @end macro`, `@unmacro`,
`@linemacro`, `@alias new=existing`, `@definfoenclose`.

## 14. Includes & printed-output formatting

**Includes:** `@include file`, `@verbatiminclude file`.

**Printed output:** `@finalout`, `@fonttextsize 10|11`, `@microtype on|off`,
`@allowcodebreaks true|false`. **Paper:** `@afourpaper` (house A4),
`@smallbook`, `@afivepaper`, `@pagesizes [w][,h]`. **Headers/footers:**
`@oddheading`/`@evenheading`/`@everyheading` (and `…footing`), with
`@thischapter`, `@thissection`, `@thispage`, `@thistitle`. **Preferences**
(usually omit so users can override): `@paragraphindent`, `@exampleindent`,
`@firstparagraphindent`, `@setchapternewpage`, `@headings`.
