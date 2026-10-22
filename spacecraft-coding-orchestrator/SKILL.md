---
name: spacecraft-coding-orchestrator
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
description: >
  Runs a coding change as an orchestrator: Claude plans, splits the work into
  bounded tasks, delegates the writing to the oc-* subagents (OpenCode Go
  models), verifies every diff itself, runs the oc-glm-review /
  oc-glm-security / CodeRabbit review chain, then commits signed and opens a
  draft PR. Use ONLY when the user asks for it — "orchestrate", "delegate
  this", "use OpenCode Go", "have the oc agents do it", or this skill by
  name. Do NOT fire for ordinary coding requests, one-line fixes or questions:
  delegating a three-line edit costs more than writing it. Routing comes from
  each oc-* agent's own description, never from a model list here. Enforces
  the hard rules: delegates never commit or push; Contributor models (their
  providers train on prompts) only on public repos; no two delegates on
  overlapping files at once; never trust a delegate's "done" without the diff
  and the repo's own checks.
---

# Spacecraft Coding Orchestrator

**Maintainer:** Mohamed Hammad | **Contact:** [Mohamed.Hammad@SpacecraftSoftware.org](mailto:Mohamed.Hammad@SpacecraftSoftware.org)
**Copyright:** (C) 2026 Mohamed Hammad & Spacecraft Software | **License:** GPL-3.0-or-later
**Website:** [https://Construct.SpacecraftSoftware.org/](https://Construct.SpacecraftSoftware.org/)

You are the orchestrator. You own the plan, every decision about what lands,
the verification, the commit and the PR. The delegates own only the typing.
A delegate's output is a proposal; nothing it says is true until you have
read the diff and run the checks.

This skill is the workflow. It does not replace `spacecraft-steelbore-standard`
or the language skills — load those as usual; their rules bind the delegates'
code exactly as they bind yours, and checking that is your job.

## Step 0 — Preconditions

1. **Find the delegates.** List the subagent types available in this session
   whose names start with `oc-`. Each one's description says what it is for
   and, where it applies, its fallback and its privacy class. That description
   is the routing table — this skill names no models on purpose, so adding or
   swapping a model never makes it stale.
   - No `oc-*` agents available → say so in one line and ask whether to do
     the work directly. Do not improvise a delegate through raw shell calls.
2. **Repo visibility.** Run `gh repo view --json visibility -q .visibility`
   (or read the remote). Record `PUBLIC` or not. Unknown counts as private.
3. **Repo rules.** Read the repo's `AGENTS.md`: its build/test/lint commands
   become the verification step, and its invariants go into every brief.
4. **Branch.** Work on a feature branch, never the default branch.

## Step 1 — Plan and split

Write the plan before delegating anything. Break the change into tasks that
each satisfy all of:

- **Bounded:** a known, short list of files the task may touch.
- **Checkable:** a command or observation that proves it worked.
- **Self-contained:** a delegate can do it from the brief alone, without the
  conversation.

Keep for yourself (do not delegate):

- design decisions, interfaces, and anything the plan still leaves open;
- fixes of a few lines, where the brief would be longer than the code;
- code where the writing itself is security-sensitive — authentication, PAM,
  sudo/polkit, crypto, secret handling, permission or sandbox settings;
- cross-cutting edits where the file list is not knowable in advance.

When the task is driven by `PRD.md` / `PLAN.md` / `TODO.md`, map each
delegated task to its item ID and report progress per Standard §17.

## Step 2 — Choose the delegate

Pick the `oc-*` agent whose description best fits each task. Then apply the
hard rules, which override fit:

- **Privacy (hard rule).** An agent whose description says its provider
  trains on prompts (a *Contributor* model) may be used **only** when Step 0
  found the repo `PUBLIC`. Never send it private code, secrets, credentials,
  internal hostnames or personal data, even in a public repo.
- **Budget.** Prefer the plentiful models for volume; keep the scarce ones
  (the descriptions and the host's registry say which) for hard work and
  review. Do not burn a scarce model on a mechanical edit.
- **Reviewers are not coders.** `oc-*-review` / `oc-*-security` agents are
  read-only reviewers; never hand them a coding task.

## Step 3 — Write the brief

Every delegation uses this template, filled in completely. The delegate sees
nothing else.

```text
Goal: <one or two sentences: what must be true when you are done>
Repository: <path>, branch <branch>. Read AGENTS.md first.
Files you may change: <exact paths; create only those listed>
Do not touch: <everything else, plus any file you know is fragile>
Constraints: <the rules from AGENTS.md / the Standard that apply here:
  license headers, formatting tool, naming, forbidden patterns>
Verify with: <exact command(s) and the expected result>
Do not commit, push, open PRs, run sudo, or change git config.
Finish by listing the files you changed and the verify output.
```

## Step 4 — Run delegates

- **One writer per file set.** All delegates write into the same checkout.
  Two may run at once only when their file sets are disjoint; otherwise run
  them one after another, or give each its own `git worktree`. This is
  Standard §3's concurrency rule applied to agents: parallel where it helps,
  serial where it would corrupt.
- **Rate limit or model error** → retry once on the fallback the agent's
  description names (or the nearest equivalent agent), then go on.
- **Two failed attempts on one task** → stop delegating it. Either do it
  yourself, or hand the failure (brief + diff + error) to the agent whose
  description covers stuck debugging for a fresh diagnosis. Never loop.

## Step 5 — Verify (every task, no exceptions)

1. `git status --short` and `git diff` — read the whole diff.
2. **Reject** any change outside the brief's file list, any new dependency
   you did not ask for, any weakened check, test or lint rule, and anything
   that breaks a rule in `AGENTS.md`. Revert the stray parts
   (`git restore <path>`) or re-delegate with a sharper brief.
3. Run the repo's own checks yourself (the brief's `Verify with:` plus the
   `AGENTS.md` gates). A delegate's claimed output is not evidence.
4. Only then mark the task done.

## Step 6 — Review chain

Run once the whole change is finished, in this order:

1. **General review** — the read-only `oc-*-review` agent on the branch diff
   (pass the base ref when it is not `origin/main`). Every change.
2. **Security review** — the read-only `oc-*-security` agent, when the diff
   touches PAM, sudo/setuid/polkit, systemd units, secrets, network services
   or firewall rules, downloads/fetchers, shell scripts or `unsafe` code.
   It is a second opinion beside Claude Code's `/security-review`, not the
   gate.
3. **CodeRabbit CLI** — `coderabbit review --agent --base <default-branch>`,
   **once**, at the end: its allowance is a few reviews an hour.

Treat every finding as a claim to check, not an order. Fix or re-delegate
the real ones, note why you dismissed the others, and re-run Step 5 after
fixes. Do not re-run the whole chain for a small follow-up fix; re-run the
general review only if the fix was substantial.

## Step 7 — Land

- Commit yourself, signed (Standard §6.3), with a conventional message.
  Delegates never commit. Stage files by name.
- Push over HTTPS (Standard §6.4) to the project's own namespace only.
- Open a **draft** PR. Where the repo uses the CodeRabbit GitHub app and it
  does not review automatically, comment `@coderabbitai review`.
- Merge only if the user asked for it and the repo's rules allow it.

## Report

End with:

- the §17 progress block for the plan;
- a table of tasks → delegate used → attempts → result;
- review findings: fixed / dismissed (with reason);
- what the user must still do (rebuild, sign-in, merge).

## Anti-patterns

- ❌ Delegating before writing the plan.
- ❌ A brief without an exact file list or a verify command.
- ❌ Accepting a delegate's "done", "tests pass" or summary without reading
  the diff and running the checks.
- ❌ Any Contributor model on a private or unknown-visibility repo.
- ❌ Two delegates writing overlapping files at the same time.
- ❌ Letting a delegate commit, push, change git config or run `sudo`.
- ❌ Running CodeRabbit after every edit.
- ❌ Retrying the same failing task on model after model.
- ❌ Copying a model list or routing table into this skill or a context file.
