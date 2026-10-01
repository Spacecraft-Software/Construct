---
name: tasks-todo-checklist
description: >
  Keeps a single, evidence-backed task checklist for the whole conversation and
  shows it at the top of every response while multi-step work is in progress.
  ALWAYS use when a request contains more than one deliverable, when work spans
  several turns, when implementing from a plan, PRD, TODO list, or issue, or
  when the user asks to track tasks, keep a checklist, or not stop until
  everything is done. A task is ticked only once its result has been checked
  and the evidence is named on the same line; a task that cannot be finished is
  marked blocked with the reason, never dropped or quietly reworded. The work
  is not complete until every item is ticked or blocked with a reason the user
  can act on. Sits beside the Standard §17 progress block and the closing
  TL;DR, never replacing either. Do NOT use for one-shot questions or
  single-step edits.
license: GPL-3.0-or-later
maintainer: Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
website: https://Construct.SpacecraftSoftware.org/
metadata:
  spdx: "SPDX-License-Identifier: GPL-3.0-or-later"
  author: "Mohamed Hammad & Spacecraft Software"
---

# Task Checklist

A checklist exists so that nothing the user asked for is lost between turns,
and so that "done" means *checked*, not *attempted*. Keep one checklist for the
whole conversation and treat it as the record of what was asked and what is
proven finished.

---

## §1 — When to keep a checklist

Keep one when any of these hold:

- the request names more than one deliverable, or one deliverable with
  several distinct steps;
- the work will take more than one turn;
- the work is driven by a plan, PRD, TODO list, issue, or review;
- the user asks for task tracking, a checklist, or for the work not to stop
  until everything is done.

Skip it for a one-shot question, a single small edit, or a reply that hands no
work back. A checklist on a one-line answer is noise.

---

## §2 — Building the list

1. **Start from the request.** Before doing any work, split the request into
   tasks — one line each, phrased as an outcome ("README lists the new skill"),
   not an activity ("look at README").
2. **Include the implicit tasks** the request depends on: tests to run, files
   that must change together, a context file or catalogue to update.
3. **Add tasks as you discover them.** New work found mid-way goes on the list
   with a note on where it came from. Never remove a task silently — if one
   turns out to be unnecessary, mark it `[-]` and say why.
4. **Keep the user's wording** for tasks they named. Do not narrow a task to
   make it easier to finish.

---

## §3 — Format

Show the checklist at the **top** of every response while work is ongoing:

```
[x] Task 1: <description> — <evidence>
[~] Task 2: <description> — in progress
[!] Task 3: <description> — blocked: <reason / what is needed from the user>
[-] Task 4: <description> — dropped: <why it is no longer needed>
[ ] Task 5: <description>
```

| Mark  | Meaning |
|-------|---------|
| `[ ]` | Not started. |
| `[~]` | Started, or done but not yet checked. |
| `[x]` | Done **and** checked, with the evidence named on the line. |
| `[!]` | Blocked. The line says what blocks it and what would unblock it. |
| `[-]` | Dropped. The line says why; only with the user's agreement for a task they named. |

One line per task. Keep unchanged items short; the list is a status board, not
a narrative.

---

## §4 — Rules

1. **`[x]` means checked, not attempted.** Mark a task done only after
   confirming it — a test run, a build, a diff, a command's output, or
   re-reading the result. Put the evidence on the same line (`cargo test: 42
   passed`, `src/lib.rs:118`, a PR link). Without evidence the task stays
   `[~]`.
2. **Never silently skip or shrink a task.** If a task cannot be finished, mark
   it `[!]` and say exactly what blocks it. Do not reword it into something
   easier to tick.
3. **Do not stop early.** The work is complete only when every task is `[x]`,
   `[-]` with a reason, or `[!]` with a reason the user can act on.
4. **Check before ending a turn.** Compare the list against the original
   request and anything added since. Name any gap explicitly rather than
   leaving it to be noticed.
5. **One list per conversation.** Update it in place; do not start a fresh
   list each turn or keep two lists that can disagree.

---

## §5 — Fitting with progress reporting and the TL;DR

On Spacecraft Software work, Standard §17 also applies. The pieces stack in a
fixed order and never replace one another:

1. **Checklist** — at the top of the response.
2. **Prose** — the work, findings, and detail.
3. **§17.1 progress block** — after the prose. Its figures must agree with the
   checklist: a track reads 100% only if every task in it is `[x]`.
4. **§17.4 TL;DR** — the last two lines of the turn. It may say the job is done
   only when the checklist has no `[ ]` or `[~]` items left; any `[!]` item
   makes the turn end in the question that would unblock it.

Outside Spacecraft Software work, keep the checklist and drop whatever
reporting format does not apply.

---

## §6 — Example

```
[x] Task 1: Parser accepts the new flag — src/cli.rs:57, cargo test: all passed
[x] Task 2: Help text documents the flag — `tool --help` output checked
[~] Task 3: Changelog entry — written, not yet re-read
[!] Task 4: Release tag — blocked: needs the maintainer's go-ahead to push
[ ] Task 5: Manual page updated
```

---

## §7 — Cross-References

| Need                                        | Skill to load                    |
|---------------------------------------------|----------------------------------|
| Progress block format and the closing TL;DR | `spacecraft-steelbore-standard`  |

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
