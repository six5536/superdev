---
type: Schema
id: schema-plan
title: Plan Schema
description: The canonical implementing record for one issue, carrying approved scope, stable work blocks, durable BUILD state, decisions, evidence, and acceptance.
---

# Plan Schema

A plan is the readable durable state of one SCOPE → BUILD → ACCEPT workflow.
Every plan links to exactly one primary issue with `rel: implements`; incoming
relationships provide the issue's plan history. Open plans are mutable only by
the phase owner declared below. Done and abandoned plans are immutable history.

SCOPE owns sections 1–8. The Rust workflow service alone updates completion and
evidence fields in Work blocks, Build state, and Completion evidence. BUILD may
append Implementation decisions and Follow-up issues. Re-scoping may revise the
approved specification but never erase execution history.

A work block has a stable number and title, done checkbox, dependencies,
affected areas, required outcome, focused commands, tests bound to contract
keys where applicable, structural evidence otherwise, and documentation
surfaces with generation and verification commands. Active evidence is executable;
completed migrated records may preserve old manual-case text only under an explicit
`Legacy evidence (non-executable)` marker.

````yaml
description: >
  One issue's approved scope, stable work blocks, durable progress, executable
  evidence, review, acceptance, and integration history.
line-limit: 1500
variant-key: lifecycle
frontmatter:
  type:
    required: true
    const: Plan
  id:
    required: true
    pattern: '^plan-\d{3}-[a-z0-9-]+$'
  title:
    required: true
  description:
    required: true
  lifecycle:
    required: true
    enum: [open, done, abandoned]
  phase:
    required: true
    enum: [scope, build, accept, done, abandoned]
  branch:
    required: true
    pattern: '^work/\d{3}-[a-z0-9-]+$'
  links:
    required: true
sections-ordered: true
sections:
  - heading-pattern: '^Plan: .+$'
    level: 1
    required: true
    description: Title heading matching the plan title.
  - heading: "Goal and boundaries"
    level: 2
    required: true
    content: prose
    description: Desired outcome, current gap, constraints, exclusions, and evidence.
  - heading: "Requirements"
    level: 2
    required: true
    content: prose
    description: Settled user intent and non-contract requirements, with no unresolved questions.
  - heading: "Contract changes"
    level: 2
    required: true
    content: bullet-list
    description: Complete keyed contract changes, or the single item none.
  - heading: "ADR decisions"
    level: 2
    required: true
    content: bullet-list
    description: Decisions created, changed, superseded, or the single item none.
  - heading: "Source and interface changes"
    level: 2
    required: true
    content: prose
    description: Source declarations and expected materialized interface definitions.
  - heading: "Knowledge changes"
    level: 2
    required: true
    content: prose
    description: Approved normative edits and BUILD-owned current-state edits.
  - heading: "Documentation changes"
    level: 2
    required: true
    content: prose
    description: Every applicable declared surface and its owner and commands, or checked none evidence.
  - heading: "Work blocks"
    level: 2
    required: true
    description: >
      Dependency-ordered stable blocks with no prose outside their subsections.
      A block closing a contract-implementation gap comes before independent
      blocks, then remaining blocks are ordered by dependency and risk.
  - heading-pattern: '^Block \d+: .+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: Stable block carrying completion, dependencies, outcomes, executable evidence, and documentation.
  - heading: "Build state"
    level: 2
    required: true
    content: prose
    description: Machine-maintained current block, attempts, fingerprints, correction count, and blocker state.
  - heading: "Implementation decisions"
    level: 2
    required: true
    content: prose
    description: Concise BUILD-time local choices with block, reason, and affected paths, or none.
  - heading: "Follow-up issues"
    level: 2
    required: true
    content: prose
    description: Links to separately filed unrelated work, or none.
  - heading: "Completion evidence"
    level: 2
    required: true
    content: prose
    description: Scope approval, block checks, candidate verification and review, acceptance, and integration evidence.
example:
  open: |
    ---
    type: Plan
    id: plan-041-pack-source-allowlist
    title: Pack source transport allowlist
    description: Refuse unsupported pack transports before invoking Git.
    lifecycle: open
    phase: build
    branch: work/041-pack-source-allowlist
    links:
      - rel: implements
        to: issue-041-pack-source-allowlist
    ---

    # Plan: Pack source transport allowlist

    ## Goal and boundaries

    Unsupported transports fail before Git. Pack caching is excluded.

    ## Requirements

    Preserve supported HTTPS, SSH, and file behavior and never invoke a shell.

    ## Contract changes

    - contract-004: add `P_scheme-refused` and `AC_git-refused`.

    ## ADR decisions

    - none.

    ## Source and interface changes

    Update the manifest source parser; no generated interface shape changes.

    ## Knowledge changes

    Update configuration after behavior is implemented.

    ## Documentation changes

    README is not triggered because pack transport is not documented there;
    canonical-knowledge is owned by Block 1 and checked with `superdev validate`.

    ## Work blocks

    ### Block 1: Parse and refuse unsupported schemes

    - [ ] Done.
    - Dependencies: none.
    - Areas: `crates/lib/superdev-core/src/pack` and configuration knowledge.
    - Outcome: unsupported schemes fail before Git.
    - Verification: `cargo test -p superdev-core pack::`.
    - Tests: `rejects_git_scheme` covers contract-004 AC_git-refused.
    - Structural evidence: Git is invoked with argument arrays and no shell.
    - Documentation: canonical-knowledge; run `cargo run -- validate --fix` then `cargo run -- validate`.

    ## Build state

    Current block: 1. Attempts: 0. Final corrections: 0. Blocker: none.

    ## Implementation decisions

    none.

    ## Follow-up issues

    none.

    ## Completion evidence

    Scope review and approval are recorded by the scope transition commit; BUILD evidence is pending.
  done: |
    ---
    type: Plan
    id: plan-041-pack-source-allowlist
    title: Pack source transport allowlist
    description: Refuse unsupported pack transports before invoking Git.
    lifecycle: done
    phase: done
    branch: work/041-pack-source-allowlist
    links:
      - rel: implements
        to: issue-041-pack-source-allowlist
    ---

    # Plan: Pack source transport allowlist

    ## Goal and boundaries

    Unsupported transports fail before Git. Pack caching is excluded.

    ## Requirements

    Preserve supported transports and never invoke a shell.

    ## Contract changes

    - contract-004: added `P_scheme-refused` and `AC_git-refused`.

    ## ADR decisions

    - none.

    ## Source and interface changes

    The manifest parser changed; no generated interface shape changed.

    ## Knowledge changes

    Configuration is current.

    ## Documentation changes

    canonical-knowledge was generated and checked.

    ## Work blocks

    ### Block 1: Parse and refuse unsupported schemes

    - [x] Done.
    - Dependencies: none.
    - Areas: manifest parser and configuration.
    - Outcome: unsupported schemes fail before Git.
    - Verification: `cargo test -p superdev-core pack::` passed.
    - Tests: `rejects_git_scheme` covers contract-004 AC_git-refused.
    - Structural evidence: no shell invocation exists.
    - Documentation: canonical-knowledge generation and validation passed.

    ## Build state

    All blocks complete; attempts 1; final corrections 0; blocker none.

    ## Implementation decisions

    none.

    ## Follow-up issues

    none.

    ## Completion evidence

    Candidate verification and isolated review passed; configured acceptance passed and the closure was integrated.
  abandoned: |
    ---
    type: Plan
    id: plan-041-pack-source-allowlist
    title: Pack source transport allowlist
    description: Refuse unsupported pack transports before invoking Git.
    lifecycle: abandoned
    phase: abandoned
    branch: work/041-pack-source-allowlist
    links:
      - rel: implements
        to: issue-041-pack-source-allowlist
    ---

    # Plan: Pack source transport allowlist

    ## Goal and boundaries

    The desired refusal was not integrated; partial product work was excluded.

    ## Requirements

    Preserve the approved requirement as issue history.

    ## Contract changes

    - none.

    ## ADR decisions

    - none.

    ## Source and interface changes

    none.

    ## Knowledge changes

    Approved knowledge disposition only.

    ## Documentation changes

    none; no behavior changed.

    ## Work blocks

    ### Block 1: Preserve disposition

    - [x] Done.
    - Dependencies: none.
    - Areas: issue and plan.
    - Outcome: abandonment is recorded without product changes.
    - Verification: `superdev validate`.
    - Tests: none; structural knowledge-only evidence applies.
    - Structural evidence: default branch contains no partial product diff.
    - Documentation: canonical-knowledge validation passed.

    ## Build state

    Terminal abandonment; blocker none.

    ## Implementation decisions

    none.

    ## Follow-up issues

    none.

    ## Completion evidence

    Human abandonment approval and knowledge-only integration are recorded.
````
