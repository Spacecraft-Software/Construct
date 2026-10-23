# Backend: OpenAI Codex (ChatGPT plan)

Back to [`../SKILL.md`](../SKILL.md). Used when the user names Codex for this
request ("have Codex write it", "Codex review") or picks it from the §0.1
question, where it is the recommended default. Codex is a full agent:
it reads the tree, edits files and runs commands, so every run goes through
the whole of §4.

Flags change between Codex releases. Before the first run of a session,
confirm the ones below with `codex exec --help` and `codex review --help`,
and follow the help text where it differs from this file.

## Reachability (§0.2)

- `codex --version` succeeds, and `codex login status` shows the ChatGPT
  account signed in. If not, stop and tell the user; never run `codex login`
  for them.
- Do not set `CODEX_API_KEY` or `OPENAI_API_KEY`: they switch Codex from the
  subscription to metered API billing.

## Data use (§0.3)

Whether OpenAI trains on a ChatGPT account's Codex prompts follows that
account's data controls (on consumer plans, the "Improve the model for
everyone" setting). On a non-public repository, Codex gets the code only after
the user has said that setting is off, or that the account is a Business or
Enterprise one.

## Writing code: `codex exec`

Run in the checkout (or worktree, §6) that §4 recorded, with the brief from
[`task-brief.md`](task-brief.md) on stdin:

```sh
codex exec --sandbox workspace-write --cd <checkout> \
  -m <model> -c model_reasoning_effort=<effort> \
  -o <scratch>/codex-<task>.txt - < <scratch>/brief-<task>.txt
```

- `--sandbox workspace-write` lets it edit the checkout and run the gates,
  with network off by default, which also keeps it from pushing. Never pass
  `--sandbox danger-full-access` or `--dangerously-bypass-approvals-and-sandbox`.
- Leave `-m` and `-c model_reasoning_effort` out unless the user named a model
  or effort; Codex then uses the account's default. List what the account
  offers with `/model` in an interactive session, not from memory.
- `-o` keeps the final message (the brief asks for files changed, CHECK output
  and doubts) out of the scrollback. Progress goes to stderr; `--json` gives a
  JSON Lines event stream when a script needs one.
- Wrap the call in `timeout 20m` unless the user set another limit. A task
  that needs longer is two tasks.
- A follow-up on the same task (a re-brief, §5) can reuse the session with
  `codex exec resume --last`, so Codex keeps what it learned; add the
  `PREVIOUS ATTEMPT FAILED` section as the prompt.

Use Codex for the work its strength fits: real multi-file features and fixes,
changes that need the tests run in a loop, and Rust where it must satisfy
`clippy -D warnings`. Small mechanical edits are cheaper on OpenCode Go or
Poe, when the user has allowed those too.

## Reviewing locally: `codex review`

Read-only; it reports prioritized findings and does not touch the tree.

```sh
codex review --base <base>          # the branch against its base, PR-style
codex review --uncommitted          # staged, unstaged and untracked changes
codex review --commit <sha>         # one commit
codex review --base <base> "focus on unsafe Rust and error handling"
```

Triage the findings as §7 says. When Codex also wrote the change, add a
reviewer from another model family if the user allowed one.

## Reviewing on GitHub: `@codex review`

Only when the user asked for a Codex review on the PR. It needs the repository
connected to Codex and code review turned on at
<https://chatgpt.com/settings/code-review>; if no review appears, say so
rather than retrying. If the Codex connector answers with "create an
environment for this repo" instead, the repository has no Codex cloud
environment yet: tell the user to create one at
<https://chatgpt.com/codex/cloud/settings/environments>, and do not post
the comment again until they have.

- After the PR is open, post one comment: `@codex review` (optionally followed
  by what to focus on). Codex reacts with 👀 and posts a review.
- If the user turned on automatic reviews there, do not post the comment as
  well.
- Codex reads review rules from the repository's `AGENTS.md` (the current
  docs name a `## Code Review Rules` section), so repository-specific review
  rules belong there, not in the comment.
- Never post `@codex fix …` or any other `@codex <task>` comment unless the
  user asked for exactly that: those start a cloud task that can push to the
  branch, which this skill's delegates never do.
- Triage its findings like any other reviewer's, and fix them in the local
  checkout; the push is Claude's, as §8 says.

## Report fields

Provider `OpenAI`, harness `Codex` (or `Codex cloud` for a GitHub review),
model and effort as Codex printed them. Do not report a model you did not see.
