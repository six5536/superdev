---
type: Schema
id: schema-adhoc-plan
title: Ad-hoc Plan Schema
description: Implementation plans for one-off work outside the feature workflow, filed in knowledge/plans/.
---

# Ad-hoc Plan Schema

Structural rules for the ad-hoc plans filed at
`knowledge/plans/plan-{nnn}-adhoc-{slug}.md` and listed in that directory's
`index.md`. For a feature going through the workflow, `schema-feature-plan`
applies instead: it is a different concept with its own type, sharing this
directory and this number series, produced by the feature-plan phase and read
by build, verify, and integrate.

The document separates what is known from what is intended: Facts carry
evidence, Requirements and Decisions carry intent, Acceptance and Definition
of done carry the test. Ids run through the document — an outcome is named
`O1`, a requirement `FR-1`, a constraint `NFR-1`, a decision `D-1`, a
workstream `W1` — so a later section can point at an earlier one instead of
restating it.

````yaml
description: >
  Ad-hoc implementation plan for one-off work outside the feature
  workflow — what is true, what is being built, in what order, and what
  must hold before the work is done.
line-limit: 800
frontmatter:
  type:
    const: AdhocPlan
  id:
    pattern: '^plan-\d{3}-adhoc-[a-z0-9-]+$'
  status:
    enum: [draft, active, done, abandoned]
    description: >
      draft while the plan is still being written; active once work has
      started against it; done when every Definition of done item holds;
      abandoned when the work was dropped, with the reason recorded in
      Out-of-band notes. The plan records no finer progress than this —
      per-step state belongs to the tracker, not to the document.

sections-ordered: true
sections:
  - heading-pattern: "^Plan: .+$"
    level: 1
    required: true
    description: >
      "Plan:" followed by the short title of the task, matching the
      frontmatter title.

  - heading: "Context"
    level: 2
    required: true
    content: prose
    description: >
      1-3 sentences: the problem this solves, why it is being solved now,
      and the constraint that shaped the approach. Link the request or
      issue if one exists. Nothing about the fix — that is Workstreams.

  - heading: "Facts"
    level: 2
    required: true
    content: bullet-list
    description: >
      What was verified before planning, one bullet each, each carrying
      its evidence: a path/to/file.ts:123 reference, a command and its
      result, or a link. Anything believed but not checked is an Open
      question instead. This is the current state of the system with a
      citation attached to every claim.

  - heading: "Goal"
    level: 2
    required: true
    content: prose
    description: >
      One sentence naming what is true when the plan is done and is not
      true today. Not a task list, and not Context restated.

  - heading: "Outcomes"
    level: 2
    required: true
    content: bullet-list
    description: >
      The observable end states the Goal decomposes into, one bullet
      each, phrased so a reader could confirm one for themselves. Each
      opens with an id — O1, O2 — so Requirements can point at it.

  - heading: "Non-goals"
    level: 2
    required: true
    content: bullet-list
    description: >
      What is deliberately out of scope, one bullet each, so a reviewer
      sees it was excluded rather than forgotten. Give the reason, or
      name where it is handled instead.

  - heading: "Requirements"
    level: 2
    required: true
    description: >
      Carries the two requirement tables and no prose of its own.

  - heading: "Functional"
    level: 3
    required: true
    content: table
    columns: [ID, Requirement, Outcome]
    description: >
      ID is FR-1, FR-2. Requirement is one testable statement of
      behaviour in the present tense. Outcome is the O-id it serves; a
      requirement serving none is scope that crept in.

  - heading: "Non-functional"
    level: 3
    content: table
    columns: [ID, Constraint, Budget]
    description: >
      ID is NFR-1, NFR-2. Constraint is the quality being held — latency,
      memory, compatibility, security. Budget is the number or bound that
      makes it checkable; without one it is an aspiration and belongs in
      Context. Omit the section when the work carries none.

  - heading: "Decisions"
    level: 2
    required: true
    content: table
    columns: [ID, Decision, Alternative, Why]
    description: >
      ID is D-1, D-2. One row per choice a reader could reasonably have
      made differently. Alternative is the strongest option not taken,
      Why the single reason it lost. A decision later reversed stays in
      the table with the reversal noted, so the reasoning survives.

  - heading: "Workstreams"
    level: 2
    required: true
    description: >
      Carries one subsection per workstream and no prose of its own.

  - heading-pattern: '^W\d+: .+$'
    level: 3
    required: true
    repeatable: true
    content: numbered-list
    description: >
      A slice of the work that can be carried to completion on its own.
      The heading is the W-id and the workstream name. The first line is
      "Depends on:" naming the W-ids that must land first, or none. Then
      the ordered steps: step name — what changes, in which files, and
      why it comes before the next. Mark any step that is hard to
      reverse and say what makes it so.

  - heading: "Files affected"
    level: 2
    required: true
    content: table
    columns: [File, Change, Workstream]
    description: >
      One row per file the plan touches. Change is new, modified or
      deleted, with a one-line description. Workstream is the W-id that
      touches it; a file claimed by two workstreams is a sign the split
      is wrong. Every file named in a step appears here, including the
      ones the plan creates and deletes.

  - heading: "Acceptance"
    level: 2
    required: true
    content: table
    columns: [Check, Verifies]
    description: >
      Check is the exact command to run or the observation to make,
      copy-pasteable, with the result that counts as a pass. Verifies is
      the FR, NFR or O id it covers. Every FR appears in at least one
      row; an FR nothing checks is a wish.

  - heading: "Definition of done"
    level: 2
    required: true
    content: bullet-list
    description: >
      The gate for the plan as a whole — everything beyond the Acceptance
      rows passing that must hold before status becomes done: docs
      updated, index entry changed, migration run, follow-ups filed.
      Each bullet is checkable by someone who did not do the work.

  - heading: "Risks"
    level: 2
    required: true
    content: bullet-list
    description: >
      What could go wrong, one bullet each, with the mitigation or the
      early signal that it is happening. A risk with neither is an Open
      question.

  - heading: "Open questions"
    level: 2
    content: bullet-list
    description: >
      A decision needed from the user, one bullet each, with the
      recommended default and what it blocks. Omit the section when
      there are none rather than leaving it holding the word "None".

  - heading: "Out-of-band notes"
    level: 2
    content: prose
    description: >
      What lands outside the code: migrations, follow-up work, docs to
      update, the reason an abandoned plan was abandoned. Omit the
      section when empty.

  - heading: "Appendix"
    level: 2
    description: >
      Carries the free sections and no prose of its own. Omit the section
      when there are none.

  - heading-pattern: "^.+$"
    level: 3
    repeatable: true
    description: >
      A free section, named and shaped by the author: a transcript, a
      table of measurements, a derivation, a log excerpt. Free sections
      live here and nowhere else. The pattern dialect has no lookaround,
      so a heading invented elsewhere in the document cannot be told
      apart from a mistyped one — collecting them under Appendix is what
      makes both checkable.

sections-prohibited:
  - "Summary"
  - "Overview"
  - "Introduction"
  - "Conclusion"
  - "Status"
  - "Progress"
  - "Timeline"
  - "Estimates"
  - "Current state"
  - "Proposed approach"
  - "Testing & verification"
  - "Risks & open questions"

example: |
  ---
  type: AdhocPlan
  id: adhoc-plan-002-scheme-match-cleanup
  title: Consolidate pack source scheme matching
  description: One helper owns pack source scheme matching, and both call sites use it.
  status: draft
  ---

  # Plan: Consolidate pack source scheme matching

  ## Context

  Pack source scheme matching is implemented twice, once in resolve and
  once in manifest validation, and the two copies have drifted far
  enough that a manifest can validate and then fail to resolve. Fixing
  the drift in place would leave the second copy free to drift again.

  ## Facts

  - `crates/lib/superdev-core/src/pack/resolve.rs:88-140` matches the
    scheme inline and accepts `file`, `git`, `https`.
  - `crates/lib/superdev-core/src/pack/manifest.rs:210-247` accepts
    those three plus `http`, added in `abc123f` and never mirrored into
    resolve.
  - `cargo test -p superdev-core pack::` passes on main, 61 tests, so
    neither copy is untested — they are tested against different
    expectations.
  - Nothing outside `superdev-core` matches a scheme by hand:
    `rg 'scheme' crates --type rust` returns hits only under
    `src/pack`.

  ## Goal

  Scheme matching has one implementation, and a manifest that validates
  resolves.

  ## Outcomes

  - O1 — one helper decides whether a scheme is accepted, and both call
    sites call it.
  - O2 — `http` is treated identically by validation and resolve.

  ## Non-goals

  - Changing which transports are allowed. Whether `http` should be
    accepted at all is a behaviour change; it is in Open questions.
  - The pack cache layer, which reads schemes but never matches them.

  ## Requirements

  ### Functional

  | ID | Requirement | Outcome |
  |----|-------------|---------|
  | FR-1 | `scheme::accepts` is the only place a pack source scheme is matched | O1 |
  | FR-2 | Manifest validation rejects exactly the schemes resolve rejects | O2 |

  ### Non-functional

  | ID | Constraint | Budget |
  |----|------------|--------|
  | NFR-1 | The matcher stays allocation-free on the resolve hot path | zero heap allocations per call |

  ## Decisions

  | ID | Decision | Alternative | Why |
  |----|----------|-------------|-----|
  | D-1 | A new `scheme.rs` owns the matcher | Export resolve's copy and call it from validation | validation would then depend on resolve for a rule neither owns |
  | D-2 | `http` stays accepted for now | Drop it as part of this change | it is a behaviour change with its own blast radius, and this plan is a refactor |

  ## Workstreams

  ### W1: Extract the helper

  Depends on: none.

  1. Add the module — new `scheme.rs` with `accepts`, covering the
     union of both copies' schemes, so neither call site loses a case
     when it switches.
  2. Point resolve at it — delete the inline match; resolve's existing
     tests now exercise the helper.

  ### W2: Adopt in validation

  Depends on: W1.

  1. Switch validation — call `accepts` and delete the local table.
     Hard to reverse: that table is the only record of validation's
     scheme set, so this step and the next land in one commit.
  2. Assert the two agree — one test feeding the same scheme list
     through both paths.

  ## Files affected

  | File | Change | Workstream |
  |------|--------|------------|
  | `crates/lib/superdev-core/src/pack/scheme.rs` | new — the matcher and its unit tests | W1 |
  | `crates/lib/superdev-core/src/pack/mod.rs` | modified — declare the module | W1 |
  | `crates/lib/superdev-core/src/pack/resolve.rs` | modified — call the helper; inline match deleted | W1 |
  | `crates/lib/superdev-core/src/pack/manifest.rs` | modified — call the helper; local table deleted | W2 |
  | `crates/lib/superdev-core/tests/scheme_agreement.rs` | new — both paths refuse the same schemes | W2 |

  ## Acceptance

  | Check | Verifies |
  |-------|----------|
  | `rg 'https' crates/lib/superdev-core/src/pack --type rust` returns hits only in `scheme.rs` | FR-1 |
  | `cargo test -p superdev-core pack::` passes, 61 tests plus the new ones | FR-2 |
  | `cargo test -p superdev-core --test scheme_agreement` passes | FR-2, O2 |
  | `cargo bench -p superdev-core resolve` stays within noise of the main baseline | NFR-1 |

  ## Definition of done

  - Every Acceptance row passes on a clean checkout of the branch.
  - `knowledge/plans/index.md` lists this plan, and its status reads done.
  - The `http` question from D-2 is filed as its own plan or issue and
    linked below.

  ## Risks

  - Risk: W1's union briefly makes resolve accept `http` — mitigation:
    W1 and W2 land in one pull request, so no release sees the widened
    set.
  - Risk: a caller matches schemes by hand somewhere `rg` missed —
    early signal: the agreement test fails on a scheme neither author
    expected.

  ## Open questions

  - Should `http` be accepted at all? Recommended default: keep it, per
    D-2, and decide in a follow-up. Blocks nothing here.

  ## Out-of-band notes

  The pack authoring guide lists accepted schemes in prose. It stays
  correct under this plan and needs updating only when the set changes.

  ## Appendix

  ### Scheme sets as of main

  | Scheme | resolve | validation |
  |--------|---------|------------|
  | `file` | yes | yes |
  | `git` | yes | yes |
  | `https` | yes | yes |
  | `http` | no | yes |
````
