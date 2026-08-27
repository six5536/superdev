---
type: FeatureRequest
id: issue-017-the-format-has-no-agent-facing-document
title: The format the agent must write in has no document, and the renderer that would produce one has no consumer
description: Every skill and schema is written in superdev-format, and the only statement of it is a 700-line grammar file the agent is never pointed at; the doc renderer ported for exactly this now exists in the binary with nothing calling it but a flag nobody runs.
status: draft
tags: [needs-triage]
links:
  - rel: relates-to
    to: adhoc-plan-006-rust-format-validator
---

# Bug: the format the agent must write in has no document

## Summary

`.agents/core.md`, 21 skills and 39 schemas are written in superdev-format,
and an agent asked to write or edit one has nothing to read about it. The
grammar is the only statement of the language, and it is 700 lines of YAML the
agent is never told about. The consequence is a validator finding rather than a
rule followed: the first fixture written for the port broke four rules its
author had no way to know.

## Environment

- Version/commit: superdev 0.2.0, AOKF 0.3
- Platform: any

## Steps to reproduce

1. `grep -rn 'superdev-format\|grammar.yaml' .agents/` — nothing outside
   `.agents/format/grammar.yaml` itself mentions the format.
2. Ask an agent to add a skill. Nothing in its instructions names the element
   vocabulary, the condition forms, or the one-home-per-statement rule.
3. `cargo run --quiet -- validate --doc` prints the whole language, in prose,
   in 191 lines. Nothing reads it.

## Expected behaviour

The format has an agent-facing document beside its spec, the way AOKF has
`.agents/aokf.md` beside `.agents/aokf/SPEC.md`, and the agent is pointed at
it before it writes a unit. The renderer exists precisely so that document
cannot drift from the grammar it describes.

## Actual behaviour

`superdev validate --doc` is the only consumer, and it is a flag a person
types by hand. `format/doc.rs` is 190 lines of ported code held by one golden,
carrying a language nobody is shown.

## Root cause (if known)

The [format validator plan](../adhoc-plans/P006-rust-format-validator.md)
ports the renderer under D-13 — deleting the reference was one-way, so the
option had to survive the port — and lists generating `.agents/format.md` from
it as a non-goal, because wiring it up decides what that file is and who owns
it. That is a pack question: `.agents/aokf.md` is binary-owned, and a format
document would have to be too, since the binary is what enforces the grammar.
The question was deferred and never asked.

## Proposed fix / workaround

- Decide what the document is: the rendered grammar as it stands, a written
  guide with the render as an appendix, or a short instruction file pointing at
  the grammar. The render is complete and stays correct for free; a written
  guide reads better and drifts.
- Decide who owns it. `.agents/aokf/SPEC.md` and `.agents/aokf.md` are the
  precedent: a spec its compiled validator enforces, and an instruction file
  beside it, both binary-owned and written into every managed repository. The
  grammar has the same shape and none of the plumbing —
  `.agents/format/grammar.yaml` is not in the blueprint at all, so a repository
  superdev manages gets the checks without the rules.
- Whichever is chosen, generate it rather than writing it, and hold it with a
  test the way the doc golden is held.
- Meanwhile `cargo run --quiet -- validate --doc` answers any question about
  the language, and is worth naming wherever the format is discussed.

## Regression risk

Low as code: the renderer is already tested against a golden captured from the
reference. The risk is in the blueprint — adding a binary-owned file writes it
into every managed repository on the next `sync`, and doing that while
[I016](I016-sync-would-revert-the-schema-migration.md) stands adds a line to a
drift report already carrying 65.
