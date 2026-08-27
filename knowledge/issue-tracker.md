---
type: IssueTracker
id: issue-tracker
title: Issue Tracker & Triage
description: Where issues live — one SOKF concept per ticket under knowledge/issues/ — plus the triage label vocabulary.
status: stable
---

Issues live as markdown files in this knowledge, not on GitHub. Specs and
plans already live here; issues follow the same conventions. The
workflow skills read this concept to learn where to publish and fetch:
`/accept` files gap issues here, `/feature-plan` picks them up, and
`/maintain` audits them.

# Conventions

- One flat directory: one file per ticket at
  `knowledge/issues/Innn-<slug>.md`, numbered after the highest
  existing issue, and never a single combined tickets file.
- The spec, when one exists, is
  `knowledge/specs/Snnn-<feature-slug>.md`.
- An issue's feature is declared by its `implements` or `references`
  link to the spec, not by the path; an issue without a feature has
  neither.
- Each issue is a SOKF concept with a unique id `issue-nnn-<slug>`, a
  `title`, a `description`, and `status: draft` while open. Its `type`
  names which of the three shapes it takes, and so which schema governs
  it:
  - `BugReport` — a defect: something behaves against its own
    specification. Symptom, environment, repro, root cause, regression
    risk.
  - `FeatureRequest` — something absent that should exist. Motivation,
    proposed behaviour, alternatives considered, scope. Never asked for
    an error log.
  - `Chore` — scoped mechanical work whose shape is already known: a
    rename, a migration, a sweep. Surfaces and a definition of done.
    Never asked for a root cause it does not have.
  Pick the shape the thing actually is. A rename filed as a bug report
  has to invent a symptom, which is what having one shape cost.
- The triage role is a string in the frontmatter `tags` list (see
  [Triage labels](#triage-labels)).
- An issue or plan that implements a spec declares the link from its own
  side (`implements` → the spec's id), so deleting it leaves no dangling
  edges in the canonical knowledge; one that merely cites or affects a spec uses
  `references` the same way.
- Comments and conversation history append under a `## Comments` heading.
- A resolved issue stays: swap its state tag to `done` (or keep
  `wontfix`, reasoning in the body) in the resolving commit — search
  down-ranks settled work. Durable knowledge found while resolving
  moves into the core concepts.
- Keep `knowledge/issues/index.md` current (open issues, grouped by
  feature heading), and list it from the root `knowledge/index.md`.
  Create the directory with the first issue.

When a skill says "publish to the issue tracker": create a new file in
`knowledge/issues/`, creating the directory and index entries if
needed. When a skill says "fetch the relevant ticket": read the
file at the referenced path — the user normally passes the path or issue
number directly.

# Triage labels

The skills speak in five canonical triage roles. This repo keeps the
default strings; a "label" here is a string in the issue's frontmatter
`tags` list.

| Role              | Label             | Meaning                                  |
| ----------------- | ----------------- | ---------------------------------------- |
| `needs-triage`    | `needs-triage`    | Maintainer needs to evaluate this issue  |
| `needs-info`      | `needs-info`      | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, ready for an AFK agent  |
| `ready-for-human` | `ready-for-human` | Requires human implementation            |
| `wontfix`         | `wontfix`         | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"),
use the matching tag from this table.
