---
type: Schema
id: schema-postmortem
title: Postmortem Schema
description: Blameless incident write-ups — impact, timeline, root cause and typed action items.
---

# Postmortem Schema

Structural rules for incident postmortems, matched by name
(`**/*postmortem*.md`); the source names no filing directory, and the
document is not an AOKF concept, so it carries no frontmatter. Blameless is
a structural property here, not a reminder: the document names systems and
processes, and the only place a person appears is as the owner of an action
item.

````yaml
target-files: "**/*postmortem*.md"
description: >
  Blameless incident write-up: impact, the timeline from trigger to
  resolution, the root cause including why the defenses missed it, what went
  well and poorly, and typed action items with owners.
line-limit: 800

sections-ordered: true
sections:
  - heading-pattern: '^Postmortem: .+$'
    level: 1
    required: true
    content: bullet-list
    description: >
      The incident title, then bullets for Date of incident (with times and
      zone), Duration (detection to resolution), Severity, Authors, and Status
      (draft | reviewed | action items complete). Followed by the blameless
      note: this document names systems and processes, not people.
  - heading: "Summary"
    level: 2
    required: true
    content: prose
    description: >
      Three or four sentences: what broke, who was affected and how badly, how
      it was resolved. Written for someone who reads nothing else.
  - heading: "Impact"
    level: 2
    required: true
    content: bullet-list
    description: >
      Users, requests or data affected, quantified where possible, and the
      business or downstream impact.
  - heading: "Timeline"
    level: 2
    required: true
    content: table
    columns: ["Time (TZ)", Event]
    description: >
      One row per event, in order: the trigger, detection, the key
      investigation steps including the wrong turns, mitigation, and confirmed
      resolution. The wrong turns are the rows that make the timeline useful.
  - heading: "Root cause"
    level: 2
    required: true
    content: prose
    description: >
      The mechanism, not just the trigger: the chain from initial condition to
      user-visible failure. Include why the defenses — tests, review, alerts,
      rollout — did not catch it.
  - heading: "What went well / what went poorly"
    level: 2
    required: true
    content: bullet-list
    description: >
      Bullets for Well, Poorly, and Lucky — where luck rather than process
      limited the damage. The Lucky line is the one that generates action
      items nobody thought to ask for.
  - heading: "Action items"
    level: 2
    required: true
    content: table
    columns: ["#", Action, Type, Owner, Due]
    description: >
      One row per action: a specific, verifiable change; its type — prevent,
      detect or mitigate; the owner; and the due date. An action with no owner
      is a wish.

example: |
  # Postmortem: Pack sync outage 2026-08-24

  - Date of incident: 2026-08-24, 09:12–10:41 UTC
  - Duration: 89 minutes, detection to resolution
  - Severity: SEV2 — sync unavailable, no data loss
  - Authors: the pack team
  - Status: reviewed

  > Blameless: this document names systems and processes, not people.

  ## Summary

  A pack source began returning redirects, and the resolver followed them
  without a depth limit until it exhausted its file descriptors. Every sync
  failed for 89 minutes. It was resolved by capping redirect depth and
  restarting the resolver.

  ## Impact

  - All sync operations failed for 89 minutes; roughly 400 attempts.
  - No data was lost and no lockfile was corrupted: the failure was before
    the write.

  ## Timeline

  | Time (TZ) | Event |
  |-----------|-------|
  | 09:12 | Upstream begins redirecting the pack source |
  | 09:19 | Sync failure alert fires — detection |
  | 09:34 | Investigation focuses on upstream availability — wrong turn |
  | 10:22 | Descriptor exhaustion found in the resolver logs |
  | 10:38 | Redirect depth cap deployed |
  | 10:41 | Sync confirmed working |

  ## Root cause

  The resolver followed HTTP redirects with no depth limit and opened a
  connection per hop without closing the previous one. A redirect loop
  upstream turned that into unbounded descriptor growth. Tests never covered
  redirects because the fake transport resolves in one hop, and the alert
  fired on sync failure rather than on descriptor count, so detection came
  after user impact rather than before it.

  ## What went well / what went poorly

  - Well: the alert fired within seven minutes of the first failure.
  - Poorly: 43 minutes were spent on upstream availability because the
    resolver logged the symptom and not the descriptor count.
  - Lucky: the loop was upstream and temporary. A permanent loop would have
    taken the process down rather than degrading it.

  ## Action items

  | # | Action | Type | Owner | Due |
  |---|--------|------|-------|-----|
  | 1 | Cap redirect depth at 5 and close each hop | prevent | pack team | 2026-08-26 |
  | 2 | Alert on descriptor count, not just sync failure | detect | infra | 2026-09-02 |
  | 3 | Add a redirect-loop case to the transport fake | prevent | pack team | 2026-09-09 |
````
