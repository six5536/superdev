---
type: Spec
id: spec-implements-rel
title: Implements in the Core Vocabulary
description: Promote implements/implemented-by into the AOKF core relationship vocabulary — spec bumped to 0.2, validator and graph taught the pair, and the issue-tracker convention realigned.
status: stable
links:
  - rel: relates-to
    to: spec-aokf-mcp-server
  - rel: relates-to
    to: issue-tracker
---

# Problem

The plan-implements-spec edge is first-class in superdev's own
workflow: the shipped plan and issue templates mint `rel: implements`,
and the implement and code-review skills navigate by it to find the
spec behind a piece of work. Yet the AOKF core vocabulary does not
know the rel, so the format's own tooling degrades it: the validator
warns on every run (a permanent warning trains readers to ignore
warnings), every generic consumer — the
[graph traversal](S002-aokf-mcp-server-design.md), search — reads the
edge as bare `relates-to`, and with no defined inverse the graph
cannot label the back-edge, so a spec cannot show what implements it.
The canonical knowledge even disagrees with itself: the
[issue tracker](../issue-tracker.md) prescribes `references` for the
same relationship the templates write as `implements`.

# Solution

Promote the pair into the core vocabulary. The embedded AOKF spec's
relationship table gains `implements` (inverse `implemented-by`,
meaning: delivers or realises the target — a plan or issue
implementing a spec), and the document bumps to AOKF 0.2 under its own
versioning rule that additions are a minor bump. The validator's
core-rel set and the graph's inverse map learn the pair, and the
issue-tracker convention aligns on `implements` while keeping its
declare-from-the-issue-side rule.

# Behaviour

1. The embedded SPEC.md declares AOKF 0.2 and its §8 table carries
   `implements` / `implemented-by` with the meaning above; `sync`
   rewrites the owned `.agents/aokf/SPEC.md` copy in managed repos.
2. The validator accepts `implements` and `implemented-by` as core
   rels: no non-core warning. Unknown rels still warn, read as
   `relates-to`.
3. The graph synthesises inverses for the pair both ways: a
   plan → spec `implements` edge shows from the spec side as
   `implemented-by`, not as `relates-to`.
4. Knowledge declaring `aokf: "0.1"` stays conformant — the change is
   additive. This repo's knowledge manifest moves to `aokf: "0.2"`.
5. The live knowledge validates with zero warnings: the P002 plan's
   `implements` link is now core.
6. The issue-tracker concept prescribes `rel: implements` for an
   issue or plan that implements a spec, declared from the
   issue/plan side as before; `references` remains the rel for
   merely citing or affecting a spec.
7. The shipped plan and issue templates are unchanged — the rel they
   already mint is now conformant.

# Design decisions

- Promoted rather than demoted to `depends-on` (decided with the
  user): the edge is load-bearing for the product's own spec → plan →
  implement flow, and `depends-on` would trade the precise name for
  silence — skills would need target-type filtering to find the spec
  among a plan's other dependencies. A format whose reference tooling
  warns about edges its reference skills create is the inconsistency
  being fixed.
- Version 0.2, not an in-place edit of 0.1 (decided with the user):
  §12 defines additions as a minor bump, and the first vocabulary
  change should exercise that discipline, not sidestep it.
- The meaning is scoped to delivery ("delivers or realises the
  target"), not general conformance, so the rel stays an edge between
  work records and their decision record rather than a licence for
  arbitrary X-implements-Y claims.
- Plans keep declaring the edge from their side (one declaration per
  edge, from the more natural side, per §8); the issue tracker's
  original reason for its rule — deleting the issue must leave no
  dangling edge — was about direction, not the rel, so it survives
  the realignment.

# Testing

Seams, all existing, as confirmed with the user: validator unit tests
(the pair passes with no warning; an unknown rel still warns), graph
unit tests (`inverse_rel` maps the pair both ways; a synthesized
back-edge is labelled `implemented-by`), and the live-knowledge CLI
end-to-end tightened to assert zero warnings — the behaviour-level
proof the P002 warning is gone. Prior art: the existing non-core-rel
warning tests, the `inverse_rel` table test, and
`aokf_validate_passes_the_live_bundle`.

# Out of scope

- Any other vocabulary addition; the core stays minimal.
- Changing how custom rels are treated — unknown rels still warn and
  read as `relates-to`.
- A manifest-declared-version compatibility gate in the validator
  beyond what exists today.
- Rewriting historical concepts' links other than the issue-tracker
  prescription.

# Open questions

None.
