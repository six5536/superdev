---
type: Schema
id: schema-adr
title: ADR Schema
description: Architecture decision records — context, the decision, options considered and consequences — filed among the ADRs.
---

# ADR Schema

Structural rules for architecture decision records, filed among the
ADRs as `adr-{nnn}-{slug}`, numbered after the highest across all of its kind's folders — a duplicate number is an error — and placed in its lifecycle folder by `superdev validate --fix`,
and listed in the ADRs' `index.md`. A superseded ADR is never edited: the new
ADR names it, and its Status line is updated to point forward.

````yaml
description: >
  Architecture decision record — context, the decision, options
  considered, and consequences.
line-limit: 800

frontmatter:
  type:
    const: Decision
  id:
    pattern: '^adr-\d{3}-[a-z0-9-]+$'
  lifecycle:
    enum: [active, deprecated]
    description: >
      The folder is the value: active while the decision stands,
      deprecated when superseded. The decision's own state — proposed,
      accepted, superseded — rides in the body Status line.

sections-ordered: true
sections:
  - heading-pattern: '^ADR-\d{3}: .+$'
    level: 1
    required: true
    content: bullet-list
    description: >
      The decision title, e.g. "ADR-012: Use SQLite for local
      persistence", followed by a bullet list of Status (proposed |
      accepted | superseded by ADR-NNN | deprecated), Date (YYYY-MM-DD),
      and Deciders (who made/approved the call).
  - heading: "Context"
    level: 2
    required: true
    content: prose
    description: >
      The forces at play: the technical or product situation that
      demands a decision, and the constraints (deadlines, team skills,
      existing systems) that narrow the options. Written so a newcomer
      in a year understands why this came up.
  - heading: "Decision"
    level: 2
    required: true
    content: prose
    description: >
      The decision, stated in one or two active sentences: "We will …".
      No hedging.
  - heading: "Options considered"
    level: 2
    required: true
    content: table
    columns: [Option, Pros, Cons]
    description: >
      A table of Option / Pros / Cons rows: the chosen option first,
      then each alternative.
  - heading: "Consequences"
    level: 2
    required: true
    content: bullet-list
    description: >
      Bullets for Positive (what gets easier), Negative (what gets
      harder or is given up — be honest, every decision has costs), and
      Follow-ups (work this decision creates, if any).

example: |
  ---
  type: Decision
  id: adr-012-pack-transport-allowlist
  title: Refuse non-allowlisted pack transports
  description: superdev fetches packs only over https, ssh, or file.
  lifecycle: active
  ---

  # ADR-012: Refuse non-allowlisted pack transports

  - Status: accepted
  - Date: 2026-08-26
  - Deciders: superdev maintainers

  ## Context

  A cloned manifest can name any git transport; unauthenticated
  transports expose users to tampered pack content.

  ## Decision

  We will refuse any pack source whose transport is not https, ssh, or
  file, at manifest parse time.

  ## Options considered

  | Option | Pros | Cons |
  |--------|------|------|
  | Allowlist at parse | Fails fast, clear error | New transports need a code change |
  | Warn and fetch | No breakage | Tampering risk remains |

  ## Consequences

  - Positive: no pack content over unauthenticated channels.
  - Negative: exotic-but-legitimate transports need an allowlist change.
  - Follow-ups: document the allowlist in the pack docs.
````
