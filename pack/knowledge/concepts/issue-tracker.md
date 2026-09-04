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
`/file` files an issue here in the user's words, `/accept` files gap
issues here, and `/maintain` audits them.

# Conventions

- One concept per ticket, id `issue-<nnn>-<slug>`, numbered after the
  highest existing issue across all of the tracker's folders — a
  duplicate number is an error — and never a single combined tickets
  file.
- Every issue is one document kind, `type: Issue`, governed by
  `schema-issue` (ADR-050). Its `kind` says what it is — `bug`,
  `feature` or `chore` — and its body takes the same six headings
  whichever it is: Summary, Context, Behaviour, Scope, Resolution and
  Comments. Summary says what and for whom; Context says why now, with
  the evidence, the environment and the reproduction for a bug and the
  surfaces for a chore; Behaviour says what is expected and what
  happens for a bug, what is proposed for a feature and what done means
  for a chore, in prose and bullets; Scope draws the boundary and lists
  the alternatives considered; Resolution says how it ended; Comments
  append. No key, no EARS tag and no `TBD` rule holds an issue — keys
  and EARS live in the contracts.
  - `bug` — a defect: something behaves against its own
    specification.
  - `feature` — something absent that should exist.
  - `chore` — scoped mechanical work whose shape is already known: a
    rename, a migration, a sweep.
- Each issue carries a `lifecycle` with one of three values, and sits in
  the folder named its value — `issues/open/`, `issues/done/`,
  `issues/wontfix/`; `superdev validate --fix` places the file, so
  nothing writes a path by hand.
  - `open` — outstanding, whether just filed or being worked. An open
    issue carries no Resolution.
  - `done` and `wontfix` — settled. Resolution is required: what
    shipped and where, or why it will not be done.
- An issue's feature is declared by its `implements` or `references`
  link to the feature issue, the plan, or the contracts the feature
  touches — never by a path; an issue without a feature has no such
  link.
- The triage role is a string in the frontmatter `tags` list (see
  [Triage labels](#triage-labels)).
- An issue or plan that delivers a feature or realises a contract
  declares the link from its own side (`implements` → the target's
  id), so deleting it leaves no dangling edges in the canonical
  knowledge; one that merely cites or affects a concept uses
  `references` the same way.
- Comments and conversation history append under a `## Comments` heading.
- A resolved issue stays: set its `lifecycle` to `done` (or `wontfix`,
  reasoning under Resolution) in the resolving commit and let
  `superdev validate --fix` refile it — search down-ranks settled work.
  Durable knowledge found while resolving moves into the core concepts.
- Keep the tracker's `index.md` current (every issue on file, grouped
  by feature heading, its folder named by the link's path), and list it
  from the root `knowledge/index.md`.

When a skill says "publish to the issue tracker": create a new issue
concept with `lifecycle: open` and run `superdev validate --fix` to
file it, adding the index entry. When a skill says "fetch the relevant
ticket": `sokf_read` the issue's id, or `sokf_search` with `lifecycle:
["open"]` when only the topic is known.

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
implied — an open issue carrying no triage tag has not been triaged.
