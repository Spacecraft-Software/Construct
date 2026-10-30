# Backend: Poe API

Back to [`../SKILL.md`](../SKILL.md). Used only when the user names Poe or a
model to reach through it ("ask mistral-large-4 on Poe", "give this to
grok-4.7 through Poe"). Poe's API is OpenAI-compatible and stateless: the
model sees only the request, cannot read the tree, run commands or edit files.
That makes it a fit for small, self-contained tasks and second opinions, not
for anything that needs the tests run in a loop.

## Reachability (§0.2)

- `POE_API_KEY` is set in the environment: `test -n "$POE_API_KEY"`. Never
  print it, write it to a file, put it in a brief, a log, a commit or a
  command's arguments, and never ask the user to paste it into the chat. If
  it is unset, stop and ask the user to export it in their shell.
- The examples pass the key through a process substitution so it never
  appears in `ps` output.

## Data use (§0.3)

Poe forwards each request to the model's own provider, whose data policy
applies. On a non-public repository, send file contents to a Poe model only
after the user has cleared that model's provider.

## Points

The plan gives a daily points allowance. Read the balance before the first
call and after the last one, and report the difference:

```sh
curl -sS https://api.poe.com/usage/current_balance \
  -H @<(printf 'Authorization: Bearer %s\n' "$POE_API_KEY") \
  | jq .current_point_balance
```

If the balance is too low for the planned calls, say so before spending it.

## Resolving the model

Poe model ids are bot names and are not always the spelling the user typed.
List them (cache the result for the session) and pick the exact `id` that
matches the user's name, ignoring case:

```sh
curl -sS https://api.poe.com/v1/models \
  -H @<(printf 'Authorization: Bearer %s\n' "$POE_API_KEY") \
  | jq -r '.data[].id' | grep -i -- '<name the user gave>'
```

No match, or more than one plausible match: tell the user what the list
offers and let them choose. Never substitute a different model.

## Calling it

Build the request with `jq` so file contents need no quoting by hand. The
system message carries the standing orders; the user message carries the
brief from [`task-brief.md`](task-brief.md) with every file it may edit
quoted in full under a `CURRENT FILES` section (path, then contents in a
fence).

```sh
jq -n --arg model '<exact id>' \
      --rawfile sys <scratch>/poe-system.txt \
      --rawfile brief <scratch>/brief-<task>.txt \
  '{model: $model, max_tokens: 8000,
    messages: [{role: "system", content: $sys},
               {role: "user",   content: $brief}]}' \
| curl -sS https://api.poe.com/v1/chat/completions \
    -H 'Content-Type: application/json' \
    -H @<(printf 'Authorization: Bearer %s\n' "$POE_API_KEY") \
    --data-binary @- \
| tee <scratch>/poe-<task>.json \
| jq -r '.choices[0].message.content'
```

The system message (`poe-system.txt`):

~~~text
You are a code delegate. You cannot run anything. Reply with ONE unified
diff (git format, paths relative to the repository root, a/ and b/
prefixes) inside a ```diff fence, touching only the files listed under
FILES YOU MAY EDIT, followed by a short list of anything you were unsure
of. No prose instructions in place of a diff. If the task cannot be done
as written, say why and send no diff.
~~~

Then apply it as §4a says: extract the fenced diff, `git apply --check`, and
only then `git apply`. The result goes through §4 steps 2 to 6 like any other
run; the orchestrator runs the CHECK, since the model could not.

Pass a model-specific setting (a reasoning effort, a thinking budget) only
when the user asked for one, under `extra_body` in the request, as Poe's docs
describe for that bot. `response_format` and the `strict` tool flag are
ignored by Poe, so never rely on structured output.

## Reviewing

For a second-opinion review, send `git diff <base>` with this instruction,
and triage the reply as §7 says:

```text
Review this diff as a senior reviewer. List concrete defects only (bugs,
security issues, broken edge cases, violations of the quoted repository
rules), each with the file, the line and why it is wrong. Say "no findings"
if there are none. Do not restate the diff or suggest style changes.
```

Keep the diff under the model's context; split by file when it is not.

## Errors

- `401` / `403`: the key is wrong or revoked. Stop and tell the user.
- `402` or an insufficient-points error: stop and report the balance.
- `429`: Poe allows 500 requests a minute; one retry after the
  `x-ratelimit-reset-requests` time, then report it.
- An unknown model: back to *Resolving the model*.

## Report fields

Provider `Poe` (and the underlying provider when the model's listing names
it in `owned_by`), harness `Poe API`, model the exact `id` sent, plus the
points the balance readings showed.
