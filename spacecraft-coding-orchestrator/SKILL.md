---
name: spacecraft-coding-orchestrator
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
description: >
  Makes the session's own model (Claude, GPT, Kimi, any other) the planner
  and reviewer while other models write or review the code: OpenCode Go through the oc-* subagents, OpenAI Codex (codex exec,
  codex review, @codex review), Poe API models, Google Antigravity (agy), xAI
  Grok Build (grok), Qwen Code, or Qwen Coder.
  Use ONLY when the user explicitly asks to orchestrate or delegate coding, or
  names one of those backends or this skill; never on its own for ordinary
  edits, and only the backends the user named. The orchestrator plans, splits the work
  into bounded tasks, writes a brief for each, checks every diff and runs the
  repository's own gates, then runs the review chain and opens the pull
  request. Delegates never commit to or push the working branch. Private code
  goes only to backends whose data use the user has cleared. Also use for
  "orchestrator off", "no delegation" or "<family> only" (only the
  orchestrator's model family works for the session) and "orchestrator on".
---

# Spacecraft coding orchestrator

The **orchestrator** plans, delegates, checks and ships; other models write or
review the code. The orchestrator is whichever model is running this session
and reading this skill: Claude in Claude Code, a GPT model in Codex, Kimi in
its own CLI, or any other. Nothing below depends on which one it is. Where a
step names a Claude Code tool (`AskUserQuestion`, subagents), use the
harness's equivalent; where a backend works only from one harness (the `oc-*`
subagents need Claude Code), its reference says so. A backend that is the
orchestrator's own model through the same harness (Codex while GPT
orchestrates in Codex) is not a delegate: that work is the orchestrator's
own. This file is the procedure every backend shares; each backend's
commands, limits and quirks live in its own reference, read only when that
backend is in use:

| Backend | Reference | What it is good for |
|---------|-----------|---------------------|
| OpenCode Go (`oc-*` subagents) | [`references/backend-opencode-go.md`](references/backend-opencode-go.md) | Volume coding across many cheap models; the GLM review pair |
| OpenAI Codex (ChatGPT plan) | [`references/backend-codex.md`](references/backend-codex.md) | Main coder for real multi-file work; local and GitHub PR review |
| Poe API | [`references/backend-poe.md`](references/backend-poe.md) | One-shot calls to a named model (Mistral, Grok, …) for small, self-contained tasks or a second opinion |
| Google Antigravity (`agy`, IDE, Desktop) | [`references/backend-antigravity.md`](references/backend-antigravity.md) | Gemini models as a headless coder; the IDE or Desktop for work the user drives |
| xAI Grok Build (`grok`, SuperGrok plan) | [`references/backend-grok.md`](references/backend-grok.md) | Grok models as a headless coder on the subscription; a second-family reviewer |
| Qwen Code (Token Plan, Coding Plan) | [`references/backend-qwen.md`](references/backend-qwen.md) | Qwen models in a session the user drives (the plans forbid headless use); headless only on a pay-as-you-go key |
| Qwen Coder (cloud agent, Qwen Go plan) | [`references/backend-qwen-coder.md`](references/backend-qwen-coder.md) | A cloud task the user starts in the web app; it publishes a branch and PR that the orchestrator checks and re-applies |

This skill holds **no model list and no routing table**. Models change
monthly; read what each backend offers on the machine, every run.

```
plan ──> brief ──> delegate writes ──> orchestrator checks the diff + gates
  ^                                                    │
  └──────────── rejected: re-brief or fall back ───────┘
                          │ all tasks accepted
                          v
        review chain (§7) ──> commit ──> PR
```

## Asking the user

Every decision this skill needs from the user is asked the same way: one
multiple-choice question per decision, never an open-ended one. Put it to the
user with the harness's question tool (in Claude Code, `AskUserQuestion`; in a
thread or chat surface without one, its decision card, or else a numbered list
the user answers with one number). Each question:

- offers two or more **concrete options** the skill can act on as written,
  each with a line on what happens if it is picked;
- lists the **recommended option first** and marks it recommended, with the
  reason in a few words;
- leaves room for an answer of the user's own (the tool's free-text "other",
  or a closing "or tell me something else");
- carries only the context needed to choose.

Ask several independent decisions together, one question each, in a single
round when the tool allows it, rather than one after another. Ask each
decision once; do not re-ask what the user already answered in this request.
While waiting, do nothing the answer could change. Only when the session
cannot ask at all (headless, CI) take the recommended option and say so in the
plan or report.

## Turning orchestration off and on

The user can switch delegation off at any point in a session and back on
later. **Off means single-family mode: only the orchestrator's model family
works**, and the orchestrator is whichever model is running this session and
reading this skill. A family is one vendor's line of models: Claude (Opus,
Sonnet, Haiku, …), GPT, Kimi, Gemini, and so on. That is Claude in Claude
Code, but the switch is the same when the orchestrator is a GPT model in
Codex, Kimi in its own CLI, or any other model: off returns the session to
that family, never to a particular vendor, and not to the one model running
now. The switch is session state: it lasts until the user flips it again or
the session ends, and a new session starts with the skill unloaded, which is
the same as off.

**Off.** Any of "orchestrator off", "no delegation", "stop delegating", or
"<family> only" ("Claude only", "GPT only", "Kimi only", "only use Claude
models"), or the like, turns it off. From then on:

- no task, review or second opinion goes to a model of any other family, or
  to a service that runs one: none of this skill's backends whose models are
  outside the family (for a Claude orchestrator that is all of them: `oc-*`
  subagents, `codex`, Poe's non-Claude models, `agy`, `grok`, `qwen`, a Qwen
  Coder hand-off, `coderabbit`, an `@codex review` comment), and no CLI, API
  or agent that reaches another family, including Claude (`claude -p`) when
  Claude is not the orchestrator's family;
- the same holds for every other route to another family: another skill,
  plugin or subagent whose job is delegating to one (for example a Codex
  dispatch or OpenCode rescue plugin), even where its own description says to
  use it proactively;
- **models of the same family stay allowed** through the orchestrator's own
  harness: its subagents on any model of the family (in Claude Code, a Haiku
  subagent for a search or an Opus one for a hard task), and any switch
  between the family's models. The orchestrator still plans, checks every
  result as §4 says, runs the repository's gates (§0.4) and commits as §8
  says. A same-family model reached through a separate provider or API (a
  Claude model on Poe while Claude Code orchestrates) is a backend, not the
  harness: it stays off unless the user asks for it;
- the §0.3 data-use guard has nothing new to guard (no code leaves the
  orchestrator's own provider) and is not asked.

A user who names a family that is not the orchestrator's ("GPT only" while
Claude orchestrates) is asking for something else: ask (see *Asking the
user*) whether they mean **the orchestrator's own family** (recommended: that
is what off does), or to hand the whole job to that family, which turns
delegation on for that one backend.

If a delegate is running or a hand-off is open when the switch goes off, start
nothing new and ask once (see *Asking the user*): **wait for it, then check
its result as §4 says and use it** (recommended: the work is already spent),
stop it and discard whatever it produced, or stop it and keep the diff only
for the orchestrator to read as a draft. A Qwen Coder PR that is already open
is left as it is; say so.

Confirm the switch in one line that names the state and the family left
working, for example
`Orchestrator: off. Claude only until you say "orchestrator on".` (or `GPT
only`, `Kimi only`), and repeat the state at the head of each later progress
block (§9). Writing it into the conversation is what lets it survive a context
summary, so do not skip it.

**On.** "Orchestrator on", or a later request that again asks to orchestrate or
delegate, or names a backend, turns it back on. Start again at §0 as for a new
request: the backends are picked from that request, and an earlier choice is
not reused without asking. Confirm in one line (`Orchestrator: on.`).

Asking about delegation while it is off ("could Codex do this faster?") is a
question, not the switch: answer it and stay off.

## 0. Before the first task

1. **Pick the backends from the request, not from habit.** Use only the
   backends the user named in this request ("use Codex", "ask Poe's
   mistral-large-4", "let Antigravity do it", "give it to Grok", "Qwen").
   When they asked to orchestrate without naming one, **ask** (see *Asking
   the user*): one option per backend that is reachable
   (§0.2), **Codex marked recommended** and listed first, then OpenCode Go, a
   Poe model, Antigravity, Grok Build, Qwen Code and Qwen Coder (noting that
   both are hand-offs the user drives), and a mix where the work splits
   naturally (for example Codex writes, another model reviews). Each option says in a line what it
   is good for. When the tool takes fewer options than there are backends
   (`AskUserQuestion` takes four), keep Codex first and recommended, offer the
   ones that fit the task best, and name the rest in the question text so the
   user can still pick one through its free-text answer. Wait for the
   answer. Only when the session cannot ask (headless, CI) use Codex and say
   so in the plan. Never add a backend the user did not choose, and never
   move to another backend as a fallback without telling them.
2. **Check each named backend is reachable**, as its reference says (the
   `oc-*` agents are listed, `codex` / `agy` / `grok` / `qwen` are on `PATH`
   and signed in, `POE_API_KEY` is set, the user confirms Qwen Coder's GitHub
   app is installed). If one is not, stop and say which, rather than
   installing anything, signing in, or quietly writing everything yourself.
3. **Check the data-use guard.** Read the repository's visibility:
   `gh repo view --json visibility -q .visibility`. On anything but `PUBLIC`,
   including no answer, a backend may receive the repository's code only when
   the user has said its provider does not train on it, for this backend on
   this account. Until they have, ask once before the first delegation (for
   example: send it to this backend, use a backend already cleared, or keep
   the work local), and send nothing in the meantime. OpenCode Go delegates whose description
   marks them *Contributor* are never used on a non-public repository, whatever
   the answer.
4. **Read the repository's own rules** (`AGENTS.md`, `CLAUDE.md`,
   `CONTRIBUTING.md`) and note its gate commands: build, lint, format, test.
   Those are what every delegated diff must pass.
5. **Report the plan** as a Standard §17 progress block, one row per
   milestone of the plan, naming the backend each task goes to, before
   delegating anything.

## 1. What the orchestrator keeps, what it delegates

Delegate a task only when all of these hold:

- its **goal fits in one sentence** and its **file list is known**;
- **a command proves it** (a build, a test, a lint, an eval);
- it fits one delegate run (each backend's reference gives its time limit);
- writing the brief costs less than writing the code.

The orchestrator keeps, and writes itself:

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
brief, whatever the repository's visibility. A backend that cannot read the
tree (Poe) gets the file contents it needs inside the brief, so the §0.3 guard
applies to those contents too.

## 3. Choosing the delegate

Within the backends §0.1 allows, match the task to the model whose stated
purpose fits it: the `oc-*` agent descriptions, `codex` / `agy` model lists,
the `grok` / `qwen` defaults, or the Poe model the user named. Prefer the
cheap, high-allowance models for routine work and keep the expensive ones for
hard, multi-file or stuck work.
When the user named a specific model, use that model; if it is unavailable,
say so instead of substituting another.

## 4. Checking what comes back

A delegate's "done" is a claim, not a result. This applies to every backend
that writes into the tree (OpenCode Go, Codex, Antigravity, Grok Build, Qwen
Code, including a session the user drove) and to Qwen Coder, whose published
branch the orchestrator re-applies as a patch. Qwen Coder is the one delegate allowed to
push, and only a single new branch of its own (its reference says how that is
checked). A Poe reply is text: the orchestrator applies it (§4a) and then checks it the
same way.

**A delegate only ever writes into a clean tree.** Before each run,
`git status --porcelain --untracked-files=all` must print nothing in the
checkout it will use. If the user has uncommitted work, never stash, move or
restore it: run the delegate in a separate worktree off `HEAD` (§6) instead,
and bring the accepted result back as a commit or patch. A clean starting point is what
makes the rollback below safe, since nothing in that tree belongs to anyone
but the delegate. Record where the run starts, too:
`branch=$(git symbolic-ref -q --short HEAD || echo DETACHED)`,
`base=$(git rev-parse HEAD)` and
`remote=$(git ls-remote --heads origin)`, so a delegate that commits,
switches branches or pushes against orders cannot hide it.

Git does not report ignored files (build output, caches, a local `.env`), so a
clean status says nothing about them. The brief forbids touching them; where
the checkout holds ignored files that matter, such as local secrets or
configuration, run the delegate in a fresh worktree off `HEAD`, which has none.

**After every delegate run:**

1. If `git ls-remote --heads origin` no longer matches `$remote` and the user
   did not push in the meantime, the delegate pushed (Qwen Coder's one new
   branch excepted, as its reference says): stop and tell the user,
   and change nothing on the remote yourself.
   If the branch (or detached state) is no longer `$branch`, the delegate
   switched branches: reset nothing, since a reset would now move a branch the
   task never owned. Stop, reject the run, and tell the user what changed.
   Otherwise, if `git rev-parse HEAD` is no longer `$base`, the delegate
   moved `HEAD` against orders. Only when `$base` is an ancestor of the new
   `HEAD` (`git merge-base --is-ancestor "$base" HEAD`), that is, the delegate
   added commits on top of it, undo them but keep their changes with
   `git reset --soft "$base"`, check them below like any other run, and name
   the breach in the report. Any other movement (a reset, a checkout of
   another commit while detached) gets no reset: stop, reject the run and
   tell the user.
2. `git status --porcelain --untracked-files=all` and
   `git diff "$base" --stat` (against the recorded commit, so staged changes
   are seen too). Any path outside the brief's file list is a rejection:
   revert it with `git restore --staged --worktree --source="$base" -- <path>`,
   or remove it if untracked. In a tree that was clean before the run, every
   change is the delegate's, so this cannot touch the user's work.
3. Read the whole diff (`git diff "$base"`, plus every new untracked file).
   Reject on a wrong approach, a silenced test, a disabled lint, a hard-coded
   value the repository's rules forbid, or code the brief did not ask for.
4. Run the brief's check command **and** the repository's gates from step 0.4.
   A delegate saying it ran them is not evidence. A check that writes files
   (a formatter, a code generator, a lockfile refresh) changes what you
   reviewed: if status or the diff moved, repeat steps 2 to 4 on the result,
   until a pass leaves both unchanged, before accepting.
5. Accept, re-brief, or fall back (§5). Record which backend, model and
   effort wrote what; the final report names them.
6. On acceptance, the orchestrator commits the task on the working branch before the
   next delegate starts, so the next run begins from a clean tree again.
   These commits may be squashed before the PR if the repository prefers.

### 4a. Text-only replies (Poe)

A model with no access to the tree answers with text. Ask it for a unified
diff against the files the brief quoted, or for whole replacement files, never
for prose instructions. The orchestrator applies the reply with `git apply --check` then
`git apply` (or writes the files), touching only paths in the brief's list,
and the reply then goes through steps 2 to 6 above like any other run. A diff
that does not apply cleanly is a failed attempt (§5), not something to repair
by hand into a different change.

## 5. Retries and fallbacks

- **Rate limit, sign-in, quota or unknown-model error:** report the error
  text. Within the backend the user named, move to the next model whose stated
  purpose fits; crossing to another backend needs the user's word (§0.1),
  asked as a choice between the reachable backends, waiting, and the
  orchestrator writing it.
- **Wrong or failing result:** re-brief once with the concrete failure (the
  failing command and its output), to the same model or a stronger one.
- **Two failed attempts on one task:** the orchestrator either writes it itself or hands
  it to a model meant for stuck debugging, for a diagnosis only, then writes
  the fix; when the right move is not clear, offer those as options. Never loop a third time on the same brief.
- **A backend's binary or key is missing:** stop and tell the user, rather
  than installing anything; offer the choices left (another reachable
  backend, waiting while they fix it, or the orchestrator writing the task).

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

Run once, on the finished change. The reviewers are the ones the user asked
for, else the ones paired with the backend they chose in §0.1 (Codex:
`codex review`; OpenCode Go: its chain below); with a mix, a reviewer from a
different model family than the writer.

- **OpenCode Go:** `oc-glm-review` on every change, plus
  `oc-glm-security` when the diff is security-relevant (authentication,
  cryptography, PAM, sudo or setuid, permissions, systemd units, secrets,
  network services, downloads or fetchers, sandbox settings, `unsafe` Rust;
  when in doubt, run it), then `coderabbit review --agent --base <base>` once.
  Details in the OpenCode Go reference.
- **Codex as reviewer:** `codex review --base <base>` locally, before the PR,
  and, when asked, an `@codex review` comment on the PR once it is open. See
  the Codex reference.
- **Another named model** (a Poe, Antigravity, Grok or Qwen model): the same
  read-only review prompt the reference gives, on `git diff <base>`.

For every reviewer:

1. Triage each finding: fix it (the orchestrator, or a re-brief), or note why it does
   not apply. A review model is a second opinion, not the gate.
2. A model that wrote a task does not review it alone; add a reviewer from
   another model family when one is available.
3. Re-run the repository's gates after the last fix.

Rate-limited or unavailable review services (CodeRabbit's free allowance, a
Codex quota) are reported and skipped, never retried in a loop.

## 8. Commit and pull request

The orchestrator commits, following the repository's rules (message convention,
signing, timestamps). The delegates' work is attributed in the PR body, not in
`Co-Authored-By` trailers: per task, the model, its reasoning effort where it
has one, the provider that served it and the harness that drove it
(Standard §15.4 records these separately, because the same model through a
different provider or harness is a different result). Never estimate token
counts or points spent; report them only where a tool printed them. Open the
PR as the repository's rules say; a review bot that needs a non-draft PR or a
trigger comment is covered in its backend's reference. Merging stays the
maintainer's call unless they have said otherwise.

## 9. Reporting

- At the start, after each accepted milestone, and at the end: the §17
  progress block for the plan, headed by the switch state once the user has
  used it (`Orchestrator: off` / `on`).
- At the end, one line per task: what it did, which backend and model wrote
  it, how many attempts, and the check that proved it. Then the review
  findings and what happened to each, any quota or points readings the tools
  printed, and the PR link.
- Close with the §17.4 TL;DR.
