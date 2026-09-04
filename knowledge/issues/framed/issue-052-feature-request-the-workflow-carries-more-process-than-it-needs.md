---
type: FeatureRequest
id: issue-052-feature-request-the-workflow-carries-more-process-than-it-needs
title: The workflow carries more process than it needs — keyed EARS issues, a framed state, and a verification pass per slice
description: Keys and EARS are demanded of issues, where no test binds them, and of three tracker kinds with two states each; a feature passes through five phases with a full verification and a code review on every slice; the product is described by the user's needs, the ADRs, the contracts and the code, and every document beyond those is text that drifts — so keys and EARS retreat to the contracts, which gain keyed criteria, issues become one plain template with open, done and wontfix, frame, contract-design and the two plan kinds become one scope phase that writes the plan and makes the contract changes, build runs the full suite once at the end, and accept becomes the manual, optional last step carrying the code review.
lifecycle: framed
links:
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: Keyed the contracts and the tracker's lists alike; the tracker half is withdrawn.
  - rel: references
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: The framed state, the three per-state schemas and the gates on it; withdrawn — an issue is open, done or wontfix.
  - rel: references
    to: adr-031-ears-criteria-are-checked-by-item-pattern
    note: EARS criteria on the issue; withdrawn — a criterion lives on the contract.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Gains `nested` and `item-key-optional`, the declarations behind a promise's nested criteria (ADR-051); built by plan-027 slice 1.
  - rel: references
    to: adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept
    note: The decision this feature delivers.
  - rel: references
    to: adr-051-a-section-rule-declares-nested-items-and-an-optional-key
    note: The schema mechanism behind a promise's nested criteria.
---

# Feature: the workflow carries more process than it needs

## Summary

An agent filing an issue is held to keys and EARS that no test binds,
across three schemas with two states each; a feature crosses five
phases, each slice verified in full and reviewed; and the framed issue
and the plan restate what the contracts and the code then say again.
The product is described by the user's needs, the ADRs, the contracts
and the code. Every document beyond those is text that drifts.

## Motivation

[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
and
[ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]
put keys and EARS on every cited list of the tracker and split every
issue into an unframed and a framed form, so the three tracker schemas
now carry seven per-state rules and four examples each, and framing
I052 the first time spent four interviews on where a key sits in an
issue. A contract's promise is bound by a test; an issue's criterion
is bound by nothing, and is copied into a plan case and a test by
hand. Each slice runs the full build, tests, lint and a review before
the next begins, so a six-slice feature verifies six times what the
last slice verifies once. `/frame`, `/contract-design`, `/feature-plan` and `/adhoc-plan`
are four phases producing two documents that hold one design.

## Proposed behaviour

Keys and EARS live in the contracts alone, and a contract carries its
criteria. A promise under Behaviour or Stability MAY carry a nested
bullet list of the `AC_` criteria that check it — each keyed and
tagged as a promise is, the key unique across the contract with the
`P_` keys — so a plan case and a test cite the criterion by key: bare
where the contract is the subject, `<contract id> AC_<slug>`
elsewhere. A promise with no criterion is its own check. The schema
grammar's support for keyed and nested items stays, whether or not an
issue schema uses it.

An issue is one document kind, `schema-issue`, id `issue-<nnn>-<slug>`,
with `kind` one of `bug`, `feature` and `chore` and `lifecycle` one of
`open`, `done` and `wontfix`, the folder the value. Its headings are
Summary, Context, Behaviour, Scope, Resolution and Comments: Summary
says what and for whom; Context says why now, with the evidence, the
environment and the reproduction for a bug; Behaviour says what is
expected and what happens for a bug, and what is proposed for a
feature, in prose and bullets; Scope draws the boundary; Resolution
appears once the issue is done or wontfix; Comments append. No key, no
EARS tag and no `TBD` rule holds an issue. `/file` writes one from the
user's words. The bug-report, feature-request and chore schemas retire
and the 51 issues on file are rewritten to the template, every
`framed` and `unframed` issue becoming `open`.

A plan is one document kind, `schema-plan`, id `plan-<nnn>-<slug>`,
replacing the feature plan and the ad-hoc plan: a Goal; the Contract
changes — each contract touched and the promises and criteria added,
changed or withdrawn, or "none"; the Work blocks, each with the blocks
it depends on, a done-check and its cases, a case citing the contract
criteria it covers where one exists; and the Deferred decisions. The
36 plans on file are rewritten to it.

The workflow is FILE → SCOPE → BUILD → ACCEPT, the last optional:

- `/file` files an issue or an idea, as today.
- `/scope` replaces `/frame`, `/contract-design`, `/feature-plan` and
  `/adhoc-plan`: given an issue, or a one-off piece of work, it
  creates the branch — `feature/<nnn>-<slug>` after the issue,
  `adhoc/<nnn>-<slug>` after the plan where there is no issue —
  interviews the user where the design is open, makes the contract
  changes, writes the plan and commits both. It calls sub-skills for
  the parts that are their own: `/grill-me` for the open decisions,
  `/contract-design` for the contract changes and their ADRs where the
  plan names any, `/research` for an external fact, `/design` or
  `/prototype` for UI, `/double-check` before the commit. None of them
  is a phase.
- `/build` works the blocks in order: tests, then code, then the
  block's own tests and the tests it touches, and no review. After the last block it runs the full build, tests, lint and
  `superdev validate` once, updates the changelog and the knowledge,
  and merges on the branch. `/integrate` retires.
- `/execute-plan` replaces `/execute-feature-plan` and drives `/build`
  over the blocks unattended, with `superdev run` and the Stop hook
  unchanged and a deferred decision recorded as today.
- `/accept` is manual and optional, the last step: a code review of
  the whole change, returning to `/build` on a finding the user wants
  fixed; the contract criteria walked on the merged code; the user
  documentation checked. It closes the issue.

The `.agents/superdev.md` workflow, the definition of done, the
development procedure, the tracker concept, the glossary, the README
and the changelog say the same. ADR-046's tracker half,
[ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]
and [ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern]
are superseded by
[ADR-050][sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept];
[contract-010][sokf:contract-010-interface-document-schemas] carries
the nested-item declarations
([ADR-051][sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key]).

## Acceptance criteria

1. `AC_contract-criteria` [ubiquitous] THE SYSTEM SHALL accept a
   contract whose promise carries a nested bullet list of `AC_` items,
   each opening with its key and its EARS tag, and report an error
   naming the item for a nested item without them, or a key used twice
   across the contract's `P_` and `AC_` keys.
2. `AC_contract-criteria-optional` [ubiquitous] THE SYSTEM SHALL
   accept a contract whose promises carry no nested list.
3. `AC_issue-schema` [ubiquitous] THE SYSTEM SHALL ship one issue
   schema with `kind` in `bug`, `feature`, `chore` and `lifecycle` in
   `open`, `done`, `wontfix`, the headings Summary, Context,
   Behaviour, Scope, Resolution and Comments, and no `item-key`,
   `item-pattern` or nested declaration on any of them.
4. `AC_issue-plain` [ubiquitous] THE SYSTEM SHALL accept an issue whose
   Behaviour is prose, bullets or both, with no key and no tag.
5. `AC_issue-resolution` [state] WHILE an issue is `done` or `wontfix`
   THE SYSTEM SHALL require its Resolution heading, and WHILE it is
   `open` refuse one.
6. `AC_old-kinds-gone` [ubiquitous] THE SYSTEM SHALL ship no
   bug-report, feature-request, chore, feature-plan or adhoc-plan
   schema, and report a document whose `type` names one.
7. `AC_issue-sweep` [ubiquitous] THE SYSTEM SHALL carry every issue on
   file in the issue template under `issues/open/`, `issues/done/` or
   `issues/wontfix/`, with `superdev validate` passing.
8. `AC_plan-schema` [ubiquitous] THE SYSTEM SHALL ship one plan schema
   with the headings Goal, Contract changes, Work blocks and Deferred
   decisions, and carry every plan on file in it with `superdev
   validate` passing.
9. `AC_scope-skill` [ubiquitous] THE SYSTEM SHALL ship a `/scope`
   skill that creates the branch, writes the plan from an issue or a
   one-off request, calls `/contract-design` for the plan's contract
   changes where it names any, commits both and hands to `/build`, and
   ship no `/frame`, `/feature-plan` or `/adhoc-plan` skill.
10. `AC_contract-design-sub` [ubiquitous] THE SYSTEM SHALL ship a
    `/contract-design` skill that makes one plan's contract changes
    and records their ADRs and hands back to `/scope`, listed in the
    workflow as no phase.
11. `AC_build-verifies-once` [ubiquitous] THE SYSTEM SHALL ship a
    `/build` skill that runs a block's own tests after each block and
    the full build, tests, lint and `superdev validate` once after the
    last, with no code-review step, and ship no `/integrate` skill.
12. `AC_execute-plan` [ubiquitous] THE SYSTEM SHALL ship an
    `/execute-plan` skill driving `/build` over the plan's blocks with
    `superdev run`, and ship no `/execute-feature-plan` skill.
13. `AC_accept-reviews` [ubiquitous] THE SYSTEM SHALL ship an `/accept`
    skill that runs the code review, walks the contract criteria on
    the merged code, checks the documentation and closes the issue,
    invoked by the user alone.
14. `AC_workflow-text` [ubiquitous] THE SYSTEM SHALL carry the
    `.agents/superdev.md` workflow as FILE → SCOPE → BUILD → ACCEPT
    with accept marked optional and the sub-skills listed under scope,
    and the definition of done, development procedure, tracker
    concept, glossary and README agreeing with it.
15. `AC_live-lifecycles` [ubiquitous] THE SYSTEM SHALL rank `open` and
    `active` documents as live in search and no other lifecycle value.
16. `AC_adrs` [ubiquitous] THE SYSTEM SHALL carry ADR-050 active with
    `supersedes` links to ADR-031 and ADR-048, both `deprecated`, and
    a `references` link to ADR-046 naming the tracker half withdrawn.

## Alternatives considered

- Keyed EARS behaviour on the issue, criteria nested beneath — the
  first framing of this issue; the issue is ephemeral and no test
  binds it, so the keys were text.
- A requirements document per capability above the contracts — a
  fourth description of the product to keep in step with three.
- Keep frame, contract-design and plan as three phases — two
  documents and three turn boundaries holding one design.
- Name the phase `/plan` — Claude Code carries a plan skill of its own.
- Keep per-slice verification — six full verifications where the last
  one decides.
- Keep the automatic review in integrate — the review that matters is
  the one a human asks for, at accept.
- Keep three issue kinds with keys removed — three schemas for one
  record.

## Scope

- In: the issue and plan schemas and the retirement of five; the
  contract schema's nested criteria and contract-010's declarations;
  the sweep of 51 issues and 36 plans; the `/scope`, `/build`,
  `/execute-plan`, `/accept`, `/file` and `/contract-design` skills and
  the retirement of `/frame`, `/feature-plan`, `/adhoc-plan`,
  `/integrate` and `/execute-feature-plan`; the workflow text and the
  concepts that describe it; `LIVE_LIFECYCLES`; the ADRs.
- Out: changing what a contract promises; the `superdev run` verbs and
  the Stop hook; the code-review and investigation report schemas; a
  migration of the contracts on file to nested criteria — a contract
  gains them as its promises are next touched.

<!-- sokf:links -->
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/deprecated/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/deprecated/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
[sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]: /knowledge/adrs/active/adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept.md
[sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key]: /knowledge/adrs/active/adr-051-a-section-rule-declares-nested-items-and-an-optional-key.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
