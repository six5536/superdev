---
type: Decision
id: adr-004-base-pack-identity
title: The Base Pack Is Identified by Normalised Source
description: A pack entry replaces the embedded snapshot when its source normalises to the blueprint's default source; status names which entry it treated as the base, so a wrong match is visible rather than silent.
status: stable
links:
  - rel: implements
    to: spec-content-packs
---

# ADR-004: The base pack is identified by normalised source

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[The spec](../specs/S014-content-packs-design.md) gives a pack entry two
possible meanings. From the snapshot's own source it *replaces* layer 0, so
what that rev drops leaves the repo. From anywhere else it *layers* on top,
adding and superseding but never removing. superdev has to tell them apart.

The obvious test — does the source string equal the blueprint's default —
fails on the same repository written a different way. `github:six5536/superdev`,
`https://github.com/six5536/superdev.git` and `git@github.com:six5536/superdev.git`
are one place. Comparing them literally means three of the four forms are
treated as a stranger's pack, so removals stop propagating with nothing on
screen to say why. A silent behaviour change is the worst failure available
here.

## Decision

We will normalise a source before comparing it — strip the scheme, any
userinfo, a `.git` suffix and a trailing slash, and lowercase host and path —
and treat an entry whose normalised source equals the blueprint's default as
the base. The same comparison decides whose pin `update` may move. `status`
prints which entry it treated as the base and which layered, so a wrong
match shows up on the next command.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Normalised source, reported by `status` | Handles every realistic spelling; one rule serves both replace and `update`; the report converts a silent miss into a visible one | Normalisation rules are a small surface that can itself be wrong |
| First entry is the base, positionally | No comparison for the replace decision, so aliasing cannot affect it; a fork gets replacement free | `update` still needs identity to know whose pin to move, so the comparison does not go away; reordering silently changes meaning |
| An explicit `base = true` field | Nothing inferred; visible in a diff | Must be remembered; needs defined behaviour for two bases and for none — states that exist only because the field does |
| Exact string match | Trivial | The first differently-spelled address silently gets the wrong behaviour |

## Consequences

- Positive: a user may write the source in whatever form their tooling
  produces; `status` gains a content line that makes layering legible.
- Negative: normalisation is a comparison superdev must get right, and two
  genuinely different sources that normalise alike would be conflated.
- Follow-ups: the `status` content line is part of the observable surface and
  belongs in [api-contracts](../api-contracts.md) at integrate.
