---
name: spacecraft-coding-orchestrator
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
description: >
  Makes Claude Code the planner and reviewer while OpenCode Go models write the
  code through the oc-* delegate subagents. Use ONLY when the user explicitly
  asks to orchestrate, to delegate coding, or to use OpenCode Go or the oc-*
  agents, or invokes this skill by name; never on its own for ordinary edits.
  Claude plans, splits the work into bounded tasks, writes a brief for each,
  picks the delegate from its agent description, checks every diff and runs the
  repository's own gates, then runs the review chain (oc-glm-review,
  oc-glm-security when the change is security-relevant, the CodeRabbit CLI
  once) and opens the pull request. Delegates never commit or push. Contributor
  models are used only on public repositories.
---

# Spacecraft coding orchestrator

Claude Code plans, delegates, checks and ships. OpenCode Go models write the
code. This skill is the procedure; it holds **no model list and no routing
table**. The delegates are Claude Code subagents named `oc-<slug>`, each a thin
forwarder around one `opencode run -m opencode-go/<id>` call, and each one's
description says what it is for. On a Steelbore OS host those files are
rendered by Home Manager from one registry (Bravais `lib/opencode-go.nix`), so
a model added or swapped there reaches this skill with no edit here.

```
plan ──> brief ──> oc-<model> writes ──> Claude checks the diff + gates
  ^                                              │
  └──────────── rejected: re-brief or fall back ─┘
                          │ all tasks accepted
                          v
oc-glm-review (+ oc-glm-security) ──> coderabbit review ──> commit ──> PR
```

## 0. Before the first task

1. **List the delegates.** Read the `oc-*` agents available in this session
   and their descriptions. If there are none, stop and say so: this skill
   needs them, and Claude does not fall back to writing everything itself
   without telling the user.
2. **Check the repository's visibility** when any delegate is marked
   *Contributor* in its description (its provider trains on the prompts):
   `gh repo view --json visibility -q .visibility`. Anything but `PUBLIC`,
   including no answer, means **no Contributor delegate for this whole run**.
3. **Read the repository's own rules** (`AGENTS.md`, `CLAUDE.md`,
   `CONTRIBUTING.md`) and note its gate commands: build, lint, format, test.
   Those are what every delegated diff must pass.
4. **Report the plan** as a Standard §17 progress block, one row per
   milestone of the plan, before delegating anything.

## 1. What Claude keeps, what it delegates

Delegate a task only when all of these hold:

- its **goal fits in one sentence** and its **file list is known**;
- **a command proves it** (a build, a test, a lint, an eval);
- it fits one delegate run (each `oc-*` call has a 10-minute ceiling);
- writing the brief costs less than writing the code.

Claude keeps, and writes itself:

- design decisions, interfaces between tasks and anything the plan is unsure of;
- one-line and few-line fixes, where the brief would be longer than the change;
- **security-sensitive code**: authentication, PAM, sudo or privilege, secrets
  handling, cryptography, sandbox or permission boundaries;
- the final integration of the delegated pieces, and every commit.

## 2. The brief

Every delegation carries the full brief in
[`references/task-brief.md`](references/task-brief.md): goal, the files it may
touch, the files it must not touch, the repository rules that apply, the check
command, and the standing orders (no commits, no pushes, no new dependencies,
no edits outside the list, stop and report rather than guess). A delegate sees
nothing of this conversation, so the brief is all it knows.

Never put a secret, a token, a private URL or a private file's content in a
brief, whatever the repository's visibility.

## 3. Choosing the delegate

Match the task to the agent whose **description** fits it, reading the
descriptions each run rather than remembering them. Prefer the delegates whose
descriptions call them high-volume or default for routine work, and keep the
ones described as for hard, multi-file or stuck work for exactly that, since
the cheaper allowances are the larger ones. Never pick a Contributor delegate
when step 0.2 ruled them out.

## 4. Checking what comes back

A delegate's "done" is a claim, not a result.

**A delegate only ever writes into a clean tree.** Before each run,
`git status --porcelain --untracked-files=all` must print nothing in the
checkout it will use. If the user has uncommitted work, never stash, move or
restore it: run the delegate in a separate worktree off `HEAD` (§6) instead,
and bring the accepted result back as a commit or patch. A clean starting point is what
makes the rollback below safe, since nothing in that tree belongs to anyone
but the delegate. Record where the run starts, too:
`branch=$(git symbolic-ref -q --short HEAD || echo DETACHED)` and
`base=$(git rev-parse HEAD)`, so a delegate that commits or switches branches
against orders cannot hide its work behind a moved `HEAD`.

Git does not report ignored files (build output, caches, a local `.env`), so a
clean status says nothing about them. The brief forbids touching them; where
the checkout holds ignored files that matter, such as local secrets or
configuration, run the delegate in a fresh worktree off `HEAD`, which has none.

**After every delegate run:**

1. If the branch (or detached state) is no longer `$branch`, the delegate
   switched branches: reset nothing, since a reset would now move a branch the
   task never owned. Stop, reject the run, and tell the user what changed.
   Otherwise, if `git rev-parse HEAD` is no longer `$base`, the delegate
   committed against orders. Undo the commit but keep its changes with
   `git reset --soft "$base"`, check them below like any other run, and name
   the breach in the report; if it also pushed, stop and tell the user.
2. `git status --porcelain --untracked-files=all` and
   `git diff "$base" --stat` (against the recorded commit, so staged changes
   are seen too). Any path outside the brief's file list is a rejection:
   revert it with `git restore --staged --worktree --source="$base" -- <path>`,
   or remove it if untracked. In a tree that was clean before the run, every
   change is the delegate's, so this cannot touch the user's work.
3. Read the whole diff (`git diff "$base"`, plus every new untracked file).
   Reject on a wrong approach, a silenced test, a disabled lint, a hard-coded
   value the repository's rules forbid, or code the brief did not ask for.
4. Run the brief's check command **and** the repository's gates from step 0.3.
   A delegate saying it ran them is not evidence. A check that writes files
   (a formatter, a code generator, a lockfile refresh) changes what you
   reviewed: if status or the diff moved, repeat steps 2 and 3 on the result
   before accepting.
5. Accept, re-brief, or fall back (§5). Record which delegate wrote what; the
   final report names it.
6. On acceptance, Claude commits the task on the working branch before the
   next delegate starts, so the next run begins from a clean tree again.
   These commits may be squashed before the PR if the repository prefers.

## 5. Retries and fallbacks

- **Rate limit, sign-in or unknown-model error:** the forwarder returns the
  error text. Move to the next delegate whose description fits the same task
  (descriptions name the overflow or fallback model where one exists).
- **Wrong or failing result:** re-brief once with the concrete failure (the
  failing command and its output), to the same delegate or a stronger one.
- **Two failed attempts on one task:** Claude either writes it itself or hands
  it to the delegate described as for stuck debugging, for a diagnosis only,
  then writes the fix. Never loop a third time on the same brief.
- **`opencode` not found:** the forwarders call `opencode` from
  `~/.opencode/bin`; if that is missing, stop and tell the user, rather than
  installing anything.

## 6. Parallel delegates

Two delegates never share a checkout at the same time: each one's rollback
would see the other's files as out of scope and undo them. To run delegates
concurrently, give each its own clean worktree off `HEAD`
(`git worktree add ../<repo>-<task> -b <task-branch>`), check and accept each
result there, then merge the accepted branches yourself, one at a time,
re-running the gates after each merge. Only tasks whose file lists do not
overlap go in parallel; the rest run one after another. Remove each worktree
(`git worktree remove`) once its branch is merged or dropped.

## 7. The review chain

Run once, on the finished change, in this order:

1. **`oc-glm-review`** on every change (it diffs against `origin/main`; pass
   another base ref when the branch targets something else).
2. **`oc-glm-security`** as well when the diff is security-relevant,
   including authentication, cryptography, PAM, sudo or setuid, permissions,
   systemd units, secrets, network services, downloads or fetchers, sandbox
   settings, or `unsafe` Rust. When in doubt, run it.
3. Triage each finding: fix it (Claude, or a re-brief), or note why it does
   not apply. A review model is a second opinion, not the gate.
4. **`coderabbit review --agent --base <base>`** once, after the GLM findings
   are dealt with. The free and open-source allowance is a few reviews an
   hour, so never run it per task. If it is unavailable or rate-limited, say
   so and continue.
5. Re-run the repository's gates after the last fix.

Do not use the OpenCode Claude Code plugin's `/opencode:*` commands or its
stop-time review gate unless `/opencode:setup` reports ready; on a mismatched
OpenCode version the gate blocks every stop.

## 8. Commit and pull request

Claude commits, following the repository's rules (message convention,
signing, timestamps). The delegates' work is attributed in the PR body, not in
`Co-Authored-By` trailers. Open the PR as the repository's rules say; when
CodeRabbit's GitHub app is installed and the repository needs a manual
trigger, the PR must not be a draft and gets an `@coderabbitai review`
comment. Merging stays the maintainer's call unless they have said otherwise.

## 9. Reporting

- At the start, after each accepted milestone, and at the end: the §17
  progress block for the plan.
- At the end, one line per task: what it did, which delegate wrote it, how
  many attempts, and the check that proved it. Then the review findings and
  what happened to each, and the PR link.
- Close with the §17.4 TL;DR.
