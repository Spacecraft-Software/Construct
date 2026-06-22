# AGENTS.md — Construct skill catalogue

Tool-agnostic guidance for any agent (Claude, Codex, Gemini, Grok, …) editing
skills in this repo. The full, version-controlled workflow lives in
[`CONTRIBUTING.md`](CONTRIBUTING.md); this file is the committed digest of the
rules that bite. (A maintainer-local `CLAUDE.md` overlay adds host-specific notes
— Home Manager fan-out, local config — and is gitignored.)

## Hard rules

- **`SKILL.md` frontmatter `description` ≤ 1024 characters.** The loader rejects
  any skill over the limit (`field 'description' in SKILL.md must be at most
  1024 characters`). Folded `description: >` blocks render by joining lines with
  spaces (blank lines → newlines, plus a trailing newline); that *rendered*
  length is what counts, not the raw line count. Keep a margin — target ≤ 1000.
  Some descriptions already run close, so re-check after any description edit.
- **Rebuild BOTH bundles after any skill-dir edit**, in the same commit:
  `<name>.zip` (`zip -qr`, keeps dir entries) and `<name>.skill` (`zip -qrD`,
  drops them). A bundle that lags its `SKILL.md`/`references/` ships broken
  content to every consumer.
- **Stage explicitly by name — never `git add -A` / `git add .`.** Other root
  `.skill` files carry unrelated uncommitted changes that must not be swept in.
- **Commit in UTC, signed** (signing is global, no flag needed); assistant
  commits add a `Co-Authored-By: Claude …` trailer.
- **Keep the README §2 catalogue row in sync** when adding, removing, or
  re-scoping a skill.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the bundling commands, the drift
sweep, and the push procedure. The maintainer-local `CLAUDE.md` adds the Home
Manager local fan-out (host-specific).
