---
type: Schema
id: schema-bug-report
title: Bug Report Schema
description: Bug reports filed in the issue tracker — symptom, repro, root cause and regression risk.
---

# Bug Report Schema

Structural rules for bug reports, filed in the issue tracker as
`issue-{nnn}-bug-{slug}`, numbered after the highest across all of its kind's folders — a duplicate number is an error — and placed in its lifecycle folder by `superdev validate --fix`. The feature is declared — when there
is one — by an `implements` or `references` link to its feature-request,
plan or contract. It shares the
tracker with `schema-feature-request` and `schema-chore`, and is the shape
for a defect alone: something that behaves against its own specification.
The `issue-tracker` concept holds the filing conventions. An issue's
`lifecycle` is its folder: `open` on arrival, and a resolved issue stays,
refiled under `done` or `wontfix` by `superdev validate --fix`.

````yaml
description: >
  Bug report: symptom, environment, exact repro steps, expected vs
  actual, root cause, and regression risk. One of the three shapes the
  issue tracker holds.
line-limit: 800

frontmatter:
  type:
    required: true
    const: BugReport
  id:
    required: true
    pattern: '^issue-\d{3}-bug-[a-z0-9-]+$'
  title:
    required: true
    description: The one-line symptom.
  description:
    required: true
  lifecycle:
    enum: [open, done, wontfix]
    description: >
      The folder is the value: open while the bug is outstanding, done
      when the fix shipped, wontfix when it will not be fixed.

sections-ordered: true
sections:
  - heading-pattern: '^Bug: .+$'
    level: 1
    required: true
    description: >
      Title heading carrying the one-line symptom, e.g. "Sync fails
      with ETIMEDOUT on large payloads".
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
      One or two sentences: what is broken and the impact — who
      hits it, how often, how bad.
  - heading: "Environment"
    level: 2
    required: true
    content: bullet-list
    description: >
      Version/commit and platform (OS, runtime version, relevant
      config). Bullet list.
  - heading: "Steps to reproduce"
    level: 2
    required: true
    content: numbered-list
    description: >
      Numbered exact steps — commands verbatim so anyone can rerun
      them.
  - heading: "Expected behaviour"
    level: 2
    required: true
    content: prose
    description: "What should happen."
  - heading: "Actual behaviour"
    level: 2
    required: true
    content: prose
    description: >
      What happens instead. Paste the exact error output/logs in a
      code block, trimmed to the relevant lines.
  - heading: "Root cause (if known)"
    level: 2
    required: true
    content: prose
    description: >
      Where the defect lives (path/to/file.ts:123) and the
      mechanism: the specific input/state that takes the code down
      the wrong path. If not yet known, state the leading
      hypothesis and what would confirm it.
  - heading: "Proposed fix / workaround"
    level: 2
    required: true
    content: bullet-list
    description: >
      Fix: the change that removes the defect. Workaround: how
      users can avoid it meanwhile, if any. Bullet list.

  - heading: "Regression risk"
    level: 2
    required: true
    content: prose
    description: >
      What else touches this code path; which tests would catch a
      recurrence.

  - heading: "Comments"
    level: 2
    content: prose
    description: >
      Conversation history, appended as it happens — the tracker's
      convention says append, so this sits last, where the verdict does not.

example: |
  ---
  type: BugReport
  id: issue-042-bug-pack-sync-etimedout
  title: Pack sync fails with ETIMEDOUT on large payloads
  description: Pack sync fails with ETIMEDOUT on large payloads.
  lifecycle: open
  ---

  # Bug: Pack sync fails with ETIMEDOUT on large payloads

  ## Summary

  Syncing a content pack larger than 50 MB times out on slow links;
  every user behind such a link hits it on first sync.

  ## Environment

  - Version/commit: v0.1.0 / 4127a3b
  - Platform: Linux x86_64, default network config

  ## Steps to reproduce

  1. Add a pack source larger than 50 MB to the manifest.
  2. Run the pack sync command.
  3. Wait about 30 seconds.

  ## Expected behaviour

  The pack downloads to completion regardless of size.

  ## Actual behaviour

  Sync aborts with a timeout naming the source host:

  ```text
  Error: connect ETIMEDOUT 203.0.113.7:443
  ```

  ## Root cause (if known)

  Leading hypothesis: a fixed whole-download socket timeout in the
  pack resolver; a per-read timeout would confirm it.

  ## Proposed fix / workaround

  - Fix: apply the timeout per read, not per download.
  - Workaround: sync the pack from a faster link.

  ## Regression risk

  Small-pack sync shares the resolver path; the resolver's sync tests
  would catch a recurrence.
````
