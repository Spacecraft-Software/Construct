# The JavaScript Trap and Free JavaScript

Read this whenever the work ships **any JavaScript to a browser** — a web page, a
web app, an embedded widget, or a snippet. The problem is subtle because the usual
free-software reflexes don't fire: the script is never "downloaded" or "installed"
in the obvious sense, yet the user's browser fetches and **runs it automatically,
silently, on every visit**. If that script is nonfree, the user is running nonfree
software without ever choosing to — the *JavaScript Trap*.

The goal: a page should run **only free JavaScript**. Either make the JS free (free
license + a machine-readable license notice + available source) or don't ship it.

## When it matters: trivial vs. nontrivial

The concern is **nontrivial** JavaScript. A script is nontrivial — and therefore
must be free and labeled — if it does substantive work, in particular if it:

- makes network requests (`fetch` / `XMLHttpRequest` / AJAX),
- defines and calls its own functions or methods (beyond a one-line inline
  handler), or dynamically loads further scripts,
- is more than a few lines, or is minified/obfuscated.

Genuinely trivial scripts — a tiny inline declaration, a handful of lines doing
nothing networked — are not the target. When in doubt, treat it as nontrivial and
label it; labeling a trivial script costs nothing.

## Making JavaScript free (three requirements)

1. **A free license.** License the script under a free, GPL-compatible license —
   default `GPL-3.0-or-later`, consistent with the rest of the project.
2. **A machine-readable license notice** that a tool (and a person) can detect.
   Two recognized methods; use either or both:
   - **Inline magic comments** wrapping the script, giving a recognized license
     identifier and a canonical URL for the license text:

     ```javascript
     // @license magnet:?xt=urn:btih:... GPL-3.0-or-later
     ...script...
     // @license-end
     ```

     A prose variant uses an `@licstart … @licend` block that names the license
     and links to it; the essential content is the same — *which* free license,
     and *where* its text is.
   - **JavaScript License Web Labels.** Publish a labels page (conventionally at
     `/jslicense`), linked from every page that loads scripts via
     `<a rel="jslicense" href="/jslicense">`. The page is a table with one row per
     script file: the **file**, its **license** (name linked to the canonical
     license text), and its **source** (the file itself if already readable, or a
     link to the non-minified source). This method scales better than inline
     comments for a site with many scripts.
3. **Available source.** Freedom 1 must be real: if you serve minified or generated
   JavaScript, link to the **corresponding non-minified source** (and to build
   instructions if it is produced by a build step). Shipping only minified blobs
   with no source path fails this even if the license is free.

## Practical defaults

- **Prefer no JavaScript, or the least that does the job.** A page that works
  without scripts sidesteps the trap entirely and is more robust.
- **Serve scripts first-party.** Pulling scripts from third-party CDNs or analytics
  providers usually reintroduces nonfree JavaScript (and surveillance) and breaks
  your labeling. Host the free scripts yourself.
- **Keep source readable or linked.** Either don't minify, or always link the
  non-minified source from the labels page.
- **Test with LibreJS.** GNU LibreJS is the browser extension that detects and
  blocks nonfree nontrivial JavaScript; a correctly licensed and labeled page
  passes it. Use it as the acceptance check for any page you produce.
- **Auditing existing pages:** flag every nontrivial script that lacks a free
  license notice or a source link; flag third-party script/tracker includes; then
  fix by labeling, self-hosting, providing source, or removing the script.

## Tie-in: don't link nonfree-JS sites either

The same principle reaches outward (see `SKILL.md`, "Do not promote proprietary
software or services"): **don't use or promote any website that serves nonfree
JavaScript** — don't link to it, host the project on it, or recommend it.
`github.com` is the prominent example (its account flow and much of its UI need
nonfree JS), but the criterion is the JavaScript itself, not the brand or its
popularity. Describe or quote such a destination instead of linking to it, and host
free projects on **GNU Savannah** or self-host.
