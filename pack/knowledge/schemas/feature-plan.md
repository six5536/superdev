---
type: Schema
id: schema-feature-plan
title: Feature Plan Schema
description: The feature's slice list — per slice a done-check, its test-plan cases and a done marker — filed among the plans.
---

# Feature Plan Schema

Structural rules for feature plans, filed among the plans as
`plan-{nnn}-feature-{slug}`, numbered after the highest across all of its kind's folders — a duplicate number is an error — and placed in its lifecycle folder by `superdev validate --fix`,
and listed in the plans `index.md`. A feature plan and an ad-hoc plan are different
concepts, not two shapes of one: `schema-adhoc-plan` governs one-off work
outside the feature workflow, and the two share the directory and the number
series. The frontmatter `type` is what selects this schema; the `feature` in
the path is for whoever reads the listing.

````yaml
description: >
  The feature's slice list — per slice a done-check, the assigned
  test-plan cases, and a done marker. Produced by the feature-plan phase;
  read by build and integrate.
line-limit: 800

frontmatter:
  type:
    const: FeaturePlan
  id:
    pattern: '^plan-\d{3}-feature-[a-z0-9-]+$'
  lifecycle:
    enum: [open, done, abandoned]
    description: >
      The folder is the value: open while slices are outstanding;
      integrate ticks each slice's Done at merge and sets done after the
      last slice; abandoned when the feature was dropped.

sections-ordered: true
sections:
  - heading-pattern: '^Feature plan: .+$'
    level: 1
    required: true
    content: prose
    description: >
      Title heading ("Feature plan: {feature title}"), followed by a
      Request line linking the framed issue `issue-{nnn}-{kind}-{slug}`
      whose acceptance criteria this plan delivers — for a bug, the repro
      and expected behaviour serve as the criteria.
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
      changes, and where; Done-check: the pass/fail check integrate runs
      against this slice; Cases: the slice's test cases, written inline,
      one per line, each naming the acceptance criteria it covers
      ("covers 1, 3"). A case belongs to exactly one slice — an
      integration or e2e case to the slice that completes its boundary,
      which usually puts the heaviest cases last — and every acceptance
      criterion in the feature-request is covered by at least one case
      across the plan.

example: |
  ---
  type: FeaturePlan
  id: plan-001-feature-pack-source-allowlist
  title: Pack source transport allowlist — feature plan
  description: Slices delivering the pack source transport allowlist.
  lifecycle: open
  ---

  # Feature plan: Pack source transport allowlist

  Request: [issue-041-feature-request-pack-source-allowlist][sokf:issue-041-feature-request-pack-source-allowlist]

  ## Slices

  ### Slice 1: Scheme parsing and refusal

  - [ ] Done — ticked by integrate at merge.
  - Change: parse the pack-source scheme in the manifest loader; refuse
    any transport that is not https, ssh, or file.
  - Done-check: a git:// source fails at parse naming the source; an
    https source resolves as before.
  - Cases:
    - unit: a git:// source is refused at parse, naming the source —
      covers 1.
    - unit: an https source resolves as before — covers 2.

  ### Slice 2: Refusal message

  - [ ] Done — ticked by integrate at merge.
  - Change: error output naming the offending source.
  - Done-check: the e2e run shows the refusal naming the source.
  - Cases:
    - e2e: `superdev sync` against a git:// manifest prints the refusal
      with the source named — covers 1, 3.
````
