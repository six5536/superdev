---
type: Decision
id: adr-002-resolve-before-plan
title: Content Resolves Before Planning
description: Pack resolution is an engine-owned phase that runs before plan_repo and hands components a resolved content set through Ctx, so Component::plan stays side-effect free.
lifecycle: active
links:
  - rel: implements
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: architectural-rules
---

# ADR-002: Content resolves before planning

- Date: 2026-08-25
- Deciders: project owner

## Context

Components today read their content from `include_str!` constants, so
`Component::plan` needs nothing but the repo and the manifest. Packs make
content something superdev must go and get — from the network, from a local
path, or from the machine-local cache — which is I/O.

[architectural-rules][sokf:architectural-rules] settles what may happen
where: "Components observe and plan; they never change anything… The engine
is the only side-effect site." Resolution is a side effect, so it cannot
happen inside `plan`. This is not a preference the feature may trade away;
`status` and `--dry-run` are free precisely because planning is pure, and
[the pack-resolution contract][sokf:contract-007-interface-pack-resolution] leans on that when it
promises `status` never reaches the network.

## Decision

We will add a resolution phase the engine owns, running before
`plan_repo`. It turns the manifest's pack entries plus the lock plus the
embedded snapshot into one resolved content set, which `Ctx` carries as a
borrowed field. `Component::plan` keeps its present signature and its
purity: it reads content from `Ctx` instead of from a constant.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Resolve before plan, carry in `Ctx` | Keeps `plan` pure and `status` free; one place fetches, one place applies; components change only in where content comes from | `Ctx` gains a field every component construction must supply |
| Resolve lazily inside `plan` | No new phase; components stay self-contained | Breaks the architectural rule outright: `status` and `--dry-run` would fetch, and planning twice would stop being free |
| Resolve during apply | Fetching sits with the other side effects | The plan could not name the files it would write, so `--dry-run` and `status` would both be wrong |
| Keep content compiled in, fetch nothing | No change at all | Is the feature |

## Consequences

- Positive: `status` provably never fetches, because it only ever calls
  `plan_repo`; the no-network acceptance criteria are structural rather than
  a rule someone must remember.
- Negative: `Ctx` grows, and every component test constructs one — a wide but
  mechanical change across the component suite.
- Follow-ups: `sync --dry-run` resolves, and therefore may fetch. It writes
  nothing to the repo, so the promise it makes — "prints the plan only" —
  still holds; the alternative is a dry run that cannot name what it would
  write.

<!-- sokf:links -->
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
