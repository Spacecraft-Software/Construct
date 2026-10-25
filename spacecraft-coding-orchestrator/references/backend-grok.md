# Backend: xAI Grok Build (`grok`, SuperGrok plan)

Back to [`../SKILL.md`](../SKILL.md). Used only when the user names Grok,
Grok Build or a Grok model for this request. Grok Build is xAI's coding agent
CLI, `grok`: it reads the tree, edits files and runs commands, so every run
goes through the whole of §4. It is included in the SuperGrok plans (Lite,
SuperGrok, Plus, Heavy), which draw on the plan's weekly usage pool. A Grok
model reached through Poe is the Poe backend, not this one.

Flags change between releases. Before the first run of a session, confirm the
ones below with `grok --help`, and follow the help text where it differs.

## Reachability (§0.2)

- `grok --version` succeeds, and a one-line smoke test answers:
  `grok --no-auto-update -p 'Say ok.'`. Headless runs use the token an
  interactive `grok login` cached (`~/.grok/auth.json`). If it is missing or
  expired, stop and ask the user to run `grok login` themselves; never sign in
  for them.
- Do not set `XAI_API_KEY`. It moves the run from the subscription to the
  separately billed, pay-per-token xAI API. Use it only when the user asks
  for API billing, and then never print it, store it or pass it in argv.
- Pass `--no-auto-update` on every run, so a headless call never stalls on an
  update check.

## Data use (§0.3)

Whether xAI trains on a consumer Grok account's conversations follows that
account's data settings. On a non-public repository, Grok gets the code only
after the user has said model training is off for this account, or that it is
a Business or Enterprise one.

## Permissions

`grok -p` is non-interactive: a tool call that would need approval fails and is
reported to the model, so a run without pre-approval tends to finish having
changed nothing. Run with `--always-approve`, and fence it with deny rules,
which win over every approval mode:

- `--deny 'Bash(git *)'` and `--deny 'Bash(gh *)'` on every run, so it cannot
  commit, switch branches or push. §4's branch, `HEAD` and remote checks still
  run afterwards, since a pattern does not catch every way of reaching `git`.
- When `grok --help` offers a sandbox (`--sandbox strict` or a profile), use
  it.

Never edit the user's `~/.grok/config.toml` (its `[permission]` rules and
`permission_mode` are theirs). When a task needs more than the flags give,
propose the narrowest rules and let the user add them.

## Writing code: `grok -p`

Run from the checkout (or worktree, §6) that §4 recorded, with the brief from
[`task-brief.md`](task-brief.md):

```sh
timeout 20m grok --no-auto-update --no-alt-screen \
  -p "$(cat <scratch>/brief-<task>.txt)" \
  --cwd <checkout> --always-approve \
  --deny 'Bash(git *)' --deny 'Bash(gh *)' \
  --output-format json --max-turns 60 \
  > <scratch>/grok-<task>.json
```

- Leave `-m` out unless the user named a model; the account's default then
  applies. A model the user named that the run rejects is reported, not
  swapped for another.
- `--output-format json` prints one object at the end of the run; read the
  final message from it. `plain` is the default, `streaming-json` gives an
  event stream.
- A follow-up on the same task (a re-brief, §5) can keep the session with
  `-s <uuid>` on the first run and `-r <uuid>` on the next, adding the
  `PREVIOUS ATTEMPT FAILED` section as the prompt.
- A task that needs more than the timeout is two tasks.

## Reviewing

`grok -p` with the review instruction from the Poe reference and
`git diff <base>` in the prompt, read-only: no `--always-approve`, and
`--deny 'Bash(*)'` so it reads and answers only. Triage as §7 says.

## Errors

- An authentication error: the cached login expired. Stop and ask the user to
  run `grok login`.
- A usage-limit error: the plan's weekly pool is spent. Report it with the
  reset time the CLI printed; do not switch to `XAI_API_KEY` on your own.

## Report fields

Provider `xAI`, harness `Grok Build CLI`, model as the run reported it (the
`-m` value when one was set).
