---
type: Schema
id: schema-issue-tracker
title: Issue Tracker Schema
description: Where issues live, the filing conventions and the triage label vocabulary, in knowledge/issue-tracker.md.
---

# Issue Tracker Schema

Structural rules for `knowledge/issue-tracker.md`, the canonical knowledge's Convention
concept for how issues are filed and triaged. This is the concept
`schema-bug-report` defers to for the label vocabulary and the lifecycle.

````yaml
description: >
  Where issues live, how they are filed and what happens to them when
  resolved, and the triage label vocabulary with who acts on each.
line-limit: 800

frontmatter:
  type:
    const: IssueTracker
  id:
    const: issue-tracker
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    Where issues live — in-repo files, GitHub, elsewhere — and how they relate
    to specs and plans.

sections-ordered: true
sections:
  - heading: "Conventions"
    level: 1
    required: true
    content: bullet-list
    description: >
      The filing rule — naming, location, one issue per what — and the
      lifecycle rule: what happens to an issue when it is resolved.
  - heading: "Triage labels"
    level: 1
    required: true
    content: table
    columns: [Role, Label, Meaning]
    description: >
      One row per label: the role it belongs to, the label itself, and who acts
      on it and what they do.

example: |
  ---
  type: Convention
  id: issue-tracker
  title: Issue Tracker & Triage
  description: Where issues live and the triage label vocabulary.
  status: stable
  ---

  Issues live in the canonical knowledge as Issue concepts under `knowledge/issues/`, not
  in GitHub, so they travel with the tree an agent reads. An issue that turns
  out to need a behaviour decision gets a spec; one that needs work gets a
  plan, linked both ways.

  # Conventions

  - Filed as `knowledge/issues/issue-{nnn}-{slug}.md`, one issue per
    reproducible symptom. Two symptoms with one cause stay two issues until
    the cause is proven.
  - A resolved issue is never deleted. It stays, retagged `done` or
    `wontfix`, so the search that found it once finds the answer next time.

  # Triage labels

  | Role | Label | Meaning |
  |------|-------|---------|
  | triage | `needs-triage` | unreviewed; a maintainer sets severity and role |
  | state | `done` | resolved, with the fix linked from the issue |
  | state | `wontfix` | closed deliberately, with the reasoning recorded |
````
