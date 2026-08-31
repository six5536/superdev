---
type: Decision
id: adr-008-one-command-per-release
title: One Command Per Release, Two Tag Series
description: The release script cuts the binary and the pack together from one commit, and a second command cuts a pack release alone — so neither release has a second step a human could get wrong.
lifecycle: active
links:
  - rel: implements
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: release-procedure
---

# ADR-008: One command per release, two tag series

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[Content packs][sokf:contract-007-interface-pack-resolution] give the binary a
default pack pin, `DEFAULT_PACK.rev`, which must name a rev whose `/pack/`
is the content that binary embedded. Left to a human, that is a second thing
to get right at every binary release: tag the pack, then cut a binary naming
that tag. A compile-time assertion can detect the mismatch; it cannot remove
the step.

The feature's whole premise is also that a pack releases without a binary, so
that path must be one command too.

## Decision

We will make each release one command, and give the pack its own version
series.

- `npm run release X.Y.Z` sets `pack.toml`'s version, sets
  `DEFAULT_PACK.rev` to the pack tag it is about to create, commits, and
  creates both `vX.Y.Z` and `assets-vA.B.C` — one commit, one push.
- `npm run release:pack` bumps `pack.toml` and creates `assets-vA.B.C`
  alone. The binary workflow ignores `assets-v*`, so no five-platform build
  runs.

The pack keeps a version of its own rather than sharing the binary's, so
binary semver stays contiguous — no absent `0.3.2` on crates.io for a content
release to explain — and `blueprint` keeps meaning the binary version it
means today.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Two series, one command each | Both release paths are single commands; binary semver stays contiguous; `blueprint` unchanged | Two version numbers to hold in mind, and the script must set `DEFAULT_PACK.rev` before tagging |
| One shared series | Snapshot and pin are literally the same commit, so mismatch is inexpressible rather than merely prevented | Content releases consume binary version numbers, leaving holes on crates.io and npm; `blueprint` starts doubling as a content version |
| Pack version primary, binary tracks it | Fits a project whose content changes faster than its code | Rewrites what `blueprint` means and how the binary is versioned — far more than this feature needs |
| Two independent releases | Cleanest conceptual split | The two-things-must-both-be-right shape this ADR exists to remove |

## Consequences

- Positive: neither release has a step a human can forget or mistype; the
  `npm run release` script [release-procedure][sokf:release-procedure]
  describes, which already keeps 18 version locations in lockstep, keeps two
  more.
- Positive: a content fix ships with no binary build, which is the point of
  the feature.
- Negative: two version series coexist in one repository, so the changelog
  and the release notes must be clear about which is being cut.
- Follow-ups: `update` reaches a content release by asking the default source
  for its newest release
  ([ADR-009][sokf:adr-009-update-queries-default-source]); without that, a content
  release reaches only repos that hand-pin it.

<!-- sokf:links -->
[sokf:adr-009-update-queries-default-source]: /knowledge/adrs/active/adr-009-update-queries-default-source.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:release-procedure]: /knowledge/release-procedure.md
