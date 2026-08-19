---
name: to-spec
description: Turn the current conversation into a Spec concept in the knowledge bundle — no interview, just synthesis of what you've already discussed.
disable-model-invocation: true
---

This skill takes the current conversation context and codebase understanding and produces a spec. Do NOT interview the user — just synthesize what you already know.

## Process

1. Explore the repo to understand the current state of the codebase, if you haven't already. Use the glossary concept's vocabulary throughout the spec, and respect the Decision concepts and stable specs in the area you're touching.

2. Sketch out the seams at which you're going to test the feature. Existing seams should be preferred to new ones. Use the highest seam possible. If new seams are needed, propose them at the highest point you can. The fewer seams across the codebase, the better - the ideal number is one.

Check with the user that these seams match their expectations.

3. Write the spec as an AOKF concept at `knowledge/specs/Snnn-<topic>-design.md` using [SPEC-FORMAT.md](./SPEC-FORMAT.md), and add it to `knowledge/specs/index.md`. The validator must pass: `superdev aokf validate knowledge` (in Claude Code the PostToolUse hook runs it for you).

Specs are permanent decision records: `status: draft` while in flight, `stable` once implemented. When a spec lands, move the durable knowledge into the core concepts and keep the spec as the record of why.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
