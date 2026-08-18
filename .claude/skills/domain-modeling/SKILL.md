---
name: domain-modeling
description: Build and sharpen a project's domain model. Use when the user wants to pin down domain terminology or a ubiquitous language, record an architectural decision, or when another skill needs to maintain the domain model.
---

# Domain Modeling

Actively build and sharpen the project's domain model as you design. This is the *active* discipline — challenging terms, inventing edge-case scenarios, and writing the glossary and decisions down the moment they crystallise. (Merely *reading* the glossary for vocabulary is not this skill — that's a one-line habit any skill can do. This skill is for when you're changing the model, not just consuming it.)

## Where the model lives

The domain model is part of the AOKF bundle at `knowledge/`:

- **Glossary** — the `knowledge/glossary.md` concept, seeded by `superdev init`. Format: [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md).
- **Decisions** — one concept per decision under `knowledge/decisions/`, named `Dnnn-<slug>.md`. Format: [DECISION-FORMAT.md](./DECISION-FORMAT.md). Create the directory lazily — only when the first decision is recorded — and list each new concept in the bundle's `index.md`.

There is no separate context map. The bundle stays single-glossary until the same word means different things in different areas; then the glossary splits into per-context glossary concepts, each linked `part-of` to the subsystem or component concept that owns that language. `aokf_graph` renders the map.

## During the session

### Challenge against the glossary

When the user uses a term that conflicts with the glossary, call it out immediately. "Your glossary defines 'cancellation' as X, but you seem to mean Y — which is it?"

### Sharpen fuzzy language

When the user uses vague or overloaded terms, propose a precise canonical term. "You're saying 'account' — do you mean the Customer or the User? Those are different things."

### Discuss concrete scenarios

When domain relationships are being discussed, stress-test them with specific scenarios. Invent scenarios that probe edge cases and force the user to be precise about the boundaries between concepts.

### Cross-reference with code

When the user states how something works, check whether the code agrees. If you find a contradiction, surface it: "Your code cancels entire Orders, but you just said partial cancellation is possible — which is right?"

### Update the glossary inline

When a term is resolved, update the glossary concept right there. Don't batch these up — capture them as they happen. Use the format in [GLOSSARY-FORMAT.md](./GLOSSARY-FORMAT.md).

The glossary should be totally devoid of implementation details. Do not treat it as a spec, a scratch pad, or a repository for implementation decisions. It is a glossary and nothing else.

After any bundle edit the validator must pass: `superdev aokf validate knowledge` (in Claude Code the PostToolUse hook runs it for you).

### Offer decision records sparingly

Only offer to record a decision when all three are true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will wonder "why did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If any of the three is missing, skip it. Use the format in [DECISION-FORMAT.md](./DECISION-FORMAT.md).

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
