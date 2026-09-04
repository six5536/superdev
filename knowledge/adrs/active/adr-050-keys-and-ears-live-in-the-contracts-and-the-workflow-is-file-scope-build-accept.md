---
type: Decision
id: adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept
title: Keys and EARS live in the contracts, and the workflow is file, scope, build, accept
description: Keys and EARS are demanded of contracts alone, whose promises may nest keyed criteria; an issue is one plain template with kind and an open, done or wontfix lifecycle; a plan is one document holding the contract changes and the work blocks; the workflow is file, scope — which writes the plan and makes the contract changes through its sub-skills — build, which runs each block's tests and the full suite once, and an optional manual accept carrying the code review; the issues and plans on file are rewritten.
lifecycle: active
links:
  - rel: supersedes
    to: adr-031-ears-criteria-are-checked-by-item-pattern
    note: EARS criteria on the issue; a criterion lives on the contract now.
  - rel: supersedes
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: The framed state, the per-state schemas and the gates on it; an issue is open, done or wontfix.
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: Its contract half stands; its tracker half — keys on an issue's lists — is withdrawn here.
  - rel: references
    to: adr-051-a-section-rule-declares-nested-items-and-an-optional-key
    note: The declarations a promise's nested criteria use.
---

# ADR-050: Keys and EARS live in the contracts, and the workflow is file, scope, build, accept

- Date: 2026-09-04
- Deciders: superdev maintainers

## Context

The product is described by the user's needs, the ADRs, the contracts
and the code. Every document beyond those restates one of them and
drifts. By September 2026 the workflow had grown a framed issue whose
keyed EARS criteria
([ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern],
[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items])
no test binds, three tracker kinds in two states
each
([ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]), a feature plan and an ad-hoc plan holding one kind of design,
and a verification pass with a review on every slice. Framing I052 the
first time spent four interviews on where a key sits in an issue.

## Decision

Keys and EARS are demanded of contracts alone. A promise under
Behaviour or Stability MAY carry a nested bullet list of `AC_`
criteria
([ADR-051][sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key]),
each keyed and tagged as a promise is, the key unique across
the contract; a plan case and a test cite a criterion by key. A
promise with no criterion is its own check. The grammar's keyed and
nested item support stays for any schema that wants it.

An issue is one template: `kind` in `bug`, `feature`, `chore`;
`lifecycle` in `open`, `done`, `wontfix`; Summary, Context, Behaviour,
Scope, Resolution and Comments; prose and bullets, no key, no tag, no
`TBD` rule. `/file` writes one. The three tracker schemas retire and
the issues on file are rewritten, `framed` and `unframed` becoming
`open`.

A plan is one template — Goal, Contract changes, Work blocks, Deferred
decisions — replacing the feature plan and the ad-hoc plan; the plans
on file are rewritten.

The workflow is FILE → SCOPE → BUILD → ACCEPT. `/scope` replaces
`/frame`, `/contract-design`, `/feature-plan` and `/adhoc-plan`: it
creates the branch, interviews where the design is open, makes the
contract changes, writes the plan and commits both, calling sub-skills
for the parts that are their own — `/grill-me`, `/contract-design`,
`/research`, `/design`, `/prototype`, `/double-check` — none of them a
phase. `/build` works the blocks — tests, code, the block's own tests
— with no review, and after the last block runs
the full build, tests, lint and validate once, updates the changelog
and the knowledge, and merges on the branch; `/integrate` retires.
`/execute-plan` drives `/build` unattended. `/accept` is manual and
optional, the last step: the code review, the contract criteria on the
merged code, the documentation; it closes the issue.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Keys and EARS in contracts alone; one issue template; one plan; build verifies once; accept optional | Every keyed item is bound by a test; one record per thing; the design is written once | 51 issues and 36 plans rewritten; five schemas and five skills retire |
| Keyed EARS behaviour on the issue with nested criteria | A disciplined issue | The issue is ephemeral; the keys bind nothing |
| A requirements document per capability | A durable user's view | A fourth description of the product to keep in step |
| Frame, contract-design and plan as three phases | The problem stated before the design | Two documents and three turn boundaries holding one design |
| `/plan` as the phase's name | Says what it writes | Claude Code carries a plan skill of its own |
| Contract-design retired outright | One skill fewer | The contract interview and the ADRs are a piece of work with its own persona, called from scope |
| Verification per slice | A fault found at its slice | Six full passes where the last decides |
| An automatic review per feature | Findings without asking | The review that matters is the one a human asks for |
| Three issue kinds, keys removed | Familiar | Three schemas for one record |

## Consequences

- Positive: an agent files an issue in prose; the keyed vocabulary is
  where tests bind it; a feature has one design document and one full
  verification; the workflow has three fewer phases.
- Negative: every issue and plan on file changes; a criterion that
  was cited by issue and key is cited by contract and key; a fault a
  per-slice pass would have caught early surfaces at the end.
- Follow-ups: the issue and plan schemas; the contract schema's nested
  criteria; the skills; the workflow text; `LIVE_LIFECYCLES`; the
  sweep; the concepts, glossary, README and changelog.

<!-- sokf:links -->
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/deprecated/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/deprecated/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
[sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key]: /knowledge/adrs/active/adr-051-a-section-rule-declares-nested-items-and-an-optional-key.md
