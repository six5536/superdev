---
type: Decision
id: adr-053-sokf-file-tools-delegate-to-pi
title: SOKF file tools delegate semantics and presentation to Pi
description: SOKF resolves and protects canonical knowledge targets while Pi 0.85.1 remains authoritative for file-tool semantics, results, and rendering.
lifecycle: active
status: draft
---

# ADR-053: SOKF file tools delegate semantics and presentation to Pi

- Date: 2026-09-06
- Deciders: superdev maintainers

## Context

The Pi extension currently routes SOKF identities through separate read and mutation result dialects. The adapter reconstructs edit details, exposes mutation envelopes, and couples semantic retrieval to exact file reads. Those choices diverge from Pi 0.85.1 and make equivalent physical and virtual targets behave differently.

SOKF still owns identity resolution, repository containment, generated ownership, agent-safe policy, stable identity, verification preservation, automatic repair, refiling, and validation. Applied mutations can leave actionable validation findings, and Pi sessions need bounded correction follow-ups that survive active-branch lifecycle events.

## Decision

We will resolve SOKF identities before execution, then delegate file semantics and presentation to Pi 0.85.1 factories and exported helpers. A narrow adapter is permitted only where package exports expose no preparation or presentation seam, and paired tests will prove that adapter byte-for-byte against Pi's operation boundaries.

We will queue virtual and physical aliases by canonical physical target. Edits will persist through an agent-safe compare-and-swap source transport that re-resolves the logical ingress and rejects target drift. A missing physical destination will canonicalize through its nearest existing ancestor. Repository-contained symlink targets are accepted, including targets outside `knowledge/`; repository escapes are rejected.

An authoritative `applied: true` response is the mutation outcome boundary. The adapter will retain the requested bytes and every successfully persisted repair or refiling change, shield dispatched persistence from late cancellation, hold the queue through mutation-state capture, and return Pi's unchanged success result. Failures without an authoritative applied response remain tool errors. An indeterminate local MCP failure requires target inspection and `superdev validate`, not durable outcome reconciliation.

Validation diagnostics will remain outside file-tool content. Repository-scoped non-context snapshots will preserve the `clean`, `pending-auto(0..2)`, `manual`, and `pending-manual` follow-up states on the active session branch. Only valid final validation resets the sequence. Snapshot failure after apply preserves success and in-memory pending validation while producing separate non-triggering manual guidance.

The unreleased MCP surface will replace `sokf_read` with `sokf_resolve_source` and `sokf_retrieve`. The resolver carries no semantic content. The retrieval operation retains semantic overview, rendered-concept, section, and line-window behavior.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Delegate to Pi factories with narrow byte-proven adapters | Exact parity and one file-tool experience | Couples parity tests to pinned Pi 0.85.1 |
| Reimplement Pi file algorithms in the extension | Full local control | Creates an approximate fork that drifts |
| Keep rendered virtual reads and mutation envelopes | Smallest source change | Preserves the current contract divergence |
| Roll back after repair or validation findings | Leaves only valid knowledge | Loses acknowledged work and forces expensive rewrites |
| Persist mutation identities and reconcile outcomes | Resolves indeterminate transport failures | Adds durable protocol and lifecycle complexity outside this issue |
| Preserve `sokf_read` as a compatibility alias | Avoids an MCP rename | Keeps mixed exact-source and semantic semantics in an unreleased API |

## Consequences

- Positive: physical paths and equivalent SOKF identities share Pi's tested file semantics, results, metadata, and rendering.
- Positive: SOKF safety, repair, refiling, and validation remain authoritative without replacing successful file-tool results.
- Positive: bounded follow-ups retain enough active-branch state to prevent lifecycle events from resetting the correction cap.
- Negative: parity evidence pins Pi 0.85.1 and requires synchronization when Pi changes its built-in contracts.
- Negative: an indeterminate local MCP failure requires manual filesystem inspection and validation.
- Follow-ups: changes to semantic ranking, unrelated MCP lifecycle behavior, durable outcome reconciliation, and packaging design require separate issues.
