---
type: Spec
id: spec-013-implements-rel
title: Implements in the Core Vocabulary
description: Promote implements/implemented-by into the SOKF core relationship vocabulary — spec bumped to 0.2, validator and graph taught the pair, and the issue-tracker convention realigned.
status: stable
links:
  - rel: relates-to
    to: spec-002-aokf-mcp-server
  - rel: relates-to
    to: issue-tracker
---

# Summary

The plan-implements-spec edge is first-class in superdev's own workflow: the
shipped plan and issue templates mint `rel: implements`, and the implement and
code-review skills navigate by it to find the spec behind a piece of work. The
core vocabulary did not know the rel, so the format's own tooling degraded it —
a permanent warning on every run, every generic consumer — including the
[graph traversal][sokf:spec-002-aokf-mcp-server] — reading the edge as bare
`relates-to`, and no inverse, so a spec could not show what implements it.
The knowledge disagreed with itself: the [issue tracker][sokf:issue-tracker]
prescribed `references` for the relationship the templates wrote as
`implements`.

This promotes the pair into the core vocabulary, bumps the format to 0.2 under
its own rule that additions are a minor bump, teaches the validator and the
graph, and aligns the issue-tracker convention.

# Behaviour

- The embedded SPEC.md declares 0.2 and its §8 table carries `implements` /
  `implemented-by`, meaning: delivers or realises the target — a plan or issue
  implementing a spec. `sync` rewrites the owned copy in managed repos.
- The validator accepts both as core rels and emits no non-core warning.
  Unknown rels still warn and are read as `relates-to`.
- The graph synthesises inverses both ways: a plan → spec `implements` edge
  shows from the spec side as `implemented-by`, not as `relates-to`.
- Knowledge declaring `0.1` stays conformant, because the change is additive.
  This repository's own manifest moves to `0.2`.
- The issue-tracker concept prescribes `rel: implements` for an issue or plan
  that implements a spec, declared from the issue side as before;
  `references` remains the rel for merely citing or affecting a spec.
- The shipped plan and issue templates are unchanged — the rel they already
  mint is now conformant.

# Acceptance criteria

1. `.agents/sokf/SPEC.md` §8 lists `implements` with inverse `implemented-by`,
   and the document's version line reads 0.2.
2. A concept declaring `rel: implements` validates with no warning; one
   declaring an unknown rel still warns.
3. `inverse_rel("implements")` is `implemented-by` and `inverse_rel`
   ("implemented-by") is `implements`.
4. The graph shows a spec's back-edge labelled `implemented-by`.
5. The live knowledge validates with zero warnings, which is the
   behaviour-level proof the P002 warning is gone.
6. `knowledge/issue-tracker.md` prescribes `implements` for the
   implements-a-spec case and keeps its declare-from-the-issue-side rule.

# Edge cases & errors

- Knowledge whose manifest still declares `0.1` and which uses the new rel:
  conformant, because the addition is backwards compatible. No gate refuses it.
- An unknown rel: unchanged. It warns and is read as `relates-to`, so
  promoting one pair does not widen what the validator accepts silently.
- A concept declaring both `implements` and `implemented-by` to the same
  target: not refused. §8 asks for one declaration per edge from the more
  natural side; the graph would show the pair twice, which is visible rather
  than wrong.
- A back-edge to a concept outside the knowledge: the graph's existing
  unknown-id handling applies, unchanged by this.

# Out of scope

- Any other vocabulary addition; the core stays minimal.
- Changing how custom rels are treated — unknown rels still warn and read as
  `relates-to`.
- A manifest-declared-version compatibility gate in the validator beyond what
  exists today.
- Rewriting historical concepts' links other than the issue-tracker
  prescription.

# Open questions

None.

# Test plan: implements in the core vocabulary

## Scope

- The validator's core-rel set and its warning behaviour.
- The graph's inverse map and the back-edges it synthesises.
- The live knowledge, end to end through the CLI.
- Out: the templates, which are unchanged, and the MCP tools, which read the
  graph rather than the vocabulary.

## Risks driving this plan

1. The pair is accepted but the inverse is not mapped, so the warning goes
   and the back-edge silently stays `relates-to` — the half-fix that looks
   fixed from the validator alone.
2. Widening the core set accidentally silences the unknown-rel warning, which
   is the check that keeps the vocabulary closed.
3. The embedded spec and this repository's copy of it drift, so managed repos
   get 0.2 and this one keeps 0.1, or the reverse.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | A core rel is accepted | unit | a concept declaring `rel: implements` | no finding |
| 2 | Its inverse is accepted | unit | a concept declaring `rel: implemented-by` | no finding |
| 3 | The vocabulary stays closed | unit | a concept declaring an unknown rel | the non-core warning still fires |
| 4 | The inverse map is symmetric | unit | `inverse_rel` over both names | maps each to the other |
| 5 | The back-edge is labelled | unit | a plan → spec `implements` edge | the spec side reads `implemented-by` |
| 6 | The live knowledge is clean | end-to-end | `superdev validate` in this repo | zero warnings |

### Manual verification

1. Run `superdev validate` in this repository and read the report: zero
   warnings, where P002's `implements` link previously produced one.
2. Open `.agents/sokf/SPEC.md` and confirm §8 carries the pair and the
   version line reads 0.2.
3. Run `superdev sync` in a scratch managed repo and confirm the owned copy
   is rewritten to 0.2.

## Exit criteria

- Every automated case above passes, and the unknown-rel warning is still
  proved by a test that fails if the core set is widened by mistake.
- `superdev validate` reports zero warnings on the live knowledge.
- The embedded spec and this repository's copy agree, which the existing
  byte-equality test covers.

<!-- sokf:links -->
[sokf:issue-tracker]: /knowledge/issue-tracker.md
[sokf:spec-002-aokf-mcp-server]: /knowledge/specs/spec-002-aokf-mcp-server.md
