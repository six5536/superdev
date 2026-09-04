---
type: Schema
id: schema-plan
title: Plan Schema
description: The one design document a piece of work carries — its goal, the contract changes it makes, its work blocks with their cases, and the decisions deferred to the user — filed among the plans.
---

# Plan Schema

Structural rules for a plan, filed among the plans as
`plan-{nnn}-{slug}`, numbered after the highest across the plans'
folders — a duplicate number is an error — and placed in its lifecycle
folder by `superdev validate --fix`, and listed in the plans
`index.md`. A plan is the one design document a piece of work carries,
whether it delivers an issue or stands alone (ADR-050): it replaces the
feature plan and the ad-hoc plan, and the frontmatter `type` selects
this schema for both. `/scope` writes a plan; `/build` works its
blocks and ticks each one; `/accept` reads it once the work is merged.

A plan is ephemeral. It holds the design while the work is open; the
contracts, the ADRs and the code hold what remains true afterwards, so
a done plan is a record and nothing reads it to build. The Goal says
what is true once the work lands; the Contract changes name each
contract the work touches and the promises and criteria it adds,
changes or withdraws — or "none" — so a reader finds every binding
change in one list; the Work blocks cut the work into the pieces
`/build` commits one at a time, each carrying its dependencies, its
done-check and its cases; the Deferred decisions hold the questions
only the user can answer.

Blocks are ordered by dependency first, then by the gap a block
closes, then by risk: every block after the blocks it depends on; a
block that closes a contract-implementation gap before the blocks that
do not, so a contract's promise is not left failing the blocks that do
not own it (ADR-044); and the riskiest early among what is left. A
dependency cycle is an error the planner refuses. A forward reference
is legal, and adding a block never renumbers the ones already written,
so list order reads and dependencies bind.

A case cites the contract criteria it covers by key where one exists —
`AC_<slug>` bare where the contract is the plan's subject, `<contract
id> AC_<slug>` elsewhere — and otherwise states what it checks. A case
belongs to exactly one block; an integration or end-to-end case belongs
to the block that completes its boundary. Every criterion the Contract
changes add or change is covered by at least one case across the plan.

````yaml
description: >
  The one design document a piece of work carries — what is true once
  it lands, the contract changes it makes, the work blocks that deliver
  it with their cases, and the decisions deferred to the user. Written
  by scope; worked and ticked by build; read by accept.
line-limit: 800

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
    enum: [open, done, abandoned]
    description: >
      The folder is the value: open while a block is outstanding; build
      ticks each block's Done at its commit and sets done after the last
      block; abandoned when the work was dropped, with the reason
      recorded under Goal.

sections-ordered: true
sections:
  - heading-pattern: '^Plan: .+$'
    level: 1
    required: true
    description: >
      Title heading ("Plan: {title}"), followed by a Request line
      linking the issue the plan delivers, where one exists; a plan for
      one-off work carries no Request line.
  - heading: "Goal"
    level: 2
    required: true
    content: prose
    description: >
      What is true once the work lands and is not true today, and the
      constraint that shaped the approach, in prose. The evidence a
      reader needs to trust the design sits here too — a path, a count,
      a command and its result — and so does what is deliberately out
      of scope. An abandoned plan records the reason here.
  - heading: "Contract changes"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per contract the work touches, opening with the
      contract's id and naming the promises and criteria it adds,
      changes or withdraws, by key; or the single bullet "none" when the
      work touches no contract. The list is complete: a contract not
      listed here is not changed by this plan.
  - heading: "Work blocks"
    level: 2
    required: true
    description: >
      Ordered by dependency first, then by the gap a block closes, then
      by risk: every block after the blocks it depends on; a block that
      closes a contract-implementation gap before the blocks that do
      not, so a contract's promise is not left failing the blocks that
      do not own it (ADR-044); and the riskiest early among what is
      left. A dependency cycle is an error the planner refuses. Carries
      the block subsections and no prose of its own.
  - heading-pattern: '^Block \d+: .+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One block, named ("Block {n}: {name}"), small enough to build and
      commit in one pass. Body: a "- [ ] Done — ticked by build at its
      commit." checkbox; Depends-on: the numbers of the blocks this one
      needs done first, or "none" — a forward reference is legal, and
      adding a block never renumbers the ones already written; Change:
      what this block changes, and where; Done-check: the pass/fail
      check build runs against this block; Cases: the block's test
      cases, one per line, each citing the contract criteria it covers
      by key where one exists — bare ("covers AC_stale-include") where
      the contract is the plan's subject, the contract's id then the key
      ("covers contract-010 AC_stale-include") elsewhere — and otherwise
      stating what it checks. A case belongs to exactly one block; an
      integration or end-to-end case to the block that completes its
      boundary, which usually puts the heaviest cases last.
  - heading: "Deferred decisions"
    level: 2
    content: bullet-list
    description: >
      Questions only the user can answer, one bullet per question,
      naming the block it blocks or "blocks nothing": written by an
      unattended run when a gate returns to scope, or by scope when the
      user is not there to ask. The run ends by putting these to the
      user in sequence; the answers are recorded here and the next run
      reads them. Omit the section when there are none.

example: |
  ---
  type: Plan
  id: plan-001-pack-source-allowlist
  title: Pack source transport allowlist
  description: The manifest loader refuses a pack source whose transport is not https, ssh or file, naming the source.
  lifecycle: open
  ---

  # Plan: Pack source transport allowlist

  Request: [issue-041-pack-source-allowlist][sokf:issue-041-pack-source-allowlist]

  ## Goal

  A manifest naming a `git://` pack source fails at parse, naming the
  source, and an `https`, `ssh` or `file` source resolves as before.
  Today `resolve.rs:88` hands any scheme to git. Out of scope: the
  pack cache, which reads schemes and never matches them.

  ## Contract changes

  - contract-004: `P_scheme-refused` added, with `AC_git-refused` and
    `AC_https-resolves` nested under it; `P_any-scheme` withdrawn.

  ## Work blocks

  ### Block 1: Scheme parsing and refusal

  - [ ] Done — ticked by build at its commit.
  - Depends-on: none.
  - Change: parse the pack-source scheme in the manifest loader; refuse
    any transport that is not https, ssh or file.
  - Done-check: a git:// source fails at parse naming the source; an
    https source resolves as before.
  - Cases:
    - unit: a git:// source is refused at parse, naming the source
      (covers AC_git-refused).
    - unit: an https source resolves as before (covers
      AC_https-resolves).

  ### Block 2: Refusal message

  - [ ] Done — ticked by build at its commit.
  - Depends-on: 1.
  - Change: error output naming the offending source.
  - Done-check: the end-to-end run shows the refusal naming the source.
  - Cases:
    - e2e: `superdev sync` against a git:// manifest prints the refusal
      with the source named (covers AC_git-refused).
    - e2e: the refusal exits with the usage code — no criterion; the
      exit code is not promised.

  ## Deferred decisions

  - Block 2: should a refused source name the allowed transports in the
    message? Recommended default: yes. Blocks nothing.
````
