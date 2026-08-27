---
type: Schema
id: schema-spec
title: Spec Schema
description: Feature specs filed in knowledge/specs/ — the spec body plus the appended test plan.
---

# Spec Schema

Structural rules for feature specs filed at
`knowledge/specs/S{nnn}-{feature-slug}.md` and listed in
`knowledge/specs/index.md`.

This contract was reconciled against the fourteen specs on file, and is
deliberately thinner than the one it replaced. That one required sixteen
sections; one spec carried them and thirteen did not, because nothing had
ever checked. Where the documents agree with each other and not with the
schema, the schema is what was wrong. What they agree on is that a spec
opens by saying why it exists — all fourteen do — and that is what is
required here. The rest is recommended in the descriptions, and enforced
where a section appears at all.

````yaml
description: >
  Feature specification: what done looks like from outside — why the
  feature exists, its observable behaviour, and what is out of scope. Say
  what it does, never how.
line-limit: 800

frontmatter:
  type:
    const: Spec
  id:
    pattern: '^spec-\d{3}-[a-z0-9-]+$'
  status:
    enum: [draft, stable, deprecated]
    description: draft until accept tags it done.

sections-ordered: false
sections:
  - heading-pattern: '^(Problem|Context|Motivation|Goal|Summary)$'
    level: 1
    required: true
    content: prose
    description: >
      The opening section: why this exists, before anything about the
      design. Every spec in this repository has one, always first, under
      one of these five names — that agreement is what makes it the one
      thing required here. Name it for what the section actually holds:
      Problem where something is wrong, Motivation where something is
      merely wanted, Context where the reader needs the ground first.
  - heading: "Behaviour"
    level: 1
    content: bullet-list
    description: >
      What the feature does, from outside. Say what it does, never how.
  - heading: "Acceptance criteria"
    level: 1
    content: numbered-list
    description: >
      Numbered, each independently checkable, each phrased so a reader
      could confirm it without reading the code.
  - heading: "UI states"
    level: 1
    content: bullet-list
    description: Loading, empty, error and success, where there is a UI.
  - heading: "Edge cases & errors"
    level: 1
    content: bullet-list
    description: >
      The inputs and states that are easy to get wrong, and what happens
      on each.
  - heading: "Testing"
    level: 1
    description: >
      How the feature is proved. Ten of the fourteen specs on file carry
      this section and four do not, so it is recommended rather than
      required: requiring it would fail four completed records, and the
      remedy would be to invent a test plan for work already shipped.
  - heading: "Out of scope"
    level: 1
    content: bullet-list
    description: >
      What this deliberately does not cover, so a reviewer sees it was
      excluded rather than forgotten. Twelve of fourteen carry it; strongly
      recommended, and the first thing to add to a spec that lacks one.
  - heading: "Open questions"
    level: 1
    content: bullet-list
    description: A decision still needed, with the recommended default.
  - heading-pattern: '^.+$'
    level: 1
    repeatable: true
    description: >
      The design, in sections the author names: Solution, Design
      decisions, Architecture, the component being specified, whatever the
      feature needs. This catch-all is why the section list is unordered —
      a spec arranges its own argument, and the corpus shows fifty-odd
      distinct section names doing exactly that. The named sections above
      still win over this one wherever they appear, so their content
      constraints and their spelling are still checked.

example: |
  ---
  type: Spec
  id: spec-001-pack-source-allowlist
  title: Pack source transport allowlist
  description: A pack source may only use https, ssh, or file transports.
  status: draft
  ---

  # Summary

  superdev refuses to fetch a content pack over a transport that is not
  https, ssh, or file, so a cloned manifest cannot pull content over an
  unauthenticated channel.

  # Behaviour

  - When a manifest names an https, ssh, or file pack source, sync
    resolves it as before.
  - When a manifest names any other transport, the command fails at parse
    with an error naming the offending source.

  # Acceptance criteria

  1. Given a manifest with a git:// pack source, when sync runs, then it
     exits non-zero naming the source and fetches nothing.
  2. Given a manifest with an https pack source, when sync runs, then the
     pack resolves as before.

  # Edge cases & errors

  - Scheme in mixed case → treated case-insensitively, same refusal.

  # Out of scope

  - Proxying or credential storage.

  # Test plan: pack source transport allowlist

  ## Scope

  - Under test: source scheme parsing and refusal.
  - Not under test: git fetch mechanics (covered by resolver tests).

  ## Risks driving this plan

  1. A refused scheme still normalising onto the base-pack identity.

  ## Test cases

  ### Automated

  | # | Case | Type | Inputs / setup | Expected result |
  |---|------|------|----------------|-----------------|
  | 1 | git:// source refused | unit | manifest with git:// source | parse error naming source |
  | 2 | https source resolves | unit | manifest with https source | source resolves as before |

  ### Manual verification

  1. Run sync against a manifest with an http:// source; observe refusal.

  ## Exit criteria

  - All automated cases pass in CI.
````
