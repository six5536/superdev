---
type: BugReport
id: issue-020-the-schemas-do-not-ship
title: The pack ships 43 document templates and no schemas, so schema enforcement runs in this repository and nowhere else
description: superdev init writes the templates that produce documents and none of the schemas that check them, so a managed repo has nothing to dispatch against and every document passes unexamined — the capability P008 built reaches no user.
status: draft
tags: [needs-triage]
---

# Bug: the schemas do not ship, so enforcement runs nowhere but here

## Summary

P008 made `superdev validate` check each document against the schema its
frontmatter `type` names. The pack carries the 43 templates that produce
those documents and none of the 41 schemas that check them, so a managed repo
has no `knowledge/schemas/` at all. Every document in every repo but this one
passes without being examined, and the check that is superdev's reason for
existing reaches no user.

## Environment

- Version/commit: superdev 0.2.0, after P008
- Platform: any; the gap is in what the content pack carries

## Steps to reproduce

1. `superdev init` in an empty git repository.
2. `ls knowledge/schemas/` — no such directory.
3. `ls knowledge/templates/*.md | wc -l` — 43.
4. Break a starter concept: change `# Overview` to `## Overview` in
   `knowledge/architecture.md`, which `schema-architecture` requires at
   level 1.
5. `superdev validate`.

## Expected behaviour

The run reports the broken heading, exactly as it does in this repository,
because the schema that governs an `Architecture` document is present.

## Actual behaviour

No finding. Before this issue's companion fix the run said nothing at all
about it; it now says:

```
  documents: 0 checked against 0 schemas
  no schemas found — no document was checked against a contract
```

which makes the absence visible without closing it.

## Root cause (if known)

`crates/lib/superdev-core/src/content/layout.rs` classifies pack content by
position, and has rules for `knowledge/concepts/`, `knowledge/skills/` and
`knowledge/templates/` — none for `knowledge/schemas/`. Nothing under
`pack/knowledge/schemas/` would be an item even if the files were put there,
and they are not: this repository's 41 schemas are its own, tracked outside
the pack.

The templates and the schemas were built at different times — the templates
as pack content from the start, the schemas as this repository's local
contracts — and P008 turned the second into a checked contract without
noticing they had never travelled together.

## Proposed fix / workaround

- A `["knowledge", "schemas", name]` rule in `classify`, an `ItemKind` for
  it, and the SOKF component writing them, so `init` and `sync` place the
  schemas beside the templates that produce their documents.
- Decide what a repo may do with a shipped schema. A template is
  write-once and a user may edit it; a schema is a contract the validator
  enforces, so it is closer to the owned files than the scaffolds. Owned
  means `sync` repairs a user's edit, which is probably right and is a
  decision, not a default.
- Workaround: copy `knowledge/schemas/` from this repository by hand. The
  schemas are plain documents and the validator reads whatever is there.

## Regression risk

Adding 41 files to the pack widens the drift `status --drift` reports until
a repo syncs, and I016 already reports 65. The layout rules are covered by
`content::layout` unit tests, which a new position must extend rather than
replace — `paths_matching_no_rule_are_not_items` is the test that would
otherwise quietly accept a typo in the new rule.
