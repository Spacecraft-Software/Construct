# Backend: OpenCode Go (`oc-*` subagents)

Back to [`../SKILL.md`](../SKILL.md). Used when the user names it, or picks it
from the §0.1 question (Codex is the recommended default there).

## How it is wired

The delegates are Claude Code subagents named `oc-<slug>`, so this backend
is reachable only when the orchestrator runs in Claude Code. Each is a thin
forwarder around one `opencode run -m opencode-go/<id>` call, and each one's
description says what it is for. On a Steelbore OS host those files are
rendered by Home Manager from one registry (Bravais `lib/opencode-go.nix`), so
a model added or swapped there reaches this skill with no edit here. Read the
descriptions each run rather than remembering them.

## Reachability (§0.2)

- List the `oc-*` agents available in this session. If there are none, stop
  and say so.
- The forwarders call `opencode` from `~/.opencode/bin`. If that is missing,
  stop and tell the user rather than installing anything.

## Data use (§0.3)

A delegate whose description marks it *Contributor* runs on a model whose
provider trains on the prompts. On any repository that is not `PUBLIC`
(including no answer from `gh`), no Contributor delegate is used for the whole
run.

## Choosing

Prefer the delegates described as high-volume or default for routine work,
and keep the ones described as for hard, multi-file or stuck work for exactly
that, since the cheaper allowances are the larger ones. A rate-limited
delegate's description usually names its overflow model.

## Limits

Each `oc-*` call has a **10-minute ceiling**. A task that cannot finish in one
run is two tasks.

## Review chain (default, §7)

1. **`oc-glm-review`** on every change (it diffs against `origin/main`; pass
   another base ref when the branch targets something else).
2. **`oc-glm-security`** as well when the diff is security-relevant.
3. **`coderabbit review --agent --base <base>`** once, after the GLM findings
   are dealt with. The free and open-source allowance is a few reviews an
   hour, so never run it per task. If it is unavailable or rate-limited, say
   so and continue.

When CodeRabbit's GitHub app is installed and the repository needs a manual
trigger, the PR must not be a draft and gets an `@coderabbitai review`
comment.

Do not use the OpenCode Claude Code plugin's `/opencode:*` commands or its
stop-time review gate unless `/opencode:setup` reports ready; on a mismatched
OpenCode version the gate blocks every stop.

## Report fields

Provider `OpenCode Go`, harness `OpenCode`, model the `opencode-go/<id>` the
forwarder names.
