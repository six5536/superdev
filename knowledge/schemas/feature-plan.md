---
type: Schema
id: schema-feature-plan
title: Feature Plan Schema
description: The feature's slice list — per slice a done-check, its test-plan cases and a done marker — filed in knowledge/feature-plans/.
---

# Feature Plan Schema

Structural rules for feature plans filed in `knowledge/feature-plans/` and
listed in its `index.md`. A feature plan and an ad-hoc plan are different
concepts, not two shapes of one: different type, different directory.
`schema-adhoc-plan` governs one-off work outside the feature workflow. The
directory is what selects this schema, so the glob does not repeat the kind;
the id still asserts it per document.

````yaml
target-files: "knowledge/feature-plans/*.md"
description: >
  The feature's slice list — per slice a done-check, the assigned
  test-plan cases, and a done marker. Produced by the feature-plan phase;
  read by build, verify, and integrate.
line-limit: 800

frontmatter:
  type:
    const: FeaturePlan
  id:
    pattern: '^feature-plan-\d{3}-[a-z0-9-]+$'
  status:
    enum: [draft, stable, deprecated]
    description: >
      draft while slices are outstanding; integrate ticks each slice's
      Done at merge and tags the concept done after the last slice.

sections-ordered: true
sections:
  - heading-pattern: '^Feature plan: .+$'
    level: 1
    required: true
    content: prose
    description: >
      Title heading ("Feature plan: {feature title}"), followed by a Spec
      line linking the spec at knowledge/specs/spec-{nnn}-{feature-slug}.md.
  - heading: "Slices"
    level: 2
    required: true
    description: >
      Ordered by dependency first, then risk: riskiest early.
  - heading-pattern: '^Slice \d+: .+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One slice, named ("Slice {n}: {name}"). Body: a "- [ ] Done —
      ticked by integrate at merge." checkbox; Change: what this slice
      changes, and where; Done-check: the pass/fail check verify runs
      against this slice; Cases: the test-plan case numbers assigned
      to this slice — an integration case belongs to the slice that
      completes its boundary. Every test-plan case in the spec appears
      in exactly one slice's Cases line.

example: |
  ---
  type: FeaturePlan
  id: feature-plan-001-pack-source-allowlist
  title: Pack source transport allowlist — feature plan
  description: Slices delivering the pack source transport allowlist.
  status: draft
  ---

  # Feature plan: Pack source transport allowlist

  Spec: [spec-001-pack-source-allowlist](../specs/spec-001-pack-source-allowlist.md)

  ## Slices

  ### Slice 1: Scheme parsing and refusal

  - [ ] Done — ticked by integrate at merge.
  - Change: parse the pack-source scheme in the manifest loader; refuse
    any transport that is not https, ssh, or file.
  - Done-check: a git:// source fails at parse naming the source; an
    https source resolves as before.
  - Cases: 1, 2

  ### Slice 2: Refusal message

  - [ ] Done — ticked by integrate at merge.
  - Change: error output naming the offending source.
  - Done-check: manual check 1 shows the refusal naming the source.
  - Cases: manual 1
````
