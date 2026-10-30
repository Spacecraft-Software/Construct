# Backend: Google Antigravity (`agy`, IDE, Desktop)

Back to [`../SKILL.md`](../SKILL.md). Used only when the user names
Antigravity, `agy` or a Gemini model through it. There are two ways in:

- **`agy` headless**, which the orchestrator drives like any other delegate (below);
- **the Antigravity IDE or Desktop app**, which the user drives and the orchestrator
  checks (*IDE and Desktop*, further down).

Flags change between releases. Before the first run of a session, confirm the
ones below with `agy --help`, and follow the help text where it differs.

## Reachability (§0.2)

- `agy --version` succeeds. Headless runs use the credentials cached by an
  interactive `agy` sign-in; a run that is not signed in exits with
  `authentication required`. Never sign in for the user: stop and ask them to
  run `agy` once themselves.
- `agy models` lists the model slugs the account can use. Pick from that
  list, never from memory; `--model` with an unknown slug exits `1` rather
  than falling back.

## Data use (§0.3)

Gemini prompts on a personal Google account follow that account's Gemini
activity and data settings. On a non-public repository, Antigravity gets the
code only after the user has cleared it for this account.

## Permissions

A headless run that needs a tool it was not approved for is **soft-denied**:
the run continues and still exits `0`, with a notice on stderr. A run that
changed nothing and printed that notice failed on permissions, not on the
task; say so instead of counting it as an attempt.

Pre-approval lives in the user's `~/.gemini/antigravity-cli/settings.json`
under `permissions.allow`, as `action(target)` rules. That is the user's
configuration: never edit it yourself. When it lacks what a task needs,
propose the narrowest rules and let the user add them, for example:

```json
{
  "permissions": {
    "allow": [
      "write_file(src/)",
      "command(regex:cargo (build|test|clippy|fmt)( .*)?)"
    ]
  }
}
```

Never propose `command(git)` or any rule that lets it run `git` (that is how a
delegate commits or pushes), and never pass `--dangerously-skip-permissions`.

## Writing code: `agy -p`

Run from the checkout (or worktree, §6) that §4 recorded, with the brief from
[`task-brief.md`](task-brief.md):

```sh
cd <checkout> && agy -p "$(cat <scratch>/brief-<task>.txt)" \
  --model <slug> --effort <level> --sandbox \
  --output-format json --print-timeout 15m \
  > <scratch>/agy-<task>.json
jq -r '.status, .response' <scratch>/agy-<task>.json
```

- Leave `--model` and `--effort` out unless the user named them.
- `--sandbox` turns on the terminal sandbox, which is off by default.
- Anything but `"status": "SUCCESS"` is a failed run; the reason is in
  `.error` and on stderr.
- A task that needs more than `--print-timeout` is two tasks.

## Reviewing

`agy -p` with the review instruction from the Poe reference and
`git diff <base>` in the prompt, read-only: no write rules are needed, and
none should be added for it. Triage as §7 says.

## IDE and Desktop

The orchestrator cannot drive the Antigravity IDE or the Desktop app. When the user
wants a task done there:

1. Prepare the clean checkout or worktree exactly as §4 says, and record
   `$branch`, `$base` and `$remote`.
2. Write the brief to a file outside the tree (the scratch directory), and
   give the user its path and the worktree to open.
3. Wait for the user to say the agent is done. Then run every §4 step, as for
   any other delegate: the IDE agent's result is checked, not trusted.

## Report fields

Provider `Google`, harness `Antigravity CLI`, `Antigravity IDE` or
`Antigravity Desktop`, model the slug used (and the effort when one was set).
