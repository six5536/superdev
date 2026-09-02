---
type: Schema
id: schema-feature-request
title: Feature Request Schema
description: Feature requests filed in the issue tracker — motivation, proposed behaviour, EARS acceptance criteria, alternatives and scope, with no room for invented repro steps.
---

# Feature Request Schema

Structural rules for feature requests, filed in the issue tracker as
`issue-{nnn}-feature-request-{slug}`, numbered after the highest across all of its kind's folders — a duplicate number is an error — and placed in its lifecycle folder by `superdev validate --fix`. It shares the
tracker with
`schema-bug-report` and `schema-chore` — the same id shape, the same
lifecycle — and differs only in its body: a request for
something absent states why it is wanted and what it would do, and is never
asked for an error log or a regression risk it does not have. Each contract
the feature touches is declared by an `implements` or `references` link
with a note saying what changed, added as CONTRACT-DESIGN updates it.

An issue is filed `unframed` by `/file` and framed by `/frame`, which
sets `framed`; `lifecycle` is the variant key (ADR-048). While
`unframed`, a criterion is a plain sentence, a `TBD — <the open
question>` or a keyed item, and the schema checks the list kind alone.
Once `framed`, every criterion carries its `AC_` key and its EARS tag,
and a `TBD` is an error; a `done` or `wontfix` request is held to the
same form. The Acceptance criteria heading is declared once per state
(ADR-049).

````yaml
description: >
  Feature request: what is missing, why it is wanted, what it would do,
  the criteria that make done checkable, what else was considered, and
  where the work stops.
line-limit: 800

variant-key: lifecycle

frontmatter:
  type:
    required: true
    const: FeatureRequest
  id:
    required: true
    pattern: '^issue-\d{3}-feature-request-[a-z0-9-]+$'
  title:
    required: true
    description: The one-line statement of what is missing.
  description:
    required: true
  lifecycle:
    enum: [unframed, framed, done, wontfix]
    description: >
      The folder is the value: unframed while the request is filed and
      not yet framed, framed once /frame has settled its criteria,
      done when it shipped, wontfix when it will not be built. The
      value selects the variant: unframed holds the criteria to their
      list kind alone; framed, done and wontfix hold every criterion to
      its key and tag (ADR-048).

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
      shipped and where, or why it will not be done. accept records its
      verdict here when it walks the criteria on the merged code. Sits
      directly under the title, before the report itself, because a reader
      who opens a settled issue wants the verdict before the evidence —
      every settled issue on file puts it there. Absent while the issue is
      outstanding, which is what distinguishes an open one from a settled
      one at a glance.
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
  - heading: "Acceptance criteria"
    level: 2
    required: true
    content: numbered-list
    variants: [unframed]
    description: >
      Numbered criteria as the user stated them, one per item: a plain
      sentence, an open question as "TBD — <the open question>", or a
      keyed EARS item where one is already known. While unframed the
      list kind alone is checked — no key and no tag is required, and
      none is refused — because nothing cites an unframed criterion
      (ADR-048). Framing rewrites every item into the framed form
      below and retires every TBD.
  - heading: "Acceptance criteria"
    level: 2
    required: true
    content: numbered-list
    item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '^`AC_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] '
    variants: [framed, done, wontfix]
    description: >
      Numbered EARS sentences, one criterion each, every one opening
      with its key in a code span — `AC_` then a slug of lowercase
      words joined by hyphens — then its pattern tag — [ubiquitous],
      [event], [state], [conditional], [optional] or [complex] — then
      the sentence in that pattern: "`AC_json-report` [conditional] IF x
      THE SYSTEM SHALL y". The item-key binds the key and the
      item-pattern the key before the tag (ADR-046, ADR-047; the tag
      alone was ADR-031). The key is the criterion's identity, stable
      and unique within the issue: a rewording keeps it, a removed key
      is not reused, and the number is the reading order. A criterion
      keyed by the sweep carries the slug `c<n>`, `n` its number
      (`AC_c11`), so a citation of the number stands; a criterion
      written since takes a named slug. A citation is the bare key
      where the issue is the subject — a plan case, a test of the
      feature — and the issue's id followed by the key elsewhere. Each
      is checkable as pass/fail without interpretation; the
      feature-plan's cases name the keys they cover, and accept walks
      them on the merged code. A "TBD" item is an error once framed:
      framing is what retires it, and a done or wontfix request is
      held to the same form (ADR-048).
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

example:
  unframed: |
    ---
    type: FeatureRequest
    id: issue-042-feature-request-validate-reports-machine-readable-json
    title: validate has no machine-readable output, so CI cannot act on findings
    description: validate prints for humans only, so a CI job can read the exit code but not which files failed or why.
    lifecycle: unframed
    ---

    # Feature: validate has no machine-readable output

    ## Summary

    `superdev validate` prints a human report and returns an exit code.
    A CI job can tell pass from fail, but cannot annotate the pull request
    with which file failed and why.

    ## Motivation

    Three of the four checks in `.github/workflows/checks.yml` already
    parse JSON from the tools they run. This one is the exception.

    ## Proposed behaviour

    A flag emits the report as JSON. The text output is unchanged.

    ## Acceptance criteria

    1. The report is one JSON object a CI job can read.
    2. TBD — whether the object carries the counts as well as the findings.

    ## Alternatives considered

    - TBD — none considered yet.

    ## Scope

    - In: the flag and its shape.
    - Out: TBD — whether annotating the pull request belongs here.
  framed: |
    ---
    type: FeatureRequest
    id: issue-042-feature-request-validate-reports-machine-readable-json
    title: validate has no machine-readable output, so CI cannot act on findings
    description: validate prints for humans only, so a CI job can read the exit code but not which files failed or why.
    lifecycle: framed
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

    ## Acceptance criteria

    1. `AC_json-report` [conditional] IF `--json` is given THE SYSTEM SHALL
       emit the report as one JSON object carrying the verdict, the counts,
       and one entry per finding with its file, severity and message.
    2. `AC_text-unchanged` [ubiquitous] THE SYSTEM SHALL leave the text
       output byte-identical when `--json` is absent.
    3. `AC_error-exit` [event] WHEN a finding is an error THE SYSTEM SHALL
       exit non-zero, `--json` or not.

    ## Alternatives considered

    - A separate `validate-json` verb — two verbs that must be kept in step
      for one report.
    - SARIF rather than a native shape — richer, and nothing here consumes
      it.

    ## Scope

    - In: the `--json` flag, its shape, and one golden covering it.
    - Out: annotating the pull request, which is the workflow's job.
  done: |
    ---
    type: FeatureRequest
    id: issue-042-feature-request-validate-reports-machine-readable-json
    title: validate has no machine-readable output, so CI cannot act on findings
    description: validate prints for humans only, so a CI job can read the exit code but not which files failed or why.
    lifecycle: done
    ---

    # Feature: validate has no machine-readable output

    ## Resolved

    Shipped in plan-012 slice 3: `--json` emits the object below, and the
    checks workflow reads it.

    ## Summary

    `superdev validate` prints a human report and returns an exit code.
    A CI job can tell pass from fail, but cannot annotate the pull request
    with which file failed and why.

    ## Motivation

    Three of the four checks in `.github/workflows/checks.yml` already
    parse JSON from the tools they run. This one is the exception.

    ## Proposed behaviour

    `--json` emits the report as a single object: the verdict, the counts,
    and one entry per finding carrying its file, severity and message.
    The text output is unchanged.

    ## Acceptance criteria

    1. `AC_json-report` [conditional] IF `--json` is given THE SYSTEM SHALL
       emit the report as one JSON object carrying the verdict, the counts,
       and one entry per finding with its file, severity and message.
    2. `AC_text-unchanged` [ubiquitous] THE SYSTEM SHALL leave the text
       output byte-identical when `--json` is absent.

    ## Alternatives considered

    - A separate `validate-json` verb — two verbs that must be kept in step
      for one report.

    ## Scope

    - In: the `--json` flag, its shape, and one golden covering it.
    - Out: annotating the pull request, which is the workflow's job.
  wontfix: |
    ---
    type: FeatureRequest
    id: issue-042-feature-request-validate-reports-machine-readable-json
    title: validate has no machine-readable output, so CI cannot act on findings
    description: validate prints for humans only, so a CI job can read the exit code but not which files failed or why.
    lifecycle: wontfix
    ---

    # Feature: validate has no machine-readable output

    ## Won't fix

    Decided 2026-03-04 by the maintainers: the checks workflow reads the
    exit code and links the run, and no consumer asked for more.

    ## Summary

    `superdev validate` prints a human report and returns an exit code.
    A CI job can tell pass from fail, but cannot annotate the pull request
    with which file failed and why.

    ## Motivation

    Three of the four checks in `.github/workflows/checks.yml` already
    parse JSON from the tools they run. This one is the exception.

    ## Proposed behaviour

    `--json` emits the report as a single object: the verdict, the counts,
    and one entry per finding carrying its file, severity and message.
    The text output is unchanged.

    ## Acceptance criteria

    1. `AC_json-report` [conditional] IF `--json` is given THE SYSTEM SHALL
       emit the report as one JSON object carrying the verdict, the counts,
       and one entry per finding with its file, severity and message.
    2. `AC_text-unchanged` [ubiquitous] THE SYSTEM SHALL leave the text
       output byte-identical when `--json` is absent.

    ## Alternatives considered

    - A separate `validate-json` verb — two verbs that must be kept in step
      for one report.

    ## Scope

    - In: the `--json` flag, its shape, and one golden covering it.
    - Out: annotating the pull request, which is the workflow's job.
````
