# Backend: Qwen Coder (cloud agent, Qwen Go plan)

Back to [`../SKILL.md`](../SKILL.md). Used only when the user names Qwen Coder
or `coder.qwen.ai` for this request. Qwen Coder is the cloud coding agent that
comes with the Qwen consumer plans (Go and up, at chat.qwen.ai). It works in a
cloud sandbox on a GitHub repository it reaches through its GitHub app, and
publishes its work as a branch and a pull request.

It has no API and no CLI, so the orchestrator cannot start or steer it. It is a
**hand-off**: the orchestrator writes the brief, the user runs it in the web app, and
the orchestrator checks what comes back. It is not Qwen Code with a Token Plan key; that
is [`backend-qwen.md`](backend-qwen.md).

## Reachability (§0.2)

The orchestrator cannot check it from the shell. Ask the user to confirm, unless they
already said so in this request, that the Qwen GitHub app is **installed** on
the repository's account or organization (authorizing it is not enough) and
that the repository shows up in Qwen Coder.

## Data use (§0.3)

Qwen Coder clones the repository into Alibaba's cloud, and the plan's terms
govern what happens to it there. On a non-public repository, it gets the code
only after the user has cleared Qwen for this account.

## Pushing: the one exception

Qwen Coder pushes: publishing creates a branch and opens a pull request. That
is the only way its work comes back, so this backend relaxes the §4 rule that a
delegate never pushes, and only this far:

- It may create **one new branch** and open **one pull request** from it. The
  brief names the branch (`qwen-coder/<task>`) and the base.
- It must not push to any other branch. After the user says it is done, compare
  `git ls-remote --heads origin` with `$remote`: the new branch is the only
  difference allowed. Any change to an existing branch, the default branch
  included, is a breach: stop and tell the user.
- Its pull request is never the one that ships. The orchestrator never merges it, and
  the orchestrator's own commits and PR (§8) carry the accepted work.

Never give it a GitHub token or a deploy key, and never suggest it, even though
that is a known workaround for its publishing failures: a delegate holding push
credentials is what this skill exists to prevent.

## Hand-off

1. Prepare the clean checkout or worktree exactly as §4 says, and record
   `$branch`, `$base` and `$remote`.
2. Write the brief from [`task-brief.md`](task-brief.md) to a file outside the
   tree (the scratch directory). Its first standing order forbids the very
   steps that bring this work back, so replace that line (and only that line)
   with these two:
   `Commit only on a new branch named qwen-coder/<task>, created from <base
   branch>; do not reset, rewrite or switch to any other branch.` and
   `Push only that branch, and open one pull request from it. Push nothing
   else.` Give the user its path, the repository and the base branch to pick
   in Qwen Coder.
3. Wait for the user to say it published, with the PR's link or branch name.
4. Check the remote as *Pushing* says, then bring the work in without taking
   its commits:

   ```sh
   git fetch origin qwen-coder/<task>
   git diff "$base" FETCH_HEAD --stat
   git diff --binary "$base" FETCH_HEAD > <scratch>/qwen-coder-<task>.patch
   git apply --check <scratch>/qwen-coder-<task>.patch \
     && git apply <scratch>/qwen-coder-<task>.patch
   ```

   If its branch is not based on `$base`, diff against its merge base with
   `$base` instead and say so. A patch that does not apply cleanly is a failed
   attempt (§5).
5. Run §4 steps 2 to 6 on the result, as for any other delegate: the result is
   checked, not trusted, and the orchestrator commits it.
6. Once the work is accepted or dropped, close Qwen Coder's PR with a one-line
   comment linking the orchestrator's PR (or saying why it was dropped). Delete its
   branch only when the user says so.

## Errors

- **"The branch update was rejected by the remote server"** on publishing:
  users report that a failed run can leave its branch behind, and later
  publishes then fail against it. Tell the user; deleting the stale branch, or
  publishing from a fresh chat, is their call.
- **Nothing published, or a PR against the wrong base:** a failed attempt
  (§5). Re-brief once with what went wrong.

## Report fields

Provider `Alibaba (Qwen)`, harness `Qwen Coder (cloud, driven by the user)`,
model as the web app showed it, plus the link to its pull request.
