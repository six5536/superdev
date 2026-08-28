---
type: Template
id: template-adr
title: ADR Template
description: Architecture decision record — context, the decision, options considered, and consequences. Filed as a Decision concept in knowledge/decisions/.
status: stable
---

---
type: Decision
id: adr-<nnn>-<slug>
title: <decision title>
description: <one line — the decision in a sentence>.
status: stable
---

# ADR-<NNN>: <decision title, e.g. "Use SQLite for local persistence">

- Status: proposed | accepted | superseded by ADR-<NNN> | deprecated
- Date: <YYYY-MM-DD>
- Deciders: <who made/approved the call>

## Context

<The forces at play: the technical or product situation that demands a decision, and the constraints (deadlines, team skills, existing systems) that narrow the options. Written so a newcomer in a year understands why this came up.>

## Decision

<The decision, stated in one or two active sentences: "We will …". No hedging.>

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| <Chosen option> | <...> | <...> |
| <Alternative> | <...> | <...> |

## Consequences

- Positive: <what gets easier>
- Negative: <what gets harder or is given up — be honest, every decision has costs>
- Follow-ups: <work this decision creates, if any>

---

Notes on usage (not part of the document):

- File as `knowledge/decisions/adr-<nnn>-<slug>.md`, numbered after the
  highest existing decision; the id is the filename without `.md`.
- List it in `knowledge/decisions/index.md`.
- A superseded ADR is never edited: the new ADR names it, and its
  Status line is updated to point forward.
