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
`lifecycle` is its folder: `unframed` when `/file` files it, `framed`
once `/frame` has framed it, and a resolved issue stays, refiled under
`done` or `wontfix` by `superdev validate --fix`.

`lifecycle` is the variant key (ADR-048). While `unframed`, a repro
step or an expected-behaviour item is a plain sentence, a `TBD — <the
open question>` or a keyed item, and the schema checks the list kind
alone. Once `framed`, every step carries its `RS_` key and no tag,
every expected-behaviour item its `EX_` key and its EARS tag, and a
`TBD` is an error; a `done` or `wontfix` report is held to the same
form. Expected behaviour is a numbered list in every state. Each of the
two headings is declared once per state (ADR-049).

````yaml
description: >
  Bug report: symptom, environment, exact repro steps, expected vs
  actual, root cause, and regression risk. One of the three shapes the
  issue tracker holds.
line-limit: 800

variant-key: lifecycle

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
    enum: [unframed, framed, done, wontfix]
    description: >
      The folder is the value: unframed while the report is filed and
      not yet framed, framed once /frame has settled its steps and
      expected behaviour, done when the fix shipped, wontfix when it
      will not be fixed. The value selects the variant: unframed holds
      the steps and the expected behaviour to their list kind alone;
      framed, done and wontfix hold every item to its key (ADR-048).

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
    variants: [unframed]
    description: >
      Numbered steps as the reporter stated them, one per item: a plain
      sentence, an open question as "TBD — <the open question>", or a
      keyed step where one is already known. While unframed the list
      kind alone is checked — no key is required, and none is refused
      (ADR-048). Framing rewrites every step into the framed form below
      and retires every TBD.
  - heading: "Steps to reproduce"
    level: 2
    required: true
    content: numbered-list
    item-key: '^`(RS_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-prohibited-pattern: '^`RS_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\]'
    variants: [framed, done, wontfix]
    description: >
      Numbered exact steps — commands verbatim so anyone can rerun
      them — every one opening with its key in a code span, `RS_` then
      a slug of lowercase words joined by hyphens, and no EARS tag: a
      step is not a requirement (ADR-046), and a step carrying a tag
      after its key is an error. The key is the step's
      identity, stable and unique within the issue, and the number is
      the reading order; a step keyed by the sweep carries the slug
      `c<n>`, `n` its number (`RS_c2`). A citation is the bare key
      where the issue is the subject — a plan case, a test of the fix
      — and the issue's id followed by the key elsewhere. A "TBD" step
      is an error once framed, and a done or wontfix report is held to
      the same form (ADR-048).
  - heading: "Expected behaviour"
    level: 2
    required: true
    content: numbered-list
    variants: [unframed]
    description: >
      What should happen, as a numbered list, one expected behaviour
      per item: a plain sentence, an open question as "TBD — <the open
      question>", or a keyed EARS item where one is already known.
      While unframed the list kind alone is checked — no key and no tag
      is required, and none is refused (ADR-048). Framing rewrites
      every item into the framed form below and retires every TBD.
  - heading: "Expected behaviour"
    level: 2
    required: true
    content: numbered-list
    item-key: '^`(EX_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '^`EX_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] '
    variants: [framed, done, wontfix]
    description: >
      What should happen, as numbered EARS sentences, one expected
      behaviour each, every one opening with its key in a code span —
      `EX_` then a slug of lowercase words joined by hyphens — then its
      pattern tag — [ubiquitous], [event], [state], [conditional],
      [optional] or [complex] — then the sentence in that pattern: an
      expected behaviour is a requirement the fix is held to (ADR-048),
      so it takes the tag as a criterion does (ADR-046). The key is the
      item's identity, stable and unique within the issue, and the
      number is the reading order; an item keyed by the sweep carries
      the slug `c<n>`, `n` its number (`EX_c1`). A citation is the bare
      key where the issue is the subject — a plan case, a test of the
      fix — and the issue's id followed by the key elsewhere. A "TBD"
      item is an error once framed, and a done or wontfix report is
      held to the same form.
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

example:
  unframed: |
    ---
    type: BugReport
    id: issue-042-bug-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Pack sync fails with ETIMEDOUT on large payloads.
    lifecycle: unframed
    ---

    # Bug: Pack sync fails with ETIMEDOUT on large payloads

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Environment

    - Version/commit: v0.1.0 / 4127a3b
    - Platform: TBD — the reporter's OS is not yet known.

    ## Steps to reproduce

    1. Add a pack source larger than 50 MB to the manifest.
    2. Run the pack sync command and wait.

    ## Expected behaviour

    1. The pack downloads to completion regardless of size.
    2. TBD — whether a slow link should be reported before the download.

    ## Actual behaviour

    Sync aborts with a timeout naming the source host.

    ## Root cause (if known)

    TBD — not yet investigated.

    ## Proposed fix / workaround

    - TBD — none proposed yet.

    ## Regression risk

    TBD — not yet assessed.
  framed: |
    ---
    type: BugReport
    id: issue-042-bug-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Pack sync fails with ETIMEDOUT on large payloads.
    lifecycle: framed
    ---

    # Bug: Pack sync fails with ETIMEDOUT on large payloads

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Environment

    - Version/commit: v0.1.0 / 4127a3b
    - Platform: Linux x86_64, default network config

    ## Steps to reproduce

    1. `RS_large-source` Add a pack source larger than 50 MB to the manifest.
    2. `RS_sync` Run the pack sync command.
    3. `RS_wait` Wait about 30 seconds.

    ## Expected behaviour

    1. `EX_completes` [ubiquitous] The pack SHALL download to completion
       regardless of size.
    2. `EX_slow-link` [event] WHEN a read stalls THE SYSTEM SHALL report
       the source host and the bytes received so far.

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
  done: |
    ---
    type: BugReport
    id: issue-042-bug-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Pack sync fails with ETIMEDOUT on large payloads.
    lifecycle: done
    ---

    # Bug: Pack sync fails with ETIMEDOUT on large payloads

    ## Resolved

    Fixed in plan-009 slice 2: the resolver applies its timeout per
    read, and a 200 MB pack syncs over a throttled link in the test.

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Environment

    - Version/commit: v0.1.0 / 4127a3b
    - Platform: Linux x86_64, default network config

    ## Steps to reproduce

    1. `RS_large-source` Add a pack source larger than 50 MB to the manifest.
    2. `RS_sync` Run the pack sync command.

    ## Expected behaviour

    1. `EX_completes` [ubiquitous] The pack SHALL download to completion
       regardless of size.

    ## Actual behaviour

    Sync aborts with a timeout naming the source host.

    ## Root cause (if known)

    A fixed whole-download socket timeout in the pack resolver.

    ## Proposed fix / workaround

    - Fix: apply the timeout per read, not per download.

    ## Regression risk

    Small-pack sync shares the resolver path; the resolver's sync tests
    would catch a recurrence.
  wontfix: |
    ---
    type: BugReport
    id: issue-042-bug-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Pack sync fails with ETIMEDOUT on large payloads.
    lifecycle: wontfix
    ---

    # Bug: Pack sync fails with ETIMEDOUT on large payloads

    ## Won't fix

    Decided 2026-03-04 by the maintainers: the pack format caps a pack
    at 20 MB from 0.3.0, so no shipped pack reaches the timeout.

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Environment

    - Version/commit: v0.1.0 / 4127a3b
    - Platform: Linux x86_64, default network config

    ## Steps to reproduce

    1. `RS_large-source` Add a pack source larger than 50 MB to the manifest.
    2. `RS_sync` Run the pack sync command.

    ## Expected behaviour

    1. `EX_completes` [ubiquitous] The pack SHALL download to completion
       regardless of size.

    ## Actual behaviour

    Sync aborts with a timeout naming the source host.

    ## Root cause (if known)

    A fixed whole-download socket timeout in the pack resolver.

    ## Proposed fix / workaround

    - Workaround: sync the pack from a faster link.

    ## Regression risk

    None: nothing changes.
````
