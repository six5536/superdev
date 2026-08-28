---
type: Decision
id: adr-009-update-queries-default-source
title: Update Queries the Default Source for a Newer Pack
description: superdev update asks the blueprint's default pack source for its newest release and moves that pin there, so a content release reaches repos whose binary has not changed; other sources' pins are never moved.
status: stable
links:
  - rel: implements
    to: spec-014-content-packs
  - rel: relates-to
    to: security-requirements
---

# ADR-009: Update queries the default source for a newer pack

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

The default pack pin is compiled into the binary, so it can never advance
past that binary on its own. Left there, a content release
([ADR-008](adr-008-one-command-per-release.md)) reaches only repos whose owner
hand-edits `config.toml` — and
[the spec](../specs/spec-014-content-packs.md)'s goal, shipping a skill
fix without a five-platform release, is mostly unmet.

`update` already means "bring pins current", is always typed deliberately,
and already rewrites the manifest. It is the one verb where going to look is
in character.

## Decision

We will have `update` ask the blueprint's **default** source for its newest
release tag and move that pin there, even when the result is ahead of the
blueprint's default. A pin naming any other source is reported and left
alone, as decided for the offline case. With no network, `update` moves the
pin no further than the blueprint's default and says it could not check.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Query the default source only | The one path by which a content fix reaches an unchanged binary; explicit, typed, and limited to the source superdev itself ships | `update` now reaches the network on a path that was offline before |
| Never query; pin moves to the blueprint default only | `update` stays entirely offline and wholly predictable | A content release reaches nobody on a default pin until the next binary — most of the feature's value |
| Query every source, including third-party | Uniform | Silently pulls unreviewed third-party content on a routine command, which the trust model most needs a human for |
| Query only under an explicit flag | Default stays offline; opt-in exists | Most people never find the flag, so the feature quietly does not work for them |

## Consequences

- Positive: `superdev update` is the single answer to "how do I get the
  latest skills", whether the fix shipped in a binary or not.
- Negative: `update` becomes a network-touching verb. The narrowing of the
  "local by default" guarantee in
  [security-requirements](../security-requirements.md) must cover it as well
  as resolution, and its failure must degrade to the blueprint default rather
  than erroring.
- Negative: third-party pins never move by themselves, so their owners must
  track releases some other way. That is the trust model working as intended,
  and the docs should say so plainly.
