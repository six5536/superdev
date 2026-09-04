---
type: Schema
id: schema-issue
title: Issue Schema
description: The one shape an issue takes in the tracker — a bug, a feature or a chore, told apart by `kind` — with its summary, context, behaviour, scope, resolution and comments in prose and bullets, and no key or EARS tag on any of them.
---

# Issue Schema

Structural rules for an issue, filed in the issue tracker as
`issue-{nnn}-{slug}`, numbered after the highest across the tracker's
folders — a duplicate number is an error — and placed in its lifecycle
folder by `superdev validate --fix`. The tracker holds one kind of
document: `kind` in the frontmatter says whether the issue is a `bug`,
a `feature` or a `chore`, and the body takes the same six headings
whichever it is. The `issue-tracker` concept holds the filing
conventions.

An issue is written in prose and bullets: every section carries at
least one line of prose, and bullets follow it where a list reads
better. No key, no EARS tag and no `TBD` rule holds an issue: keys and
EARS live in the contracts, whose promises carry the criteria a plan
case and a test cite (ADR-050). An issue says what is wanted and why;
a contract says what the software promises.

`lifecycle` is the variant key (ADR-045): `open` while the issue is
outstanding, `done` once it shipped, `wontfix` once it was declined.
Resolution is required under `done` and `wontfix` and prohibited under
`open`, so a reader tells a settled issue from an open one by the
heading alone.

````yaml
description: >
  An issue in the tracker: a bug, a feature or a chore, with what it
  is, why now, what is expected or proposed, where the work stops, and
  — once settled — how it ended.
line-limit: 800

variant-key: lifecycle

frontmatter:
  type:
    required: true
    const: Issue
  id:
    required: true
    pattern: '^issue-\d{3}-[a-z0-9-]+$'
  title:
    required: true
    description: The one-line statement of the issue.
  description:
    required: true
  kind:
    required: true
    enum: [bug, feature, chore]
    description: >
      What the issue is: a bug is something that behaves against its
      own specification, a feature is something absent that should
      exist, a chore is scoped mechanical work whose shape is already
      known. The kind selects nothing in the schema; it tells a reader
      and a search what they are looking at.
  lifecycle:
    enum: [open, done, wontfix]
    description: >
      The folder is the value: open while the issue is outstanding,
      done once it shipped, wontfix once it was declined. The value
      selects the variant: done and wontfix require Resolution, open
      prohibits it.

sections-ordered: true
sections:
  - heading-pattern: '^(Bug|Feature|Chore): .+$'
    level: 1
    required: true
    description: >
      Title heading opening with the kind's word, e.g. "Bug: sync fails
      with ETIMEDOUT on large payloads".
  - heading: "Summary"
    level: 2
    required: true
    content: prose
    description: >
      One or two sentences: what the issue is and for whom — what is
      broken and its impact, what is missing and who is slowed by its
      absence, or what changes and why it is worth doing.
  - heading: "Context"
    level: 2
    required: true
    content: prose
    description: >
      Why now, with the evidence: the case that hit it, the count that
      makes it worth doing, or the rule it would let the project keep.
      For a bug, the environment — version, platform, configuration —
      and the steps that reproduce it, as a bullet list where that
      reads better. For a chore, the surfaces the work reaches and
      the counts that bound them.
  - heading: "Behaviour"
    level: 2
    required: true
    content: prose
    description: >
      What is expected. For a bug, what should happen and what happens
      instead, with the error output in a code block trimmed to the
      relevant lines, and the root cause where it is known. For a
      feature, what exists once it is done, described so a reader could
      recognise it — behaviour, not implementation. For a chore, what
      done means. Prose, bullets or both; bullets are encouraged for a
      list of expectations a reader will check one by one. No key and
      no EARS tag: the criteria a test binds live on the contract the
      work touches.
  - heading: "Scope"
    level: 2
    content: prose
    description: >
      Where the work stops: what is in and, separately, what is
      deliberately out, so a reader sees the boundary was drawn rather
      than forgotten; the alternatives considered and why each lost,
      where any were. Bullets are encouraged. Absent when the boundary
      is the issue itself.
  - heading: "Resolution"
    level: 2
    required: true
    content: prose
    variants: [done, wontfix]
    description: >
      How it ended: what shipped and where, what was decided and by
      whom, or why it will not be done. Present once the issue is done
      or wontfix and prohibited while it is open, so the heading alone
      tells a settled issue from an outstanding one.
  - heading: "Comments"
    level: 2
    content: prose
    description: >
      Conversation history, appended as it happens — the tracker's
      convention says append, so this sits last.

sections-prohibited:
  - heading: "Resolution"
    variants: [open]

example:
  open: |
    ---
    type: Issue
    id: issue-042-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Syncing a content pack larger than 50 MB times out on slow links, so every user behind such a link fails on first sync.
    kind: bug
    lifecycle: open
    ---

    # Bug: pack sync fails with ETIMEDOUT on large payloads

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Context

    Reported from a site on a throttled link, v0.1.0 at 4127a3b on
    Linux x86_64 with the default network configuration.

    - Add a pack source larger than 50 MB to the manifest.
    - Run the pack sync command and wait about 30 seconds.

    ## Behaviour

    The pack downloads to completion regardless of size, and a stalled
    read reports the source host and the bytes received so far.

    Instead, sync aborts with a timeout naming the source host:

    ```text
    Error: connect ETIMEDOUT 203.0.113.7:443
    ```

    The leading hypothesis is a fixed whole-download socket timeout in
    the pack resolver; a per-read timeout would confirm it.

    ## Scope

    The resolver's timeout alone.

    - In: the timeout the resolver applies to a read.
    - Out: reporting a slow link before the download starts.
  done: |
    ---
    type: Issue
    id: issue-042-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Syncing a content pack larger than 50 MB times out on slow links, so every user behind such a link fails on first sync.
    kind: bug
    lifecycle: done
    ---

    # Bug: pack sync fails with ETIMEDOUT on large payloads

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Context

    Reported from a site on a throttled link, v0.1.0 at 4127a3b on
    Linux x86_64 with the default network configuration.

    - Add a pack source larger than 50 MB to the manifest.
    - Run the pack sync command and wait about 30 seconds.

    ## Behaviour

    The pack downloads to completion regardless of size.

    Instead, sync aborts with a timeout naming the source host. The
    cause was a fixed whole-download socket timeout in the pack
    resolver.

    ## Resolution

    Fixed in plan-009: the resolver applies its timeout per read, and
    a 200 MB pack syncs over a throttled link in the test.
  wontfix: |
    ---
    type: Issue
    id: issue-042-pack-sync-etimedout
    title: Pack sync fails with ETIMEDOUT on large payloads
    description: Syncing a content pack larger than 50 MB times out on slow links, so every user behind such a link fails on first sync.
    kind: bug
    lifecycle: wontfix
    ---

    # Bug: pack sync fails with ETIMEDOUT on large payloads

    ## Summary

    Syncing a content pack larger than 50 MB times out on slow links;
    every user behind such a link hits it on first sync.

    ## Context

    Reported from a site on a throttled link, v0.1.0 at 4127a3b on
    Linux x86_64 with the default network configuration.

    ## Behaviour

    The pack downloads to completion regardless of size. Instead, sync
    aborts with a timeout naming the source host.

    ## Resolution

    Decided 2026-03-04 by the maintainers: the pack format caps a pack
    at 20 MB from 0.3.0, so no shipped pack reaches the timeout. Sync
    the pack from a faster link meanwhile.
````
