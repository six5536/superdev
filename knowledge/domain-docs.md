---
type: Convention
id: domain-docs
title: Domain Docs
description: Which bundle concepts serve as the domain docs and how engineering skills use them.
status: stable
---

This repo is single-context. Its domain docs are bundle concepts, not
`CONTEXT.md` or `docs/adr/` ([Matt Pocock skills
overrides](/.agents/MATT-POCOCK-SKILLS.md)). Before exploring the
codebase, read:

- [Domain Glossary](glossary.md) — the `CONTEXT.md` equivalent.
- [Architecture](architecture.md) and
  [Architectural Rules](architectural-rules.md) — the system context.
- Decision records: concepts with `type: Decision`, and the
  [specs](specs/index.md) — each spec is a permanent decision record.

# Where /domain-modeling writes

ADRs become AOKF concepts (`type: Decision`) in the bundle. Glossary
terms and context go into the glossary and architecture concepts. Never
create `CONTEXT.md` or `docs/adr/`.

# Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor
proposal, a hypothesis, a test name), use the term as defined in the
[glossary](glossary.md). Don't drift to synonyms the glossary avoids.

If the concept you need isn't in the glossary yet, that's a signal —
either you're inventing language the project doesn't use (reconsider) or
there's a real gap (note it for `/domain-modeling`).

# Flag decision conflicts

If your output contradicts an existing Decision concept or spec, surface
it explicitly rather than silently overriding:

> _Contradicts the workflows-provider-default spec — but worth reopening
> because…_
