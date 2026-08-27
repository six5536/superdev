---
type: Schema
id: schema-feature-request
title: Feature Request Schema
description: Feature requests filed in knowledge/issues/ — motivation, proposed behaviour, alternatives and scope, with no room for invented repro steps.
---

# Feature Request Schema

Structural rules for feature requests filed at
`knowledge/issues/I{nnn}-{slug}.md`. It shares the tracker with
`schema-bug-report` and `schema-chore` — the same id shape, the same triage
tags, the same lifecycle — and differs only in its body: a request for
something absent states why it is wanted and what it would do, and is never
asked for an error log or a regression risk it does not have.

````yaml
description: >
  Feature request: what is missing, why it is wanted, what it would do,
  what else was considered, and where the work stops.
line-limit: 800

frontmatter:
  type:
    const: FeatureRequest
  id:
    pattern: '^issue-\d{3}-[a-z0-9-]+$'
  title:
    description: The one-line statement of what is missing.
  status:
    enum: [draft, stable, deprecated]
    description: >
      draft while the request is outstanding; the resolution rides in
      tags, not here.

sections-ordered: true
sections:
  - heading-pattern: '^Feature: .+$'
    level: 1
    required: true
    description: >
      Title heading naming what is missing, e.g. "The tracker has no
      shape for a feature request".
  - heading-pattern: "^(Decided|Resolved|Resolved in part|Won't fix)$"
    level: 2
    repeatable: true
    content: prose
    description: >
      How it ended, added when it does: what was decided and by whom, what
      shipped and where, or why it will not be done. Sits directly under the
      title, before the report itself, because a reader who opens a settled
      issue wants the verdict before the evidence — every settled issue on
      file puts it there. Absent while the issue is outstanding, which is
      what distinguishes an open one from a settled one at a glance.
  - heading: "Summary"
    level: 2
    required: true
    content: prose
    description: >
      One or two sentences: what does not exist, and who is blocked or
      slowed by its absence.
  - heading: "Motivation"
    level: 2
    required: true
    content: prose
    description: >
      Why this is wanted now, with the evidence: the case that hit it,
      the count that makes it worth doing, or the rule it would let the
      project keep. An absence measured, not asserted.
  - heading: "Proposed behaviour"
    level: 2
    required: true
    content: prose
    description: >
      What exists once this is done, described so a reader could
      recognise it. Behaviour, not implementation.
  - heading: "Alternatives considered"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per option not taken, each with the single reason it
      lost. A request with no alternatives has not been thought about
      yet; say so rather than leaving the section out.
  - heading: "Scope"
    level: 2
    required: true
    content: bullet-list
    description: >
      What is in, and — separately — what is deliberately out, so a
      reader sees the boundary was drawn rather than forgotten.

  - heading: "Comments"
    level: 2
    content: prose
    description: >
      Conversation history, appended as it happens — the tracker's
      convention says append, so this sits last, where the verdict does not.

example: |
  ---
  type: FeatureRequest
  id: issue-042-validate-reports-machine-readable-json
  title: validate has no machine-readable output, so CI cannot act on findings
  description: validate prints for humans only, so a CI job can read the exit code but not which files failed or why.
  status: draft
  tags: [needs-triage]
  ---

  # Feature: validate has no machine-readable output

  ## Summary

  `superdev validate` prints a human report and returns an exit code.
  A CI job can tell pass from fail, but cannot annotate the pull request
  with which file failed and why.

  ## Motivation

  Three of the four checks in `.github/workflows/checks.yml` already
  parse JSON from the tools they run. This one is the exception, so its
  failures arrive as a wall of text in the job log that a reviewer has to
  open the run to read.

  ## Proposed behaviour

  `--json` emits the report as a single object: the verdict, the counts,
  and one entry per finding carrying its file, severity and message.
  The text output is unchanged.

  ## Alternatives considered

  - A separate `validate-json` verb — two verbs that must be kept in step
    for one report.
  - SARIF rather than a native shape — richer, and nothing here consumes
    it.

  ## Scope

  - In: the `--json` flag, its shape, and one golden covering it.
  - Out: annotating the pull request, which is the workflow's job.
````
