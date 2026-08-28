---
type: Schema
id: schema-status-update
title: Status Update Schema
description: Status updates — TL;DR, done since last update, in progress, blockers with recommended defaults, and next.
---

# Status Update Schema

Structural rules for status updates, matched by name
(`**/*status-update*.md`); the source names no filing directory, and filed in the knowledge as a concept. The heading
carries the date, so a series of these sorts and reads as a series.

````yaml
description: >
  One update in a series: the state in a sentence, what landed since the last
  one, what is moving, what is blocked and on whom, and what comes next.
line-limit: 800

frontmatter:
  type:
    const: StatusUpdate

sections-ordered: true
sections:
  - heading-pattern: '^Status: .+ — \d{4}-\d{2}-\d{2}$'
    level: 1
    required: true
    description: >
      "Status:" then the task or project name, an em dash, and the date as
      YYYY-MM-DD.
  - heading: "TL;DR"
    level: 2
    required: true
    content: prose
    description: >
      One or two sentences: overall state and the single most important thing
      the reader should know. Green/yellow/red if the audience uses that
      convention.
  - heading: "Done since last update"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per completed item, with a link to the PR, commit or
      artifact. Outcomes, not activity.
  - heading: "In progress"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per item: where it stands and what is next on it.
  - heading: "Blocked / needs input"
    level: 2
    content: bullet-list
    description: >
      One bullet per decision or dependency holding work up, each with who can
      unblock it and a recommended default. Omit the section when empty — an
      empty section here is good news worth stating in the TL;DR instead.
  - heading: "Next"
    level: 2
    required: true
    content: bullet-list
    description: >
      What will be worked on before the next update.
  - heading: "Risks & notes"
    level: 2
    content: prose
    description: >
      Anything trending badly, scope changes, dates at risk. One line each.
      Omit the section when empty.

example: |
  # Status: Pack transport allowlist — 2026-08-26

  ## TL;DR

  Green. The allowlist landed and the migration guide is out; what remains is
  the extraction hardening the security review turned up.

  ## Done since last update

  - Transport allowlist merged, refusing at manifest parse (#118).
  - 0.1-to-0.2 migration guide published alongside the release notes.

  ## In progress

  - Path traversal fix in archive extraction — normalising before join; tests
    written, fix in review.

  ## Blocked / needs input

  - Whether `http://` stays refused for local development. Needs a call from
    the maintainers; recommendation: keep it refused and document the `file`
    transport as the local path.

  ## Next

  - Land the extraction fix and cut 0.2.1.
  - Start schema-driven document validation.

  ## Risks & notes

  Scope has grown by one unplanned security fix; 0.2.1 slips by roughly two
  days as a result.
````
