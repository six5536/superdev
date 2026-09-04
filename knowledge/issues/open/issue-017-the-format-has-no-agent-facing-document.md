---
type: Issue
id: issue-017-the-format-has-no-agent-facing-document
title: The format the agent must write in has no document, and the renderer that would produce one has no consumer
description: Every skill and schema is written in superdev-format, and the only statement of it is a 700-line grammar file the agent is never pointed at; the doc renderer ported for exactly this now exists in the binary with nothing calling it but a flag nobody runs.
kind: feature
lifecycle: open
links:
  - rel: relates-to
    to: plan-006-rust-format-validator
---

# Feature: the format the agent must write in has no document

## Summary

`.agents/core.md`, 21 skills and 39 schemas are written in superdev-format,
and an agent asked to write or edit one has nothing to read about it. The
grammar is the only statement of the language, and it is 700 lines of YAML the
agent is never told about. The consequence is a validator finding rather than a
rule followed: the first fixture written for the port broke four rules its
author had no way to know.

## Context

Ask an agent to add a skill: nothing in its instructions names the element
vocabulary, the condition forms, or the one-home-per-statement rule.
`superdev validate --doc` is the only consumer, and it is a flag a person
types by hand. `format/doc.rs` is 190 lines of ported code held by one golden,
carrying a language nobody is shown.

The [format validator plan][sokf:plan-006-rust-format-validator]
ports the renderer under D-13 — deleting the reference was one-way, so the
option had to survive the port — and lists generating `.agents/format.md` from
it as a non-goal, because wiring it up decides what that file is and who owns
it. That is a pack question: `.agents/aokf.md` is binary-owned, and a format
document would have to be too, since the binary is what enforces the grammar.
The question was deferred and never asked.

## Behaviour

The format has an agent-facing document beside its spec, the way AOKF has
`.agents/aokf.md` beside `.agents/aokf/SPEC.md`, and the agent is pointed at
it before it writes a unit. The renderer exists precisely so that document
cannot drift from the grammar it describes.

- superdev ships an agent-facing description of superdev-format beside its
  grammar, rendered from the grammar so the two cannot drift.
- When an agent is directed to write a skill or schema, superdev points it
  at that document first.

## Scope

Two decisions and the generation that follows them.

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

Alternatives considered:

- Point the agent at `grammar.yaml` directly — 700 lines of machine-readable
  rules, written for a parser rather than a reader.
- Write the document by hand beside the grammar — it would go stale the
  first time the grammar changed, which is the failure this project already
  knows well.
- Leave the renderer unused and let the schemas carry the guidance — they
  describe documents, not the language skills are written in.

<!-- sokf:links -->
[sokf:plan-006-rust-format-validator]: /knowledge/plans/done/plan-006-rust-format-validator.md
