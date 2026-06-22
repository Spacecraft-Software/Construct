# Credits

## Agent registry — vercel-labs/skills

`registry/agents.toml` — the set of supported AI coding agents and their
project/global skills directories — is **derived from the
[vercel-labs/skills](https://github.com/vercel-labs/skills) project**,
specifically its `src/agents.ts` registry, © Vercel, Inc., licensed under the
**MIT License**.

The Spacecraft Software adaptation reorganizes that data into a static TOML
registry embedded in the `construct` binary. Per Standard §4.2 (third-party-
derived artifacts preserve their upstream license), `registry/agents.toml` is
dual-licensed **`GPL-3.0-or-later OR MIT`** — the MIT arm preserves Vercel's
notice — as recorded in the repository `REUSE.toml`. Only home-relative path
defaults are modeled; the upstream's environment-variable overrides
(`CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `XDG_CONFIG_HOME`, …) are not.

The rest of the `construct` CLI is original Spacecraft Software work licensed
`GPL-3.0-or-later`.
