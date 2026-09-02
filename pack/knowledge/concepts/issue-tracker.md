---
type: IssueTracker
id: issue-tracker
title: Issue Tracker & Triage
description: Where issues live — one SOKF concept per ticket in the issue tracker, filed by lifecycle — plus the triage label vocabulary.
status: stable
---

Issues live as markdown files in this knowledge, not on GitHub. Plans
already live here; issues follow the same conventions. The
workflow skills read this concept to learn where to publish and fetch:
`/accept` files gap issues here, `/feature-plan` picks them up, and
`/maintain` audits them.

# Conventions

- One concept per ticket, id `issue-<nnn>-<kind>-<slug>`, where `<kind>`
  is `bug`, `feature-request` or `chore` — the type's own word, so a
  listing sorts by number and reads by kind. Numbered after the highest
  existing issue across all of the tracker's folders — a duplicate
  number is an error — and never a single combined tickets file.
- Each issue carries a `lifecycle` — `unframed`, `framed`, `done` or
  `wontfix` (ADR-048) — and sits in the folder named its value;
  `superdev validate --fix` places the file, so nothing writes a path
  by hand.
- An issue's feature is declared by its `implements` or `references`
  link to the feature-request, the plan, or the contracts the feature
  touches — never by a path; an issue without a feature has no such
  link.
- Each issue is a SOKF concept with a unique id `issue-<nnn>-<kind>-<slug>`, a
  `title`, a `description`, and `lifecycle: open` while open. Its `type`
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
- An issue or plan that delivers a feature request or realises a
  contract declares the link from its own side (`implements` → the
  target's id), so deleting it leaves no dangling edges in the canonical
  knowledge; one that merely cites or affects a concept uses
  `references` the same way.
- Comments and conversation history append under a `## Comments` heading.
- A resolved issue stays: set its `lifecycle` to `done` (or `wontfix`,
  reasoning in the body) in the resolving commit and let
  `superdev validate --fix` refile it — search down-ranks settled work.
  Durable knowledge found while resolving moves into the core concepts.
- Keep the tracker's `index.md` current (open issues, grouped by
  feature heading), and list it from the root `knowledge/index.md`.

When a skill says "publish to the issue tracker": create a new issue
concept with `lifecycle: open` and run `superdev validate --fix` to file
it, adding the index entry. When a skill says "fetch the relevant
ticket": `sokf_read` the issue's id, or `sokf_search` with
`lifecycle: ["open"]` when only the topic is known.

# Triage labels

The skills speak in five canonical triage roles. This repo keeps the
default strings; a "label" here is a string in the issue's frontmatter
`tags` list.

| Role              | Label             | Meaning                                  |
| ----------------- | ----------------- | ---------------------------------------- |
| `needs-info`      | `needs-info`      | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, ready for an AFK agent  |
| `ready-for-human` | `ready-for-human` | Requires human implementation            |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"),
use the matching tag from this table. Two former labels are lifecycle
values now: `wontfix` is `lifecycle: wontfix`, and `needs-triage` is
implied — an `open` issue carrying no triage tag has not been triaged.
