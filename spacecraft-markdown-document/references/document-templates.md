<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) 2026 Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->

<!-- GFM Document
     title:      Document Templates
     author:     Mohamed Hammad & Spacecraft Software
     date:       2026-05-25
     license:    GPL-3.0-or-later
     website:    https://Construct.SpacecraftSoftware.org/
-->

# Document Templates

Ready-to-fill GFM templates for the `spacecraft-markdown-document` skill.
Invoke via: `/spacecraft-markdown-document template <slug>`

Replace every `<placeholder>` before delivering. Placeholders in angle-brackets
are mandatory; those in `[square brackets]` are optional.

---

## `readme` — Project README

```markdown
<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) <YYYY> Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->

<!-- GFM Document
     title:      <ProjectName>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
     website:    https://<ProjectName>.SpacecraftSoftware.org/
-->

# <ProjectName>

> <One-sentence description of the project.>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)

## Overview

<Two to four sentences describing what the project does, who it is for,
and why it exists.>

## Features

- <Feature one>
- <Feature two>
- <Feature three>

## Requirements

- <Dependency or runtime requirement>
- <Another requirement>

## Installation

```sh
<installation command(s)>
```

## Usage

```sh
<usage example>
```

[Further usage examples or link to documentation.]

## Building from Source

```sh
git clone https://github.com/Spacecraft-Software/<ProjectName>.git
cd <ProjectName>
<build command>
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

GPL-3.0-or-later — see [`LICENSE`](LICENSE).

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `changelog` — CHANGELOG.md (Keep a Changelog)

```markdown
<!--
SPDX-License-Identifier: GPL-3.0-or-later
Copyright (C) <YYYY> Mohamed Hammad & Spacecraft Software
https://Construct.SpacecraftSoftware.org/
-->

<!-- GFM Document
     title:      Changelog — <ProjectName>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->

# Changelog

All notable changes to **<ProjectName>** are documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [Unreleased]

### Added

- <New feature>

### Changed

- <Change to existing behaviour>

### Fixed

- <Bug fix>

### Removed

- <Removed feature>

---

## [<X.Y.Z>] — <YYYY-MM-DD>

### Added

- <First release feature list>

[Unreleased]: https://github.com/Spacecraft-Software/<ProjectName>/compare/v<X.Y.Z>...HEAD
[<X.Y.Z>]: https://github.com/Spacecraft-Software/<ProjectName>/releases/tag/v<X.Y.Z>

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `contributing` — CONTRIBUTING.md

```markdown
<!-- GFM Document
     title:      Contributing to <ProjectName>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->

# Contributing to <ProjectName>

Thank you for considering a contribution to **<ProjectName>**.
This document explains how to contribute effectively.

## Code of Conduct

Be respectful and constructive. Harassment of any kind is not tolerated.

## How to Contribute

### Reporting Issues

Open an issue at <https://github.com/Spacecraft-Software/<ProjectName>/issues>
with:

- A clear title.
- Steps to reproduce (for bugs).
- Expected vs. actual behaviour.
- Environment details (OS, compiler version, relevant config).

### Submitting Changes

1. Fork the repository and create a feature branch.
2. Write your changes following [The Steelbore Standard](https://Standard.SpacecraftSoftware.org/).
3. Run tests: `<test command>`.
4. Sign your commits (Ed25519 SSH signing required — see §6.3 of the Standard).
5. Open a pull request against `main`.

### Commit Message Format

```
<type>(<scope>): <short summary>

<body — explain what and why, not how>

[BREAKING CHANGE: <description>]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`.

## License

By contributing, you agree that your contributions are licensed under
GPL-3.0-or-later.

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `prd` — Product Requirements Document

```markdown
<!-- GFM Document
     title:      PRD — <Feature or Product Name>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     version:    0.1.0
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->

# PRD — <Feature or Product Name>

**Status:** Draft | In Review | Approved | Superseded
**Author:** Mohamed Hammad
**Date:** <YYYY-MM-DD>
**Version:** 0.1.0

---

## 1 — Problem Statement

<What problem does this feature or product solve? Who experiences it? What is
the current state without this solution?>

## 2 — Goals

- <Measurable goal 1>
- <Measurable goal 2>

## 3 — Non-Goals

- <What is explicitly out of scope>

## 4 — Requirements

### 4.1 — Functional Requirements

| ID    | Requirement                          | Priority  |
|-------|--------------------------------------|-----------|
| FR-01 | <Requirement description>            | Must Have |
| FR-02 | <Requirement description>            | Should Have |

### 4.2 — Non-Functional Requirements

| ID     | Requirement                         | Priority  |
|--------|-------------------------------------|-----------|
| NFR-01 | <Performance / security / etc.>     | Must Have |

## 5 — Design Notes

<Architecture notes, diagrams, or references to design documents.>

## 6 — Open Questions

- [ ] <Question requiring resolution>

## 7 — References

- <Link or citation>

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `adr` — Architecture Decision Record

```markdown
<!-- GFM Document
     title:      ADR-<NNN> — <Decision Title>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->

# ADR-<NNN> — <Decision Title>

**Date:** <YYYY-MM-DD>
**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-<NNN>
**Deciders:** Mohamed Hammad

---

## Context

<What is the issue that motivates this decision? What are the forces at play?>

## Decision

<What is the change that is proposed and/or adopted?>

## Consequences

### Positive

- <Benefit>

### Negative

- <Trade-off or downside>

### Neutral

- <Side effect without clear valence>

## Alternatives Considered

| Option | Reason Rejected |
|--------|-----------------|
| <Alt 1> | <Why not chosen> |

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `release-notes` — Release Notes

```markdown
<!-- GFM Document
     title:      Release Notes — <ProjectName> v<X.Y.Z>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->

# Release Notes — <ProjectName> v<X.Y.Z>

**Released:** <YYYY-MM-DD>
**Tag:** [`v<X.Y.Z>`](https://github.com/Spacecraft-Software/<ProjectName>/releases/tag/v<X.Y.Z>)

---

## Highlights

<Two to three sentences summarising the most important changes in this release.>

## What's New

- <New feature or improvement>
- <New feature or improvement>

## Bug Fixes

- <Bug fix description> ([#<issue>](https://github.com/Spacecraft-Software/<ProjectName>/issues/<issue>))

## Breaking Changes

> **Note:** The following changes require action when upgrading.

- <Breaking change and required migration step>

## Upgrading

```sh
<upgrade command>
```

[Full diff](https://github.com/Spacecraft-Software/<ProjectName>/compare/v<PREV>...v<X.Y.Z>)

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

## `companion` — GFM Companion for ODF/MS Office Document

```markdown
<!-- GFM Document
     title:      <Document Title>
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     project:    <ProjectName>
-->
<!-- companion: <source-basename>.<ext> | palette: Spacecraft Software §9 -->

# <Document Title>

> **Note:** This file is the GFM companion to `<source-basename>.<ext>`.
> Visual styling (Void Navy background `#000027`, Spacecraft Software palette,
> Share Tech Mono / Inconsolata typography) lives in the source file only.
> This companion carries the content for text-only tools, diffs, and agents.

---

<Mirror the full document content here in plain GFM.
Heading levels must match the source file: H1 = document title,
H2 = sections, H3 = subsections.
Do not add colour, font hints, or page-geometry cues — those are visual-only.>

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) — companion to `<source-basename>.<ext>` —*
```

---

## `skill-readme` — Spacecraft Software Skill README

```markdown
<!-- GFM Document
     title:      <skill-name> — Spacecraft Software Skill
     author:     Mohamed Hammad & Spacecraft Software
     date:       <YYYY-MM-DD>
     license:    GPL-3.0-or-later
     website:    https://Construct.SpacecraftSoftware.org/
-->

# `<skill-name>`

> <One-sentence description of what this skill enables.>

**Author:** Mohamed Hammad & Spacecraft Software
**License:** GPL-3.0-or-later
**Registry:** <https://Construct.SpacecraftSoftware.org/>

---

## Purpose

<Describe what the skill does, when it should be invoked, and what it
produces. Be specific about the trigger pattern (auto / slash command).>

## Invocation

[Auto-triggered when: <describe trigger conditions>]

[Slash command: `/skill-name [subcommand] [options]`]

## Directory Layout

```
<skill-name>/
├── SKILL.md
└── references/
    └── <reference-file>.md
```

## License

GPL-3.0-or-later — see [`LICENSE`](../../LICENSE).

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
```

---

*— Built by [Spacecraft Software](https://SpacecraftSoftware.org/) —*
