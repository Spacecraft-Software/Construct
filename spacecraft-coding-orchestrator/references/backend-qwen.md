# Backend: Qwen Code (QwenCloud Token Plan, Alibaba Coding Plan)

Back to [`../SKILL.md`](../SKILL.md). Used only when the user names Qwen,
Qwen Code or a Qwen model for this request. Qwen Code is Alibaba's coding
agent CLI, `qwen`: it reads the tree, edits files and runs commands. Its free
Qwen OAuth tier ended on 2026-04-15, so it runs on one of these keys:

| Plan | Key (prefix) | Environment variable Qwen Code reads | Non-interactive use |
|------|--------------|--------------------------------------|---------------------|
| QwenCloud Token Plan (Lite, Essential, Standard, Pro) | `sk-sp-` | `QWENCLOUD_TOKEN_PLAN_API_KEY` | **Not allowed** |
| Alibaba Model Studio Coding Plan | `sk-sp-` | `BAILIAN_CODING_PLAN_API_KEY` | **Not allowed** |
| A pay-as-you-go API key (QwenCloud or Model Studio) | `sk-` / `sk-ws-` | `QWENCLOUD_API_KEY`, or the provider set in `~/.qwen/settings.json` | Allowed, billed per token |

Both subscription plans state that their keys are for interactive use inside
coding tools, never for scripts, backends or non-interactive calls, and that
breaking this can suspend the subscription or revoke the key. A headless
`qwen -p` that Claude starts is a non-interactive call. So:

- **On a subscription key, Claude never runs `qwen` itself.** The user runs
  Qwen Code, and Claude briefs and checks (*Hand-off*, below).
- **Headless runs only on a pay-as-you-go key,** and only after the user has
  said that this key may be billed for the request.

"Qwen Go!", the personal agent in the Qwen app, is not a coding plan and has
no programmatic access; it is not a backend.

## Reachability (§0.2)

- `qwen --version` succeeds. Ask the user which plan their Qwen Code is signed
  in with, unless they already said in this request; never read
  `~/.qwen/settings.json` or the environment for a key, and never print,
  store or pass one in argv.
- If no key is set up, stop and ask the user to run `/auth` in Qwen Code
  themselves.

## Data use (§0.3)

The plans run in Alibaba Cloud's regions (the Token Plan in Singapore), so
prompts and outputs leave the user's country, and the plans' pages do not say
whether they are used for training. On a non-public repository, Qwen gets the
code only after the user has cleared it for this account.

## Hand-off (subscription keys)

1. Prepare the clean checkout or worktree exactly as §4 says, and record
   `$branch`, `$base` and `$remote`.
2. Write the brief from [`task-brief.md`](task-brief.md) to a file outside the
   tree (the scratch directory), and give the user its path, the checkout to
   run in, and the line to start with, for example
   `cd <checkout> && qwen --approval-mode auto-edit`, then paste the brief.
   Ask them not to approve any `git` or `gh` command it proposes.
3. Wait for the user to say Qwen Code is done. Then run every §4 step, as for
   any other delegate: the result is checked, not trusted.

For a review, give the user the review instruction from the Poe reference and
the output of `git diff <base>` the same way, and triage what they paste back
as §7 says.

## Writing code headless: `qwen -p` (pay-as-you-go key only)

Run from the checkout (or worktree, §6) that §4 recorded:

```sh
cd <checkout> && qwen -p "$(cat <scratch>/brief-<task>.txt)" \
  --approval-mode auto-edit --sandbox \
  --exclude-tools agent --max-wall-time 20m \
  --output-format json > <scratch>/qwen-<task>.json
jq -r '.[] | select(.type == "result") | .is_error, .result' \
  <scratch>/qwen-<task>.json
```

- `auto-edit` approves file edits but not shell commands, so the delegate
  cannot run the CHECK itself; Claude runs it in §4, as for Poe. Never pass
  `--yolo` or `--approval-mode yolo`: it runs every shell command, `git`
  included, with the host's privileges.
- `--sandbox` runs shell, write and edit tools in the sandbox image; where no
  sandbox runtime is available, say so rather than dropping the flag.
- Leave `-m` out unless the user named a model.
- `is_error: true`, exit `53` (turn limit) or `55` (budget) is a failed run.
- A task that needs more than `--max-wall-time` is two tasks.

## Report fields

Provider `Alibaba Cloud` (`QwenCloud` or `Model Studio`, as the plan says),
harness `Qwen Code`, and for a hand-off `Qwen Code (driven by the user)`;
model as the run reported it.
