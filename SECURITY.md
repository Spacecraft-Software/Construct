<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# Security Policy

Standard §26.1 requires every Spacecraft Software repository to state where a
vulnerability report goes and what happens to it.

## Scope

This repository holds **agent skills** — instructions a language model loads and
acts on — plus `construct-cli`, the Rust tool that packages and installs them.
That shapes what a vulnerability looks like here, and two classes matter more
than they would elsewhere:

- **A skill that instructs an agent to do something unsafe.** A skill is
  executed by a model with tool access. Text that tells it to run a destructive
  command, disable a safety gate, exfiltrate a file, or trust an untrusted input
  is a vulnerability in this repository, not a documentation bug — and it
  reaches every session that loads the skill.
- **Injection through skill content.** Skills carry reference files, code
  samples, and quoted upstream material. Content that smuggles instructions into
  a context where a model will read them as directives is in scope.

Also in scope: `construct-cli` itself (it writes to `~/.claude/skills/` and
unpacks archives), the CI gates, and any credential or private path committed by
mistake.

Out of scope: a vulnerability in a *project governed by* a skill here belongs to
that project's own repository.

## Reporting

Email `Mohamed.Hammad [at] SpacecraftSoftware.org`. Do not open a public issue
for anything that should not be public.

For a skill-content report, quote the passage and say what an agent would do
having read it. That is the whole of the report — no proof-of-concept is needed,
and we would rather not have one.

## What to expect

| | |
|---|---|
| Acknowledgement | within 7 days |
| Assessment | within 30 days |
| Supported versions | `main`, and the most recently published bundle of the affected skill |
| Disclosure | coordinated; the fix lands before any public description |
| Credit | offered by default, declined on request |

A fix to skill content ships as a new bundle; installed copies are stale until
reinstalled, so an advisory will say which skill and which version.

This is a personal hobby project (§5.1). There is no SLA, and the targets above
are intentions rather than commitments.
