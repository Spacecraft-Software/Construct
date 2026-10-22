# Task brief template

Back to [`../SKILL.md`](../SKILL.md). Fill every section; an empty section is
a brief that is not ready to send. The delegate sees nothing but this text.

```text
GOAL
<one sentence: the observable result when this task is done>

CONTEXT
<what the code around it does, in two or three sentences; the decision
already taken that this task implements, so the delegate does not reopen it>

FILES YOU MAY EDIT
- <path>
- <path>  (create)

FILES YOU MUST NOT TOUCH
- everything not listed above, in particular <paths a delegate is likely to
  reach for: lockfiles, CI config, other modules, generated files>

RULES THAT APPLY
- <the repository rules this change touches, quoted or named precisely>
- match the surrounding code's style, naming and comment density

CHECK
<the exact command that proves the task, and what passing looks like>

STANDING ORDERS
- Do not stage, commit, push, create branches or open pull requests.
- Do not add dependencies, change lockfiles or edit CI.
- Do not disable, skip or weaken a test or a lint to make the check pass.
- Do not edit any file outside FILES YOU MAY EDIT.
- If the task cannot be done as written, stop and report why instead of
  guessing; partial work is fine if you say what is missing.
- End with: the files you changed, the CHECK output, and anything you were
  unsure of.
```

## Re-brief after a rejection

Send the same brief with one section added at the top:

```text
PREVIOUS ATTEMPT FAILED
<the failing command and the relevant lines of its output, or the specific
review finding; what must be different this time>
```

## Sizing

A brief whose FILES list grows past a handful, or whose CHECK needs more than
one command, is usually two tasks. Split it; the 10-minute ceiling on each
delegate run is a hard limit.
