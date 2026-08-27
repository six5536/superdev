---
type: Schema
id: schema-spec
title: Spec Schema
description: Feature specs filed in knowledge/specs/ — the spec body plus the appended test plan.
---

# Spec Schema

Structural rules for feature specs filed at
`knowledge/specs/spec-{nnn}-{feature-slug}.md` and listed in
`knowledge/specs/index.md`. The test plan is appended to the spec as further
sections.

````yaml
target-files: "knowledge/specs/spec-*.md"
description: >
  Feature specification: what done looks like from outside — observable
  behaviour, acceptance criteria, UI states, edge cases, and what is out
  of scope — with the test plan appended as further sections. Say what it
  does, never how.
line-limit: 800

frontmatter:
  type:
    const: Spec
  id:
    pattern: '^spec-\d{3}-[a-z0-9-]+$'
  status:
    enum: [draft, stable, deprecated]
    description: draft until accept tags it done.

sections-ordered: true
sections:
  - heading: "Summary"
    level: 1
    required: true
    content: prose
    description: >
      One or two sentences: what the feature does and for whom, as a user
      or caller would describe it. No implementation.
  - heading: "Behaviour"
    level: 1
    required: true
    content: bullet-list
    description: >
      The feature from outside — what a user sees or a caller gets, stated
      as observable facts. "When X, the system does Y." Each line something
      a tester could watch happen. Bullet list.
  - heading: "Acceptance criteria"
    level: 1
    required: true
    content: numbered-list
    description: >
      Numbered and walkable: each one checkable as pass/fail without
      interpretation. Given/When/Then where it helps.
  - heading: "UI states"
    level: 1
    content: bullet-list
    description: >
      For UI features, the list of states is most of the spec — Empty,
      Loading, Populated, Error, Edge. Delete this section for non-UI work.
  - heading: "Edge cases & errors"
    level: 1
    required: true
    content: bullet-list
    description: >
      Inputs and situations that must be handled, with the expected
      behaviour for each — invalid input, limits, concurrency, offline,
      permission denied. Format: - Case → expected behaviour.
  - heading: "Out of scope"
    level: 1
    required: true
    content: bullet-list
    description: >
      Adjacent behaviour deliberately excluded, so nobody assumes it was
      forgotten. Bullet list.
  - heading: "Open questions"
    level: 1
    content: bullet-list
    description: >
      Behavioural decisions still unmade, each with a recommended answer
      and who decides. Delete if none.
  - heading-pattern: '^Test plan: .+$'
    level: 1
    required: true
    description: >
      The appended test plan for the feature under test. Scope, risks
      driving the plan, automated and manual cases, regression coverage,
      and exit criteria.
  - heading: "Scope"
    level: 2
    required: true
    content: bullet-list
    description: "Under test / not under test, with why for exclusions."
  - heading: "Risks driving this plan"
    level: 2
    required: true
    content: numbered-list
    description: >
      The 2-4 ways this change is most likely to break — the plan
      should visibly attack these. Numbered list.
  - heading: "Test cases"
    level: 2
    required: true
    description: >
      The cases, split by how they are run: automated under Automated,
      step-by-step manual checks under Manual verification. UI is still
      tested.
  - heading: "Automated"
    level: 3
    required: true
    content: table
    columns: ["#", Case, Type, "Inputs / setup", "Expected result"]
    description: >
      One row per automated case. Type is the level it runs at — unit,
      integration, e2e. The case numbers are what a feature plan's slices
      claim in their Cases lines.
  - heading: "Manual verification"
    level: 3
    required: true
    content: numbered-list
    description: >
      Step-by-step checks a person runs, in order, each with what they
      should see. Numbered so a slice can claim "manual 1".
  - heading: "Regression coverage"
    level: 2
    content: bullet-list
    description: >
      Existing tests that must keep passing; areas adjacent to the
      change worth a smoke check.
  - heading: "Environments / data"
    level: 2
    content: bullet-list
    description: >
      Required services, fixtures, env vars, seeded data. How to set
      them up.
  - heading: "Exit criteria"
    level: 2
    required: true
    content: bullet-list
    description: >
      All automated cases pass in CI; manual checks signed off; known
      gaps accepted and listed.

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
