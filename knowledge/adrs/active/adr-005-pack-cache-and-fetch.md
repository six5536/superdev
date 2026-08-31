---
type: Decision
id: adr-005-pack-cache-and-fetch
title: Resolved Packs Are Cached Locally and Fetched Only on Demand
description: A resolved pack is kept under .superdev/cache/packs/, and superdev reaches the network only when it needs bytes it does not have — a new pin, or repairing a drifted file on a machine that never fetched it.
lifecycle: active
links:
  - rel: implements
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: security-requirements
---

# ADR-005: Resolved packs are cached locally and fetched only on demand

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

`sync` does not merely detect drift, it repairs it: a managed file that no
longer matches is rewritten from the content superdev ships. With content
compiled in, the desired bytes are always to hand. With packs they are not —
and the lock's hashes can prove a file changed without supplying what to
write in its place.

That collides with the promise in
[the pack-resolution contract][sokf:contract-007-interface-pack-resolution] that a repo whose content
is committed keeps working offline. It does, while nothing has drifted. The
moment something has, superdev needs the pack's actual bytes.

## Decision

We will keep a resolved pack under `.superdev/cache/packs/<digest>/` — the
gitignored machine-state directory that already holds the search index and
the backup tree — and reach the network only for bytes superdev does not
have: a pin not yet resolved on this machine, or a repair whose content is
neither committed nor cached. A steady-state `sync`, a CI `status --drift`,
and a `--dry-run` after any previous resolve all stay offline.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Cache under `.superdev/cache/packs/` | Repairs work offline on a machine that has resolved once; reuses a directory whose lifecycle is already defined; `dry-run` then `sync` fetches once | A cache directory whose growth and invalidation must be defined |
| No cache, fetch when bytes are needed | Nothing to go stale; one less directory | `dry-run` then `sync` fetches twice; every drift repair needs the network even moments after a resolve |
| Fetch once at pin time, never again | Strongest offline guarantee | Weakens `sync` from "repairs drift" to "reports drift" for pack content — a real loss of behaviour, quietly |
| Fetch only under an explicit command | Most predictable network behaviour | A changed pin makes `sync` fail until a second command is run |

## Consequences

- Positive: the no-network acceptance criteria hold on every path a CI runner
  or an offline developer actually takes.
- Negative: `.superdev/cache/` grows a second sizeable tree; eviction of
  digests no lock references is work this decision creates.
- Follow-ups: the "local by default" guarantee in
  [security-requirements][sokf:security-requirements] narrows from *no
  network input at runtime* to *no network except resolving a pin superdev
  does not already hold*; [constraints-non-goals][sokf:constraints-non-goals]
  cites that sentence for the non-PIE musl acceptance and must be revisited
  with it, at integrate.

<!-- sokf:links -->
[sokf:constraints-non-goals]: /knowledge/constraints-non-goals.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:security-requirements]: /knowledge/security-requirements.md
